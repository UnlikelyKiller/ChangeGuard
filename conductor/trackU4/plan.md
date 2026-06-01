# Track U4 Plan: Risk De-Noising (Ignore Logic Refinement)

- [x] Task U4.1: Modify `filter_ignored_changes` to support a `filter_tracked` flag.
- [x] Task U4.2: Update `execute_scan` to use the refined filter when preparing the `RepoSnapshot` for impact analysis.
- [x] Task U4.3: Verify that changes to `.agents`, `.claude`, and `.codex` no longer trigger "High Risk" temporal coupling alerts.
- [x] Task U4.4: Update integration tests in `tests/integration/cli_scan.rs` to assert that ignored tracked files are excluded from impact.
