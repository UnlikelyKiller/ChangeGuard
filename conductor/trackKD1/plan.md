## Plan: Track KD1 - CozoDB-Redux Dependency Upgrade

### Phase 1: Cargo Dependency Update
- [ ] Task 1.1: Edit `Cargo.toml` to replace the `cozo` dependency reference with the latest `cozo-redux` GitHub branch.
- [ ] Task 1.2: Run `cargo check` and resolve any missing type definitions or compilation issues.
- [ ] Task 1.3: Clean old build cache and execute `cargo build --workspace`.

### Phase 2: System Validation
- [ ] Task 2.1: Run `cargo run -- doctor` to verify that `CozoStorage` connects to SQLite and In-Memory instances, and starts cleanly.
- [ ] Task 2.2: Ensure cold start verification does not encounter filesystem lock errors on Windows.

### Phase 3: Verification
- [ ] Task 3.1: Execute `cargo test --workspace` to ensure all existing knowledge-graph and vector-store tests pass.
