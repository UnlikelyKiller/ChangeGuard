## Plan: Track 22 — Structural Completion and Plan Reconciliation

### Phase 0: Evidence and Scope Check
- [ ] Task 22.1: Review each remaining plan/file gap against current code usage before adding new modules or schema.
- [ ] Task 22.2: For each gap, decide explicitly between implement, shim, or documented deferral.

### Phase 1: Scan Diff-Summary Integration
- [ ] Task 22.3: Decide where diff summaries belong in scan output/reporting without making scan noisy.
- [ ] Task 22.4: Wire `src/git/diff.rs` into scan or a scan-adjacent report path with deterministic truncation.
- [ ] Task 22.5: Add tests for diff-summary generation and capped output behavior.

### Phase 2: Index Storage and Symbol Persistence
- [ ] Task 22.6: Decide whether persisted symbol storage has a concrete consumer in the current product.
- [ ] Task 22.7: If yes, create `src/index/storage.rs` as the symbol persistence seam and add only the minimum schema/hooks needed.
- [ ] Task 22.8: If not, add the seam or documentation needed to make the deferral explicit without speculative DB growth.
- [ ] Task 22.9: Add tests for whichever path is chosen: persistence round-trips or explicit architectural deferral coverage in docs/tests where feasible.

### Phase 3: Remaining Planned Module Gaps
- [ ] Task 22.10: Create `src/index/normalize.rs` only if normalization logic is now shared; otherwise document why it remains unnecessary.
- [ ] Task 22.11: Create `src/gemini/wrapper.rs` and move/re-export wrapper logic from `gemini/mod.rs` only if it improves ownership clarity without churn.
- [ ] Task 22.12: Create `src/output/table.rs` or an explicit shim if table configuration remains intentionally local.
- [ ] Task 22.13: Add minimal `src/util/fs.rs`, `src/util/hashing.rs`, `src/util/process.rs`, and `src/util/text.rs` seams only where existing duplication justifies them.
- [ ] Task 22.14: Decide whether `src/state/locks.rs` is a minimal placeholder or an explicitly documented deferral.

### Phase 4: Documentation Reconciliation
- [ ] Task 22.15: Create `docs/prd.md`.
- [ ] Task 22.16: Add `docs/implementation-plan.md` or a clearly documented alias/redirect to `docs/Plan.md`.
- [ ] Task 22.17: Update architecture/conductor docs for any explicit deferrals adopted in this track.

### Phase 5: Test and CI Gaps
- [ ] Task 22.18: Add at least one black-box CLI test invoking the built binary.
- [ ] Task 22.19: Strengthen scan-related assertions beyond `is_ok()`.
- [ ] Task 22.20: Add `cargo deny` to CI or explicitly document deferral in the repo docs and conductor.

### Phase 6: Final Verification
- [ ] Task 22.21: `cargo fmt --check`
- [ ] Task 22.22: `cargo clippy --all-targets --all-features`
- [ ] Task 22.23: `cargo test -j 1 -- --test-threads=1`
