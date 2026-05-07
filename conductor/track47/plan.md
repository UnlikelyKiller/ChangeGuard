# Implementation Plan - Track 47: Harden Service Detection

## Goal
Close three service-detection gaps: renames/deletes, root-level LIKE '%', and cross-service edge name collisions.

## Proposed Changes

### 1. Deleted File Service Tracing [src/commands/impact.rs]
- In `map_snapshot_to_packet()` at line 155, change `ChangeType::Deleted` to also capture `old_path`:
  ```rust
  ChangeType::Deleted => ("Deleted".to_string(), Some(c.path.clone())),
  ```
  This preserves the former path of deleted files so `ServiceProvider::enrich()` can look up their previous `service_name`.
- The `ServiceProvider::enrich()` lookup at `src/impact/enrichment/services.rs:28` already uses `change.old_path.as_ref().unwrap_or(&change.path)` — no change needed there.

### 2. Root-Level Service Containment [src/index/project_index.rs]
- The depth-sorting already exists at lines 1373-1378 (deepest services first). This is correct.
- The root-level guard at lines 1382-1386 uses `file_path NOT LIKE '%/%'` which correctly matches only files with no path separator. This is actually correct behavior — it only matches root-level files.
- The issue Codex flagged is actually more subtle: when `dir_str.is_empty()` or `dir_str == "."` fires for a service with an empty directory, the query `file_path NOT LIKE '%/%'` is used. This is a reasonable heuristic but could be tightened by requiring that the root service directory is actually at the repo root.
- **Change**: Add a `root_services` detection step: if a service has an empty/`.` directory, only match files where extracting the parent directory from `file_path` yields an empty string or `.` (i.e., truly root-level files). The current query is already correct for this case.
- **Additional safety**: Add a guard in `infer_services()` that prevents empty-directory services from being created unless there are routes/files that are genuinely at the repo root.

### 3. Cross-Service Edge Name Collision Fix [src/coverage/services.rs]
- In `compute_cross_service_edges()`, change the `symbol_to_service` mapping from bare name to `(name, file_path)` or use qualified name:
  ```rust
  // Change from:
  symbol_to_service.insert(route.clone(), service.name.clone());
  // To a file-disambiguated key:
  // Use the service directory + symbol name as the key
  ```
- Since `Service` doesn't carry file paths per route (only route names), the disambiguation must happen at the service level. The simplest fix: when building `symbol_to_service`, prefix the symbol with the service name to create a composite key. Then lookup uses the same prefixing.
- Alternative simpler fix: Instead of storing bare symbol names, store `qualified_name` when available from the call graph edges. The enrichment provider at `src/impact/enrichment/services.rs:118-121` already loads `COALESCE(ps_caller.qualified_name, ps_caller.symbol_name)` — so qualified names are already being used there. The `compute_cross_service_edges()` in `services.rs` receives a `CallGraph` whose edges may have bare names.
- **Fix**: Update `compute_cross_service_edges()` to use `(service_name, symbol_name)` tuples as keys instead of bare symbol names.

### 4. Tests
- `tests/service_detection.rs` [NEW]:
  - `test_deleted_file_service_tracing`: Create a repo, index it with services, simulate a deletion, verify affected service is detected.
  - `test_root_service_containment`: Monorepo with root handler + nested service. Verify root service doesn't claim nested files.
  - `test_cross_service_edge_dedup_by_name`: Two services both have handler `index`. Verify edges resolve to distinct services.
  - `test_rename_service_detection`: Renamed file traced to correct service via old_path.

## Verification Plan

### Automated Tests
- `cargo test --test service_detection`
- `cargo test --workspace`

## Definition of Done (DoD)
- [x] **Deleted File Tracing**: Deleted files contribute to `affected_services` via `old_path`.
- [x] **Root Service Containment**: Root-level services don't claim files in subdirectories.
- [x] **Edge Name Uniqueness**: Symbol name collisions between services don't cause edge misattribution.
- [x] **Test Coverage**: 4+ new integration tests covering the three gaps.
- [x] **Zero Regression**: All existing tests pass.
- [x] **Clean CI**: `cargo fmt`, `cargo clippy`, full test suite pass.
