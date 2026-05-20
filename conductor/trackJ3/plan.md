# Track J3 Plan: Temporal Coupling Row Cap and Relevance Filter

## Steps

### Red Phase (failing tests)
1. [ ] Add unit test in `src/impact/enrichment/coupling.rs`: build a coupling list where only 2 of 10 pairs involve a "changed file"; assert that after filtering exactly 2 remain
2. [ ] Add unit test: build 200 relevant coupling pairs; with `max_coupling_pairs = 50`, assert result len ≤ 50
3. [ ] Add unit test: `max_coupling_pairs = 0` with 200 pairs → returns all 200 (no cap)
4. [ ] Add unit test: `changed_files` is empty → no relevance filter applied, cap still applies
5. [ ] Run CI gate — new tests expected to fail

### Green Phase (implementation)
6. [ ] Locate `TemporalConfig` struct (likely `src/config/model.rs`); add `max_coupling_pairs: usize` with `serde(default)` returning `50`
7. [ ] In `enrich_temporal()`: after getting couplings, build `HashSet<&str>` from `changed_files`
8. [ ] Apply relevance filter: `if !changed_files.is_empty()` then keep only pairs where `file_a` or `file_b` is in the set
9. [ ] Apply cap: if `config.temporal.max_coupling_pairs > 0`, truncate to that length; emit `debug!` with counts
10. [ ] Add `max_coupling_pairs = 50` to `.changeguard/config.toml` under `[temporal]`
11. [ ] Run `cargo build` — fix any type errors
12. [ ] Run CI gate — all tests expected to pass

### Verification
13. [ ] `cargo install --path .` to rebuild binary
14. [ ] `changeguard scan --impact` with agent dotfiles present → ≤ 50 coupling rows in output
15. [ ] Verify all shown rows involve a changed file
16. [ ] `RUST_LOG=debug changeguard scan --impact` → shows before/after/cap counts in stderr
17. [ ] `changeguard verify` passes

### Finalization
18. [ ] Mark all tasks complete; update `conductor/conductor.md` status to Completed
19. [ ] `changeguard ledger commit` with summary and reason
