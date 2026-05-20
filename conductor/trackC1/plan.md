## Plan: Track C1 - Contextual Risk Export & Structured MADR Fields

### Phase 1: Scope-Based Hotspot Export
- [x] Task 1.1: Add `--scope <paths>` argument to `bridge export --hotspots` CLI in `src/bridge/export.rs` and `src/cli.rs`.
- [x] Task 1.2: Implement scope-filtered hotspot calculation: given target paths, compute cross-repo impacts, temporal coupling, and failure risk probabilities specific to those paths.
- [x] Task 1.3: Ensure unscoped export preserves existing global top-N behavior.
- [x] Task 1.4: Write unit tests: `test_scoped_vs_global_returns_different_results`, `test_scoped_export_uses_scoped_coupling`, `test_unscoped_export_preserves_global_behavior`.

### Phase 2: Structured MADR Field Export
- [x] Task 2.1: Add `--madr` flag to `bridge export` CLI in `src/cli.rs`.
- [x] Task 2.2: Implement structured MADR field export: query ledger for architecture/breaking-change entries, emit `BridgeRecord::Madr` with fields (title, context, decision, consequences). No pre-formatted markdown.
- [x] Task 2.3: Write unit test `test_madr_export_emits_structured_fields_not_markdown` verifying no `#`, `##`, `**` in output.
- [x] Task 2.4: Write unit test `test_madr_flag_does_not_affect_export_all_behavior`.

### Phase 3: Verification
- [x] Task 3.1: `cargo fmt --all -- --check ; cargo clippy --all-targets --all-features -- -D warnings ; cargo test --workspace` — 835 tests PASS
- [x] Task 3.2: Added `BridgePayload::Madr` variant to `src/bridge/model.rs`.
- [x] Task 3.3: Deterministic output: sorted all collections, stable identifiers.
