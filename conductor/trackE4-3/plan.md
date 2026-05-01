## Plan: Track E4-3 Environment Variable Schema Extraction

### Phase 1: Database Schema
- [ ] Task 1.1: Add `env_schema` table creation to migration M18 in `src/state/migrations.rs` with columns `id`, `var_name`, `source`, `default_value`, `description`, `required`, `file_path`, `last_indexed_at` and indices on `var_name`, `source`, and `file_path`.
- [ ] Task 1.2: Write a test verifying the `env_schema` table is created and supports insert/query operations, including querying by `var_name` and `source`.
- [ ] Task 1.3: Add `EnvVarDep` struct to `src/impact/packet.rs` with fields `var_name`, `file_path`, `source`, `is_new`.
- [ ] Task 1.4: Add `env_var_deps: Vec<EnvVarDep>` field to `ImpactPacket` with `#[serde(default)]`.
- [ ] Task 1.5: Write tests verifying `EnvVarDep` serialization/deserialization and default-empty behavior.

### Phase 2: Config File Parsers
- [ ] Task 2.1: Create `src/index/env_schema.rs` module with an `EnvVarEntry` struct holding `var_name`, `source`, `default_value`, `description`, `required`, `file_path`.
- [ ] Task 2.2: Implement `.env.example` and `.env.template` parser: extract `KEY=VALUE` pairs, comments as descriptions, mark `required` based on whether a default value exists.
- [ ] Task 2.3: Implement `config.toml` parser: flatten nested keys with `.` separator, extract key-value pairs, skip non-env-var-like keys.
- [ ] Task 2.4: Implement `config.json` parser: flatten nested keys with `.` separator, extract key-value pairs, skip non-env-var-like keys.
- [ ] Task 2.5: Write unit tests for each parser using fixture config files with known structure.
- [ ] Task 2.6: Write test: `.env` files (without `.example` or `.template` suffix) are never read or parsed.
- [ ] Task 2.7: Write test: malformed lines in `.env.example` are skipped with warnings, not crashes.

### Phase 3: Source Code Env Var Extraction
- [ ] Task 3.1: Implement code-based env var extraction by reading existing `runtime_usage` data from `ChangedFile` and creating `env_schema` entries with `source = 'CODE'`.
- [ ] Task 3.2: Implement dynamic env var detection: mark `process.env[dynamicKey]` and similar patterns with `var_name = 'DYNAMIC'` and exclude from risk scoring.
- [ ] Task 3.3: Implement common env var exclusion: skip `PATH`, `HOME`, `USER`, `LANG`, `SHELL`, `TERM`, `PWD` from risk scoring.
- [ ] Task 3.4: Write test: Rust `std::env::var("DATABASE_URL")` extracts `var_name = 'DATABASE_URL'` with `source = 'CODE'`.
- [ ] Task 3.5: Write test: TypeScript `process.env.API_KEY` extracts `var_name = 'API_KEY'` with `source = 'CODE'`.
- [ ] Task 3.6: Write test: Python `os.getenv("CONFIG")` extracts `var_name = 'CONFIG'` with `source = 'CODE'`.
- [ ] Task 3.7: Write test: dynamic env var references are stored with `var_name = 'DYNAMIC'` and `required = 0`.

### Phase 4: Index Integration
- [ ] Task 4.1: Add `extract_env_schema` function to `src/index/env_schema.rs` that detects config files by path pattern and dispatches to the appropriate parser.
- [ ] Task 4.2: Wire env schema extraction into `src/commands/index.rs`: detect and parse config files, read `runtime_usage` env var references, insert all results into `env_schema`.
- [ ] Task 4.3: Implement upsert logic: on re-index, delete existing `env_schema` rows for a file before inserting new ones.
- [ ] Task 4.4: Write integration test: run `changeguard index` on a fixture repo with `.env.example`, `config.toml`, and source code env var references, and verify `env_schema` rows are populated correctly.
- [ ] Task 4.5: Write security test: verify that `.env` files (not `.env.example`) are never read or parsed.

### Phase 5: Impact Integration
- [ ] Task 5.1: Modify `analyze_risk()` in `src/impact/analysis.rs` to collect `runtime_usage.env_vars` from changed files and compare against `env_schema`.
- [ ] Task 5.2: For each env var reference in a changed file that is NOT in `env_schema`, create an `EnvVarDep` entry with `is_new = true` and add it to `ImpactPacket.env_var_deps`.
- [ ] Task 5.3: Exclude common env vars (`PATH`, `HOME`, `USER`, `LANG`, `SHELL`, `TERM`, `PWD`) from the new-dependency check.
- [ ] Task 5.4: Add risk reason: "New environment variable dependency: X" for each new env var dependency found.
- [ ] Task 5.5: Write test: adding `std::env::var("NEW_VAR")` to a source file produces a "New env var dependency: NEW_VAR" warning in the impact output.
- [ ] Task 5.6: Write test: adding `process.env.PATH` does NOT produce a new dependency warning (common env var exclusion).

### Phase 6: Verify Integration
- [ ] Task 6.1: Modify `src/verify/predict.rs` to include env-var-based prediction reasons when a changed file introduces new env var dependencies.
- [ ] Task 6.2: Add prediction reason: "New env var dependency: X" for each new env var dependency found in the changed file.
- [ ] Task 6.3: Write test: changing a file that introduces a new env var dependency produces a prediction reason mentioning the env var.

### Phase 7: Final Validation
- [ ] Task 7.1: Run full test suite (`cargo test`) and verify no regressions in existing `impact`, `hotspots`, `verify`, or `ledger` tests.
- [ ] Task 7.2: Run `changeguard index` on a fixture repo with `.env.example` and source code env var references, and verify `env_schema` rows are created with correct `source` values.
- [ ] Task 7.3: Run `changeguard impact` on a fixture repo and verify `env_var_deps` appears in JSON output when new env var dependencies are introduced.
- [ ] Task 7.4: Verify that `.env` files are never read by the index or impact commands.