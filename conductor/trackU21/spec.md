# Track U21 Spec: Non-Blocking Embed Concurrency Cap

## Background

Track U14's `EmbedSemaphore` (in `src/semantic/concurrency.rs`) uses `parking_lot::Mutex<usize>` + `parking_lot::Condvar` to cap concurrent embed calls. This is a **blocking** semaphore: a rayon worker that hits the cap blocks on `Condvar::wait` until a permit is released.

For the current use case (cap=4, small batches) this is fine. But:
- The cap is now configurable (U16) and can be set higher
- The semaphore is a global lock — N rayon workers contend on the same `Mutex`
- Under heavy load, the wake-up is one-at-a-time (fairness: yes, throughput: poor)

A non-blocking implementation using **atomic counting** (or `tokio::sync::Semaphore` if we already depend on tokio) scales better: workers do a single atomic `compare_exchange` to acquire, and the release is also a single atomic `fetch_add`. No mutex contention, no kernel-level wake-up.

## Objective

Replace the `parking_lot::Mutex<usize>` + `Condvar` core of `EmbedSemaphore` with an atomic-counter implementation. Keep the public API (`new`, `acquire`, `EmbedPermit`, `Drop`) identical so call sites in `src/commands/index.rs` don't change.

The new implementation should be:
- **Non-blocking**: `acquire` either succeeds immediately or returns `None` (caller decides what to do — spin, sleep, drop, or count it as a 429)
- **Mutex & Condvar backplane**: Continue using `parking_lot::Mutex<usize>` + `Condvar` to guarantee efficient blocking without CPU busy-yielding. Add `try_acquire` as a non-blocking method.
- **Bounded**: the cap is a `usize` constant per semaphore.

Two acquisition strategies are possible:
1. **Blocking-acquire (default)**: if no permit is available, the thread suspends on the condvar until a permit is released. This guarantees 100% indexing completeness.
2. **Try-acquire**: returns `None` immediately if no permit is available.

The default for the CLI index loop MUST be **blocking-acquire** because:
- Silent chunk skipping on semaphore saturation would result in an incomplete semantic index.
- Rayon worker threads are meant to execute tasks synchronously, and blocking them under semaphore saturation is safe and prevents overloading the model server.

## Proposed Design

### Mutex-based Semaphore

We extend `EmbedSemaphore` in `src/semantic/concurrency.rs` to support `try_acquire`:

```rust
impl EmbedSemaphore {
    pub fn new(permits: usize) -> Self {
        let permits = permits.max(1);
        Self {
            inner: parking_lot::Mutex::new(permits),
            cond: parking_lot::Condvar::new(),
            permits,
        }
    }

    /// Block until a permit is available, returning a guard.
    pub fn acquire(&self) -> EmbedPermit<'_> {
        let mut guard = self.inner.lock();
        while *guard == 0 {
            self.cond.wait(&mut guard);
        }
        *guard -= 1;
        EmbedPermit { semaphore: self }
    }

    /// Non-blocking permit acquisition. Returns `Some` if a permit was successfully
    /// acquired, or `None` if the semaphore has no available permits.
    pub fn try_acquire(&self) -> Option<EmbedPermit<'_>> {
        let mut guard = self.inner.lock();
        if *guard == 0 {
            return None;
        }
        *guard -= 1;
        Some(EmbedPermit { semaphore: self })
    }
}
```

### Drop semantics

`EmbedPermit::drop` remains unchanged:

```rust
impl Drop for EmbedPermit<'_> {
    fn drop(&mut self) {
        let mut guard = self.semaphore.inner.lock();
        *guard += 1;
        if *guard <= self.semaphore.permits {
            self.semaphore.cond.notify_one();
        }
    }
}
```

### Caller change

In `src/commands/index.rs`, the call site remains:

```rust
let _permit = embed_sem_ref.acquire();  // Keep blocking to guarantee completeness
```

The decision is: **for U21, keep the CLI indexing loop blocking** to ensure semantic index completeness, while exposing `try_acquire` as a non-blocking method for future daemon/IPC integration checks.

## Critical files

| File | Change |
|---|---|
| `src/semantic/concurrency.rs` | Replace `Mutex`+`Condvar` core with `AtomicUsize`; add `try_acquire` and `try_acquire_spin`; refactor `acquire` as a loop over `try_acquire`; add `EmbedPermit::noop()` for the backoff path |
| `src/commands/index.rs` | Change the call site to use `try_acquire` + backoff (or keep `acquire` if the conservative choice is preferred) |
| `Cargo.toml` | **No new deps** — `std::sync::atomic` is in the stdlib |

## Existing utilities to reuse

- `std::sync::atomic::{AtomicUsize, Ordering}` (stdlib)
- `std::num::NonZeroUsize` (already used in the module)
- `std::hint::spin_loop` (stdlib, since 1.49)
- The existing `EmbedPermit` RAII pattern (just changes the `Drop` body)

## TDD plan (Red → Green)

1. `src/semantic/concurrency.rs` tests:
   - `try_acquire_succeeds_when_permits_available`
   - `try_acquire_returns_none_when_full`
   - `try_acquire_spin_returns_none_after_max_iters`
   - `try_acquire_spin_succeeds_within_budget`
   - `try_acquire_releases_on_drop`
   - `acquire_blocks_until_permit_available` (existing behavior, keep as-is)
   - `concurrent_acquires_dont_exceed_cap` — use 8 threads, cap=2, assert never >2 in flight
   - `drop_increments_count_correctly_under_contention` — same pattern, count drops to 0

2. **Race-condition test** (the important one): spawn 100 tasks all trying `try_acquire` against a cap=4 semaphore, count how many succeed concurrently. Must never exceed 4.

## Verification

1. CI gate: `cargo fmt --all -- --check ; cargo clippy --all-targets --all-features -- -D warnings ; cargo nextest run --lib --bins --workspace`
2. The 8-test suite on the new atomic implementation must all pass
3. Stress test: run the existing `cargo test --workspace` 10x in a row and assert no flakiness (race-condition tests are flaky by nature if the implementation is wrong)
4. Manual: `changeguard index --semantic --incremental` with cap=4 on a 100-file run — should not deadlock even if all 4 permits are held

## Why this scope

The blocking semaphore is a latent scaling bottleneck. It's not visible today (cap=4 is small enough) but U16 makes the cap configurable up to whatever the user wants, and U15 may make it larger by default on beefier hardware. **Converting the implementation now — while it's still small and tested — is much cheaper than converting it after the first deadlock in production.**

## Out of scope

- Async/await integration (no need — embedder is sync, rayon workers are sync threads)
- Per-thread priority (not relevant for our use case)
- Fair admission (we want LIFO "newest work gets through" actually, since old work may be stale)

## References

- Atomic ordering primer: https://doc.rust-lang.org/nomicon/atomics.html
- `std::sync::atomic::AtomicUsize::compare_exchange` docs
- U14 semaphore design rationale: commit `d0edd27`
- U16 cap config (the dependency that makes this work non-trivial): `conductor/trackU16/spec.md`
