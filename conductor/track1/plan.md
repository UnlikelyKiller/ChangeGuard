## Plan: Track 1 - Repo-Local State Layout and Init

### Phase 2: Repo-Local State Layout and Init
- [ ] Task 2.1: Define State Layout Abstractions
  - Create constants and functions in `src/state/layout.rs` to manage paths within `.changeguard/`.
  - Ensure all paths are deterministic, absolute, and relative to the discovered repository root.
  - Define custom `miette::Diagnostic` error types for any directory resolution or creation failures.
- [ ] Task 2.2: Implement `.gitignore` Mutator
  - Create a utility in `src/git/ignore.rs` or directly within the init command flow.
  - Read the existing `.gitignore` (if any), check for an existing `.changeguard/` or `.changeguard` entry.
  - Safely append `.changeguard/` if missing, preserving the file's original line endings.
- [ ] Task 2.3: Generate Starter Configurations
  - Add static string templates for `config.toml` and `rules.toml`.
  - Implement idempotent file creation for these files in `.changeguard/`.
- [ ] Task 2.4: Implement the `init` CLI Command
  - Update `src/commands/init.rs` to use `clap` for the `--no-gitignore` boolean flag.
  - Wire up the layout creation, file generation, and `.gitignore` update steps.
  - Log progress appropriately without `unwrap()` or panics on failure.
- [ ] Task 2.5: Add Verification Tests
  - Add unit tests for state layout resolution and directory creation.
  - Add unit tests for the `.gitignore` mutator (idempotency, line-ending preservation).
  - Add integration-style tests for the complete `init` command.
  - **Explicit Verification Step**: Run `cargo test -j 1` to ensure tests are stable and do not race on filesystem state.
