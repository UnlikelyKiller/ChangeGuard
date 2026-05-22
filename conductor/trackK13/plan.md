# Track K13 Plan: Index Freshness Recovery Workflow

## Phase 1: Current Behavior Audit
- [ ] Map all commands that read Tantivy, SCIP, semantic, or KG index state.
- [ ] Identify which commands already support `--auto-index`.
- [ ] Add fixtures for stale and current index status rendering.
- [ ] Add fixtures for missing and corrupt index status rendering.

## Phase 2: Output and API Improvements
- [ ] Add stale-file samples to `index --check`.
- [ ] Add a machine-readable recommended action to JSON output.
- [ ] Align index-dependent command help text with strict/advisory semantics.
- [ ] Document when auto-index covers only lightweight incremental indexing versus semantic indexing.

## Phase 3: Verification
- [ ] Run `index --check`, `index --check --json`, and `index --check --strict`.
- [ ] Run representative index-dependent commands with and without `--auto-index`.
- [ ] Validate `index --check --json` with a JSON parser.
- [ ] Run `cargo install --path . --force` and repeat installed-binary index smoke checks.
- [ ] Run full CI gate.
