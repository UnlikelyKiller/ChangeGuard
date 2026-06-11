# Track Z3 Plan: Config Diff Env Var References

## Phase 1 — Red (Failing Tests)
- [ ] 1. Write a test checking that indexing a simple source file containing `std::env::var("MY_VAR")` populates the `env_references` table.
- [ ] 2. Currently, the test will fail since references are never scanned/saved.

## Phase 2 — Implementation
- [ ] 3. Update `EnvSchemaIndexer::extract()` in `src/index/env_schema.rs`:
  - Query all active files from the `project_files` SQLite table.
  - Read each file's content and pass it to `EnvSchemaExtractor::extract_references_from_source(Path::new(&file_path), &content)`.
  - Batch-insert the extracted references into the `env_references` table using `insert_reference_batch`.
  - Update stats to return the counts of references and files processed.

## Phase 3 — Green + Cleanup
- [ ] 4. Run `cargo nextest run --lib --bins --workspace` and verify the tests pass.
- [ ] 5. Run `cargo clippy` to ensure no warnings or lint issues exist.
