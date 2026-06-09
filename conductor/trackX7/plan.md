# Track X7 Plan: `doctor` Embedding Model Name Placeholder

## Phase 1 — Implementation (no failing test needed — display only)
- [ ] 1. In `src/commands/doctor.rs`, extract a helper at the top of `execute_doctor`:
  ```rust
  fn model_name_or_placeholder(name: &str) -> String {
      if name.is_empty() {
          "(not configured)".yellow().to_string()
      } else {
          name.to_string()
      }
  }
  ```
- [ ] 2. In the embedding probe `Ok(dims)` arm (line ~45), replace:
  ```rust
  config.local_model.embedding_model,
  ```
  with:
  ```rust
  model_name_or_placeholder(&config.local_model.embedding_model),
  ```
- [ ] 3. Apply the same to `generation_model` in the completion probe display.

## Phase 2 — Verification
- [ ] 4. Temporarily remove `embedding_model` from `config.toml`, run `changeguard doctor`, confirm `(not configured)` appears in yellow.
- [ ] 5. Restore config, run `changeguard doctor`, confirm normal model name displays.
- [ ] 6. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 7. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 8. Run `cargo fmt --all -- --check` — clean.
- [ ] 9. Update `conductor/conductor.md` status to Completed.
