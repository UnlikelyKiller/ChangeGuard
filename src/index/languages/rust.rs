use crate::index::symbols::{Symbol, SymbolKind};
use miette::{IntoDiagnostic, Result};
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

pub fn extract_symbols(content: &str) -> Result<Option<Vec<Symbol>>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser
        .set_language(&language.into())
        .into_diagnostic()?;

    let tree = parser.parse(content, None).ok_or_else(|| {
        miette::miette!("Failed to parse Rust content")
    })?;

    let query_str = r#"
        (function_item name: (identifier) @name) @symbol
        (struct_item name: (type_identifier) @name) @symbol
        (enum_item name: (type_identifier) @name) @symbol
        (trait_item name: (type_identifier) @name) @symbol
        (mod_item name: (identifier) @name) @symbol
        (type_item name: (type_identifier) @name) @symbol
    "#;

    let query = Query::new(&language.into(), query_str).into_diagnostic()?;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

    let mut symbols = Vec::new();

    while let Some(m) = matches.next() {
        let mut name = String::new();
        let mut is_public = false;
        let mut kind = SymbolKind::Function;

        for capture in m.captures {
            let capture_name = query.capture_names()[capture.index as usize];
            match capture_name {
                "name" => {
                    name = capture.node.utf8_text(content.as_bytes()).into_diagnostic()?.to_string();
                }
                "symbol" => {
                    let node = capture.node;
                    match node.kind() {
                        "function_item" => kind = SymbolKind::Function,
                        "struct_item" => kind = SymbolKind::Struct,
                        "enum_item" => kind = SymbolKind::Enum,
                        "trait_item" => kind = SymbolKind::Trait,
                        "mod_item" => kind = SymbolKind::Module,
                        "type_item" => kind = SymbolKind::Type,
                        _ => {}
                    }
                    
                    // Check for visibility by looking at children
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "visibility_modifier" {
                            is_public = true;
                            break;
                        }
                    }
                }
                _ => {}
            }
        }

        if !name.is_empty() {
            symbols.push(Symbol {
                name,
                kind,
                is_public,
            });
        }
    }

    Ok(Some(symbols))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_symbols() {
        let content = r#"
            pub fn public_fn() {}
            fn private_fn() {}
            pub struct PublicStruct;
            struct PrivateStruct {
                pub field: i32,
            }
            pub enum PublicEnum { Variant }
            pub trait PublicTrait {}
            pub mod public_mod {}
            pub type PublicType = i32;
        "#;

        let symbols = extract_symbols(content).unwrap().unwrap();
        
        assert!(symbols.iter().any(|s| s.name == "public_fn" && s.kind == SymbolKind::Function && s.is_public));
        assert!(symbols.iter().any(|s| s.name == "private_fn" && s.kind == SymbolKind::Function && !s.is_public));
        assert!(symbols.iter().any(|s| s.name == "PublicStruct" && s.kind == SymbolKind::Struct && s.is_public));
        assert!(symbols.iter().any(|s| s.name == "PrivateStruct" && s.kind == SymbolKind::Struct && !s.is_public));
        assert!(symbols.iter().any(|s| s.name == "PublicEnum" && s.kind == SymbolKind::Enum && s.is_public));
        assert!(symbols.iter().any(|s| s.name == "PublicTrait" && s.kind == SymbolKind::Trait && s.is_public));
        assert!(symbols.iter().any(|s| s.name == "public_mod" && s.kind == SymbolKind::Module && s.is_public));
        assert!(symbols.iter().any(|s| s.name == "PublicType" && s.kind == SymbolKind::Type && s.is_public));
    }
}
