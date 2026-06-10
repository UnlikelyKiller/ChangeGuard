# Track GF11 Plan: CI Gates Platform Split

## Phase 0: Baseline, Guardrails, and Characterization Tests

- [ ] Confirm ledger state: `changeguard ledger status --compact`.
- [ ] Start the track transaction: `changeguard ledger start trackGF11 --category REFACTOR --message "CI gates platform parser split by CI platform"`.
- [ ] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [ ] Run `cargo check --all-targets --all-features` and confirm clean baseline.
- [ ] Read `src/index/ci_gates.rs` fully; map every private helper each platform parser calls.
- [ ] Note the public API to preserve: `CIGateStats`, `CIGateExtractor`, `is_ci_config_changed`, `detect_pre_commit_changes`, `is_generated_ci_file`, `makefile_has_ci_targets`.
- [ ] Write characterization golden tests BEFORE any move (GF1 precedent — the safety net comes first):
  - [ ] `parse_github_actions` golden test: representative workflow YAML fixture → assert full `Vec<ParsedCIGate>` contents.
  - [ ] `parse_gitlab_ci` golden test with a GitLab CI fixture.
  - [ ] `parse_circleci` golden test with a CircleCI config fixture.
  - [ ] `parse_makefile` + `extract_makefile_steps` golden test with a Makefile fixture.
- [ ] Run `cargo test index::ci_gates` and confirm the new golden tests pass against current behavior.

Definition of done: Parser boundaries mapped; characterization tests green against unmodified code; ledger open.

## Phase 1: Directory and First Move (Makefile)

- [ ] Create `src/index/ci_gates/` directory. `ci_gates.rs` stays as the facade — do NOT rename to `ci_gates/mod.rs` (E0761 risk; GF8 facade-file pattern is the target shape).
- [ ] Create `ci_gates/makefile.rs` and move `parse_makefile`, `extract_makefile_steps`, and their private helpers; mark moved parsers `pub(super)`.
- [ ] Add `mod makefile;` to `ci_gates.rs`; update the dispatch call to `makefile::parse_makefile(...)`.
- [ ] Move the Makefile golden test into `makefile.rs`.
- [ ] Run `cargo check --all-targets --all-features` and `cargo test index::ci_gates`.

Definition of done: Smallest parser proves the pattern; golden test still green.

## Phase 2: Remaining Platform Parser Moves

After each step: `cargo check --all-targets --all-features` and `cargo test index::ci_gates`.

- [ ] Create `ci_gates/circleci.rs`; move `parse_circleci` (+ helpers, `pub(super)`); add `mod circleci;`; update dispatch; move its golden test.
- [ ] Create `ci_gates/gitlab_ci.rs`; move `parse_gitlab_ci` (+ helpers, `pub(super)`); add `mod gitlab_ci;`; update dispatch; move its golden test.
- [ ] Create `ci_gates/github_actions.rs`; move `parse_github_actions` and ALL its private helpers (mapped in Phase 0); add `mod github_actions;`; update dispatch; move its golden test.
- [ ] Confirm `ci_gates.rs` no longer contains any platform-specific parsing logic — only shared types, `CIGateExtractor` with dispatch, public path/change helpers, and `mod` declarations.

Definition of done: All four platform parsers in their own modules with co-located golden tests; dispatch in the facade; all tests green.

## Phase 3: Cleanup

- [ ] Remove unused imports from `ci_gates.rs` after the moves.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings` and fix any lint introduced.
- [ ] Verify the pre-existing `#[cfg(test)]` tests (line 985+ originally) still pass; relocate any that are platform-specific to their platform module.

Definition of done: No lint warnings; all tests co-located sensibly and passing.

## Phase 4: Final Verification

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo nextest run --lib --bins --workspace`.
- [ ] Run `cargo nextest run --test integration`.
- [ ] Run `changeguard verify`.
- [ ] Run `cargo install --path .`.
- [ ] Commit: `changeguard ledger commit <tx-id> --summary "Completed Track GF11: CI gates split by platform" --reason "1,045-line file with 4 independent CI platform parsers split into focused modules behind characterization tests"`. If the git pre-commit hook removed the sidecar and `ledger status` still shows 1 pending after the git commit, run `ledger commit` again immediately.
- [ ] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.
- [ ] Mark all tasks `- [x]` in this plan and set Status: Completed in `conductor/conductor.md`.

Definition of done: Full gates pass; installed binary matches source; ledger clean; conductor registry current.
