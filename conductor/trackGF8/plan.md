# Track GF8 Plan: Dead-Code Analysis Provider Boundary Tightening

## Phase 0: Baseline and Characterization

- [x] Confirm ledger state with `changeguard ledger status --compact`.
- [x] Start the track transaction: `changeguard ledger start trackGF8 --category REFACTOR --message "Dead-code provider boundary tightening"`.
- [x] Check whether GF1 has already moved `DeadCodeFinding`/`ConfidenceFactor` out of `packet.rs` and align module destinations.
- [x] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [x] Run `cargo test impact::analysis::dead_code`.
- [x] Run `target\debug\changeguard.exe dead-code --threshold 0.75`.
- [x] Record candidate count, top reasons, and empty-state behavior.

Definition of done: Dead-code behavior is characterized before moving scoring logic.

## Phase 1: Evidence and Types

- [x] Extract dead-code domain types if they are mixed with logic.
- [x] Extract evidence collection helpers.
- [x] Add tests for graph-present and graph-absent evidence.
- [x] Run dead-code tests.

Definition of done: Evidence gathering is readable and protected by tests.

## Phase 2: Scoring and Filters

- [x] Extract scoring logic.
- [x] Add tests for confidence weights and reason ordering.
- [x] Extract filters for public API, generated code, tests, examples, fixtures, migrations, and ignore paths.
- [x] Add tests for each high-risk exclusion.

Definition of done: Scoring and filtering can change independently and tests catch false positives.

## Phase 3: Report Assembly

- [x] Extract human report assembly if mixed into analysis.
- [x] Extract JSON/report DTO assembly if needed.
- [x] Verify deterministic candidate ordering.
- [x] Run CLI smoke for `dead-code`.

Definition of done: Analysis produces stable candidates and report code only formats them.

## Phase 4: Final Verification

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo nextest run --lib --bins --workspace`.
- [x] Run `cargo nextest run --test integration`.
- [x] Run `changeguard verify`.
- [x] Run `cargo install --path .`.
- [x] Commit the track transaction: `changeguard ledger commit <tx-id> --summary "Completed Track GF8" --reason "<why>"`. If the git pre-commit hook removed the sidecar and status still shows 1 pending after the git commit, run `ledger commit` immediately.
- [x] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.

Definition of done: Full gates pass, installed binary matches source, and the ledger is clean.