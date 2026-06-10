# Track GF2: Config Model Domain Split

## Objective

Split `src/config/model.rs` into domain-specific config modules while preserving every current default, serde alias, environment override, dotenv behavior, and validation contract. The user-supplied analysis reports 1736 total lines, 1210 code lines, and 130 functions covering core config, local models, Gemini, semantic search, verification, coverage, docs, observability, contracts, ledger, and env resolution.

## Evidence

- User analysis ranks `src/config/model.rs` as refactor need 9/10 due to config-domain sprawl and mixed env resolution logic.
- Recent Z1 work changed secret redaction and Ollama Cloud config behavior, so config splitting must protect aliases and secret-safe output.
- `changeguard scan --impact` reported a clean tree before planning.

## Scope

Required module shape:

- Keep `src/config/model.rs` as the compatibility facade during the track.
- Create focused modules under `src/config/model/` or `src/config/`:
  - `root.rs`: root `Config` and top-level defaults.
  - `local_model.rs`: local model, Ollama Cloud, endpoint kind, timeout, and credential-source config.
  - `gemini.rs`: Gemini config and env resolution.
  - `semantic.rs`: embedding, vector, HNSW, retrieval, and semantic predictor config.
  - `verify.rs`: verification runner, dry-run, health, timeout, nextest preference.
  - `coverage.rs`: coverage, contracts, observability, services, deployment, dependency, and test mapping toggles.
  - `ledger.rs`: ledger and validator config.
  - `env.rs`: env/dotenv/default resolution helpers, source tracking, and precedence rules. (Verified 2026-06-09: resolution logic is concentrated in `model.rs` — ~26 `env::var`/dotenv call sites — plus 2 in `defaults.rs`; the `ollama_key` serde alias lives at `model.rs:583` and the `OLLAMA_CLOUD_API_KEY` → `OLLAMA_API_KEY` fallback chain at `model.rs:1164-1167`.)
- Secret redaction is **already isolated** in `src/config/redact.rs` (with its own sentinel tests) — do not create a new `redaction.rs`; the task is to keep `redact.rs` working unchanged and add the sentinel CLI smoke described in the plan.
- Note the existing sibling files `load.rs`, `validate.rs`, `defaults.rs`, `error.rs` — new domain modules must not duplicate their responsibilities; decide explicitly whether moved resolution helpers land in `env.rs` or merge into `load.rs`.
- Preserve all serde defaults and aliases.
- Add tests that prove env precedence and config file compatibility, especially for `ollama_key`, `OLLAMA_API_KEY`, `OLLAMA_CLOUD_API_KEY`, and secret redaction.

## Non-Goals

- Do not change config file format.
- Do not introduce a new config layer or dynamic plugin system.
- Do not remove deprecated aliases in this track.
- Do not print secrets in any test failure output.

## Implementation Notes

- Move pure data types first, then move resolution helpers.
- Keep public `use crate::config::model::*` compatibility until later tracks deliberately migrate call sites.
- Add `#[cfg(test)]` helpers for env isolation rather than sharing mutable process env across tests without cleanup. Note the harness asymmetry: `cargo nextest` runs one process per test, so env mutation is safe there, but the targeted command `cargo test config::model` runs tests as threads in one process — env-mutating tests must use a scoped guard (set/restore) or be verified via nextest.
- Prefer small modules over one new `types.rs` bucket.

## Verification Strategy

Targeted:

- `cargo test config::model`
- `cargo test commands::config`
- CLI smoke for `config view --json`, `config verify --json`, and `doctor`.

Final:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --lib --bins --workspace`
- `cargo nextest run --test integration`
- `changeguard verify`
- `cargo install --path .`

## Definition of Done

- `src/config/model.rs` is reduced to a facade and narrowly scoped shared definitions.
- Config domain modules can be read independently.
- Existing config files and env aliases continue to work.
- Secret redaction still protects human, JSON, and diagnostic output.
- Final verification and reinstall pass.

## Risks

- Process environment tests can be order-dependent.
- Serde alias movement can break old config files without compile errors.
- Config defaults are often indirectly tested through commands, so integration coverage matters.
