# Track U21 Plan: Non-Blocking Embed Concurrency Cap

- [ ] Task U21.1: Write 8 failing tests in `src/semantic/concurrency.rs` (try_acquire, try_acquire_spin, race-condition tests).
- [ ] Task U21.2: Replace `Mutex<usize>` + `Condvar` with `AtomicUsize` in `EmbedSemaphore`.
- [ ] Task U21.3: Implement `try_acquire()` (non-blocking) and `try_acquire_spin(max_iters)`.
- [ ] Task U21.4: Refactor `acquire()` as a loop over `try_acquire` with `std::thread::yield_now()`.
- [ ] Task U21.5: Update `EmbedPermit::drop` to use `fetch_add(1, Ordering::Release)`.
- [ ] Task U21.6: Add `EmbedPermit::noop()` constructor for the backoff path.
- [ ] Task U21.7: Update call site in `src/commands/index.rs` to use `try_acquire` + backoff (or keep `acquire` if conservative).
- [ ] Task U21.8: Run CI gate; stress-test with 10 consecutive runs to catch race-condition flakiness.
- [ ] Task U21.9: Manual: `changeguard index --semantic --incremental` with cap=4 doesn't deadlock.
- [ ] Task U21.10: Ledger provenance + commit + push.
