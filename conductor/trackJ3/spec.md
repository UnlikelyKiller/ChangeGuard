# Track J3: Temporal Coupling Row Cap and Relevance Filter

## Status
Planned

## Milestone
J: Developer Experience Hardening

## Problem
`changeguard scan --impact` and `changeguard impact` produce 500+ temporal coupling rows when agent dotfiles (`.claude/`, `.codex/`) are present in git history, because:

1. `enrich_temporal()` in `src/impact/enrichment/coupling.rs` calls `temporal_engine.calculate_couplings()` which returns **all** historical coupling pairs with no cap.
2. There is no filter to restrict results to pairs where at least one file is in the current changeset.
3. Agent dotfiles are committed together with code, producing hundreds of 100%-coupled pairs (`.claude/memory/x.md` ↔ `.claude/memory/y.md`) that are noise from a risk-triage perspective.

The result is an impact report that is practically unreadable and takes significantly longer to render.

## Fix Strategy
Two-pronged:

1. **Relevance filter**: Only surface coupling pairs where **at least one file is in `changed_files`** (the current scan's changeset). Cross-dotfile coupling that does not involve a changed file is irrelevant.
2. **Row cap**: After filtering, cap the result set at `max_coupling_pairs` (configurable, default 50). Emit a `debug!` log when rows are trimmed.

The cap and filter are both applied in `enrich_temporal()` before building the `ProviderResult`. The underlying `calculate_couplings()` is unchanged to avoid breaking other callers.

## Scope of Changes

### 1. `src/impact/enrichment/coupling.rs`
- After receiving couplings from `temporal_engine.calculate_couplings()`, filter to pairs where `pair.file_a` or `pair.file_b` is in `changed_files` (convert to a `HashSet<&str>` for O(1) lookup).
- Apply `config.temporal.max_coupling_pairs` cap (default 50) after filtering.
- Emit `debug!("Temporal coupling: {} pairs before filter, {} after, {} after cap", ...)`.

### 2. `src/config/model.rs` (or equivalent config struct)
- Add `max_coupling_pairs: usize` field to `TemporalConfig` with `#[serde(default = "default_max_coupling_pairs")]` returning `50`.

### 3. `.changeguard/config.toml` default template
- Add `max_coupling_pairs = 50` under `[temporal]` section with a comment.

### 4. `src/commands/scan.rs` and `src/commands/impact.rs`
- No changes needed; the fix is contained in the enrichment provider.

## Success Criteria
- `changeguard scan --impact` with agent dotfiles present produces ≤ 50 temporal coupling rows in the report.
- All coupling rows in the report involve at least one file from the current changeset.
- `RUST_LOG=debug changeguard scan --impact` logs the before/after counts.
- `max_coupling_pairs = 200` in `config.toml` raises the cap to 200.
- If changed files have zero coupling history, the coupling section is empty (not an error).
- All existing tests pass.

## Files Changed
- `src/impact/enrichment/coupling.rs`
- `src/config/model.rs` (or wherever `TemporalConfig` is defined)
- `.changeguard/config.toml`

## Edge Cases
- **`changed_files` is empty** (e.g., `changeguard impact` with no active git changes): Skip the relevance filter entirely; apply only the row cap. Return the top-N pairs by coupling score so the command is still useful.
- **All couplings filtered out**: Return an empty coupling list, not an error. The `ProviderResult` status should be `Ok` with an empty vec.
- **`max_coupling_pairs = 0`**: Treat 0 as "no cap" (unlimited) to allow power users to opt out of the cap via config.
- **Very large `max_coupling_pairs`**: No upper bound enforcement needed; the user accepts the performance trade-off by setting a large value.
- **Files renamed since last index**: `changed_files` uses the current paths from the scan; old paths in coupling data may not match. Accept as-is — renamed files will simply not match, which is conservative (fewer false positives).

## Definition of Done
- [ ] With agent dotfiles in git history, `changeguard scan --impact` produces ≤ 50 coupling rows and all rows involve a changed file.
- [ ] `max_coupling_pairs = 0` in config disables the cap (returns all filtered rows).
- [ ] `changeguard impact` with no changes returns top-N coupling rows (no filter, cap applies).
- [ ] `RUST_LOG=debug` shows before/after/cap counts.
- [ ] CI gate passes: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace`.
