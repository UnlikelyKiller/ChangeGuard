# Track X9 Plan: Empty-State Hints

## Phase 1 — Implementation

### observability coverage
- [x] 1. In `src/commands/observability.rs`, locate the coverage table render section. After building `rows`, check if empty:
  ```rust
  if rows.is_empty() && !json {
      println!("  {}", "No observability coverage data found.".yellow());
      println!("  Run {} to populate.", "changeguard index --analyze-graph".cyan().bold());
      return Ok(());
  }
  ```

### deploy impact --changed
- [x] 2. In `src/commands/deploy.rs`, locate the impact output section. After building the impact result list, check if empty (non-JSON):
  ```rust
  if results.is_empty() && !json {
      println!("  {}", "No deployment impact detected for current changes.".yellow());
  }
  ```

### tests <file>
- [x] 3. In `src/commands/test_mapping.rs`, after building `mappings`, check if empty (non-JSON):
  ```rust
  if mappings.is_empty() && !json {
      println!("  {}", format!("No test mappings found for '{}'.", entity).yellow());
      println!("  Run {} to populate test mappings.", "changeguard index".cyan().bold());
  }
  ```

## Phase 2 — Verification
- [x] 4. Verify each command manually with an empty state.
- [x] 5. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 6. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 7. Run `cargo fmt --all -- --check` — clean.
- [x] 8. Update `conductor/conductor.md` status to Completed.
