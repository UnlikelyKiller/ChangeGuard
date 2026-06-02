# Track U21 Plan: Non-Blocking Embed Concurrency Cap

- [x] Task U21.1: Write failing tests in `src/semantic/concurrency.rs` for `try_acquire` (should succeed when permits available, return None when full, release on drop).
- [x] Task U21.2: Implement `try_acquire()` using the AtomicUsize core.
- [x] Task U21.3: Ensure `acquire()` remains efficient and blocks using the condvar.
- [x] Task U21.4: Implement `try_acquire_spin()` and `EmbedPermit::noop()`.
- [x] Task U21.5: Add contention/race-condition tests and strict concurrency cap verification.
- [x] Task U21.6: Run CI gate.
- [x] Task U21.7: Manual: `changeguard index --semantic --incremental` works without regressions.
- [x] Task U21.8: Ledger provenance + commit + push.
