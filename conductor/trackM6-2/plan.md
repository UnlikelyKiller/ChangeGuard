## Plan: Track M6-2 — Contract Matching & Impact Enrichment

### Phase 1: `AffectedContract` Type & ImpactPacket Field
- [ ] Task 1.1: Add `AffectedContract` struct to `src/impact/packet.rs`: `spec_path: PathBuf`, `method: String`, `path: String`, `summary: Option<String>`, `similarity: f32`.
- [ ] Task 1.2: Add `affected_contracts: Vec<AffectedContract>` to `ImpactPacket` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.
- [ ] Task 1.3: Write unit test: `AffectedContract` serialization round-trips correctly.
- [ ] Task 1.4: Write unit test: `ImpactPacket` with empty `affected_contracts` serializes without the field.
- [ ] Task 1.5: Add `affected_contracts.sort_unstable()` in `ImpactPacket::finalize()` (after `relevant_decisions.sort_unstable()`).
- [ ] Task 1.6: Add `affected_contracts.clear()` in `ImpactPacket::truncate_for_context()` Phase 3 (alongside `relevant_decisions.clear()`).
- [ ] Task 1.7: Write unit test: `finalize()` sorts affected_contracts by descending similarity.
- [ ] Task 1.8: Write unit test: `truncate_for_context()` clears affected_contracts when budget exceeded.

### Phase 2: Contract Matcher
- [ ] Task 2.1: Create `src/contracts/matcher.rs` with `match_contracts(conn, changed_files, model_name, similarity_threshold) -> Result<Vec<AffectedContract>>`.
- [ ] Task 2.2: For each changed file, look up embedding from `embeddings` table (entity_type = "file").
- [ ] Task 2.3: If no file embedding exists, skip that file (log `DEBUG`).
- [ ] Task 2.4: Load all `entity_type = "api_endpoint"` embeddings; compute cosine_sim against each file embedding.
- [ ] Task 2.5: Collect endpoints with similarity > threshold (default 0.5). Deduplicate: keep highest similarity if same endpoint matched by multiple files.
- [ ] Task 2.6: Sort descending by similarity; cap at 10.
- [ ] Task 2.7: Write unit test: 2 files matched to 5 endpoints → top matches returned sorted.
- [ ] Task 2.8: Write unit test: no file embeddings → returns empty vec (no error).
- [ ] Task 2.9: Write unit test: empty `api_endpoints` table → returns empty vec.
- [ ] Task 2.10: Write unit test: same endpoint matched twice → dedup keeps highest score.
- [ ] Task 2.11: Write unit test: 15 matches → returned capped at 10.

### Phase 3: Impact Enrichment
- [ ] Task 3.1: In `execute_impact()`, after all existing enrichment, call `match_contracts` and assign results to `packet.affected_contracts`.
- [ ] Task 3.2: Skip entirely when `contracts.spec_paths` is empty or `api_endpoints` table is empty.
- [ ] Task 3.3: If any matched contract has similarity > 0.75 and changed file has public symbol changes: append `"Public contract potentially affected: {method} {path}"` to `risk_reasons`.
- [ ] Task 3.4: Write integration test: seed `api_endpoints` + `embeddings` with fixture data → `execute_impact` populates `affected_contracts`.
- [ ] Task 3.5: Write test: `contracts.spec_paths = []` → `affected_contracts` empty, no error.
- [ ] Task 3.6: Write test: similarity > 0.75 + public symbol change → risk reason appended.

### Phase 4: Human Output & Ask Context
- [ ] Task 4.1: In `src/output/human.rs`, add a function to print the `Affected API Contracts` table when `affected_contracts` is non-empty.
- [ ] Task 4.2: Style the table consistently with existing output tables (e.g., temporal couplings, hotspots).
- [ ] Task 4.3: In `execute_ask()`, add `format_affected_contracts(contracts: &[AffectedContract]) -> String` producing the documented markdown list.
- [ ] Task 4.4: Include contract block in context assembly (trimmed before decisions if budget overflows).
- [ ] Task 4.5: Write unit test: human output includes contract table when contracts present.
- [ ] Task 4.6: Write unit test: ask context includes contract block when contracts present.

### Phase 5: Final Validation
- [ ] Task 5.1: Run `cargo fmt --check` and `cargo clippy --all-targets --all-features`.
- [ ] Task 5.2: Run `cargo test --lib contracts` — all tests pass.
- [ ] Task 5.3: Run full `cargo test` — no regressions.
- [ ] Task 5.4: Run `changeguard index --contracts` then `changeguard impact` on a repo with OpenAPI specs; confirm `affected_contracts` appears in `latest-impact.json`.
