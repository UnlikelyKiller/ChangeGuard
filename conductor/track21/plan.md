## Plan: Track 21 — Verification Process Hardening

### Phase 0: Compatibility Guardrails
- [ ] Task 21.1: Preserve support for existing `required_verifications` string commands and manual `--command`.
- [ ] Task 21.2: Decide whether runner structure changes can remain additive; if not, document the schema/report impact before implementation.

### Phase 1: Verify Module Structure
- [ ] Task 21.3: Create `src/verify/runner.rs` and move execution logic out of `commands/verify.rs`.
- [ ] Task 21.4: Create `src/verify/timeouts.rs` and centralize timeout defaults.
- [ ] Task 21.5: Update `src/verify/mod.rs` exports.
- [ ] Task 21.6: Reduce `commands/verify.rs` to orchestration, report writing, and user-facing flow.

### Phase 2: Structured Execution
- [ ] Task 21.7: Introduce a structured execution model that can coexist with the current string command representation.
- [ ] Task 21.8: Execute structured steps without `cmd /C` or `sh -c` where possible.
- [ ] Task 21.9: Keep manual `--command` on an explicit shell-fallback path, clearly separated from auto-planned steps.
- [ ] Task 21.10: Add an explicit compatibility path for legacy rule strings that cannot yet be losslessly tokenized.

### Phase 3: Process Policy and Timeouts
- [ ] Task 21.11: Apply `platform::process_policy::check_policy(...)` before verification execution using the actual executable identity where possible.
- [ ] Task 21.12: Add actionable diagnostics for denied commands and shell-fallback execution mode.
- [ ] Task 21.13: Make timeout selection flow through policy/timeouts helpers rather than scattered literals.
- [ ] Task 21.14: Normalize runner environment/current-dir/stdin-stdout-stderr handling so execution is deterministic enough for tests.

### Phase 4: Tests
- [ ] Task 21.15: Add tests for direct-process execution, manual shell execution, timeout handling, policy denial, and legacy rule-string execution.
- [ ] Task 21.16: Add `tests/platform_windows.rs` for Windows-specific verification and path/policy seams.
- [ ] Task 21.17: Add `tests/platform_wsl.rs` or conditionally executed WSL seam tests where feasible.

### Phase 5: Documentation
- [ ] Task 21.18: Update verify-related docs/comments to explain direct execution, shell fallback, and compatibility constraints.

### Phase 6: Final Verification
- [ ] Task 21.19: `cargo fmt --check`
- [ ] Task 21.20: `cargo clippy --all-targets --all-features`
- [ ] Task 21.21: `cargo test -j 1 -- --test-threads=1`
