You are a senior Rust reviewer performing a read-only audit of a config model domain split refactor.

## Context
`src/config/model.rs` (1,739 lines) was split into 8 focused domain modules under `src/config/model/` while preserving every serde contract, default, alias, env override, dotenv behavior, and validation contract. `src/config/model.rs` is now a ~366-line compatibility facade.

## Previous review cycles
Two prior subagent review cycles both returned PASS. One minor finding about `pub(crate)` visibility was addressed by an existing code comment.

## Files to review
Please review the following files (read-only) and report any findings.

- `src/config/model.rs` (facade)
- `src/config/model/root.rs`
- `src/config/model/local_model.rs`
- `src/config/model/gemini.rs`
- `src/config/model/semantic.rs`
- `src/config/model/verify.rs`
- `src/config/model/coverage.rs`
- `src/config/model/ledger.rs`
- `src/config/model/env.rs`
- `tests/compile_fail/config_model_submodule.rs`
- `tests/compile_fail/config_model_submodule.stderr`

## What to look for
1. **Serde contract preservation**: Are ALL `#[serde(default)]`, `#[serde(default = "...")]`, `#[serde(alias = "...")]`, `#[serde(rename_all = "...")]`, and `#[serde(skip_serializing_if = "...")]` preserved exactly after the move?
2. **Facade completeness**: Does `src/config/model.rs` re-export every public type that existed before? Are any types missing?
3. **Env/dotenv behavior**: Is the `OLLAMA_CLOUD_API_KEY` → `OLLAMA_API_KEY` fallback chain preserved? Is `ollama_key` alias preserved?
4. **Secret safety**: Does `config view --json` redact secrets? Are there any accidental secret leaks in test output?
5. **Compile-fail contract**: Does the compile-fail test assert that submodule paths are private?
6. **Regression risks**: Are there any broken imports outside the config module?

## Expected outcome
Return either:
- **CLEAR** — no actionable findings.
- **ACTIONABLE: <list>** — specific findings with line references.
