# Specification: Track E0-4 Token Budget Consistency

## Overview

The `execute_ask` function in `src/commands/ask.rs` hardcodes a character limit
of `409_600` for truncating the impact packet before sending to Gemini:

```rust
let char_limit = 409_600;
let truncated = latest_packet.truncate_for_context(char_limit);
```

Meanwhile, `GeminiConfig` has a `context_window` field that defaults to
`128_000` (tokens) but is never used in the truncation logic. The hardcoded
`409_600` corresponds to approximately 102,400 tokens at 4 chars/token, which
is inconsistent with the configured `context_window` of 128,000 tokens.

This track derives the truncation budget from `config.gemini.context_window`
instead of hardcoding it. The formula: **character limit = context_window * 4 *
0.8** (80% of context window in chars, reserving 20% for system prompt and
response).

## Components

### 1. Token Budget Calculation (`src/commands/ask.rs`)

Replace the hardcoded `let char_limit = 409_600;` with a calculation derived
from the config:

```rust
// 4 chars per token, reserve 20% for system prompt and response
let char_limit = (config.gemini.context_window as u64 * 4 * 80 / 100) as usize;
```

For the default `context_window` of 128,000, this yields:
- `128_000 * 4 * 80 / 100 = 409_600`

So the default behavior is identical to the current hardcoded value. The
difference is that users who change `context_window` in their config will now
have the truncation budget adjusted automatically.

Note: This formula uses integer arithmetic (`u64 * 4 * 80 / 100`) rather than
floating-point multiplication (`as f64 * 0.8 * 4.0`). Integer arithmetic
avoids floating-point rounding issues and is more idiomatic for this kind of
budget calculation.

### 2. Context Window Documentation (`src/config/model.rs`)

Add a doc comment to the `context_window` field in `GeminiConfig` explaining
that it is used to derive the truncation budget for Gemini prompt submission.
The comment should document the formula: `char_limit = context_window * 4 * 80 / 100`
(integer arithmetic). Example:

```rust
/// Context window size in tokens for the Gemini model.
/// Used to derive the truncation budget for prompt submission:
/// char_limit = context_window * 4 * 80 / 100 (4 chars/token, 80% reserved for context).
```

### 3. Default Config Alignment (`src/config/defaults.rs`)

Verify that the default config string in `DEFAULT_CONFIG` includes
`context_window = 128000` in the `[gemini]` section. It already does. No
change needed, but verify.

### 4. Unit Tests (`src/commands/ask.rs`)

Add tests to verify the token budget derivation. All tests use integer
arithmetic matching the production formula:

- `test_default_context_window_yields_hardcoded_budget`: Create a default
  `GeminiConfig`, compute `char_limit = (context_window as u64 * 4 * 80 / 100).max(MIN_CONTEXT_CHARS as u64) as usize`,
  and assert it equals `409_600`.
- `test_custom_context_window_adjusts_budget`: Create a `GeminiConfig` with
  `context_window = 200_000`, compute `char_limit`, and assert it equals
  `640_000` (200_000 * 4 * 80 / 100).
- `test_small_context_window_budget`: Create a `GeminiConfig` with
  `context_window = 32_000`, compute `char_limit`, and assert it equals
  `102_400`.
- `test_zero_context_window_fallback`: Create a `GeminiConfig` with
  `context_window = 0`, compute `char_limit = (0u64 * 4 * 80 / 100).max(MIN_CONTEXT_CHARS as u64) as usize`,
  and assert it equals `32_768` (the minimum floor is applied).

### 5. Minimum Budget Floor (`src/commands/ask.rs`)

Add a minimum floor constant:

```rust
const MIN_CONTEXT_CHARS: usize = 32_768; // 8K tokens minimum
```

After computing `char_limit`, apply `.max(MIN_CONTEXT_CHARS)`:

```rust
let char_limit = (config.gemini.context_window as u64 * 4 * 80 / 100)
    .max(MIN_CONTEXT_CHARS as u64) as usize;
```

This prevents accidental truncation to nothing if someone misconfigures
`context_window` to 0 or a very small number.

## Constraints & Guidelines

- **Default behavior unchanged**: With the default `context_window = 128_000`,
  the computed `char_limit` is exactly `409_600`, identical to the current
  hardcoded value. No existing user sees a behavior change.
- **Single-line change**: The core fix is replacing one line. The surrounding
  changes (doc comment, floor constant, tests) are defensive but minimal.
- **No new crate dependency**: This is pure arithmetic.
- **Overflow safety**: `context_window` is `usize`. For realistic values (up to
  ~2M tokens), `context_window * 4 * 80` fits in `u64` without overflow. The
  cast to `u64` before multiplication prevents overflow on 32-bit platforms.
- **TDD**: Write the budget calculation tests first, confirm the formula works,
  then make the one-line change.

## Acceptance Criteria

1. With the default `context_window = 128_000`, the truncation budget is exactly
   `409_600` characters (no behavior change).
2. With `context_window = 200_000`, the truncation budget is `640_000`
   characters.
3. With `context_window = 0`, the truncation budget falls back to `32_768`
   characters (minimum floor).
4. The `GeminiConfig.context_window` field has a doc comment explaining its
   role in deriving the truncation budget.
5. The hardcoded `409_600` value is removed from `execute_ask`; the budget is
   now computed from config.
6. All existing `ask.rs` tests continue to pass.
7. No new crate dependencies are introduced.

## Definition of Done

- All acceptance criteria pass
- All unit tests pass
- `cargo fmt --all -- --check` passes
- `cargo clippy --all-targets --all-features -- -D warnings` passes
- `cargo test` passes with no regressions
- No deviations from this spec without documented justification