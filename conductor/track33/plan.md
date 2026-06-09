## Plan: Federated Intelligence Completion
### Phase 1: Storage and Schema Safety Hardening
- [ ] Task 1.1: Update `src/federated/schema.rs` to add explicit `schema_version` validation when loading. Return a typed error for version mismatches.
- [ ] Task 1.2: Update `src/federated/scanner.rs` `load_schema` to wrap `serde_json::from_str` in `std::panic::catch_unwind`. Return a safe error on panic.
- [ ] Task 1.3: Update `src/federated/scanner.rs` to canonicalize paths using `fs::canonicalize`. Verify the resolved path is exactly `parent_dir.join(sibling_name)` to prevent traversal attacks.
- [ ] Task 1.4: Implement a configurable cap in `FederatedScanner` to stop at 20 siblings.
- [ ] Task 1.5: Refactor `scan_siblings` to return `Result<(Vec<(Utf8PathBuf, FederatedSchema)>, Vec<String>)>`, where the second element is a list of deterministic user-visible warnings for malformed or skipped schemas.

### Phase 2: Dependency Edge Discovery
- [ ] Task 2.1: Implement `discover_dependencies` in `src/federated/scanner.rs` (or a new module) that cross-references local files/symbols against discovered sibling `public_interfaces`. A lightweight text-based presence check of sibling symbols in local source files is acceptable for v1.
- [ ] Task 2.2: Update `src/commands/federate.rs` (`execute_federate_scan`) to clear existing dependencies for a sibling, call `discover_dependencies`, and save the new edges via `save_federated_dependencies`.
- [ ] Task 2.3: In `src/commands/federate.rs` (`execute_federate_export`), remove `unwrap_or("unknown")` for the repo name. Return `miette::miette!("Could not determine repository name for export")` if it fails.
- [ ] Task 2.4: In `execute_federate_export`, apply secret redaction patterns to the `FederatedSchema` before serializing and writing to disk.

### Phase 3: True Impact Resolution
- [ ] Task 3.1: Rewrite `check_cross_repo_impact` in `src/federated/impact.rs`. Remove the generic placeholder warning.
- [ ] Task 3.2: Iterate through stored `federated_links`. For each link, load the current `schema.json` from disk. If missing or invalid, generate an impact reason: `"Cross-repo impact: Sibling '{name}' schema is unavailable or invalid."`
- [ ] Task 3.3: Query `get_dependencies_for_sibling`. For each `(local_symbol, sibling_symbol)`, check if `sibling_symbol` exists in the newly loaded sibling schema.
- [ ] Task 3.4: If `sibling_symbol` is missing, push a specific impact reason to the packet: `"Cross-repo impact: Local symbol '{local}' depends on sibling '{sibling}' interface '{interface}' which was removed."`
- [ ] Task 3.5: Ensure all generated risk reasons are deterministically sorted before adding to the `ImpactPacket`.

### Phase 4: Testing & Verification
- [ ] Task 4.1: Write `tests/federated_discovery.rs` tests for path confinement, panic isolation (`catch_unwind`), and the 20-sibling limit.
- [ ] Task 4.2: Write tests in `tests/federated_discovery.rs` (or a new test file) that mock a sibling schema, create local dependencies, mutate the sibling schema (remove a symbol), and assert that `check_cross_repo_impact` generates the correct specific warning.
- [ ] Task 4.3: Write a test verifying `changeguard federate export` returns an error if the repo name is missing, and correctly redacts secrets if present.