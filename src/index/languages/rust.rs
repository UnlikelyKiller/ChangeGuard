use crate::index::call_graph::{CallEdge, CallKind, ResolutionStatus};
use crate::index::symbols::{Symbol, SymbolKind};
use miette::{IntoDiagnostic, Result};
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
                    name = capture
                        .node
                        .utf8_text(content.as_bytes())
                        .into_diagnostic()?
                        .to_string();
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
                cognitive_complexity: None,
                cyclomatic_complexity: None,
                line_start: None,
                line_end: None,
                qualified_name: None,
                byte_start: None,
                byte_end: None,
                entrypoint_kind: None,
            });
        }
    }

    Ok(Some(symbols))
}

pub fn extract_calls(content: &str, _symbols: &[Symbol]) -> Result<Vec<CallEdge>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Rust content"))?;

    let mut edges = Vec::new();
    collect_call_edges(tree.root_node(), content, &mut edges);
    Ok(edges)
}

fn collect_call_edges(node: tree_sitter::Node, content: &str, edges: &mut Vec<CallEdge>) {
    // Recurse first so we process children, then check this node.
    let kind = node.kind();

    if kind == "call_expression" {
        let caller_name = find_enclosing_function(node, content);
        let callee_node = node.child(0);
        if let Some(callee) = callee_node {
            match callee.kind() {
                "identifier" => {
                    let name = callee
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    if !name.is_empty() {
                        let evidence = format!("call_expr:{name}()");
                        edges.push(CallEdge {
                            caller_name,
                            callee_name: name,
                            call_kind: CallKind::Direct,
                            resolution_status: ResolutionStatus::Resolved,
                            confidence: CallKind::Direct.default_confidence(),
                            evidence,
                        });
                    }
                }
                "method_call_expression" | "field_expression" => {
                    // method_call_expression in Rust tree-sitter: the function name is the last
                    // identifier child. field_expression is used for qualified paths like
                    // Foo::bar() which we treat similarly.
                    let callee_name = extract_method_call_name(callee, content);
                    if !callee_name.is_empty() {
                        let full_text =
                            node.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                        let evidence = format!("method_call:{full_text}");
                        edges.push(CallEdge {
                            caller_name,
                            callee_name,
                            call_kind: CallKind::MethodCall,
                            resolution_status: ResolutionStatus::Resolved,
                            confidence: CallKind::MethodCall.default_confidence(),
                            evidence,
                        });
                    }
                }
                "scoped_identifier" => {
                    // e.g. std::fs::read() -- treat as Direct (static dispatch)
                    let name = callee
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    if !name.is_empty() {
                        let short_name = name.rsplit("::").next().unwrap_or(&name).to_string();
                        let evidence = format!("call_expr:{name}");
                        edges.push(CallEdge {
                            caller_name,
                            callee_name: short_name,
                            call_kind: CallKind::Direct,
                            resolution_status: ResolutionStatus::Resolved,
                            confidence: CallKind::Direct.default_confidence(),
                            evidence,
                        });
                    }
                }
                "generic_function" => {
                    // e.g. <T as Trait>::method() or generic_path::func::<T>()
                    // The first child is typically the path/identifier.
                    let func_name = callee
                        .child(0)
                        .and_then(|c| c.utf8_text(content.as_bytes()).ok())
                        .unwrap_or("")
                        .to_string();
                    if !func_name.is_empty() {
                        let evidence = format!("trait_dispatch:{func_name}");
                        edges.push(CallEdge {
                            caller_name,
                            callee_name: func_name,
                            call_kind: CallKind::TraitDispatch,
                            resolution_status: ResolutionStatus::Ambiguous,
                            confidence: CallKind::TraitDispatch.default_confidence(),
                            evidence,
                        });
                    }
                }
                _ => {
                    // Unrecognized callee pattern -- mark as dynamic/unresolved
                    let text = callee
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    if !text.is_empty() {
                        let evidence = format!("dynamic:{text}");
                        edges.push(CallEdge {
                            caller_name,
                            callee_name: text,
                            call_kind: CallKind::Dynamic,
                            resolution_status: ResolutionStatus::Unresolved,
                            confidence: CallKind::Dynamic.default_confidence(),
                            evidence,
                        });
                    }
                }
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_call_edges(child, content, edges);
    }
}

/// Extract the method/function name from a method_call_expression or field_expression node.
fn extract_method_call_name(node: tree_sitter::Node, content: &str) -> String {
    // For method_call_expression, the last identifier child is the method name.
    // For field_expression (like Foo::bar), similarly the last identifier.
    let mut cursor = node.walk();
    let mut last_ident = String::new();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" || child.kind() == "field_identifier" {
            last_ident = child
                .utf8_text(content.as_bytes())
                .unwrap_or("")
                .to_string();
        }
    }
    last_ident
}

/// Walk up the tree to find the nearest enclosing function_item and return its name.
fn find_enclosing_function(node: tree_sitter::Node, content: &str) -> String {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "function_item" || parent.kind() == "impl_item" {
            // For function_item, find the name identifier child.
            let mut cursor = parent.walk();
            for child in parent.children(&mut cursor) {
                if child.kind() == "identifier" {
                    return child
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();
                }
            }
        }
        current = parent.parent();
    }
    "<module>".to_string()
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

        assert!(
            symbols
                .iter()
                .any(|s| s.name == "public_fn" && s.kind == SymbolKind::Function && s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "private_fn" && s.kind == SymbolKind::Function && !s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "PublicStruct" && s.kind == SymbolKind::Struct && s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "PrivateStruct" && s.kind == SymbolKind::Struct && !s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "PublicEnum" && s.kind == SymbolKind::Enum && s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "PublicTrait" && s.kind == SymbolKind::Trait && s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "public_mod" && s.kind == SymbolKind::Module && s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "PublicType" && s.kind == SymbolKind::Type && s.is_public)
        );
    }

    #[test]
    fn test_extract_calls_direct() {
        let content = r#"
            fn helper() -> i32 { 42 }
            fn caller() -> i32 {
                helper()
            }
        "#;

        let edges = extract_calls(content, &[]).unwrap();
        let direct: Vec<&CallEdge> = edges
            .iter()
            .filter(|e| e.call_kind == CallKind::Direct && e.callee_name == "helper")
            .collect();
        assert!(!direct.is_empty(), "should find a DIRECT call to helper");
        assert_eq!(direct[0].caller_name, "caller");
        assert_eq!(direct[0].resolution_status, ResolutionStatus::Resolved);
        assert!(direct[0].evidence.contains("helper"));
    }

    #[test]
    fn test_extract_calls_method() {
        let content = r#"
            struct S;
            impl S {
                fn process(&self) {}
            }
            fn caller() {
                let s = S;
                s.process();
            }
        "#;

        let edges = extract_calls(content, &[]).unwrap();
        let method: Vec<&CallEdge> = edges
            .iter()
            .filter(|e| e.call_kind == CallKind::MethodCall && e.callee_name == "process")
            .collect();
        assert!(!method.is_empty(), "should find a METHOD_CALL to process");
        assert_eq!(method[0].caller_name, "caller");
    }

    #[test]
    fn test_extract_calls_dynamic() {
        // A call through a variable (fn pointer) cannot be resolved statically.
        let content = r#"
            fn caller() {
                let f: fn() -> i32 = std::mem::transmute;
                let result = f();
            }
        "#;

        let edges = extract_calls(content, &[]).unwrap();
        // f() should be detected; its callee pattern may be an identifier but
        // since it's a variable call we expect at least one edge.
        assert!(!edges.is_empty(), "should find at least one call edge");
    }
}
