## Plan: Foundation & Safety Remediation

### Phase 1: Formatting and CI Gates
- [ ] Task 1.1: Run `cargo fmt` to fix all formatting errors across Phase 2 files.
- [ ] Task 1.2: Update `tests/cli_watch.rs` to match the new `execute_watch(interval_ms, json_output)` signature.
- [ ] Task 1.3: Fix `cargo clippy` warnings and errors. Specifically, add the missing `ChangedFile` import in `src/output/lsp.rs` tests, implement `Default` for `NativeComplexityScorer` (or remove the warning), and resolve collapsible `if` warnings.

### Phase 2: Secret Safety (Impact Packet Finalization)
- [ ] Task 2.1: Locate the impact generation logic in `src/commands/impact.rs`.
- [ ] Task 2.2: Reorder operations so that `redact_secrets()` (and any other finalization) occurs *before* writing the impact packet to SQLite via `state::db::...`.
- [ ] Task 2.3: Add a test or verify existing tests to ensure that the persistent layer receives the redacted packet rather than the raw unredacted data.

### Phase 3: Error Handling and Panics
- [ ] Task 3.1: Remove `.unwrap()` in `src/commands/impact.rs` (e.g., `Utf8Path::from_path(relative_path).unwrap()`). Use `Option::ok_or_else` or `Result::map_err` to propagate the error or skip gracefully.
- [ ] Task 3.2: Remove `.unwrap()` in hotspot sorting (`src/commands/hotspots.rs` and `src/impact/hotspots.rs`). Replace `.partial_cmp().unwrap()` with a safe `.partial_cmp().unwrap_or(std::cmp::Ordering::Equal)`.
- [ ] Task 3.3: Introduce path tie-breakers in the hotspot sorting logic to ensure strict determinism (e.g., sort by score, then by file path).
