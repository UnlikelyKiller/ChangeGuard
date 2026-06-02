# Track U16 Spec: Configurable Embed Concurrency Cap

## Background

Track U14 introduced `DEFAULT_EMBED_CAP = 4` as a hardcoded constant in `src/semantic/concurrency.rs`. The cap exists because the local ONNX server crashes when too many concurrent embed requests arrive (commit `90da256`).

U15 will split `[semantic].concurrency` into `parse_concurrency` and `embed_concurrency`. This track completes the picture by making the **embed cap itself** tunable — different hardware (a beefier GPU box) and different local-model servers (Ollama vs llama.cpp raw vs OpenAI-compatible cloud) have different crash thresholds. The constant `4` is a safe-but-arbitrary default.

## Objective

Expose the embed concurrency cap as a first-class config field, with the constant `DEFAULT_EMBED_CAP` as the fallback. The new field is distinct from `embed_concurrency` (U15) because:

- `embed_concurrency` = the user's *intent* for how many parallel embeds they want
- `embed_concurrency_cap` = a *safety ceiling* that can never be exceeded regardless of intent

The effective embed concurrency is `min(embed_concurrency, embed_concurrency_cap)`. The cap is a floor, not a ceiling — even `0` is treated as `1` (a single embed at a time) to keep the system responsive.

## Proposed Design

### Config field

In `src/config/model.rs::SemanticConfig`:

```rust
/// Safety ceiling on concurrent embed requests, regardless of `embed_concurrency`.
/// Defaults to `DEFAULT_EMBED_CAP=4` to stay below the local ONNX server's
/// crash threshold. Set to a higher value on beefier hardware; lower for
/// very constrained environments.
#[serde(default)]
pub embed_concurrency_cap: Option<usize>,
```

The accessor:

```rust
pub fn semantic_embed_concurrency_cap(&self) -> usize {
    self.embed_concurrency_cap.filter(|n| *n > 0).unwrap_or(DEFAULT_EMBED_CAP)
}
```

The validator (`src/config/validate.rs`) gets a new check:

```rust
if let Some(0) = config.semantic.embed_concurrency_cap {
    return Err(ConfigError::ValidationFailed {
        reason: "semantic.embed_concurrency_cap must be > 0".to_string(),
    }.into());
}
```

### Resolver change

In `src/semantic/concurrency.rs::ResolveOptions`, the existing `embed_cap: NonZeroUsize` is the same concept. Wire it up:

```rust
pub fn resolve_semantic_concurrency(
    cli_override: Option<usize>,
    config_value: Option<usize>,
    opts: ResolveOptions,
) -> ResolvedConcurrency {
    let raw = cli_override
        .or(config_value)
        .unwrap_or_else(|| opts.available_parallelism.map_or(1, |n| n.get()));
    let raw_nz = NonZeroUsize::new(raw.max(1)).unwrap();
    let parse_threads = raw_nz;
    let embed_threads = std::cmp::min(raw_nz, opts.embed_cap);
    ResolvedConcurrency { parse_threads, embed_threads }
}
```

In `src/commands/index.rs`, the call site changes to:

```rust
let resolve_opts = ResolveOptions {
    available_parallelism,
    embed_cap: nz_usize(config.semantic.semantic_embed_concurrency_cap()),
    ..Default::default()
};
```

The `embed_concurrency_cap` defaults to `4` if unset, so behavior is **identical to U14/U15** when the user doesn't set the new field — only users with non-standard hardware need to touch it.

### U15 interaction

U15 splits `concurrency` into `parse_concurrency` and `embed_concurrency`. U16's cap applies to the *resolved* `embed_concurrency` (the user-intent value), so the precedence chain becomes:

```
parse_threads:    CLI -j > [semantic].parse_concurrency > [semantic].concurrency (legacy) > [local_model].concurrency > auto
embed_concurrency: [semantic].embed_concurrency > [semantic].concurrency (legacy) > DEFAULT_EMBED_CAP=4
embed_concurrency_cap: [semantic].embed_concurrency_cap > DEFAULT_EMBED_CAP=4
effective:        min(embed_concurrency, embed_concurrency_cap).max(1)
```

### Dry-run integration (U15 dependency)

U15's `--semantic-dry-run` report must show all three values:

```
Semantic concurrency:
  parse_threads:          8     (auto from 8 logical CPUs)
  embed_concurrency:      4     (default, [semantic].embed_concurrency unset)
  embed_concurrency_cap:  4     (default, [semantic].embed_concurrency_cap unset)
  effective_embed:        4     (min(4, 4) = 4)
```

When the user sets a cap above the default:

```
Semantic concurrency:
  parse_threads:          8     (CLI -j 8)
  embed_concurrency:      8     (CLI -j 8)
  embed_concurrency_cap:  16    ([semantic].embed_concurrency_cap = 16)
  effective_embed:        8     (min(8, 16) = 8)
```

## Critical files

| File | Change |
|---|---|
| `src/config/model.rs` | Add `embed_concurrency_cap: Option<usize>`; accessor |
| `src/config/validate.rs` | `> 0` check |
| `src/config/defaults.rs` | Default template entry |
| `src/semantic/concurrency.rs` | Wire `ResolveOptions::embed_cap` to the config accessor at the call site (U15 should already plumb the resolver; U16 adds the wiring) |
| `src/commands/index.rs` | Read the cap and pass it into `ResolveOptions` |
| `src/commands/config.rs` | `format_semantic_line` shows the cap when non-default |
| `src/cli.rs` | No new flags |

## Existing utilities to reuse

- `crate::semantic::concurrency::DEFAULT_EMBED_CAP` (already a const, `src/semantic/concurrency.rs`)
- `crate::semantic::concurrency::ResolveOptions::embed_cap` (already a field, just needs wiring)
- The `semantic_concurrency()` accessor pattern at `src/config/model.rs:257` (filter `Some(0)`)

## TDD plan (Red → Green)

1. `src/config/model.rs`: TOML deserialization for `[semantic] embed_concurrency_cap = 16`
2. `src/config/validate.rs`: `test_zero_embed_concurrency_cap_fails`
3. `src/semantic/concurrency.rs`: `custom_embed_cap_is_respected` already exists from U14 — extend it to read from config-like options
4. `src/commands/config.rs`: `format_semantic_line_reports_cap_when_set`

## Verification

1. CI gate: `cargo fmt --all -- --check ; cargo clippy --all-targets --all-features -- -D warnings ; cargo nextest run --lib --bins --workspace`
2. Manual: with `[semantic] embed_concurrency_cap = 16` in user config, `changeguard index --semantic --incremental` should log `parse=N, embed_concurrency=min(N, 16)`
3. `changeguard config verify` shows the cap when explicit
4. End-to-end: `cargo install --path .` and re-run

## Why this scope

The `DEFAULT_EMBED_CAP = 4` is a band-aid that works for the current local ONNX setup. A user with a 16-core GPU box running Ollama can comfortably run 8-12 concurrent embeds. A user with a Raspberry Pi + cloud API should set the cap to 1. Making this configurable is the only correct response to "your tool works on my machine but not on theirs."

## Out of scope

- Auto-detecting the cap from server behavior (would need a probe protocol — could be a future U-track)
- Per-server cap overrides (different cap for `ollama_cloud_url` vs `local_model.base_url`)

## References

- ONNX cap regression: commit `90da256`
- U14 auto-tuner: `src/semantic/concurrency.rs`, commit `d0edd27`
- Resolver test pattern: `semaphore_releases_on_drop`, `embed_threads_capped_independently` in `src/semantic/concurrency.rs`
