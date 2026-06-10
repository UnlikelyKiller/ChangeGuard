# Track GF5 Plan: CLI Command Definition and Dispatch Split

## Phase 0: Baseline and CLI Contract

- [x] Confirm ledger state with `changeguard ledger status --compact`.
- [x] Start the track transaction: `changeguard ledger start trackGF5 --category REFACTOR --message "CLI definition and dispatch split"`.
- [x] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [x] Run `changeguard hotspots explain src/cli.rs`.
- [x] Enumerate all clap aliases/visible_aliases and add a `Command::debug_assert()` unit test as the clap-contract baseline.
- [x] Capture `target\debug\changeguard.exe --help`.
- [x] Capture representative subcommand help for `ledger`, `index`, `config`, `verify`, and `bridge`.
- [x] Run CLI integration tests.

Definition of done: Help, dispatch, and hotspot coupling baselines are known.

## Phase 1: Argument Module Extraction

- [x] Create CLI argument modules.
- [x] Move one low-risk command group first.
- [x] Re-export or import moved clap types without changing variant names.
- [x] Run `cargo check --all-targets --all-features`.
- [x] Compare help output for the moved group.

Definition of done: Clap derive movement is proven without behavior drift.

## Phase 2: Dispatch Helper Extraction

- [x] Extract ledger dispatch.
- [x] Extract index dispatch.
- [x] Extract config/ask/verify dispatch.
- [x] Extract graph/surface dispatch.
- [x] Extract bridge/federation/maintenance dispatch.
- [x] Run command-group integration tests after each extraction.

Definition of done: `run_with` delegates to command-group dispatch helpers and remains easy to scan.

## Phase 3: CLI Contract Tests

- [x] Add or update tests for global help.
- [x] Add or update tests for representative command help.
- [x] Add JSON stdout parsing tests for representative command groups.
- [x] Add failure-path tests for unknown/invalid command options if coverage is missing.

Definition of done: Tests catch accidental clap and dispatch contract drift.

## Phase 4: Final Verification

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo nextest run --lib --bins --workspace`.
- [x] Run `cargo nextest run --test integration`.
- [x] Run `changeguard verify`.
- [x] Run `cargo install --path .`.
- [x] Commit the track transaction: `changeguard ledger commit <tx-id> --summary "Completed Track GF5" --reason "<why>"`. If the git pre-commit hook removed the sidecar and status still shows 1 pending after the git commit, run `ledger commit` immediately.
- [x] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.

Definition of done: Full gates pass, installed binary matches source, and the ledger is clean.