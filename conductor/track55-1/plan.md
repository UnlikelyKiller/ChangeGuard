# Track 55-1: Maintenance & Migration - Implementation Plan

## Phase 1: CLI & Command Infrastructure
1.  **CLI Definition**:
    - Add `Update` subcommand to `Commands` in `src/cli.rs`.
    - Define flags: `--migrate`, `--binary`, `--force`.
2.  **Command Implementation**:
    - Create `src/commands/update.rs`.
    - Implement `execute_update`.

## Phase 2: Migration Logic
1.  **State Management**:
    - Implement `StateMigrationEngine` in `src/state/migration.rs`.
    - Add logic to detect schema version mismatches (via a new `meta` relation in Cozo).
2.  **Binary Update**:
    - Implement `update_binary()` using `std::process::Command` to invoke `cargo install --path .` if the source is detected.
3.  **Migration Paths**:
    - **Fresh Start**: If schema is significantly old, prompt to wipe and re-index.
    - **Enrichment**: Add new indices (like the FTS index from Track 54-1) without wiping existing embeddings.

## Phase 3: Integration & Testing
1.  **Integration**:
    - Link `update` command in `src/cli.rs`.
2.  **Verification**:
    - Run `changeguard update --migrate` and verify that the Knowledge Graph is healthy.
    - Test the `update --binary` flow (optional, dependent on environment).
