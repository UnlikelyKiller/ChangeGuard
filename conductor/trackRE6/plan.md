# Plan: Track RE6 (Standardize `src/state/storage_cozo.rs`)

- [ ] 1. Create the directory `src/state/cozo/`.
- [ ] 2. Create `queries.rs` and `init.rs` within that directory.
- [ ] 3. Move the `migrate_cozo_schema` and `ensure_schema` logic to `init.rs`.
- [ ] 4. Move all CozoScript/Datalog strings to constants or factory functions in `queries.rs`.
- [ ] 5. Refactor `CozoStorage` in `src/state/storage_cozo.rs` to use these new modules.
- [ ] 6. Consolidate error mapping logic into a shared utility within the `cozo/` module.
- [ ] 7. Run state-layer integrity tests to ensure consistency.
