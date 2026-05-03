use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

pub struct DocChunk {
    pub file_path: String,
    pub chunk_index: usize,
    pub heading: Option<String>,
    pub content: String,
    pub token_count: usize,
}

fn estimate_tokens(text: &str) -> usize {
    text.len().div_ceil(4)
}

/// Chunk markdown content into semantic chunks based on heading boundaries.
/// `overlap_tokens` is reserved for future use and currently ignored.
pub fn chunk_markdown(
    content: &str,
    file_path: &str,
    chunk_tokens: usize,
    _overlap_tokens: usize,
) -> Vec<DocChunk> {
    let mut chunks: Vec<DocChunk> = Vec::new();
    let mut chunk_index: usize = 0;

    for (heading, body) in split_into_sections(content) {
        let token_count = estimate_tokens(&body);
        if token_count < 50 {
            continue;
        }

        if token_count > chunk_tokens {
            for sub in split_at_paragraphs(&body, chunk_tokens) {
                let tk = estimate_tokens(&sub);
                if tk >= 50 {
                    chunks.push(DocChunk {
                        file_path: file_path.to_string(),
                        chunk_index,
                        heading: heading.clone(),
                        content: sub,
                        token_count: tk,
                    });
                    chunk_index += 1;
                }
            }
        } else {
            chunks.push(DocChunk {
                file_path: file_path.to_string(),
                chunk_index,
                heading,
                content: body,
                token_count,
            });
            chunk_index += 1;
        }
    }

    chunks
}

/// Walk pulldown-cmark events and return (heading, body) pairs for each section.
/// The first section may have heading=None.
fn split_into_sections(content: &str) -> Vec<(Option<String>, String)> {
    let parser = Parser::new_ext(content, Options::ENABLE_TABLES);
    let mut sections: Vec<(Option<String>, String)> = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut heading_buf = String::new();
    let mut body_buf = String::new();
    let mut in_heading = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                // Finalize previous section
                if !body_buf.is_empty() && !in_heading && !body_buf.trim().is_empty() {
                    sections.push((current_heading.take(), std::mem::take(&mut body_buf)));
                }
                current_heading = None;
                heading_buf.clear();
                in_heading = true;
            }
            Event::End(TagEnd::Heading(_)) => {
                current_heading = Some(heading_buf.trim().to_string());
                heading_buf.clear();
                in_heading = false;
            }
            Event::Text(text) => {
                if in_heading {
                    heading_buf.push_str(&text);
                } else {
                    body_buf.push_str(&text);
                }
            }
            Event::Code(code) => {
                if in_heading {
                    heading_buf.push_str(&code);
                } else {
                    body_buf.push_str(&code);
                }
            }
            Event::SoftBreak | Event::HardBreak if !in_heading => {
                body_buf.push('\n');
            }
            Event::Start(Tag::Paragraph) | Event::End(TagEnd::Paragraph)
                if !in_heading && !body_buf.is_empty() && !body_buf.ends_with('\n') =>
            {
                body_buf.push('\n');
            }
            _ => {}
        }
    }

    // Final section
    if !body_buf.trim().is_empty() {
        sections.push((current_heading.take(), std::mem::take(&mut body_buf)));
    }

    sections
}

/// Split text at paragraph boundaries (\n\n) so each sub-chunk fits within `max_tokens`.
/// Falls back to hard split at budget boundary if no paragraph break found.
fn split_at_paragraphs(text: &str, max_tokens: usize) -> Vec<String> {
    let max_chars = max_tokens * 4;
    let mut result = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if estimate_tokens(remaining) <= max_tokens {
            result.push(remaining.to_string());
            break;
        }

        // Find the best split point within max_chars
        let slice = if remaining.len() <= max_chars {
            remaining
        } else {
            &remaining[..max_chars]
        };

        // Try paragraph break
        if let Some(pos) = slice.rfind("\n\n") {
            let chunk = remaining[..pos].to_string();
            result.push(chunk);
            remaining = remaining[pos + 2..].trim_start();
        } else if let Some(pos) = slice.rfind('\n') {
            let chunk = remaining[..pos].to_string();
            result.push(chunk);
            remaining = remaining[pos + 1..].trim_start();
        } else if remaining.len() > max_chars {
            let chunk = remaining[..max_chars].to_string();
            result.push(chunk);
            remaining = &remaining[max_chars..];
        } else {
            result.push(remaining.to_string());
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_sections_with_headings() {
        let long_body = "X".repeat(220); // ~55 tokens per section, above minimum
        let md = format!(
            "# Title\n\n{}\n\n## Section 1\n\n{}\n\n## Section 2\n\n{}\n",
            long_body, long_body, long_body
        );
        let chunks = chunk_markdown(&md, "test.md", 512, 0);

        assert_eq!(chunks.len(), 3, "Should have 3 chunks for 3 sections");
        assert_eq!(chunks[0].heading.as_deref(), Some("Title"));
        assert_eq!(chunks[1].heading.as_deref(), Some("Section 1"));
        assert_eq!(chunks[2].heading.as_deref(), Some("Section 2"));
    }

    #[test]
    fn test_section_splits_at_paragraph_boundary_when_over_budget() {
        // Create several paragraphs that together exceed a small budget
        let paragraph = "A".repeat(200); // ~50 tokens per paragraph
        let md = format!(
            "## Big\n\n{}\n\n{}\n\n{}\n\n",
            paragraph, paragraph, paragraph
        );
        // Total ~150 tokens, budget 60 tokens => should split
        let chunks = chunk_markdown(&md, "test.md", 60, 0);

        assert!(chunks.len() >= 2, "Should split into at least 2 sub-chunks");
        for chunk in &chunks {
            assert!(
                chunk.token_count <= 60,
                "Each chunk should be under budget: got {}",
                chunk.token_count
            );
        }
    }

    #[test]
    fn test_section_under_50_tokens_discarded() {
        let md = "# Title\n\nShort.\n\n## Tiny\n\nHi.\n\n## Keep\n\n";
        let body = "K".repeat(220); // ~55 tokens
        let full = format!("{}{}", md, body);
        let chunks = chunk_markdown(&full, "test.md", 512, 0);

        // "Short." under 50 tokens should be discarded
        // "Hi." under 50 tokens should be discarded
        // "Title" section may be too small too
        // "Keep" section has the big body
        assert!(!chunks.is_empty());
        // No chunk should have heading "Tiny" or "Tiny"-related content
        let headings: Vec<&str> = chunks.iter().filter_map(|c| c.heading.as_deref()).collect();
        assert!(
            !headings.contains(&"Tiny"),
            "Tiny section should be discarded"
        );
    }

    #[test]
    fn test_no_headings_single_chunk() {
        let md = "Just plain text without any headings. ".repeat(10);
        let chunks = chunk_markdown(&md, "test.md", 512, 0);

        assert_eq!(chunks.len(), 1, "No headings -> single chunk");
        assert!(chunks[0].heading.is_none(), "Heading should be None");
    }

    #[test]
    fn test_deterministic_output() {
        let long_body = "X".repeat(220);
        let md = format!(
            "# A\n\n{}\n\n## B\n\n{}\n\n## C\n\n{}\n",
            long_body, long_body, long_body
        );
        let first = chunk_markdown(&md, "test.md", 512, 0);
        let second = chunk_markdown(&md, "test.md", 512, 0);

        assert_eq!(first.len(), second.len());
        for (a, b) in first.iter().zip(second.iter()) {
            assert_eq!(a.heading, b.heading);
            assert_eq!(a.content, b.content);
            assert_eq!(a.token_count, b.token_count);
            assert_eq!(a.chunk_index, b.chunk_index);
        }
    }

    #[test]
    fn test_empty_input() {
        let chunks = chunk_markdown("", "test.md", 512, 0);
        assert!(chunks.is_empty());
    }
}
