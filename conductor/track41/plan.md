# Implementation Plan - Track 41: Robust Doc Chunker

## Goal
Fix the Unicode boundary panic in `src/docs/chunker.rs` so that Markdown containing multi-byte characters (box-drawing, emoji, CJK) is chunked safely and correctly.

## Proposed Changes

### 1. Identify Panic Sites [src/docs/chunker.rs]
- `split_at_paragraphs` line 139: `&remaining[..max_chars]` — byte slice may split a multi-byte char.
- `split_at_paragraphs` line 148: `&remaining[..max_chars]` — same issue in fallback path.
- `split_at_paragraphs` line 162: `remaining = &remaining[advance..]` — `advance` is derived from byte counts; must also be char-boundary aligned.

### 2. Implement Unicode-Aware Slicing
- Replace `&remaining[..max_chars]` with a helper `safe_prefix(remaining: &str, max_bytes: usize) -> &str` that walks back from `max_bytes` to the nearest valid UTF-8 character boundary using `str::floor_char_boundary` (Rust 1.75+) or `char_indices()`.
- Replace `&remaining[advance..]` with a helper `safe_advance(remaining: &str, advance_bytes: usize) -> &str` that walks forward to the next valid character boundary.
- Ensure `estimate_tokens` (len/4) still produces a conservative byte budget, but slicing respects char boundaries.

### 3. Preserve Existing Behavior
- Do not change token estimation logic, section splitting, or overlap math.
- Only the mechanical slicing of `remaining` into substrings must become boundary-safe.

### 4. Add Regression Tests
- `test_box_drawing_characters`: Markdown with `│`, `├`, `─`, `└` tree diagrams. Must not panic and must preserve all characters.
- `test_emoji_boundary`: Markdown with emoji and zero-width joiners.
- `test_cjk_characters`: Markdown with Chinese/Japanese/Korean text.
- `test_combining_marks`: Markdown with combining diacritical marks.

## Verification Plan

### Automated Tests
- `cargo test` in `src/docs/chunker.rs` module.
- Run `changeguard index --docs` on a repo containing tree-diagram Markdown (e.g., `docs/` in this repo) and confirm no panic.

### Manual Verification
- Index the ChangeGuard repository's own documentation, which contains ASCII tree diagrams in `.agents/skills/`.

## Definition of Done (DoD)
- [ ] **Panic Eliminated**: `chunk_markdown` handles multi-byte Unicode without panic.
- [ ] **Boundary Safety**: Every substring slice in the chunker falls on a valid UTF-8 character boundary.
- [ ] **Regression Coverage**: New tests for box-drawing, emoji, CJK, and combining marks all pass.
- [ ] **Zero Regression**: Existing chunker tests produce identical output.
- [ ] **Clean CI**: `cargo fmt`, `cargo clippy`, and full test suite pass.
