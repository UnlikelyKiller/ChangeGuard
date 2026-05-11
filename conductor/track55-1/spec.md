# Track 55-1: Maintenance & Migration (Update Command)

## Objective
Provide a unified `update` command to manage the lifecycle of ChangeGuard installations and repository-specific state, ensuring that users can easily migrate to new versions and fix data inconsistencies.

## Problem Statement
As ChangeGuard evolves (e.g., switching to CozoDB-redux, adding new FTS indices, or changing embedding dimensions), users are currently required to manually delete `.changeguard/state` and re-index. This is friction-heavy and error-prone. Additionally, upgrading the binary itself requires knowing the `cargo install` command.

## Scope
-   **Binary Management**:
    -   Implement `update --binary` to trigger a re-compilation and installation of the current source (if available) or check for newer versions.
-   **State Migration**:
    -   Implement `update --migrate` to perform "soft" or "hard" migrations.
    -   Soft migration: Run all pending schema updates in `storage_cozo.rs`.
    -   Hard migration: Wipe non-ledger state (Knowledge Graph, Embeddings) and trigger a full re-index.
-   **Repo Hygiene**:
    -   Validate that `.gitignore` contains necessary patterns.
    -   Verify that all paths in the Knowledge Graph are properly normalized (UTF-8).

## Deliverables
-   New `update` CLI command.
-   Migration engine in `src/state/migration.rs`.
-   Integration with `index` and `storage_cozo` for automated re-initialization.
