## Plan: Track 19 — Reset and Recovery Completion

### Phase 0: Compatibility Guardrails
- [ ] Task 19.1: Keep `changeguard reset` as the command name and preserve default non-interactive behavior for derived-state cleanup only.
- [ ] Task 19.2: Add only additive flags; do not rename existing report/config paths.
- [ ] Task 19.3: Add an explicit confirmation mechanism for any mode that deletes `config.toml`, `rules.toml`, or the full `.changeguard/` tree.

### Phase 1: CLI and Command Wiring
- [ ] Task 19.4: Create `src/commands/reset.rs` with `execute_reset(...)`.
- [ ] Task 19.5: Register `reset` in `src/commands/mod.rs`.
- [ ] Task 19.6: Update `src/cli.rs` so `Reset` routes through `commands::reset` instead of printing a placeholder.
- [ ] Task 19.7: Add reset flags for `--remove-config`, `--remove-rules`, `--all`, and the explicit confirmation flag.

### Phase 2: Derived-State Cleanup
- [ ] Task 19.8: Implement cleanup of `.changeguard/logs/`, `.changeguard/tmp/`, `.changeguard/reports/`, and `.changeguard/state/` derived artifacts.
- [ ] Task 19.9: Explicitly remove SQLite artifacts (`ledger.db`, `ledger.db-wal`, `ledger.db-shm`) when present.
- [ ] Task 19.10: Preserve `config.toml` and `rules.toml` by default.
- [ ] Task 19.11: Implement optional removal for config/rules and full-tree removal for `--all`.

### Phase 3: Safety and Reporting
- [ ] Task 19.12: Canonicalize and validate reset targets so all filesystem mutations remain strictly bounded to `.changeguard/`.
- [ ] Task 19.13: Ensure reset is idempotent and performs deterministic target enumeration.
- [ ] Task 19.14: Continue best-effort cleanup on in-scope failures, then return actionable `miette` diagnostics with a deterministic summary.
- [ ] Task 19.15: Distinguish removed, absent, preserved, and failed items in user-facing output.

### Phase 4: Tests
- [ ] Task 19.16: Add integration tests for reset with missing state, populated state, repeated runs, and SQLite sidecars.
- [ ] Task 19.17: Add coverage for `--remove-config`, `--remove-rules`, and `--all`.
- [ ] Task 19.18: Add coverage proving destructive modes require explicit confirmation.
- [ ] Task 19.19: Add a boundedness test proving nothing outside `.changeguard/` is touched.

### Phase 5: Final Verification
- [ ] Task 19.20: `cargo fmt --check`
- [ ] Task 19.21: `cargo clippy --all-targets --all-features`
- [ ] Task 19.22: `cargo test -j 1 -- --test-threads=1`
