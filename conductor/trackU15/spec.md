# Track U15 Spec: Split Semantic Concurrency + Always-Visible Diagnostics

## Background

Track U14 (commit `d0edd27`) introduced a single `[semantic].concurrency` config field that drives *both* the rayon pool size for AST parsing and a separately-capped concurrent embed count. This conflation hides a real design choice: **CPU-bound parse work scales with logical cores, while network-bound embed work has a fixed crash threshold on the local ONNX server** (see commit `90da256`).

A second U14 friction point: the new `info!` log line `Semantic indexing threads: parse=N, embed_concurrency=M` sits *after* the `if files_to_process.is_empty()` early-exit. On incremental runs where nothing has changed, the line never fires and users have no visible signal that the new code is active.

A third UX gap: there is no dry-run mode for `index --semantic`. The only way to confirm a config change took effect is to actually run a real index, which on a large repo takes 4+ minutes including HNSW rebuild.

## Objective

Three concrete improvements, all in the `index --semantic` path:

1. **Split `[semantic].concurrency` into two independently-tunable fields**: `parse_concurrency` (rayon pool) and `embed_concurrency` (semaphore cap). Preserve backward-compat: if the legacy `concurrency` field is set, it populates both. Add `embed_concurrency` only if explicitly set, otherwise default to `DEFAULT_EMBED_CAP=4` unchanged.
2. **Move the Phase 2 thread-resolution log above the empty-files early-exit** so it fires on every `--semantic` invocation. Also surface the *resolution trace* (CLI → semantic.parse → semantic.embed → local_model.concurrency → auto) so users can see *why* a particular value was chosen.
3. **Add a `--semantic-dry-run` flag** that prints the resolved thread counts, embed cap, candidate file count, and estimated chunk count without doing any I/O. Uses clap's `Option<Option<PathBuf>>` pattern so `changeguard index --semantic --dry-run` works (no value) and `changeguard index --semantic --dry-run=/tmp/out.json` works (with optional output path for machine-readable consumption).

## Proposed Design

### 1. Split fields

In `src/config/model.rs`:

```rust
pub struct SemanticConfig {
    #[serde(default)]
    pub hnsw_rebuild_threshold: Option<usize>,
    /// Legacy combined field. If set and the split fields are not, populates both.
    #[serde(default)]
    pub concurrency: Option<usize>,
    /// Threads for CPU-bound AST parsing (rayon pool size).
    /// Independent from embed_concurrency because the workloads are different.
    #[serde(default)]
    pub parse_concurrency: Option<usize>,
    /// Cap on concurrent embed requests in flight. Defaults to DEFAULT_EMBED_CAP=4
    /// to stay below the local ONNX server's crash threshold.
    #[serde(default)]
    pub embed_concurrency: Option<usize>,
}
```

Resolution precedence per field (implemented in `resolve_semantic_concurrency` as two calls or a new `resolve_split_semantic_concurrency`):

```
parse_threads:    CLI -j > [semantic].parse_concurrency > [semantic].concurrency (legacy) > [local_model].concurrency > auto
embed_concurrency: CLI --embed-concurrency > [semantic].embed_concurrency > [semantic].concurrency (legacy) > DEFAULT_EMBED_CAP=4
```

The legacy `concurrency` field stays in the struct (marked `#[serde(default)]`) so existing user configs keep working. A deprecation log line is emitted when it's the value-driving key.

### 2. Always-visible log

In `src/commands/index.rs`, move the `info!("Semantic indexing threads: ...")` call to immediately after `info!("Indexing repository for semantic search...")` (line 545) and **before** the candidate-collection phase. Also log the resolution chain at `debug!` level so verbose output shows the precedence walk.

### 3. Dry-run flag

In `src/cli.rs`, add to the `Index` subcommand:

```rust
/// Print resolved semantic settings and exit. Optionally takes a path for
/// machine-readable JSON output (uses comfy-table for human, serde_json for JSON).
#[arg(long, value_name = "OUTPUT_PATH", num_args = 0..=1)]
pub semantic_dry_run: Option<Option<PathBuf>>,
```

In `src/commands/index.rs::execute_index`, branch on `args.semantic_dry_run` *before* any side effects (file walk, DB init). The dry-run report includes:

- Resolved `parse_threads` and `embed_concurrency` with the chain that produced each
- Candidate file count (walks the repo but does not parse or embed)
- Estimated chunk count (cheap: counts lines / file-size, doesn't call AstChunker)
- Detected embedding model + dimensions
- HNSW rebuild threshold + whether the batch would trigger a rebuild
- Memory graph: vector count, file count

Output to stdout (human table via `comfy-table`) or to the file path (JSON via `serde_json::to_string_pretty`).

## Critical files

| File | Change |
|---|---|
| `src/config/model.rs` | Add `parse_concurrency` and `embed_concurrency` fields; update `Default`; add `semantic_parse_concurrency()` and `semantic_embed_concurrency()` accessors |
| `src/config/defaults.rs` | Add commented `[semantic]` block entries for the new fields |
| `src/config/validate.rs` | Validate `> 0` for both new fields |
| `src/semantic/concurrency.rs` | New `resolve_split_semantic_concurrency(cli_parse, cli_embed, config) -> ResolvedConcurrency` that handles the precedence chain |
| `src/commands/index.rs` | Move thread log above early-exit; add dry-run path; refactor `execute_semantic_index` to use the new resolver |
| `src/commands/config.rs` | Update `format_semantic_line` to show split fields when explicit |
| `src/cli.rs` | Add `--semantic-dry-run` flag with optional value |
| `Cargo.toml` | **No new deps.** `comfy-table` (7.2.2) and `serde_json` (1.0) are already in the tree. |

## Existing utilities to reuse

- `crate::semantic::concurrency::{resolve_semantic_concurrency, ResolveOptions, ResolvedConcurrency, EmbedSemaphore, DEFAULT_EMBED_CAP}` — the U14 work
- `std::thread::available_parallelism` — already used at `src/commands/index.rs:623`
- `crate::semantic::chunker::AstChunker::chunk_file` — already used at `src/commands/index.rs:652`
- `comfy_table::Table` for human output (already used in `src/commands/audit.rs`)
- `clap` `Option<Option<T>>` pattern (verified against clap 4.6.1 docs) for `--semantic-dry-run[=<path>]`

## TDD plan (Red → Green)

**Red phase — write tests first:**

1. `src/semantic/concurrency.rs`:
   - `legacy_concurrency_populates_both_when_split_unset`
   - `split_field_overrides_legacy`
   - `parse_and_embed_resolve_independently`
   - `embed_defaults_to_4_when_unset`
   - `cli_embed_override_wins_over_config`
2. `src/config/model.rs`: TOML deserialization for `[semantic] parse_concurrency = 6` and `embed_concurrency = 2`
3. `src/config/validate.rs`: `test_zero_parse_concurrency_fails`, `test_zero_embed_concurrency_fails`
4. `src/commands/config.rs`: `format_semantic_line` shows split fields when both set
5. `src/commands/index.rs`: new `format_dry_run_report` unit test

**Green phase — implement:**

1. Add the three fields to `SemanticConfig`
2. Update `Default` and accessors
3. Update validation
4. Add `resolve_split_semantic_concurrency` in `src/semantic/concurrency.rs`
5. Refactor `src/commands/index.rs:execute_semantic_index` to use the new resolver
6. Move the `info!` log line above the empty check
7. Add `--semantic-dry-run` to `IndexArgs` + CLI flag
8. Add dry-run report generator
9. Update `format_semantic_line` in `src/commands/config.rs`

## Verification

1. **Unit tests** (the red/green ones above) — must pass.
2. **CI gate** — must pass:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo nextest run --lib --bins --workspace
   ```
3. **Manual end-to-end via global binary**:
   - `changeguard index --semantic --dry-run` → human-readable table, no side effects
   - `changeguard index --semantic --dry-run=/tmp/report.json` → JSON file written
   - `changeguard index --semantic --incremental` → Phase 2 log line *always* visible
   - With `[semantic] parse_concurrency = 6` and `embed_concurrency = 2`: both values reflected in dry-run output
   - With legacy `[semantic] concurrency = 4`: dry-run shows `parse=4, embed=4 (legacy)` with a deprecation note
4. **`changeguard config verify`** shows the new fields when explicit
5. **Ledger lifecycle**: start, verify, commit, push

## Why this scope

This is the highest-leverage work in the opportunity list because:
- The conflation of parse vs embed is a real correctness/design miss (opportunity #5)
- The hidden log line makes U14 look like a no-op on most runs (opportunity #2)
- The dry-run mode unlocks all future tuning work — no more 4-minute diagnostic runs (opportunity #3)
- Splitting the config field requires zero new infrastructure — clap is already type-driven

## Out of scope

- Adaptive tuning based on past run duration (would need a metrics store)
- Per-file-type concurrency (e.g., different pools for Rust vs Python) — would need per-language chunker hooks
- The `comfy_table` table styling polish — keep the dry-run report functional, not pretty

## References

- clap derive docs: https://docs.rs/clap/latest/clap/_derive/
- comfy-table 7.x: https://github.com/Nukesor/comfy-table (already at 7.2.2 in lockfile)
- tracing best practices (perl-lsp, spotifai migrations): info! for lifecycle, debug! for routine ops
- Predecessor work: U14 commit `d0edd27`, U11 commit `937e62d`, ONNX cap from commit `90da256`
