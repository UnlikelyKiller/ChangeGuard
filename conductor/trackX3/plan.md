# Track X3 Plan: `hotspots explain` Path Normalization

## Phase 1 — Red (Failing Tests)
- [x] 1. Write unit test `test_explain_resolves_relative_path`: mock a project_files row with absolute path, call the explain complexity query with a relative path, assert result > 0.
- [x] 2. Write unit test `test_explain_exact_file_filter`: create a hotspot for an exact file, call `calculate_hotspots` with exact-file filter, assert one result returned.

## Phase 2 — Implementation
- [x] 3. In `execute_hotspots_explain` (`src/commands/hotspots.rs`):
  - After receiving `entity: String`, resolve it to an absolute path:
    ```rust
    let abs_entity = {
        let p = std::path::Path::new(&entity);
        if p.is_absolute() { entity.clone() }
        else {
            layout.root.join(&entity).to_string_lossy().into_owned()
        }
    };
    ```
  - Use `abs_entity` in the complexity SQL query parameter.
- [x] 4. Add a new `HotspotQuery` field `exact_file: Option<String>` (or reuse `dir_filter` as an exact-file sentinel) in `src/impact/hotspots.rs`. When `exact_file` is set, filter git log entries to only those where the changed file matches exactly.
- [x] 5. In `execute_hotspots_explain`, pass `exact_file: Some(abs_entity.clone())` to `HotspotQuery` instead of `dir_filter`.
- [x] 6. Update temporal coupling filter: normalize `c.file_a` and `c.file_b` to the same form before comparing to `abs_entity`.
- [x] 7. When both complexity and frequency are 0 after normalization, print: `"No indexed data for '{}'. Run 'changeguard index' to populate metrics."`.

## Phase 3 — Green + Cleanup
- [x] 8. Test locally: `changeguard hotspots explain src/commands/hotspots.rs` shows non-zero values.
- [x] 9. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 10. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 11. Run `cargo fmt --all -- --check` — clean.
- [x] 12. Update `conductor/conductor.md` status to Completed.
