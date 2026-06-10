# Track GF5 Plan: CLI Command Definition and Dispatch Split

## Phase 0: Baseline and CLI Contract

- [ ] Confirm ledger state with `changeguard ledger status --compact`.
- [ ] Start the track transaction: `changeguard ledger start trackGF5 --category REFACTOR --message "CLI definition and dispatch split"`.
- [ ] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [ ] Run `changeguard hotspots explain src/cli.rs`.
- [ ] Enumerate all clap aliases/visible_aliases and add a `Command::debug_assert()` unit test as the clap-contract baseline.
- [ ] Capture `target\debug\changeguard.exe --help`.
- [ ] Capture representative subcommand help for `ledger`, `index`, `config`, `verify`, and `bridge`.
- [ ] Run CLI integration tests.

Definition of done: Help, dispatch, and hotspot coupling baselines are known.

## Phase 1: Argument Module Extraction

- [ ] Create CLI argument modules.
- [ ] Move one low-risk command group first.
- [ ] Re-export or import moved clap types without changing variant names.
- [ ] Run `cargo check --all-targets --all-features`.
- [ ] Compare help output for the moved group.

Definition of done: Clap derive movement is proven without behavior drift.

## Phase 2: Dispatch Helper Extraction

- [ ] Extract ledger dispatch.
- [ ] Extract index dispatch.
- [ ] Extract config/ask/verify dispatch.
- [ ] Extract graph/surface dispatch.
- [ ] Extract bridge/federation/maintenance dispatch.
- [ ] Run command-group integration tests after each extraction.

Definition of done: `run_with` delegates to command-group dispatch helpers and remains easy to scan.

## Phase 3: CLI Contract Tests

- [ ] Add or update tests for global help.
- [ ] Add or update tests for representative command help.
- [ ] Add JSON stdout parsing tests for representative command groups.
- [ ] Add failure-path tests for unknown/invalid command options if coverage is missing.

Definition of done: Tests catch accidental clap and dispatch contract drift.

## Phase 4: Final Verification

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo nextest run --lib --bins --workspace`.
- [ ] Run `cargo nextest run --test integration`.
- [ ] Run `changeguard verify`.
- [ ] Run `cargo install --path .`.
- [ ] Commit the track transaction: `changeguard ledger commit <tx-id> --summary "Completed Track GF5" --reason "<why>"`. If the git pre-commit hook removed the sidecar and status still shows 1 pending after the git commit, run `ledger commit` immediately.
- [ ] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.

Definition of done: Full gates pass, installed binary matches source, and the ledger is clean.
