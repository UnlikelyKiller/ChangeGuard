# Track U15 Plan: Split Semantic Concurrency + Always-Visible Diagnostics

- [x] Task U15.1: Add `parse_concurrency` and `embed_concurrency` fields to `SemanticConfig`; keep `concurrency` for back-compat; update `Default` and accessors.
- [x] Task U15.2: Add `> 0` validation for the two new fields in `src/config/validate.rs`.
- [x] Task U15.3: Add `[semantic]` template entries for the new fields in `src/config/defaults.rs`.
- [x] Task U15.4: Write `resolve_split_semantic_concurrency` in `src/semantic/concurrency.rs` honoring the precedence chain (CLI > new field > legacy > default).
- [x] Task U15.5: Update `HnswRefreshPlan` and `VectorStore` consumers to use the new resolver (or keep them on the old one and have `execute_semantic_index` translate).
- [x] Task U15.6: Move the `info!("Semantic indexing threads: ...")` log line above the empty-files early-exit at `src/commands/index.rs:612`.
- [x] Task U15.7: Add `--semantic-dry-run` to `IndexArgs` (clap `Option<Option<PathBuf>>`) in `src/cli.rs`.
- [x] Task U15.8: Implement `format_dry_run_report` in `src/commands/index.rs` using `comfy-table` (human) or `serde_json` (machine).
- [x] Task U15.9: Update `format_semantic_line` in `src/commands/config.rs` to display split fields when explicit.
- [x] Task U15.10: Write the red-phase tests: 5 in concurrency.rs, 2 model, 2 validate, 1 config, 1 dry-run report.
- [x] Task U15.11: Run the CI gate (fmt + clippy + nextest).
- [x] Task U15.12: End-to-end smoke test: dry-run, config split, log visibility, `cargo install --path .`.
- [x] Task U15.13: Ledger provenance + commit + push.
