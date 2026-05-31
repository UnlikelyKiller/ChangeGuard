# Implementation Plan - Track 41: Robust Doc Chunker

## Goal
Fix the Unicode boundary panic in `src/docs/chunker.rs` so that Markdown containing multi-byte characters (box-drawing, emoji, CJK) is chunked safely and correctly.

## Proposed Changes

### 1. Identify Panic Sites [src/docs/chunker.rs]
- `split_at_paragraphs` line 139: `&remaining[..max_chars]` — byte slice may split a multi-byte char.
- `split_at_paragraphs` line 148: `&remaining[..max_chars]` — same issue in fallback path.
- `split_at_paragraphs` line 162: `remaining = &remaining[advance..]` — `advance` is derived from byte counts; must also be char-boundary aligned.

### 2. Implement Unicode-Aware Slicing
- Replace `&remaining[..max_chars]` with a helper `safe_byte_prefix(text: &str, max_bytes: usize) -> &str` that uses `str::floor_char_boundary` (Rust 1.75+) to find the nearest valid UTF-8 character boundary at or before `max_bytes`.
  ```rust
  fn safe_byte_prefix(text: &str, max_bytes: usize) -> &str {
      let end = text.floor_char_boundary(max_bytes);
      &text[..end]
  }
  ```
- Replace `&remaining[advance..]` with boundary-safe advance using `floor_char_boundary` on the computed advance value.
- Ensure the `slice` created for paragraph-break search is also char-boundary safe: `let slice = safe_byte_prefix(remaining, max_chars);`.
- Apply `floor_char_boundary` to the overlap calculation so `advance` is always a valid boundary.
- Ensure `estimate_tokens` (len/4) still produces a conservative byte budget, but slicing respects char boundaries.
- **Document:** Grapheme cluster integrity is best-effort. Char-boundary safety eliminates panics; ZWJ emoji sequences may still be split across chunks. This is acceptable for LLM ingestion but should be noted.
- **Infinite-loop prevention:** If `advance` would be zero, force `advance = remaining.floor_char_boundary(1)` so progress is guaranteed.

### 3. Preserve Existing Behavior
- Do not change token estimation logic, section splitting, or overlap math.
- Only the mechanical slicing of `remaining` into substrings must become boundary-safe.

### 4. Add Regression Tests
- `test_box_drawing_characters`: Markdown with `│`, `├`, `─`, `└` tree diagrams. Must not panic and must preserve all characters.
- `test_emoji_boundary`: Markdown with emoji and zero-width joiners.
- `test_cjk_characters`: Markdown with Chinese/Japanese/Korean text.
- `test_combining_marks`: Markdown with combining diacritical marks.
- `test_rtl_text`: Markdown with Arabic/Hebrew right-to-left text to ensure bidirectional characters are not split.
- `test_infinite_loop_prevention`: A single multi-byte char exceeding budget must not loop infinitely; advance must always progress.
- `test_empty_after_section_split`: Ensure `split_at_paragraphs` terminates cleanly when all paragraphs are under budget.

## Verification Plan

### Automated Tests
- `cargo test` in `src/docs/chunker.rs` module.
- Run `changeguard index --docs` on a repo containing tree-diagram Markdown (e.g., `docs/` in this repo) and confirm no panic.

### Manual Verification
- Index the ChangeGuard repository's own documentation, which contains ASCII tree diagrams in `.agents/skills/`.

## Definition of Done (DoD)
- [ ] **Panic Eliminated**: `chunk_markdown` handles multi-byte Unicode without panic.
- [ ] **Infinite-Loop Prevention**: `split_at_paragraphs` terminates on all inputs, including single-char-over-budget edge cases.
- [ ] **Boundary Safety**: Every substring slice in the chunker falls on a valid UTF-8 character boundary.
- [ ] **Regression Coverage**: New tests for box-drawing, emoji, CJK, RTL, combining marks, and infinite-loop prevention all pass.
- [ ] **Zero Regression**: Existing chunker tests produce identical output.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, and full test suite pass.
