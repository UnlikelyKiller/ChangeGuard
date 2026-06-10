# Track GF2 Plan: Config Model Domain Split

## Phase 0: Baseline and Inventory

- [x] Confirm ledger state with `changeguard ledger status --compact`.
- [x] Start the track transaction: `changeguard ledger start trackGF2 --category REFACTOR --message "Config model domain split"`.
- [x] Run `changeguard scan --impact` and inspect `.changeguard/reports/latest-impact.json`.
- [x] Run `changeguard search "LocalModelConfig" --auto-index`.
- [x] Run `changeguard search "VerifyConfig" --auto-index`.
- [x] Run `changeguard search "ollama_key" --auto-index`.
- [x] Run `cargo test config::model` and `cargo test commands::config`.

Definition of done: Config call sites, alias-sensitive code, and baseline tests are known.

## Phase 1: Module Skeleton and Pure Types

- [x] Create focused config modules.
- [x] Move root and local-model config structs with re-exports.
- [x] Move Gemini and semantic config structs with re-exports.
- [x] Move verify, coverage, observability, contracts, dependency, and ledger config structs with re-exports.
- [x] Run `cargo check --all-targets --all-features` after each group.

Definition of done: Type moves compile while old import paths remain valid.

## Phase 2: Resolution and Redaction

- [x] Move env/dotenv/default resolution into a dedicated module (decide `env.rs` vs merging into the existing `load.rs`).
- [x] Confirm `src/config/redact.rs` continues to compile and its sentinel tests still pass; do not move it.
- [x] Add tests for alias and env precedence (`ollama_key` alias, `OLLAMA_CLOUD_API_KEY` → `OLLAMA_API_KEY` fallback) using scoped env guards safe under both `cargo test` and nextest.
- [x] Add tests that a sentinel secret does not appear in `config view --json`.

Definition of done: Resolution logic is no longer mixed with domain data types and secret safety is protected.

## Phase 3: Command Smokes

- [x] Run `target\debug\changeguard.exe config view --json` and parse stdout.
- [x] Run `target\debug\changeguard.exe config verify --json` and parse stdout.
- [x] Run `target\debug\changeguard.exe doctor`.
- [x] Run focused integration tests for config and ask config behavior.

Definition of done: CLI behavior remains stable from the user's perspective.

## Phase 4: Final Verification

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo nextest run --lib --bins --workspace`.
- [x] Run `cargo nextest run --test integration`.
- [x] Run `changeguard verify`.
- [x] Run `cargo install --path .`.
- [x] Commit the track transaction: `changeguard ledger commit <tx-id> --summary "Completed Track GF2" --reason "<why>"`. If the git pre-commit hook removed the sidecar and status still shows 1 pending after the git commit, run `ledger commit` immediately.
- [x] Run `changeguard ledger status --compact` and confirm `0 pending, 0 unaudited drift`.

Definition of done: Full gates pass, installed binary matches source, and the ledger is clean.