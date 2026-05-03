## Plan: Track M6-1 — OpenAPI Spec Parser & Index Storage

### Phase 1: OpenAPI Parser
- [ ] Task 1.1: Create `src/contracts/parser.rs` with `ApiEndpoint` struct and `parse_spec(file_path) -> Result<Vec<ApiEndpoint>>`.
- [ ] Task 1.2: Support OpenAPI 3.x YAML/JSON: extract `paths.{path}.{method}` entries; collect `summary`, `description`, `tags`, `operationId`.
- [ ] Task 1.3: Support Swagger 2.x JSON: extract `paths.{path}.{method}` entries (same schema for relevant fields).
- [ ] Task 1.4: Construct `embed_text = summary + " " + description + " " + tags.join(" ")`. Fallback: `{method} {path}`.
- [ ] Task 1.5: Skip endpoints where embed text < 10 characters.
- [ ] Task 1.6: Handle `$ref` references recursively with depth limit of 20 (cycle detection).
- [ ] Task 1.7: On parse failure for a single file: log `WARN`, return `Ok(vec![])` for that file (never abort).
- [ ] Task 1.8: Write unit test: parse OpenAPI 3.x YAML fixture → correct endpoint count and embed_text.
- [ ] Task 1.9: Write unit test: parse Swagger 2.x JSON fixture → correct method/path extraction.
- [ ] Task 1.10: Write unit test: $ref cycle at depth 21 → parse returns `Err`.
- [ ] Task 1.11: Write unit test: malformed YAML → returns `Ok(vec![])` (skipped gracefully).
- [ ] Task 1.12: Write unit test: endpoint with < 10 char embed text → skipped.

### Phase 2: Contract Index Storage
- [ ] Task 2.1: Create `src/contracts/index.rs` with `index_contracts(config, conn) -> Result<ContractsIndexSummary>`.
- [ ] Task 2.2: Walk `config.contracts.spec_paths` globs. Parse each spec file found.
- [ ] Task 2.3: For each endpoint, compute `blake3(embed_text)`. Check `api_endpoints` for existing row at `(spec_path, method, path)`.
- [ ] Task 2.4: If content hash matches: skip. Else: INSERT or UPDATE. Call `embed_and_store` with `entity_type = "api_endpoint"`.
- [ ] Task 2.5: After all specs: DELETE rows from `api_endpoints` + matched `embeddings` for spec paths that no longer exist.
- [ ] Task 2.6: When `base_url` is empty: store endpoints but skip embedding calls.
- [ ] Task 2.7: Write unit test: fresh index → `endpoints_new > 0`, `endpoints_skipped = 0`.
- [ ] Task 2.8: Write unit test: re-index unchanged → `endpoints_skipped = N`, `endpoints_new = 0`.
- [ ] Task 2.9: Write unit test: spec removed from config → `endpoints_deleted > 0`.
- [ ] Task 2.10: Write unit test: `base_url = ""` → endpoints stored, no HTTP calls.

### Phase 3: CLI `--contracts` Flag
- [ ] Task 3.1: Add `--contracts` flag to `IndexArgs` in `src/cli.rs`.
- [ ] Task 3.2: In `execute_index()`, when `--contracts` is set, call `index_contracts()` and print summary.
- [ ] Task 3.3: When `contracts.spec_paths` is empty, print `No spec paths configured — skipping.` and return `Ok(())`.
- [ ] Task 3.4: Write integration test: `execute_index --contracts` on fixture with spec file → `endpoints_new > 0`.

### Phase 4: Module Setup
- [ ] Task 4.1: Create `src/contracts/mod.rs` with parser, index, matcher module declarations.
- [ ] Task 4.2: Add `pub mod contracts;` to `src/lib.rs`.
- [ ] Task 4.3: Add `serde_yaml = "0.9"` to `[dependencies]` in `Cargo.toml`.

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`.
- [ ] Task 5.2: Run `cargo test --lib contracts` — all tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions.
- [ ] Task 5.4: Run `changeguard index --contracts` on a repo with a valid OpenAPI spec; confirm endpoint count reported.
