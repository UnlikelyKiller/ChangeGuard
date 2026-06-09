## Plan: Federated Intelligence (Cross-Repo)

### Phase 1: Federation Data Model & Export CLI
- [ ] Task 1.1: Create `src/federated/mod.rs` and `src/federated/schema.rs`. Define `FederatedSchema` with a version, repository name, and public interface elements. Ensure full `serde` support.
- [ ] Task 1.2: Implement secret redaction. When extracting public interfaces for export, strip all configuration or environment values (e.g., API keys, connection strings) to retain only keys/names.
- [ ] Task 1.3: Create `src/commands/federate.rs` and implement the `changeguard federate export` command. This should collect local public symbols and write them out to `.changeguard/schema.json`.
- [ ] Task 1.4: Update `src/cli.rs` to register the `federate` subcommand and ensure it uses `miette::Result` for errors.

### Phase 2: Safe Sibling Discovery
- [ ] Task 2.1: Create `src/federated/scanner.rs` to implement safe directory traversal for `../`.
- [ ] Task 2.2: Implement security constraints: Use `std::fs::symlink_metadata` to skip symlinks. Canonicalize paths and strictly reject any paths escaping higher than one directory above the root.
- [ ] Task 2.3: Implement schema parsing using `serde_json` within a failure-tolerant loop. If a sibling's `.changeguard/schema.json` is malformed, log a diagnostic warning and continue without crashing.
- [ ] Task 2.4: Implement a strict depth cap of 1 to avoid cycle problems and infinite loops.

### Phase 3: Local Database Integration
- [ ] Task 3.1: Add single-statement rusqlite schema migrations (using `i64` for integers) to create `federated_links` and `federated_dependencies` tables.
- [ ] Task 3.2: Implement DB persistence layer in `src/federated/storage.rs` to store discovered sibling schemas securely in the local repository's `.changeguard/db.sqlite3`.
- [ ] Task 3.3: Implement read access for the federated state. Ensure determinism by always sorting database results alphabetically by sibling repo name and symbol name.

### Phase 4: Impact Aggregation & CLI Expansion
- [ ] Task 4.1: Create `src/federated/impact.rs`. Implement logic to merge federated dependencies with local impact packets. If a local file depends on a sibling repo interface, flag the local file as impacted when the sibling changes.
- [ ] Task 4.2: Expand `src/commands/federate.rs` to support `changeguard federate scan` (triggering the scanner explicitly) and `changeguard federate status` (listing known links and cross-dependencies).
- [ ] Task 4.3: Implement integration tests inside `tests/federated_discovery.rs` testing mock sibling layouts, symlink escapes, and malformed schemas.
- [ ] Task 4.4: Run full `cargo check`, `cargo fmt`, and ensure all new structures do not use `unwrap()` in production paths.
