# Track Y5 Plan: CLI UX Consistency

## Phase 1 — `ledger start --category` Enum
- [ ] 1. Change `LedgerStartArgs.category` type from `Option<String>` to `Option<Category>` in `src/cli.rs`.
- [ ] 2. Update `execute_ledger_start` in `src/commands/ledger.rs` to convert `Category` enum back to string for storage (remove the manual `to_string()` on a String).
- [ ] 3. Remove any `category.to_string()` workarounds now handled by `ValueEnum`.
- [ ] 4. Write integration test: `changeguard ledger start test-entity --category invalid` → clap rejects with "invalid value".

## Phase 2 — `verify --dry-run` Plan Output
- [ ] 5. In `src/commands/verify.rs`, after building the `VerificationPlan` and before returning "skip" message:
  ```rust
  if dry_run {
      print_verify_plan(&plan);
      return Ok(());
  }
  ```
- [ ] 6. Ensure `print_verify_plan` from `src/output/human.rs` is imported.
- [ ] 7. Write integration test: run `verify --dry-run`, assert output contains `Verification Plan`.

## Phase 3 — Risk Analysis Env-Var Identity
- [ ] 8. In `src/impact/packet.rs`, add `old_env_vars: Vec<String>` and `new_env_vars: Vec<String>` to `RuntimeUsageDelta` (or equivalent structure).
- [ ] 9. In the risk scoring orchestration (`src/impact/orchestrator.rs`), compute set difference between old and new env vars:
  ```rust
  let added: HashSet<_> = new.iter().filter(|v| !old.contains(v)).collect();
  let removed: HashSet<_> = old.iter().filter(|v| !new.contains(v)).collect();
  ```
- [ ] 10. When `added` or `removed` is non-empty (even if counts match), emit a risk reason like `"Environment variable changed: {added:?} added, {removed:?} removed"`.
- [ ] 11. Verify `test_runtime_delta_same_cardinality_not_flagged` test is updated (or new test added that specifically checks identity-aware detection).

## Phase 4 — Verification
- [ ] 12. Run `changeguard ledger start test --category bad` — confirm clap rejects.
- [ ] 13. Run `changeguard verify --dry-run` — confirm plan output.
- [ ] 14. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 15. Run `cargo nextest run --test integration` — all pass.
- [ ] 16. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 17. Run `cargo fmt --all -- --check` — clean.
- [ ] 18. Update `conductor/conductor.md` status to Completed.