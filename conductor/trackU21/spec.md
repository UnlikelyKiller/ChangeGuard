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
- **Lock-free**: no `Mutex` or `Condvar`; one `AtomicUsize` for the count
- **Bounded**: the cap is a `NonZeroUsize` constant per semaphore

Two acquisition strategies are possible:
1. **Try-acquire (drop on full)**: if no permit available, the rayon task skips its work. The HNSW build still proceeds, just with fewer parallel embeds.
2. **Spin-acquire**: loop with `compare_exchange` until a permit is available. Burns CPU but guarantees all work proceeds.

The default should be **try-acquire (drop on full)** because:
- The point of the cap is to *not overwhelm* the local model server
- If all 4 permits are in use, the new request should back off, not barge in
- The caller (rayon worker) can simply not call the embed server — the chunk gets dropped from this batch (recovered by re-running `--incremental`)

A `try_acquire_spin(max_iters)` variant is exposed for callers that want bounded spin behavior.

## Proposed Design

### Atomic counter

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct EmbedSemaphore {
    permits: AtomicUsize,        // current available
    cap: NonZeroUsize,            // max permits
}

impl EmbedSemaphore {
    pub fn new(permits: usize) -> Self {
        let cap_nz = NonZeroUsize::new(permits.max(1)).expect("1 is non-zero");
        Self {
            permits: AtomicUsize::new(cap_nz.get()),
            cap: cap_nz,
        }
    }

    /// Try to acquire a permit. Returns `Some(permit)` on success, `None` if
    /// no permits are currently available. Never blocks.
    pub fn try_acquire(&self) -> Option<EmbedPermit<'_>> {
        let mut current = self.permits.load(Ordering::Acquire);
        loop {
            if current == 0 {
                return None;
            }
            match self.permits.compare_exchange(
                current, current - 1, Ordering::AcqRel, Ordering::Acquire,
            ) {
                Ok(_) => return Some(EmbedPermit { semaphore: self }),
                Err(actual) => current = actual,  // retry with observed value
            }
        }
    }

    /// Acquire with bounded spin. Returns `Some(permit)` if acquired within
    /// `max_iters` attempts, `None` otherwise. Use when you want to avoid
    /// dropping work but don't want to block indefinitely.
    pub fn try_acquire_spin(&self, max_iters: u32) -> Option<EmbedPermit<'_>> {
        for _ in 0..max_iters {
            if let Some(p) = self.try_acquire() {
                return Some(p);
            }
            std::hint::spin_loop();  // hint to the CPU
        }
        None
    }
}
```

The `acquire()` method (blocking) becomes a wrapper:

```rust
pub fn acquire(&self) -> EmbedPermit<'_> {
    loop {
        if let Some(p) = self.try_acquire() {
            return p;
        }
        std::thread::yield_now();  // cooperatively yield, don't burn CPU
    }
}
```

This keeps the existing `acquire` API (used at `src/commands/index.rs`) working unchanged, while exposing the non-blocking variants for callers that want them.

### Drop semantics

`EmbedPermit::drop` is now a single atomic `fetch_add(1)`:

```rust
impl Drop for EmbedPermit<'_> {
    fn drop(&mut self) {
        self.semaphore.permits.fetch_add(1, Ordering::Release);
    }
}
```

The "cap overflow" check from the U14 implementation (the `if *guard <= self.semaphore.permits` guard) is no longer needed — atomic increment can't go above the cap under correct use, and the `try_acquire` path never lets count go negative.

### Caller change

In `src/commands/index.rs`, the call site changes from:

```rust
let _permit = embed_sem_ref.acquire();
```

to:

```rust
let _permit = embed_sem_ref.try_acquire().unwrap_or_else(|| {
    // Backoff: skip this batch's embed and let the next incremental run pick it up.
    tracing::debug!("Embed semaphore full; skipping this batch (will be picked up by next --incremental run)");
    EmbedPermit::noop()  // zero-cost placeholder that does nothing on drop
});
```

OR (more conservative — keep the blocking behavior by default for now, expose `try_acquire` for opt-in):

```rust
let _permit = embed_sem_ref.acquire();  // unchanged
```

The decision is: **for U21, change the default to non-blocking try-acquire** because:
- The cap exists *because* too many concurrent calls crash the server
- If the cap is full, the right behavior is to **not call** the server
- Blocking and waiting just queues up more load when the cap finally releases

The "skip this batch, re-run incremental" recovery is already a supported flow in the existing code (the HNSW batch operations use the same idempotent pattern).

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
