# Track X12 Plan: Temporal Coupling Directory Filter

## Phase 1 — Implementation
- [ ] 1. Add a helper `fn is_file_path(p: &std::path::Path) -> bool`:
  ```rust
  fn is_file_path(p: &std::path::Path) -> bool {
      p.extension().is_some()
  }
  ```
- [ ] 2. In `execute_hotspots_explain`, after collecting `entity_couplings`, add a second filter pass:
  ```rust
  let file_couplings: Vec<_> = entity_couplings.iter().filter(|c| {
      let other = if c.file_a.to_string_lossy() == entity { &c.file_b } else { &c.file_a };
      is_file_path(other.as_std_path())
  }).collect();
  let dir_count = entity_couplings.len() - file_couplings.len();
  ```
- [ ] 3. Use `file_couplings.len()` in the `Temporal Couplings: N` display.
- [ ] 4. After the Top Couplings block, if `dir_count > 0`:
  ```rust
  println!("  {}", format!("({} directory-level entries hidden)", dir_count).dimmed());
  ```

## Phase 2 — Verification
- [ ] 5. Run `changeguard hotspots explain <most-coupled-file>`, confirm no directory entries in Top Couplings.
- [ ] 6. Confirm hidden-count note appears when directories were filtered.
- [ ] 7. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 8. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 9. Run `cargo fmt --all -- --check` — clean.
- [ ] 10. Update `conductor/conductor.md` status to Completed.
