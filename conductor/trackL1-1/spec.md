# Specification: Track L1-1 Ledger Data Model & Migrations

## Overview
Implement the first track of Phase L1 (Transaction Lifecycle & Data Model) from `docs/Ledger-Incorp-plan.md`. This track establishes the domain types, error handling, configuration models, and initial database migrations required to support the ledger feature in ChangeGuard.

## Components

### 1. Ledger Types (`src/ledger/types.rs`)
Defines the core enums for the Ledger domain. These enums must use `serde` for serialization/deserialization and `clap::ValueEnum` for CLI parsing.
- **`Category`**: `Architecture`, `Feature`, `Bugfix`, `Refactor`, `Infra`, `Tooling`, `Docs`, `Chore`. (Serialized as `SCREAMING_SNAKE_CASE`).
- **`ChangeType`**: `Create`, `Modify`, `Deprecate`, `Delete`. (Serialized as `SCREAMING_SNAKE_CASE`).
- **`EntryType`**: `Implementation`, `Architecture`, `Lesson`. (Serialized as `SCREAMING_SNAKE_CASE`).
- **`VerificationStatus`**: `Verified`, `Unverified`, `PartiallyVerified`, `Failed`. (Serialized as `snake_case`).
- **`VerificationBasis`**: `Tests`, `Build`, `Lint`, `Runtime`, `ManualInspection`, `Inferred`. (Serialized as `snake_case`).

### 2. Ledger Errors (`src/ledger/error.rs`)
Defines the error taxonomy for the ledger subsystem.
- Implement `LedgerError` enum.
- Derive `thiserror::Error` and `miette::Diagnostic`.
- Anticipate variants such as `DatabaseError(#[from] rusqlite::Error)`, `ConfigError`, `InvalidState`, `TransactionNotFound`, and `EntityConflict`.

### 3. Database Migrations (`src/state/migrations.rs`)
Add migrations M11 and M12 to `src/state/migrations.rs` after the existing 10 migrations.
- **M11**: Create `transactions` table with fields for UUID, entity, category, status, etc., along with indices for querying.
- **M12**: Create `ledger_entries` table, `ledger_fts` virtual table using SQLite FTS5, and the associated FTS5 content-sync triggers (`AFTER INSERT`, `AFTER DELETE`, `AFTER UPDATE`).

### 4. Configuration Model (`src/config/model.rs`)
Extend the global configuration to support ledger enforcement and workflows.
- Add `LedgerConfig` struct containing `enforcement_enabled`, `verify_to_commit`, `auto_reconcile`, `stale_threshold_hours`, `category_mappings`, and `watcher_patterns`.
- Define helper structs `CategoryMapping` and `WatcherPattern`.
- Implement `Default` traits mapping to the documented defaults (e.g., `auto_reconcile = true`, `stale_threshold_hours = 24`).
- Add a `ledger` field to the root `Config` struct.

## Constraints & Guidelines
- **TDD Requirement**: Write or update tests for types, error formatting, configuration defaults/deserialization, and migration schemas. Ensure tests fail before implementing the logic.
- **No Production Unwraps**: Handle errors gracefully by returning `Result<T, LedgerError>`.
- **Path Normalization**: Internal paths will be handled as `camino::Utf8PathBuf`. Normalization for `entity_normalized` means using forward slashes and being relative to the workspace root.
- **SQLite Compatibility**: The project uses `rusqlite` with the `bundled` feature, providing FTS5 natively.
