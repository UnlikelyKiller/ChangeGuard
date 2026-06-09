# Track X2 Plan: Cargo.lock → `dependencies list`

## Phase 1 — Red (Failing Tests)
- [x] 1. Write unit test `tests/repro_dependencies_list.rs::test_cargo_lock_populates_kg`: create a temp Cargo.lock with 3 packages, run the loader, assert CozoDB has 3 `category: 'package'` nodes.
- [x] 2. Write unit test for the Cargo.lock TOML parser: assert `[[package]]` sections are parsed to `{name, version, dependencies: [...]}`.

## Phase 2 — Implementation
- [x] 3. In `src/index/graph_loader.rs`, add Section 10 (after Cedar policies, before final insert):
  - Scan for `Cargo.lock` at `storage.root_path()`.
  - Parse `[[package]]` sections using `toml::from_str::<CargoLockFile>`.
  - For each package, create a `GraphNode` with `category = NodeKind::Package.to_string()` and metadata JSON `{version, ecosystem: "rust/cargo", manifest: "Cargo.lock"}`.
  - URN: `urn:changeguard:package:{name}:{version}`.
  - For each `dependencies` entry in a package, create a `GraphEdge` with `EdgeKind::DependsOn`.
- [x] 4. Add `CargoLockFile` and `CargoLockPackage` structs in `src/index/graph_loader.rs` (or a new `src/index/cargo_lock.rs`):
  ```rust
  #[derive(Deserialize)]
  struct CargoLockFile { package: Vec<CargoLockPackage> }
  #[derive(Deserialize)]
  struct CargoLockPackage { name: String, version: String, dependencies: Option<Vec<String>> }
  ```
- [x] 5. Emit `info!("Cargo.lock: {} packages indexed", count)` at end of ingestion.
- [x] 6. Emit `warn!("No Cargo.lock found at {:?}", path)` when absent (non-fatal).

## Phase 3 — Green + Cleanup
- [x] 7. Run `changeguard index --analyze-graph` locally, verify `dependencies list` shows packages.
- [x] 8. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 9. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 10. Run `cargo fmt --all -- --check` — clean.
- [x] 11. Update `conductor/conductor.md` status to Completed.
