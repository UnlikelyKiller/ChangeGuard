# Track U17 Spec: Fix TOML Merge Regression for `[semantic]` Defaults

## Background

When a user adds a `[semantic]` block to their `.changeguard/config.toml` to set a single field (e.g. `concurrency = 4`), the **sibling fields lose their defaults**. This is observable today:

- U14 test (working as expected): `[semantic] concurrency = 4` in user config → `config view -s semantic` shows `{"concurrency": 4, "hnsw_rebuild_threshold": null}` — the `500` default is gone.

The root cause: serde's `#[serde(default)]` on `SemanticConfig` calls `Default::default()` only when the field is *missing entirely*. When the user writes `[semantic]` with any field, serde deserializes the whole struct, and unset sibling fields are `None` (the `Option<usize>` default), not the `Some(500)` from our `Default` impl.

This is a **serde gotcha** that bites every `Option<T>` field with a non-`None` default — and ChangeGuard has it on `hnsw_rebuild_threshold`, soon on `concurrency` / `parse_concurrency` / `embed_concurrency` (U15), and on `embed_concurrency_cap` (U16).

## Objective

Make `#[serde(default)]` honor the in-struct `Default` impl for each field, not the `Option::None` default. The fix is to use `#[serde(default = "fn_name")]` per field — a small helper that returns the desired default value.

This applies to:
- `SemanticConfig::hnsw_rebuild_threshold` (currently broken — see above)
- `SemanticConfig::concurrency` (U13 — currently `None` default, which is correct for this one, but the pattern matters)
- `SemanticConfig::parse_concurrency` (U15)
- `SemanticConfig::embed_concurrency` (U15)
- `SemanticConfig::embed_concurrency_cap` (U16 — `None` default, also correct, but consistency matters)
- **Any other `Option<T>` field in the codebase with a non-`None` desired default** — audit and fix.

## Proposed Design

### Per-field default functions

In `src/config/model.rs`:

```rust
fn default_hnsw_rebuild_threshold() -> Option<usize> {
    Some(DEFAULT_HNSW_REBUILD_THRESHOLD)
}

pub struct SemanticConfig {
    #[serde(default = "default_hnsw_rebuild_threshold")]
    pub hnsw_rebuild_threshold: Option<usize>,
    // ... other fields
}
```

The accessor pattern (`hnsw_rebuild_threshold()`) already does the right thing — `self.hnsw_rebuild_threshold.unwrap_or(DEFAULT_HNSW_REBUILD_THRESHOLD)` — so the fix is just plumbing the value through deserialization.

### Audit script

Run `cargo +nightly udeps --all-features` (or grep) to find all `#[serde(default)] pub foo: Option<T>` fields, then verify each has either:
- A `default = "fn"` that returns `Some(...)`, OR
- A `None` default that is intentional (call out in a doc comment)

Track this in a follow-up audit sub-task; U17 is the [semantic] fix specifically, but the pattern applies elsewhere.

### Why not a `Deserialize` impl

Writing a custom `Deserialize` for `SemanticConfig` would also work but is heavier: ~30 lines vs ~3 per field. The `default = "fn"` pattern is the idiomatic serde fix and matches what the rest of `model.rs` already does (e.g. `default_risk_weights`, `default_chunk_top_k`).

## Critical files

| File | Change |
|---|---|
| `src/config/model.rs` | Add `default_hnsw_rebuild_threshold()` helper; change `#[serde(default)]` → `#[serde(default = "default_hnsw_rebuild_threshold")]` on `hnsw_rebuild_threshold` |
| `src/config/model.rs` | Audit other `Option<T>` fields; add `default = "fn"` where the `None` default is wrong (U18 will surface a wider audit) |
| `src/config/defaults.rs` | No change (template already has the explicit value) |

## Existing utilities to reuse

- The `default_*` helper pattern at `src/config/model.rs:251-260` (`default_risk_weights`, etc.)
- The accessor pattern at `src/config/model.rs:249-255` (`hnsw_rebuild_threshold()`)

## TDD plan (Red → Green)

1. `src/config/model.rs`:
   - `test_semantic_partial_section_preserves_defaults`: parse `t"[semantic]\nconcurrency = 4"`, assert `hnsw_rebuild_threshold()` returns `500`, not the unwrap default.
2. This is a bug fix, so the test is "write the failing test, then fix."

## Verification

1. CI gate: `cargo fmt --all -- --check ; cargo clippy --all-targets --all-features -- -D warnings ; cargo nextest run --lib --bins --workspace`
2. Manual: add `[semantic] concurrency = 4` to `.changeguard/config.toml`, run `changeguard config view -s semantic`, confirm `hnsw_rebuild_threshold` is `500` (not `null`).
3. `changeguard config verify` shows `Semantic: hnsw_rebuild_threshold=500` even with a partial `[semantic]` block.

## Why this scope

The regression is **invisible until you test it** — `config verify` doesn't show the default, the `hnsw_rebuild_threshold()` accessor still works because it `unwrap_or`s the constant, so the user sees correct *behavior* but the displayed config is wrong. The `changeguard config view -s semantic --json` output I observed during the U13 smoke test is the smoking gun: `"hnsw_rebuild_threshold": null`.

This is a small, high-leverage fix. The wider audit is U18.

## Out of scope

- Cross-section defaults (e.g., `[local_model].concurrency` and `[semantic].concurrency` merging) — would need a layered config loader (the `config` crate)
- Making `Option<T>` defaults globally "smart" — impossible without per-field helpers

## References

- Serde docs on `default`: https://serde.rs/field-attrs.html#default
- Confirmed bug surface: U13 smoke-test output `"hnsw_rebuild_threshold": null` from `config view -s semantic` after partial override
- Similar pattern in existing code: `default_risk_weights` at `src/config/model.rs:255`
