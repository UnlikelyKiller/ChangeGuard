# Specification: Track 20 — Determinism and Error Visibility Hardening

## Overview
Address the remaining `docs/audit2.md` engineering gaps around silent fallback behavior, missing rule validation on load, and silent dropping of partial-analysis failures during impact generation.

This track hardens failure visibility without turning every recoverable absence into a fatal error.

## Breaking-Risk Assessment
This track should be behavior-tightening, not format-breaking:

- missing config/rules files may continue to use documented defaults
- invalid or unreadable config/rules files must fail visibly instead of silently changing behavior
- impact JSON/report changes must be additive where possible
- if packet semantics become non-additive, bump the packet schema version and document it

## 1. Rule Validation on Load
**Priority: HIGH**

### Required Change
- `src/policy/load.rs` must call `validate_rules()` after parsing TOML and before returning `Rules`

### Required Behavior
- invalid rule files must fail deterministically and visibly
- the error must identify the bad pattern/path when possible
- no later subsystem should need to silently ignore invalid rules that were already loadable
- missing `rules.toml` may still resolve to default rules

## 2. Remove Silent Fallbacks in Commands
**Priority: HIGH**

### Current Problems
- `commands/ask.rs` uses `load_config(...).unwrap_or_default()`
- `commands/watch.rs` uses `load_config(...).unwrap_or_default()`
- `commands/verify.rs` uses `load_rules(...).unwrap_or_default()` in auto-plan mode

### Required Direction
- Replace silent defaulting with one of:
  - fail-fast with actionable diagnostics, or
  - explicit warning plus well-documented fallback

### Policy
- missing config/rules files may continue to use the default loader behavior
- parse, validation, permission, and read errors must not be collapsed into defaults
- `ask` and `watch` should fail on invalid config rather than silently changing runtime behavior
- `verify` automatic planning should fail on invalid rules instead of silently substituting defaults
- any fallback that remains must emit a visible warning and be covered by tests

## 3. Partial Analysis Must Be Explicit
**Priority: HIGH**

### Current Problem
`commands/impact.rs` currently discards parser/extractor failures by collapsing them into missing data.

### Required Outcome
Impact generation must distinguish:

- no symbols/imports/runtime usage found
- analysis skipped because file could not be read
- parser/extractor failed
- analysis intentionally unsupported for that file type

### Recommended Shape
Prefer structured status over free-form warning strings alone. Acceptable models:

- additive packet-level `analysis_warnings: Vec<...>`
- per-file analysis status fields on `ChangedFile`
- per-subsystem status objects for symbols/imports/runtime usage

Whatever shape is used must remain deterministic and inspectable.

### Output Requirements
- warnings/statuses must be sorted deterministically
- warnings/statuses must be written to the impact report JSON
- human output should summarize when partial analysis occurred
- user-visible messages should separate parser failure from unsupported language from file-read failure

## 4. Determinism Contract Reinforcement
**Priority: MEDIUM**

### Required Rules
- warning collections must be sorted and deduplicated
- validation and load failures must produce stable messages suitable for tests
- tests should verify that the same invalid inputs produce the same diagnostics shape
- packet/report changes must preserve stable field naming and ordering assumptions used in tests

## 5. Tests
**Priority: HIGH**

### Required Coverage
- invalid `rules.toml` fails during `load_rules()`
- missing `rules.toml` still yields default rules
- `ask` with invalid config fails visibly
- `watch` with invalid config fails visibly
- `verify` auto-plan with invalid rules fails visibly
- `impact` on unreadable/unsupported/failed parse content records explicit status instead of silently dropping data
- deterministic ordering for multiple warnings/status entries
- packet schema/version expectations when new fields are added

## Non-Goals
- making every internal warning fatal
- speculative parser-recovery heuristics
- ad hoc human-only warnings with no JSON/report representation

## Verification
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test -j 1 -- --test-threads=1`
