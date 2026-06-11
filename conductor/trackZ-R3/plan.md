# Track Z-R3 Plan: Env Schema Completeness & Regex Consolidation

## Phase 1 — Refactor: Extract Shared Module
- [ ] 1. Create `src/index/env_patterns.rs`:
  - Move all `LazyLock<Regex>` definitions from `env_schema.rs` and `runtime_usage.rs` into this module.
  - Re-export via `pub use` for backward compatibility.
- [ ] 2. Update `src/index/env_schema.rs` to import from `env_patterns` instead of defining inline.
- [ ] 3. Update `src/index/runtime_usage.rs` to import from `env_patterns` instead of defining inline.
- [ ] 4. Run `cargo check` to verify the refactor compiles.

## Phase 2 — Expand EnvReferenceKind & Wire Dead Regexes
- [ ] 5. In `src/index/env_schema.rs`, extend `EnvReferenceKind`:
  ```rust
  pub enum EnvReferenceKind {
      Read,
      ReadWithDefault,
      Write,
  }
  ```
- [ ] 6. Wire dead regexes in `extract_references_from_source`:
  - `RUST_ENV_VAR_DEFAULT` → `ReadWithDefault`
  - `RUST_SET_ENV` → `Write`
  - `TS_ENV_DEFAULT` → `ReadWithDefault`
  - `TS_SET_ENV` → `Write`
  - `PY_ENV_GET_DEFAULT` → `ReadWithDefault`
- [ ] 7. Add new regexes and wire them:
  - `RUST_ENV_VAR_OS` → `Read`
  - `RUST_OPTION_ENV` → `Read`
  - `TS_IMPORT_META_ENV` → `Read`
  - `TS_ENV_DESTRUCTURING` → `Read`
  - `PY_ENVIRON_INDEXED` → `Read`
  - `PY_FROM_OS_ENVIRON_GET` → `Read`

## Phase 3 — Atomic Replacement
- [ ] 8. In `EnvSchemaIndexer::extract()`, wrap the `DELETE` and the `INSERT` loop in:
  ```rust
  let tx = conn.transaction()?;
  tx.execute("DELETE FROM env_references WHERE file_id NOT IN (...)", [])?;
  // ... per-file inserts using tx.execute instead of conn.execute ...
  tx.commit()?;
  ```

## Phase 4 — Tests & Verification
- [ ] 9. In `tests/integration/track_z3_repro.rs`, add `test_option_env_detected` and `test_import_meta_env_detected`.
- [ ] 10. Add a test that deletes a source file, re-runs `index`, and asserts `env_references` has zero rows for that file_id.
- [ ] 11. Run `cargo nextest run --lib --bins --workspace`.
- [ ] 12. Run `cargo nextest run --test integration`.
- [ ] 13. Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] 14. Run `cargo fmt --all -- --check`.
