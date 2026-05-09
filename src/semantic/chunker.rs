use crate::index::symbols::SymbolKind;
use miette::{IntoDiagnostic, Result, miette};
use std::path::Path;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AstChunk {
    pub file_path: String,
    pub name: String,
    pub kind: SymbolKind,
    pub content: String,
    pub docstring: Option<String>,
    pub range: (usize, usize), // (byte_start, byte_end)
    pub lines: (usize, usize), // (line_start, line_end)
    pub offset: usize,         // offset for split chunks
}

impl AstChunk {
    pub fn to_embedding_text(&self) -> String {
        let mut text = String::new();
        if let Some(doc) = &self.docstring {
            text.push_str(doc);
            text.push_str("\n\n");
        }
        text.push_str(&self.content);
        text
    }

    pub fn split(&self, max_chars: usize, overlap: usize) -> Vec<AstChunk> {
        let embedding_text = self.to_embedding_text();
        if embedding_text.len() <= max_chars {
            return vec![self.clone()];
        }

        let mut chunks = Vec::new();
        let mut start = 0;
        while start < embedding_text.len() {
            let end = std::cmp::min(start + max_chars, embedding_text.len());
            let chunk_text = embedding_text[start..end].to_string();

            chunks.push(AstChunk {
                file_path: self.file_path.clone(),
                name: self.name.clone(),
                kind: self.kind.clone(),
                content: chunk_text,
                docstring: None, // docstring is already incorporated into content for split chunks
                range: self.range,
                lines: self.lines,
                offset: start,
            });

            if end == embedding_text.len() {
                break;
            }
            start += max_chars - overlap;
        }
        chunks
    }
}

pub struct AstChunker;

impl AstChunker {
    pub fn chunk_file(path: &Path, content: &str) -> Result<Vec<AstChunk>> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let raw_chunks = match extension {
            "rs" => Self::chunk_rust(path, content)?,
            "ts" | "tsx" | "js" | "jsx" => Self::chunk_typescript(path, content)?,
            "py" => Self::chunk_python(path, content)?,
            _ => vec![],
        };

        let mut final_chunks = Vec::new();
        for chunk in raw_chunks {
            // max_chars roughly 2000 corresponds to ~512 tokens for nomic/bge
            final_chunks.extend(chunk.split(2000, 200));
        }
        Ok(final_chunks)
    }

    fn chunk_rust(path: &Path, content: &str) -> Result<Vec<AstChunk>> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
        parser.set_language(&language.into()).into_diagnostic()?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| miette!("Failed to parse Rust content"))?;

        let query_str = r#"
            (function_item name: (identifier) @name) @symbol
            (struct_item name: (type_identifier) @name) @symbol
            (enum_item name: (type_identifier) @name) @symbol
            (trait_item name: (type_identifier) @name) @symbol
            (impl_item) @symbol
            (mod_item name: (identifier) @name) @symbol
        "#;

        let query = Query::new(&language.into(), query_str).into_diagnostic()?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        let mut chunks = Vec::new();
        let file_path = path.to_string_lossy().to_string();

        while let Some(m) = matches.next() {
            let mut name = String::new();
            let mut kind = SymbolKind::Function;
            let mut symbol_node = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        name = capture
                            .node
                            .utf8_text(content.as_bytes())
                            .into_diagnostic()?
                            .to_string();
                    }
                    "symbol" => {
                        symbol_node = Some(capture.node);
                        match capture.node.kind() {
                            "function_item" => kind = SymbolKind::Function,
                            "struct_item" => kind = SymbolKind::Struct,
                            "enum_item" => kind = SymbolKind::Enum,
                            "trait_item" => kind = SymbolKind::Trait,
                            "impl_item" => {
                                kind = SymbolKind::Type; // impl blocks are tagged as Type/Class-like
                                // Try to find the type name in the impl block
                                let mut walk = capture.node.walk();
                                for child in capture.node.children(&mut walk) {
                                    if child.kind() == "type_identifier" {
                                        name = child
                                            .utf8_text(content.as_bytes())
                                            .into_diagnostic()?
                                            .to_string();
                                        break;
                                    }
                                }
                                if name.is_empty() {
                                    name = "impl".to_string();
                                }
                            }
                            "mod_item" => kind = SymbolKind::Module,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            if let Some(node) = symbol_node {
                let chunk_content = node
                    .utf8_text(content.as_bytes())
                    .into_diagnostic()?
                    .to_string();

                // Simple docstring extraction: look for comments immediately preceding the node
                let mut docstring = Vec::new();
                let mut prev = node.prev_sibling();
                while let Some(p) = prev {
                    if p.kind() == "line_comment" || p.kind() == "block_comment" {
                        docstring.push(
                            p.utf8_text(content.as_bytes())
                                .into_diagnostic()?
                                .trim()
                                .to_string(),
                        );
                        prev = p.prev_sibling();
                    } else if p.kind() == "attribute_item" {
                        // Skip attributes but keep looking for comments
                        prev = p.prev_sibling();
                    } else {
                        break;
                    }
                }
                docstring.reverse();
                let docstring = if docstring.is_empty() {
                    None
                } else {
                    Some(docstring.join("\n"))
                };

                chunks.push(AstChunk {
                    file_path: file_path.clone(),
                    name,
                    kind,
                    content: chunk_content,
                    docstring,
                    range: (node.start_byte(), node.end_byte()),
                    lines: (node.start_position().row + 1, node.end_position().row + 1),
                    offset: 0,
                });
            }
        }

        Ok(chunks)
    }

    fn chunk_typescript(path: &Path, content: &str) -> Result<Vec<AstChunk>> {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        parser.set_language(&language.into()).into_diagnostic()?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| miette!("Failed to parse TypeScript content"))?;

        let query_str = r#"
            (function_declaration name: (identifier) @name) @symbol
            (class_declaration name: (type_identifier) @name) @symbol
            (interface_declaration name: (type_identifier) @name) @symbol
            (method_definition name: (property_identifier) @name) @symbol
            (export_statement declaration: (function_declaration name: (identifier) @name)) @symbol
        "#;

        let query = Query::new(&language.into(), query_str).into_diagnostic()?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        let mut chunks = Vec::new();
        let file_path = path.to_string_lossy().to_string();

        while let Some(m) = matches.next() {
            let mut name = String::new();
            let mut kind = SymbolKind::Function;
            let mut symbol_node = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        name = capture
                            .node
                            .utf8_text(content.as_bytes())
                            .into_diagnostic()?
                            .to_string();
                    }
                    "symbol" => {
                        symbol_node = Some(capture.node);
                        match capture.node.kind() {
                            "function_declaration" => kind = SymbolKind::Function,
                            "class_declaration" => kind = SymbolKind::Class,
                            "interface_declaration" => kind = SymbolKind::Interface,
                            "method_definition" => kind = SymbolKind::Method,
                            "export_statement" => kind = SymbolKind::Function, // usually
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            if let Some(node) = symbol_node {
                let chunk_content = node
                    .utf8_text(content.as_bytes())
                    .into_diagnostic()?
                    .to_string();

                let mut docstring = Vec::new();
                let mut prev = node.prev_sibling();
                while let Some(p) = prev {
                    if p.kind() == "comment" {
                        docstring.push(
                            p.utf8_text(content.as_bytes())
                                .into_diagnostic()?
                                .trim()
                                .to_string(),
                        );
                        prev = p.prev_sibling();
                    } else {
                        break;
                    }
                }
                docstring.reverse();
                let docstring = if docstring.is_empty() {
                    None
                } else {
                    Some(docstring.join("\n"))
                };

                chunks.push(AstChunk {
                    file_path: file_path.clone(),
                    name,
                    kind,
                    content: chunk_content,
                    docstring,
                    range: (node.start_byte(), node.end_byte()),
                    lines: (node.start_position().row + 1, node.end_position().row + 1),
                    offset: 0,
                });
            }
        }

        Ok(chunks)
    }

    fn chunk_python(path: &Path, content: &str) -> Result<Vec<AstChunk>> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE;
        parser.set_language(&language.into()).into_diagnostic()?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| miette!("Failed to parse Python content"))?;

        let query_str = r#"
            (function_definition name: (identifier) @name) @symbol
            (class_definition name: (identifier) @name) @symbol
        "#;

        let query = Query::new(&language.into(), query_str).into_diagnostic()?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        let mut chunks = Vec::new();
        let file_path = path.to_string_lossy().to_string();

        while let Some(m) = matches.next() {
            let mut name = String::new();
            let mut kind = SymbolKind::Function;
            let mut symbol_node = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        name = capture
                            .node
                            .utf8_text(content.as_bytes())
                            .into_diagnostic()?
                            .to_string();
                    }
                    "symbol" => {
                        symbol_node = Some(capture.node);
                        match capture.node.kind() {
                            "function_definition" => kind = SymbolKind::Function,
                            "class_definition" => kind = SymbolKind::Class,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            if let Some(node) = symbol_node {
                let chunk_content = node
                    .utf8_text(content.as_bytes())
                    .into_diagnostic()?
                    .to_string();

                // In Python, docstring is the first child if it's a string expression
                let mut docstring = None;
                let first_child = node
                    .child_by_field_name("body")
                    .and_then(|b| b.children(&mut b.walk()).next());

                match first_child {
                    Some(child) if child.kind() == "expression_statement" => match child.child(0) {
                        Some(expr) if expr.kind() == "string" => {
                            docstring = Some(
                                expr.utf8_text(content.as_bytes())
                                    .into_diagnostic()?
                                    .to_string(),
                            );
                        }
                        _ => {}
                    },
                    _ => {}
                }

                chunks.push(AstChunk {
                    file_path: file_path.clone(),
                    name,
                    kind,
                    content: chunk_content,
                    docstring,
                    range: (node.start_byte(), node.end_byte()),
                    lines: (node.start_position().row + 1, node.end_position().row + 1),
                    offset: 0,
                });
            }
        }

        Ok(chunks)
    }
}
