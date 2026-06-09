# Track X13 Plan: `security boundaries` Summary + Empty Hint

## Phase 1 — Implementation
- [ ] 1. In `src/commands/security.rs` `execute_security_boundaries`, after fetching all boundary nodes, build category counts:
  ```rust
  let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
  for row in &rows {
      *counts.entry(row.category.clone()).or_insert(0) += 1;
  }
  ```
- [ ] 2. If `rows.is_empty()` and `!json`:
  ```rust
  println!("  {}", "No security boundary data found.".yellow());
  println!("  Add Cedar policy files to 'policies/' and run {}.",
      "changeguard index --analyze-graph".cyan().bold());
  return Ok(());
  ```
- [ ] 3. Otherwise print header with counts:
  ```rust
  let summary = ["policy","principal","action","resource"].iter()
      .map(|k| format!("{} {}s", counts.get(*k).unwrap_or(&0), k))
      .collect::<Vec<_>>().join(" | ");
  println!("{}", format!("Security Boundaries  [{}]", summary).bold().green());
  ```
- [ ] 4. In JSON path, wrap in `{"meta": {"counts": {...}}, "boundaries": [...]}`.

## Phase 2 — Verification
- [ ] 5. Run `changeguard security boundaries` with empty KG — confirm empty hint.
- [ ] 6. Run `changeguard security boundaries` after adding a Cedar policy — confirm counts.
- [ ] 7. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 8. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 9. Run `cargo fmt --all -- --check` — clean.
- [ ] 10. Update `conductor/conductor.md` status to Completed.
