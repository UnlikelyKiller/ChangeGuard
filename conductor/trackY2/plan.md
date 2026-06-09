# Track Y2 Plan: Standardize JSON Output Contract

## Phase 1 — Audit & Fix Mixed Output
- [ ] 1. Audit every `--json` code path for interleaved human text on stdout. Fix by moving `println!` / `print!` calls for status text behind `if !json` guards.
- [ ] 2. Add a `--quiet` flag (suppress stderr hints) for scripting use.

## Phase 2 — Add Missing `--json` Flags
- [ ] 3. Add `--json` flag to `LedgerSearchArgs` and `LedgerStatusArgs` in `src/cli.rs`.
- [ ] 4. Implement `execute_ledger_search_json`: serialize search results to JSON, write to stdout.
- [ ] 5. Implement `execute_ledger_status_json`: serialize status (pending count, drift count, federated count) to JSON.
- [ ] 6. Wire both to their respective execute functions.

## Phase 3 — Standalone `impact --json` and `--out`
- [ ] 7. Add `--json` and `--out <path>` flags to `ImpactArgs` in `src/cli.rs`.
- [ ] 8. In `execute_impact`, when `--json` is set, serialize `ImpactPacket` to stdout instead of human output.
- [ ] 9. When `--out` is set, write to the specified path instead of the default reports directory.

## Phase 4 — Verification
- [ ] 10. Run `changeguard X --json | jq` for every JSON-supporting command — confirm no parsing errors.
- [ ] 11. Write integration test `test_json_output_pipeable`: invoke 3 surfaces with `--json`, pipe through `jq`, assert exit 0.
- [ ] 12. Run `cargo nextest run --lib --bins --workspace` — all pass.
- [ ] 13. Run `cargo nextest run --test integration` — all pass.
- [ ] 14. Run `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- [ ] 15. Run `cargo fmt --all -- --check` — clean.
- [ ] 16. Update `conductor/conductor.md` status to Completed.