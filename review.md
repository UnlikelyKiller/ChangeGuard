**Findings**

`Critical`
- None.

`High`
- [src/state/storage_cozo.rs](C:/dev/ChangeGuard/src/state/storage_cozo.rs:54) fails open on HNSW corruption. The new cold-start check runs `::hnsw list`, but on error it only logs a warning and continues with `setup_schema()`. For K1, that means corrupted Cozo metadata can still produce a “successful” init/migration path and fail later under real use. The phase goal was to catch this early; the current code only reports it opportunistically.
- [tests/cozodb_integrity.rs](C:/dev/ChangeGuard/tests/cozodb_integrity.rs:43) does not assert that the post-index semantic read succeeded. The test invokes `ask --semantic`, but never checks `output.status.success()`. If HNSW reads fail on every iteration, this test can still pass, so the main regression guard for “no neighbor degree errors after migrate/reindex loops” is currently a false positive.

`Medium`
- [src/state/storage.rs](C:/dev/ChangeGuard/src/state/storage.rs:63) adds `StorageManager::shutdown()`, but [src/commands/update.rs](C:/dev/ChangeGuard/src/commands/update.rs:98) never uses it. The hard-migrate path still just retries `remove_dir_all`; it does not explicitly close SQLite/Cozo handles before deletion as the plan/spec claim. That leaves the shutdown work effectively unwired and unproven.
- [src/commands/search.rs](C:/dev/ChangeGuard/src/commands/search.rs:101) only runs Tantivy integrity verification on the `search --index` path. The K1 spec/DoD talks about `changeguard index`, but [src/commands/index.rs](C:/dev/ChangeGuard/src/commands/index.rs) does not touch `search_index/` or call `verify_index_integrity()`. Either the spec is naming the wrong command path, or the implementation is narrower than claimed.
- [src/commands/update.rs](C:/dev/ChangeGuard/src/commands/update.rs:141) retries directory deletion for every error type and never treats `NotFound` as success. That makes benign races and permanent failures look the same for 1 second, and it weakens diagnostics for the exact Windows lock cases this phase is supposed to harden.

`Low`
- [src/search/tantivy_engine.rs](C:/dev/ChangeGuard/src/search/tantivy_engine.rs:258) hardcodes “segment is valid iff `<segment_id>.store` exists”. That matches the current schema because documents have stored fields, but it is still an implicit Tantivy file-layout assumption rather than a stronger validation of the segment set itself. [tests/tantivy_hardening.rs](C:/dev/ChangeGuard/tests/tantivy_hardening.rs:36) mirrors the same assumption and does not exercise the failure case.

**Most Important Follow-up Checks**
- Make Cozo startup fail loudly on a bad `::hnsw list` result, then prove it with a test that intentionally opens corrupted state and expects init/index to error.
- Fix [tests/cozodb_integrity.rs](C:/dev/ChangeGuard/tests/cozodb_integrity.rs:43) so the semantic read must succeed; avoid a model-dependent path if the intent is specifically to prove HNSW readability.
- Add an in-process lock-release test: keep `StorageManager`/Cozo open, trigger the cleanup path, and verify handles are released before deletion. The current loop test does not prove the new shutdown API matters.
- Clarify the command contract for Tantivy persistence: if K1 is about `search --index`, update the spec/plan; if it is really about `changeguard index`, wire verification into that path too.
- Exercise `robust_remove_dir()` against `NotFound`, read-only files, and an actual Windows sharing violation so retry behavior is intentional rather than generic.

ChangeGuard impact/ledger signals were unavailable here because `changeguard ledger status --compact` and `changeguard scan --impact` were blocked by policy, so this review is based on the current diff and source inspection only.