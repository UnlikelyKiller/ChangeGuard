## Plan: Track E0-4 Token Budget Consistency

### Phase 1: Write Budget Calculation Tests
- [ ] Task 1.1: Add `test_default_context_window_yields_hardcoded_budget` to `src/commands/ask.rs`. Create a `GeminiConfig::default()`, compute `char_limit = (context_window as u64 * 4 * 80 / 100).max(MIN_CONTEXT_CHARS as u64) as usize`, assert it equals `409_600`.
- [ ] Task 1.2: Add `test_custom_context_window_adjusts_budget`. Create a `GeminiConfig` with `context_window = 200_000`, compute `char_limit`, assert it equals `640_000`.
- [ ] Task 1.3: Add `test_small_context_window_budget`. Create a `GeminiConfig` with `context_window = 32_000`, compute `char_limit`, assert it equals `102_400`.
- [ ] Task 1.4: Add `test_zero_context_window_fallback`. Create a `GeminiConfig` with `context_window = 0`, compute `char_limit = (0u64 * 4 * 80 / 100).max(MIN_CONTEXT_CHARS as u64) as usize`, assert it equals `32_768`.
- [ ] Task 1.5: Run tests. Confirm the first three pass (formula is correct arithmetic) and the fourth passes (floor logic works).

### Phase 2: Replace Hardcoded Budget with Config-Derived Budget
- [ ] Task 2.1: Add a `MIN_CONTEXT_CHARS` constant: `const MIN_CONTEXT_CHARS: usize = 32_768;` in `src/commands/ask.rs`.
- [ ] Task 2.2: Replace the hardcoded `char_limit` in `execute_ask` with the integer-arithmetic formula:
  ```rust
  // Derive truncation budget from config: 4 chars/token, reserve 20% for system prompt + response
  let char_limit = (config.gemini.context_window as u64 * 4 * 80 / 100)
      .max(MIN_CONTEXT_CHARS as u64) as usize;
  ```
- [ ] Task 2.3: Add a doc comment to the `context_window` field in `GeminiConfig` (`src/config/model.rs`):
  ```rust
  /// Context window size in tokens for the Gemini model.
  /// Used to derive the truncation budget for prompt submission:
  /// char_limit = context_window * 4 * 80 / 100 (4 chars/token, 80% reserved for context).
  ```
- [ ] Task 2.4: Run all tests in `ask.rs`. Confirm they pass, including the existing `select_gemini_model` tests.

### Phase 3: Verify Default Behavior Unchanged
- [ ] Task 3.1: Compute `128_000 * 4 * 80 / 100` manually and confirm it equals `409_600`. Assert this in the test from Task 1.1.
- [ ] Task 3.2: Run `cargo test` across the full project. Confirm no regressions.
- [ ] Task 3.3: Run `cargo clippy` and resolve any new warnings.

### Phase 4: Edge Cases and Documentation
- [ ] Task 4.1: Verify `src/config/defaults.rs` includes `context_window = 128000` in the default config string. Confirm it matches the `default_context_window()` function.
- [ ] Task 4.2: Add a brief comment near the `MIN_CONTEXT_CHARS` constant explaining the floor rationale: "Prevents accidental zero-truncation if context_window is misconfigured to 0 or a very small number."
- [ ] Task 4.3: Manual smoke test: set `context_window = 32000` in a local config, run `changeguard ask`, verify the prompt is truncated to the smaller budget (or completes successfully if the packet is small enough).
- [ ] Task 4.4: Manual smoke test: remove `context_window` from config (use default), run `changeguard ask`, verify behavior is identical to before the change.