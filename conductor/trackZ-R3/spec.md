# Track Z-R3: Env Schema Completeness & Regex Consolidation

**Status:** Planned
**Milestone:** Z — Command Audit Remediation & Ollama Cloud Hardening
**Priority:** Medium

## Objective

Wire the currently dead `#[allow(dead_code)]` regexes, expand coverage for real-world environment-variable access patterns, deduplicate regex definitions across modules, and make the reference-replacement operation atomic.

## Problem Statement

The Z3 implementation in `src/index/env_schema.rs` defines eight `LazyLock<Regex>` patterns. Three are active (`RUST_ENV_VAR`, `RUST_ENV_MACRO`, `TS_ENV_DOT`, `TS_ENV_INDEXED`, `PY_ENV_GET`, `PY_ENV_INDEXED`); five are marked `#[allow(dead_code)]` and never called:

- `RUST_ENV_VAR_DEFAULT`
- `RUST_SET_ENV`
- `TS_ENV_DEFAULT`
- `TS_SET_ENV`
- `PY_ENV_GET_DEFAULT`

These dead patterns represent legitimate env access/write patterns that should be captured. Meanwhile, common real-world patterns are uncovered:

- Rust: `std::env::var_os(…)`, `option_env!(…)`, bare `env::var` via `use std::env`
- Python: `os.environ['KEY']`, `from os import environ; environ.get(…)`
- JS/TS: `import.meta.env.VAR` (Vite), `const { VAR } = process.env`

The same regexes are duplicated in `src/index/runtime_usage.rs`, creating a maintenance hazard: a fix to one module does not propagate.

Finally, the orphan cleanup (`DELETE FROM env_references WHERE file_id NOT IN (...)`) runs outside a transaction with the subsequent `INSERT`s. If the process crashes between `DELETE` and `INSERT`, the table is temporarily empty.

## Acceptance Criteria

1. **Dead regexes wired**: All six `#[allow(dead_code)]` regexes are called in `extract_references_from_source` with appropriate `EnvReferenceKind` variants.
2. **Expanded coverage**: New regexes for `var_os`, `option_env!`, `import.meta.env`, `os.environ['KEY']`, and destructuring are added and tested.
3. **Shared module**: A new `src/index/env_patterns.rs` module owns all `LazyLock<Regex>` definitions; both `env_schema.rs` and `runtime_usage.rs` re-export.
4. **Atomic replacement**: `EnvSchemaIndexer::extract()` wraps the orphan `DELETE` and all per-file `INSERT`s in a single SQLite `Transaction`.
5. **Zero behavior regression**: Existing patterns continue to match identically.

## Key Files

- `src/index/env_schema.rs` — Regex wiring, atomic transaction, new patterns.
- `src/index/runtime_usage.rs` — Re-export consolidation.
- `src/index/env_patterns.rs` — New shared module.
- `tests/integration/track_z3_repro.rs` — Coverage tests.

## Definition of Done

- `cargo nextest run --lib --bins --workspace` passes.
- `cargo nextest run --test integration` passes.
- `config diff` no longer reports false negatives for `option_env!` or `import.meta.env` usage.
- Deleting a file and re-indexing leaves zero orphaned `env_references` rows.
- `cargo clippy --all-targets --all-features -- -D warnings` is clean.
