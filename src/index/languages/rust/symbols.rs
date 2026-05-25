use crate::index::symbols::{Symbol, SymbolKind};
use miette::{IntoDiagnostic, Result};
use std::collections::BTreeMap;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

pub fn extract_symbols(content: &str) -> Result<Option<Vec<Symbol>>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Rust content"))?;

    let query_str = r#"
        (function_item name: (identifier) @name) @symbol
        (struct_item name: (type_identifier) @name) @symbol
        (enum_item name: (type_identifier) @name) @symbol
        (trait_item name: (type_identifier) @name) @symbol
        (mod_item name: (identifier) @name) @symbol
        (type_item name: (type_identifier) @name) @symbol
        (use_declaration) @symbol
        (impl_item) @symbol
    "#;

    let query = Query::new(&language.into(), query_str).into_diagnostic()?;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

    let mut symbols = Vec::new();

    while let Some(m) = matches.next() {
        let mut name = String::new();
        let mut is_public = false;
        let mut kind = SymbolKind::Function;
        let mut metadata = BTreeMap::new();
        let mut symbol_node: Option<tree_sitter::Node> = None;
        let mut skip = false;
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
                    let node = capture.node;
                    symbol_node = Some(node);
                    match node.kind() {
                        "function_item" => kind = SymbolKind::Function,
                        "struct_item" => kind = SymbolKind::Struct,
                        "enum_item" => kind = SymbolKind::Enum,
                        "trait_item" => kind = SymbolKind::Trait,
                        "mod_item" => kind = SymbolKind::Module,
                        "type_item" => kind = SymbolKind::Type,
                        "impl_item" => {
                            kind = SymbolKind::Type;
                            // Try to find the type name in the impl block
                            let mut walk = node.walk();
                            for child in node.children(&mut walk) {
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
                        "use_declaration" => {
                            // Only handle public re-exports
                            let mut cursor = node.walk();
                            let mut is_pub = false;
                            for child in node.children(&mut cursor) {
                                if child.kind() == "visibility_modifier" {
                                    is_pub = true;
                                    break;
                                }
                            }
                            if is_pub {
                                kind = SymbolKind::Type; // Fallback kind
                                is_public = true;
                                // Extract re-exported name(s)
                                name = extract_use_name(node, content);
                                metadata.insert("reexport".to_string(), "true".to_string());
                            } else {
                                skip = true;
                            }
                        }
                        _ => {}
                    }

                    // Check for visibility and metadata by looking at children and preceding siblings
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "visibility_modifier" {
                            is_public = true;
                        }
                        if child.kind() == "abi"
                            && let Ok(abi_text) = child.utf8_text(content.as_bytes())
                        {
                            metadata.insert("abi".to_string(), abi_text.to_string());
                        }
                    }

                    // Check preceding siblings for attributes
                    if let Some(parent) = node.parent() {
                        let mut pcursor = parent.walk();
                        let siblings: Vec<tree_sitter::Node> =
                            parent.children(&mut pcursor).collect();
                        if let Some(idx) = siblings.iter().position(|s| *s == node) {
                            for i in (0..idx).rev() {
                                let sibling = siblings[i];
                                if sibling.kind() == "attribute_item" {
                                    if let Ok(attr_text) = sibling.utf8_text(content.as_bytes()) {
                                        if attr_text.contains("#[cfg(") {
                                            metadata
                                                .insert("cfg".to_string(), attr_text.to_string());
                                        }
                                        if attr_text.contains("proc_macro") {
                                            metadata.insert(
                                                "macro".to_string(),
                                                "proc_macro".to_string(),
                                            );
                                        }
                                    }
                                } else if sibling.kind() != "line_comment"
                                    && sibling.kind() != "block_comment"
                                {
                                    break;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if !skip && let Some(node) = symbol_node {
            let byte_start = Some(node.start_byte() as i32);
            let byte_end = Some(node.end_byte() as i32);
            let line_start = Some((node.start_position().row + 1) as i32);
            let line_end = Some((node.end_position().row + 1) as i32);

            if !name.is_empty() {
                symbols.push(Symbol {
                    name,
                    kind,
                    is_public,
                    cognitive_complexity: None,
                    cyclomatic_complexity: None,
                    line_start,
                    line_end,
                    qualified_name: None,
                    byte_start,
                    byte_end,
                    entrypoint_kind: None,
                    metadata,
                });
            }
        }
    }

    Ok(Some(symbols))
}

fn extract_use_name(node: tree_sitter::Node, content: &str) -> String {
    let mut last_ident = String::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" || n.kind() == "type_identifier" {
            last_ident = n.utf8_text(content.as_bytes()).unwrap_or("").to_string();
        }
        let mut c = n.walk();
        let children: Vec<_> = n.children(&mut c).collect();
        for child in children.into_iter().rev() {
            stack.push(child);
        }
    }
    last_ident
}
