# Track X12 Plan: Temporal Coupling Directory Filter

## Phase 1 — Implementation
- [x] 1. Add a helper `fn is_file_path(p: &std::path::Path) -> bool`:
  ```rust
  fn is_file_path(p: &std::path::Path) -> bool {
      p.extension().is_some()
  }
  ```
- [x] 2. In `execute_hotspots_explain`, after collecting `entity_couplings`, add a second filter pass:
  ```rust
  let file_couplings: Vec<_> = entity_couplings.iter().filter(|c| {
      let other = if c.file_a.to_string_lossy() == entity { &c.file_b } else { &c.file_a };
      is_file_path(other.as_std_path())
  }).collect();
  let dir_count = entity_couplings.len() - file_couplings.len();
  ```
- [x] 3. Use `file_couplings.len()` in the `Temporal Couplings: N` display.
- [x] 4. After the Top Couplings block, if `dir_count > 0`:
  ```rust
  println!("  {}", format!("({} directory-level entries hidden)", dir_count).dimmed());
  ```

## Phase 2 — Verification
- [x] 5. Run `changeguard hotspots explain <most-coupled-file>`, confirm no directory entries in Top Couplings.
- [x] 6. Confirm hidden-count note appears when directories were filtered.
- [x] 7. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [x] 8. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [x] 9. Run `cargo fmt --all -- --check` — clean.
- [x] 10. Update `conductor/conductor.md` status to Completed.
