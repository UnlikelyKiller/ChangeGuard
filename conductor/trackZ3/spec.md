# Track Z3: Config Diff Env Var References
 
**Status:** In Progress
**Milestone:** Z — Command Audit Remediation & Ollama Cloud Hardening
**Priority:** High

## Objective

Fix the false negatives in `changeguard config diff` where all declared env vars are reported as "not referenced in code" even though they are clearly used.

## Problem Statement

In `src/index/env_schema.rs`, `EnvSchemaIndexer::extract()` only extracts declarations from `.env.example`. The `EnvSchemaExtractor::extract_references_from_source` utility is implemented but never called or integrated into the indexing pipeline. As a result, the `env_references` table remains completely empty, leading `config diff` to report all declared variables as unused.

## Acceptance Criteria

1. `changeguard index` (both full and incremental) scans all project files in `project_files`, extracts env references, and saves them to the `env_references` database table.
2. The indexer tracks `total_references` and `files_processed` accurately in `EnvSchemaStats`.
3. Running `changeguard config diff` correctly lists env vars referenced in the source files, eliminating the false-positive "Declared but not referenced" warning.

## Key Files

* `src/index/env_schema.rs` — `EnvSchemaIndexer::extract()` and database insertion helpers.
* `src/commands/config.rs` — `execute_config_diff()` verification.

## Definition of Done

* `cargo nextest run --lib --bins --workspace` passes.
* Verify `config diff` output is accurate on the `ChangeGuard` codebase itself.
