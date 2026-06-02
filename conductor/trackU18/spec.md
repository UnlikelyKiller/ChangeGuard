# Track U18 Spec: Audit and Fix All `Option<T>` Serde Defaults in `Config`

## Background

Track U17 fixes the specific case of `SemanticConfig::hnsw_rebuild_threshold` losing its `Some(500)` default when a sibling field is set in user config. The same serde gotcha (`#[serde(default)]` on `Option<T>` → `None`, ignoring the struct's `Default` impl) likely affects other `Option<T>` fields in `Config` and friends.

The pattern is invisible until tested:
- The field has a typed accessor that `unwrap_or`s a constant — behavior looks correct.
- `config view --json` shows `null` for the field — debugging surface is wrong.
- A user who sets a sibling field unknowingly disables the constant default for this one.

## Objective

Systematic audit: find every `Option<T>` field in the config model that has a non-`None` intended default, then fix each with a `#[serde(default = "fn")]` helper. Where the intended default *is* `None`, add a doc comment to make the intent explicit so future readers don't get confused.

## Proposed Design

### Audit steps

1. **Grep for the pattern** in `src/config/model.rs`:
   ```bash
   rg "pub (\w+): Option<" src/config/model.rs
   ```
   For each match, check whether the field's accessor (e.g. `field()`) or the `Default` impl sets a non-`None` value.

2. **For each affected field**, add a helper:
   ```rust
   fn default_<field>() -> Option<T> { Some(<default_value>) }
   ```
   and change `#[serde(default)]` → `#[serde(default = "default_<field>")]`.

3. **For fields where `None` is intentional** (e.g. `LocalModelConfig::concurrency`, `SemanticConfig::concurrency`), add a doc comment:
   ```rust
   /// `None` triggers auto-tuning at the call site. Intentionally NOT a `Some(N)` default.
   #[serde(default)]
   pub concurrency: Option<usize>,
   ```

### Likely candidates (initial scan)

- `SemanticConfig::hnsw_rebuild_threshold` — **U17 fixes this**
- `LocalModelConfig::ollama_cloud_url`, `ollama_cloud_api_key`, `ollama_cloud_model` — defaults to `None` (intentional, no Ollama Cloud by default)
- `LocalModelConfig::embedding_url`, `generation_url` — defaults to `None` (intentional, fall back to `base_url`)
- `LocalModelConfig::concurrency` — defaults to `None` (intentional, auto-tune)
- `CoverageConfig`, `HotspotConfig`, `TemporalConfig`, `VerifyConfig` — need to scan each

### Test strategy

For every fixed field, add a test:
```rust
#[test]
fn test_<config>_<field>_partial_section_preserves_default() {
    let toml_str = r#"
        [<section>]
        some_other_field = 42
    "#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.<section>.<field>(), <expected_default>);
}
```

If the audit finds N fixes, N tests get added. The CI gate catches regressions on any of them.

## Critical files

| File | Change |
|---|---|
| `src/config/model.rs` | Per-field `default = "fn"` helpers + accessor + doc comments for `None` defaults |
| `src/config/validate.rs` | No change |
| `src/config/defaults.rs` | No change (template already has explicit values) |

## Existing utilities to reuse

- Existing `default_*` helpers: `default_risk_weights`, `default_chunk_top_k`, `default_chunk_min_similarity`, `default_chunk_dedup_threshold`, `default_context_window_local`, `default_local_timeout` (all at `src/config/model.rs:251-565`)
- The accessor pattern that `unwrap_or`s a constant (e.g. `hnsw_rebuild_threshold()`)

## TDD plan (Red → Green)

1. Run the audit grep.
2. For each non-`None` `Option<T>` field, write a "partial section preserves default" test.
3. Confirm tests fail.
4. Add `default_<field>()` helpers and switch the `#[serde(default)]` to `#[serde(default = "default_<field>")]`.
5. Confirm tests pass.

For fields where `None` is the intended default, add the doc comment (no test needed — the absence of a default is the contract).

## Verification

1. CI gate.
2. Manual: build a "kitchen sink" test TOML with one field per section, confirm every other field's `unwrap_or`-style accessor still returns the constant default.
3. `changeguard config view --json` for the kitchen-sink config shows the right defaults populated everywhere.

## Why this scope

U17 is the visible bug; U18 is the systematic fix. Doing the audit now while the U13/U14 changes are fresh is cheaper than waiting for the next time a user reports a "missing" default. The audit will likely find 0–2 additional bugs (most `Option<T>` fields are correctly `None` by design), but the doc-comment pass alone is worth the work.

## Out of scope

- Cross-section merging (deferred — would need the `config` crate)
- A linter rule to catch the pattern in CI (could be a `clippy::pedantic` warning if one exists; would need a separate track)

## References

- U17 spec/plan
- Serde field attrs: https://serde.rs/field-attrs.html#default
- Confirmed bug surface: U13 smoke test (`hnsw_rebuild_threshold: null`)
