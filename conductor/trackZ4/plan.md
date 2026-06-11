# Track Z4 Plan: Cargo.lock Dependency Ingestion

## Phase 1 — Red (Failing Tests)
- [ ] 1. Create a unit test `tests/repro_dependencies_list.rs::test_cargo_lock_populates_kg` that creates a mock `Cargo.lock`, runs the graph loader, and asserts CozoDB contains the package nodes/edges.
- [ ] 2. Currently, the test will fail since `Cargo.lock` is ignored.

## Phase 2 — Implementation
- [ ] 3. In `src/index/graph_loader.rs`, create a package parser structure:
  ```rust
  #[derive(serde::Deserialize)]
  struct CargoLockFile { package: Vec<CargoLockPackage> }
  #[derive(serde::Deserialize)]
  struct CargoLockPackage { name: String, version: String, dependencies: Option<Vec<String>> }
  ```
- [ ] 4. Add a `phase_cargo_dependencies` function to `src/index/graph_loader.rs`:
  - Locate `Cargo.lock` at repo root.
  - Parse package entries.
  - Insert `GraphNode` nodes with `category = NodeKind::Package` and `id = urn:changeguard:package:{name}:{version}`.
  - Create `DependsOn` edges between dependent packages.
- [ ] 5. Wire `phase_cargo_dependencies` into the `build_native_graph` function in `graph_loader.rs`.

## Phase 3 — Green + Cleanup
- [ ] 6. Run `cargo nextest run --lib --bins --workspace` to ensure everything compiles and passes.
