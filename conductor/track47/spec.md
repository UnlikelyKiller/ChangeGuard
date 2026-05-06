# Track 47: Harden Service Detection — Renames, Root-Level, and Cross-Service Collisions

## Overview
Three distinct service-detection weaknesses identified by Codex:

1. **Rename/delete blind spot**: `ServiceProvider::enrich()` at `src/impact/enrichment/services.rs:28` looks up `service_name` using `change.old_path.unwrap_or(change.path)`. This handles renames by checking the old path. However, `map_snapshot_to_packet()` in `src/commands/impact.rs:156-157` only sets `old_path` for `ChangeType::Renamed`. For `Deleted` files, `old_path` is `None` and the current `change.path` is used — but deleted files may have been removed from `project_files` by a prior re-index, so the lookup returns NULL. The result: deleted files under-report affected services.

2. **Root-level `LIKE '%'` stampede**: In `src/index/project_index.rs:1380-1396`, when a service has an empty or `"."` directory, the SQL uses `WHERE file_path NOT LIKE '%/%' AND service_name IS NULL`. This means any root-level handler inadvertently claims all root-level files. In monorepos, a root handler can stamp unrelated root-level files with the wrong `service_name`, making impact results order-dependent.

3. **Cross-service edge name collisions**: `compute_cross_service_edges()` in `src/coverage/services.rs:228-236` builds `symbol_to_service` mapping using bare symbol names. Two services with handlers named `index` or `health` will collide — the last writer wins. The schema already stores `qualified_name` in `project_symbols`; using it would eliminate ambiguity.

## Objectives
- Include `old_path` from `ChangeType::Deleted` in the packet's `ChangedFile` so deleted files are traced to their former service.
- Guard root-level service assignment against claiming the entire repo: only match files that are in the root-level directory AND not already assigned to deeper services.
- Use `qualified_name` (with file-path disambiguation) in `compute_cross_service_edges` and the enrichment provider's edge loading.

## Success Criteria
- Deleted files trigger affected-service detection via their former path.
- Root-level services only claim files truly at the repo root; deeper services take precedence.
- Cross-service edges never collide on duplicate bare symbol names between services.
- New tests: rename + delete coverage, root-level + monorepo, duplicate symbol names across services.
- CI gate passes.

## Architecture
- `src/commands/impact.rs` — `map_snapshot_to_packet()`: set `old_path` for Deleted files too.
- `src/impact/enrichment/services.rs` — `ServiceProvider::enrich()`: lookup is already correct via `old_path.unwrap_or(&change.path)`.
- `src/index/project_index.rs` — `index_services()`: tighten root-level SQL; sort services by depth descending before assignment.
- `src/coverage/services.rs` — `compute_cross_service_edges()`: use `qualified_name` instead of bare name.
- `src/impact/enrichment/services.rs` — Edge loading query already uses `COALESCE(ps_caller.qualified_name, ps_caller.symbol_name)`; verify this is sufficient.

## Testing Strategy
- **Red commit**: Tests for deleted-file service tracing, root-level monorepo isolation, duplicate-name edge resolution.
- **Green commit**: Implement fixes. Verify all tests pass.
