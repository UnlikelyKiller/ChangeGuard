# Track Y3 Plan: Consolidate `scan --impact` vs Standalone `impact`

## Phase 1 — Add `--json` and `--out` to Standalone `impact`
- [ ] 1. Add `--json` and `--out <path>` flags to `ImpactArgs` in `src/cli.rs`.
- [ ] 2. In `execute_impact` in `src/commands/impact.rs`: when `--json`, serialize `ImpactPacket` to stdout using existing `serde` impl; when `--out <path>`, write to the specified path.
- [ ] 3. Merge `execute_impact_silent()` path into the main `execute_impact` with a `silent` parameter.

## Phase 2 — Deduplicate Internal Code Paths
- [ ] 4. Extract shared "compute ImpactPacket" logic into a single `compute_impact()` function called by both `scan --impact` and standalone `impact`.
- [ ] 5. Ensure `scan --impact --json` calls the same output serializer as `impact --json`.
- [ ] 6. Add cross-reference in help text: `scan --impact` help mentions `impact`, and vice versa.

## Phase 3 — Verification
- [ ] 7. Run `changeguard impact --json` and pipe through `jq` — confirm parseable.
- [ ] 8. Run `changeguard impact --out test.json` — confirm file created and parseable.
- [ ] 9. Compare `changeguard scan --impact --json` with `changeguard impact --json` — confirm identical.
- [ ] 10. Write integration test `test_impact_standalone_json` and `test_impact_out_file`.
- [ ] 11. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 12. Run `cargo nextest run --test integration` — all pass.
- [ ] 13. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 14. Run `cargo fmt --all -- --check` — clean.
- [ ] 15. Update `conductor/conductor.md` status to Completed.