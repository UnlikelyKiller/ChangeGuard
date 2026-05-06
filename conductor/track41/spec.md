# Track 41: Robust Doc Chunker

## Overview
Fix a production panic in `src/docs/chunker.rs` that occurs when indexing Markdown documentation containing multi-byte Unicode characters (e.g., box-drawing characters `│`, `├`, `─` used in directory-tree ASCII art). The panic message is `start byte index is not a char boundary`, caused by byte-index slicing that splits UTF-8 encoded characters.

## Objectives
- Eliminate all byte-index slicing in `split_at_paragraphs` and `split_into_sections`.
- Use `str::floor_char_boundary` (Rust 1.75+) as the primary boundary tool for all string slicing operations.
- Use Unicode-aware character boundary checks (`char_indices`, `floor_char_boundary`, or equivalent) for all string slicing operations.
- Ensure the chunker correctly handles CJK characters, emoji, combining marks, RTL text, and box-drawing symbols without panic or data loss.
- Prevent infinite loops when overlap or budget math produces zero advance.
- Document that grapheme cluster integrity is best-effort (char-boundary safety prevents panics; grapheme splitting may still occur for ZWJ sequences).

## Success Criteria
- `chunk_markdown` never panics on valid UTF-8 input, regardless of Unicode character class.
- Chunk boundaries always fall on valid character boundaries.
- Existing tests continue to pass with identical output.
- New unit tests cover box-drawing chars, emoji, CJK text, RTL text, and infinite-loop edge cases.
- CI gate (`cargo fmt`, `cargo clippy`, `cargo test`) passes.

## Architecture
- `src/docs/chunker.rs` — The only file changed. Focus on `split_at_paragraphs` (lines 125-166) where `&remaining[..max_chars]` performs naive byte slicing.
- No new modules or public API changes.

## Testing Strategy
- **Red commit**: Add failing tests with box-drawing Markdown, emoji, and CJK text. Verify they panic on current code.
- **Green commit**: Implement Unicode-aware boundary logic. Verify all tests pass.
