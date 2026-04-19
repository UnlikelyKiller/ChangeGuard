# Plan: Phase 2 - Repo-Local State Layout and Init

### Phase 2.1: State Layout and Core Utilities
- [ ] Task 2.1.1: Define domain-specific errors in `src/state/mod.rs` using `thiserror` and `miette::Diagnostic` for layout operations.
- [ ] Task 2.1.2: Create `src/state/layout.rs` to encapsulate path resolution for `.changeguard/` and its subdirectories (`logs/`, `tmp/`, `reports/`, `state/`).
- [ ] Task 2.1.3: Write unit/tempdir tests in `src/state/layout.rs` to ensure directory structures are created accurately and idempotently.
- [ ] Task 2.1.4: Implement the `ensure_state_dir` and `initialize_subdirs` functions in `layout.rs`.

### Phase 2.2: Starter Configurations
- [ ] Task 2.2.1: Define `src/config/defaults.rs` with static strings for default `config.toml` and `rules.toml`.
- [ ] Task 2.2.2: Add error definitions for configuration initialization in `src/config/mod.rs`.
- [ ] Task 2.2.3: Write tests verifying that starter configurations are written correctly and not overwritten if they already exist (idempotent).
- [ ] Task 2.2.4: Implement `write_starter_configs` to persist the defaults into the `.changeguard/` directory.

### Phase 2.3: Gitignore Integration
- [ ] Task 2.3.1: Create `src/util/ignore.rs` (or `src/git/ignore.rs`) and define custom IO/Git errors.
- [ ] Task 2.3.2: Write fixture-based tests covering edge cases: `.gitignore` missing, empty, no trailing newline, already contains `.changeguard/`, and contains similar but different entries.
- [ ] Task 2.3.3: Implement `append_to_gitignore` logic to safely append `.changeguard/` when missing, preserving existing contents.

### Phase 2.4: Command Wiring and CLI Integration
- [ ] Task 2.4.1: Update `src/cli.rs` (or `src/commands/mod.rs`) to include the `Init` subcommand with a `--no-gitignore` boolean flag.
- [ ] Task 2.4.2: Implement the orchestrator in `src/commands/init.rs` that sequentially calls layout creation, config writing, and `.gitignore` updating (unless `--no-gitignore` is true). Ensure all errors are wrapped into a `miette::Result`.
- [ ] Task 2.4.3: Add end-to-end integration tests in `tests/cli_init.rs` to verify a successful `changeguard init` execution within a temporary environment, validating the directory tree, `.gitignore`, and starter configs.
