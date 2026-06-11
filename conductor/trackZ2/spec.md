# Track Z2: `data-models impact --changed` Clean-Tree Message
 
**Status:** In Progress
**Milestone:** Z — Command Audit Remediation & Ollama Cloud Hardening
**Priority:** Medium

## Objective

When the working tree is clean and `changeguard data-models impact --changed` is run, it currently prints `"No data models indexed. Data models are extracted from ORM structs, SQL table definitions, and migration files..."` which is highly misleading. It should instead say `"No changed data models found."` if data models do exist in the index.

## Problem Statement

In `src/commands/data_models.rs`, if the filtered list of `impacted` data models is empty, it unconditionally prints the "No data models indexed" help text. It does not check if data models exist in the database (i.e., whether the database contains models but none of them are affected by unstaged/uncommitted changes).

## Acceptance Criteria

1. Running `changeguard data-models impact --changed` on a clean working tree (or when no data models are affected by changes) prints a graceful `"  No changed data models found."` message if the total count of indexed data models in SQLite is greater than zero.
2. If the total count of indexed data models is zero, it prints the original, descriptive help message.
3. Tests verify both behaviors.

## API Contracts

If `impacted.is_empty()`:
* If `total_indexed_models > 0`:
  ```
  Data Model Impact Analysis
    No changed data models found.
  ```
* If `total_indexed_models == 0`:
  ```
    No data models indexed. Data models are extracted from ORM structs,
    SQL table definitions, and migration files. Run `changeguard index
    --incremental` if models exist, or confirm your ORM/framework is supported.
  ```

## Key Files

* `src/commands/data_models.rs` — `execute_data_models` (`DataModelSubcommands::Impact` match arm)

## Definition of Done

* `cargo nextest run --lib --bins --workspace` passes.
* Command output is validated against the clean-tree state.
