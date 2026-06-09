# Track X5 Plan: Security Child Node Orphan Pruning

## Phase 1 — Red (Failing Tests)
- [x] 1. Write unit test: insert a `principal` node whose ID contains a deleted-policy filename, run the pruning logic, assert node is gone.
- [x] 2. Write unit test: insert a `principal` node whose ID matches a valid policy filename, run pruning, assert node is preserved.

## Phase 2 — Implementation
- [x] 3. In `src/index/graph_loader.rs` Section 9, extend the existing pruning block to cascade to child categories. After the `policy` node prune:
  ```rust
  for child_category in &["principal", "action", "resource"] {
      if valid_cedar_filenames.is_empty() {
          let script = format!(
              "?[id] := *node{{id, category: '{}'}} :rm node {{id}}",
              child_category
          );
          let _ = cozo.run_script(&script);
      } else {
          if let Ok(res) = cozo.run_script(
              &format!("?[id] := *node{{id, category: '{}'}}", child_category)
          ) {
              let stale_ids: Vec<String> = res.rows.into_iter().filter_map(|row| {
                  if let Some(cozo::DataValue::Str(id)) = row.into_iter().next() {
                      let id_lower = id.to_lowercase();
                      let is_valid = valid_cedar_filenames.iter()
                          .any(|fname| id_lower.contains(fname.as_str()));
                      if !is_valid { Some(id.to_string()) } else { None }
                  } else { None }
              }).collect();
              let _ = cozo.remove_nodes_by_id(&stale_ids);
          }
      }
  }
  ```
- [x] 4. Add `info!("Cedar child node cleanup: pruned principal/action/resource orphans")` log line.

## Phase 3 — Green + Cleanup
- [x] 5. Run `changeguard index --analyze-graph`, confirm `security boundaries` shows no orphaned entries.
- [x] 6. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 7. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 8. Run `cargo fmt --all -- --check` — clean.
- [x] 9. Update `conductor/conductor.md` status to Completed.
