# Specification: Track E4-3 Environment Variable Schema Extraction

## Overview

Implement the third track of Phase E4 (Safety Context) from `docs/expansion-plan.md`. This track extracts environment variable declarations from `.env.example` and config files, and environment variable references from source code, stores them in two separate tables (`env_declarations` and `env_references`), and integrates with the `impact` command to warn about undeclared environment variable dependencies.

## Motivation

ChangeGuard's `runtime_usage` extractor already collects environment variable references from source code, but they have zero effect on risk scoring or verification prediction. When a developer adds a new `std::env::var("DATABASE_URL")` call, ChangeGuard should warn that a new runtime dependency has been introduced. This track builds an environment variable schema from config files and source code, and uses it to detect and flag new env var dependencies.

The design uses two separate tables — `env_declarations` for variables declared in config files (`.env.example`, `config.toml`, `config.json`), and `env_references` for variables referenced in source code — to cleanly separate declarations from usage and to enable detection of undeclared dependencies.

## Components

### 1. Config File Parsing (`src/index/env_schema.rs`)

New module that parses environment variable definitions from config files and source code.

**`.env.example` and `.env.template` parsing:**
- Read each line, extract `KEY=VALUE` or `KEY: VALUE` pairs.
- Lines starting with `#` are comments; skip them.
- Lines with no `=` or `:` are malformed; skip with a warning.
- Extract: `var_name`, `default_value_redacted` (a redacted category, never the raw value — see Security below), `description` (from inline `# description` or the comment line above).
- Mark `required = 1` if the line has no default value or the default is empty.

**Security (CRITICAL):** NEVER store raw default values from `.env.example`. Store only redacted categories:
- `HAS_DEFAULT`: the variable has a non-empty, non-placeholder default value
- `EMPTY_DEFAULT`: the variable has `KEY=` (empty string default)
- `PLACEHOLDER_DEFAULT`: the variable has a placeholder like `CHANGE_ME`, `xxx`, `your-api-key-here`
- `POSSIBLE_SECRET_REDACTED`: the value looks like it could be a secret (contains patterns matching URLs, keys, tokens) — always redacted regardless of source

This is a hard requirement. Storing raw default values, even from `.env.example`, risks accidental exposure of secrets in logs, reports, and database files.

**`config.toml` parsing:**
- Parse TOML structure and extract key-value pairs at any nesting depth.
- Flatten nested keys with `.` separator (e.g., `database.url`).
- Extract: `var_name` (the flattened key), `default_value_redacted` (the redacted category for the TOML value), `required` (1 if no default, 0 if default provided).
- Skip keys that are clearly not env var references (e.g., `version`, `name` at top level).

**`config.json` parsing:**
- Parse JSON structure and extract key-value pairs.
- Flatten nested keys with `.` separator.
- Extract: `var_name`, `default_value_redacted`, `required`.
- Skip keys that are clearly not env var references.

**Source code env var extraction** (leveraging existing `runtime_usage`):
- The existing `src/index/runtime_usage.rs` already extracts env var references from source code.
- This track reads those references and creates `env_references` entries with `reference_kind` based on how the variable is used.
- For Rust: `std::env::var("KEY")` → READ, `env!("KEY")` → READ, `option_env!("KEY")` → DEFAULTED
- For TypeScript: `process.env.KEY` → READ, `process.env["KEY"]` → READ
- For Python: `os.environ["KEY"]` → READ, `os.environ.get("KEY")` → DEFAULTED, `os.getenv("KEY")` → DEFAULTED
- Dynamic env var references (e.g., `process.env[dynamicKey]`) are marked with `reference_kind = 'DYNAMIC'` and `confidence = 0.5`.

### 2. Database Schema (`src/state/migrations.rs`)

Add migration M18 to create two tables: `env_declarations` and `env_references` (alongside the `test_mapping` and `ci_gates` tables from Tracks E4-1 and E4-2):

```sql
CREATE TABLE IF NOT EXISTS env_declarations (
    id INTEGER PRIMARY KEY,
    var_name TEXT NOT NULL,
    source_file_id INTEGER NOT NULL REFERENCES project_files(id),
    source_kind TEXT NOT NULL,
    required INTEGER DEFAULT 0,
    default_value_redacted TEXT,
    description TEXT,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_indexed_at TEXT NOT NULL,
    UNIQUE(var_name, source_file_id, source_kind)
);

CREATE TABLE IF NOT EXISTS env_references (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES project_files(id),
    symbol_id INTEGER REFERENCES project_symbols(id),
    var_name TEXT NOT NULL,
    reference_kind TEXT NOT NULL,
    line_start INTEGER,
    confidence REAL NOT NULL DEFAULT 1.0,
    last_indexed_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_env_decls_var ON env_declarations(var_name);
CREATE INDEX IF NOT EXISTS idx_env_decls_file ON env_declarations(source_file_id);
CREATE INDEX IF NOT EXISTS idx_env_refs_var ON env_references(var_name);
CREATE INDEX IF NOT EXISTS idx_env_refs_file ON env_references(file_id);
```

The `source_kind` column in `env_declarations` distinguishes where the declaration was found: `DOTENV_EXAMPLE`, `CONFIG`, or `DOCS`.

The `reference_kind` column in `env_references` distinguishes how the variable is used: `READ`, `WRITE`, `DEFAULTED`, or `DYNAMIC`.

**Note:** The single `env_schema` table design has been replaced by this two-table design to cleanly separate declarations from references, enabling proper undeclared dependency detection.

### 3. Index Integration (`src/commands/index.rs`)

Wire env schema extraction into the `changeguard index` command:

- Detect config files by path pattern: `.env.example`, `.env.template`, `config.toml`, `config.json`.
- Parse each detected file using the appropriate parser and insert into `env_declarations` with the appropriate `source_kind`.
- Also read env var references from the existing `runtime_usage` extraction and create `env_references` entries with the appropriate `reference_kind`.
- Insert results into `env_declarations` and `env_references`.
- On re-index, delete existing `env_declarations` and `env_references` rows for a file before inserting new ones.

### 4. Impact Integration (`src/impact/analysis.rs`)

Add environment variable dependency detection:

- After analyzing a changed file, collect its `runtime_usage.env_vars` (env var references in the current file version).
- Query `env_declarations` for all known env var names in the project.
- For each env var reference in the changed file that is NOT in `env_declarations`, add a risk reason: "New environment variable dependency: X" to `ImpactPacket.env_var_deps`.
- Very common env vars (`PATH`, `HOME`, `USER`, `LANG`, `SHELL`, `TERM`, `PWD`) are excluded from risk scoring as too common to be meaningful.

**Risk category:** New env var dependencies contribute to the **Runtime/Config** category (max 25 points) in the category-capped scoring model (expansion plan Section 4.2).

### 5. ImpactPacket Extension (`src/impact/packet.rs`)

Add the `env_var_deps` field to `ImpactPacket`:

```rust
#[serde(default)]
pub env_var_deps: Vec<EnvVarDep>,
```

Where `EnvVarDep` is:

```rust
pub struct EnvVarDep {
    pub var_name: String,
    pub file_id: i64,           // references project_files(id)
    pub source: String,         // "DOTENV_EXAMPLE", "CONFIG", "CODE"
    pub is_new: bool,           // true if not in env_declarations
    pub confidence: f64,         // confidence score for this dependency detection
    pub evidence: String,      // e.g., "std::env::var(\"DATABASE_URL\")", "process.env.API_KEY"
}
```

All new fields must have `#[serde(default)]` to maintain backward compatibility.

### 6. Verify Integration (`src/verify/predict.rs`)

Add env-var-based verification suggestions:

- When a changed file introduces new env var dependencies (not in `env_declarations`), add a prediction reason: "New env var dependency: X".
- Suggest running with `--check-env` to validate that all required env vars are set (the `--check-env` flag itself is deferred to a future phase; for now, the prediction reason is informational).

## Constraints & Guidelines

- **Security**: NEVER read or parse actual `.env` files. Only read `.env.example` and `.env.template` files. Actual `.env` files may contain secrets.
- **Security (CRITICAL)**: NEVER store raw default values from `.env.example` or any config file. Store only redacted categories (`HAS_DEFAULT`, `EMPTY_DEFAULT`, `PLACEHOLDER_DEFAULT`, `POSSIBLE_SECRET_REDACTED`). This is a hard requirement. Violations risk exposing secrets in logs, reports, and database files.
- **Graceful degradation**: If no config files exist, skip env declaration extraction from config. If `runtime_usage` has no env var references, skip code-based extraction. Missing data is not an error.
- **No false confidence**: Dynamic env var references (e.g., `process.env[dynamicKey]`) are stored with `reference_kind = 'DYNAMIC'` and `confidence = 0.5`, and excluded from risk scoring.
- **Same env var from multiple files**: Store one `env_declarations` row per `(var_name, source_file_id, source_kind)` combination. The `source_kind` column indicates where it was found.
- **TDD Requirement**: Write or update tests for config file parsing, env var extraction, risk weight application, and ImpactPacket extension.
- **No performance regression**: Env schema extraction must not add more than 5% overhead to the `index` command.
- **Backward-compatible schema**: The `env_declarations` and `env_references` tables are new and additive. No existing tables are modified.

## Edge Cases

- **`.env` files (not `.env.example`)**: NEVER read or parse. These may contain secrets. Only parse `.env.example` and `.env.template`.
- **Dynamic env var names** (`process.env[DYNAMIC_KEY]`): Store with `reference_kind = 'DYNAMIC'` and `confidence = 0.5`, and exclude from risk scoring. Mark `required = 0` in declarations (if a declaration is somehow created).
- **Same env var referenced from multiple files**: Store one row per `(var_name, file_id)` combination in `env_references`, and one row per `(var_name, source_file_id, source_kind)` combination in `env_declarations`.
- **Malformed config files**: Extract what is possible. Skip malformed lines with a warning. Never crash.
- **Very large config files**: Skip values longer than 4096 characters. For `default_value_redacted`, always store the redacted category, never the raw value.

## Acceptance Criteria

- `changeguard index` populates `env_declarations` from `.env.example`, `config.toml`, `config.json` and `env_references` from source code references.
- `changeguard impact` warns about new env var dependencies that are not in the project's `env_declarations`.
- Actual `.env` files are never read or parsed.
- Raw default values are never stored; only redacted categories (`HAS_DEFAULT`, `EMPTY_DEFAULT`, `PLACEHOLDER_DEFAULT`, `POSSIBLE_SECRET_REDACTED`) are stored.
- Dynamic env var references are stored but excluded from risk scoring.
- Repos without config files continue to function normally with code-only env var extraction.

## Definition of Done

- [ ] All acceptance criteria pass
- [ ] All unit tests pass
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes with no regressions
- [ ] No deviations from this spec without documented justification
- [ ] Migration M18 applied cleanly to existing ledger.db
- [ ] `changeguard index` populates E4 tables for fixture repos