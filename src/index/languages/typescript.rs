use crate::index::call_graph::{CallEdge, CallKind, ResolutionStatus};
use crate::index::symbols::{Symbol, SymbolKind};
use miette::{IntoDiagnostic, Result};
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

pub fn extract_symbols(content: &str) -> Result<Option<Vec<Symbol>>> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse TypeScript content"))?;

    let query_str = r#"
        (function_declaration name: (identifier) @name) @symbol
        (class_declaration name: (type_identifier) @name) @symbol
        (interface_declaration name: (type_identifier) @name) @symbol
        (type_alias_declaration name: (type_identifier) @name) @symbol
        (enum_declaration name: (identifier) @name) @symbol
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
            if capture_name == "name" {
                name = capture
                    .node
                    .utf8_text(content.as_bytes())
                    .into_diagnostic()?
                    .to_string();
            } else if capture_name == "symbol" {
                let node = capture.node;
                match node.kind() {
                    "function_declaration" => kind = SymbolKind::Function,
                    "class_declaration" => kind = SymbolKind::Class,
                    "interface_declaration" => kind = SymbolKind::Interface,
                    "type_alias_declaration" => kind = SymbolKind::Type,
                    "enum_declaration" => kind = SymbolKind::Enum,
                    _ => {}
                }

                // Check if exported
                if let Some(parent) = node.parent()
                    && parent.kind() == "export_statement"
                {
                    is_public = true;
                }
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
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse TypeScript content"))?;

    let mut edges = Vec::new();
    collect_ts_call_edges(tree.root_node(), content, &mut edges);
    Ok(edges)
}

fn collect_ts_call_edges(node: tree_sitter::Node, content: &str, edges: &mut Vec<CallEdge>) {
    let kind = node.kind();

    if kind == "call_expression" {
        let caller_name = find_ts_enclosing_function(node, content);
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
                "member_expression" => {
                    // e.g. obj.method() -- member_expression inside call_expression
                    let callee_name = extract_ts_member_name(callee, content);
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
                _ => {
                    // Variable/lambda call or other dynamic pattern
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
    } else if kind == "new_expression" {
        let caller_name = find_ts_enclosing_function(node, content);
        // new_expression has a "constructor" named field for the type identifier.
        let ctor_node = node.child_by_field_name("constructor");
        if let Some(ctor) = ctor_node {
            let name = ctor.utf8_text(content.as_bytes()).unwrap_or("").to_string();
            if !name.is_empty() {
                let evidence = format!("new_expr:new {name}()");
                edges.push(CallEdge {
                    caller_name,
                    callee_name: name,
                    call_kind: CallKind::MethodCall,
                    resolution_status: ResolutionStatus::Resolved,
                    confidence: CallKind::MethodCall.default_confidence(),
                    evidence,
                });
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_ts_call_edges(child, content, edges);
    }
}

/// Extract the method name from a member_expression (e.g. obj.method -> "method").
fn extract_ts_member_name(node: tree_sitter::Node, content: &str) -> String {
    let mut cursor = node.walk();
    let mut last_ident = String::new();
    for child in node.children(&mut cursor) {
        if child.kind() == "property_identifier" || child.kind() == "identifier" {
            last_ident = child
                .utf8_text(content.as_bytes())
                .unwrap_or("")
                .to_string();
        }
    }
    last_ident
}

/// Walk up to find the nearest enclosing function/arrow function and return its name.
fn find_ts_enclosing_function(node: tree_sitter::Node, content: &str) -> String {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "function_declaration" | "function_expression" | "method_definition" => {
                // Find the name identifier
                let mut cursor = parent.walk();
                for child in parent.children(&mut cursor) {
                    if child.kind() == "identifier" || child.kind() == "property_identifier" {
                        return child
                            .utf8_text(content.as_bytes())
                            .unwrap_or("")
                            .to_string();
                    }
                }
            }
            "arrow_function" => {
                // Arrow functions may not have a name; try the variable binding.
                if let Some(var_parent) = parent.parent()
                    && var_parent.kind() == "variable_declarator"
                {
                    let mut cursor = var_parent.walk();
                    for child in var_parent.children(&mut cursor) {
                        if child.kind() == "identifier" {
                            return child
                                .utf8_text(content.as_bytes())
                                .unwrap_or("")
                                .to_string();
                        }
                    }
                }
                return "<arrow>".to_string();
            }
            _ => {}
        }
        current = parent.parent();
    }
    "<module>".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_typescript_symbols() {
        let content = r#"
            export function publicFn() {}
            function privateFn() {}
            export class PublicClass {}
            class PrivateClass {}
            export interface PublicInterface {}
            export type PublicType = string;
            export enum PublicEnum { A }
        "#;

        let symbols = extract_symbols(content).unwrap().unwrap();

        assert!(
            symbols
                .iter()
                .any(|s| s.name == "publicFn" && s.kind == SymbolKind::Function && s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "privateFn" && s.kind == SymbolKind::Function && !s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "PublicClass" && s.kind == SymbolKind::Class && s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "PrivateClass" && s.kind == SymbolKind::Class && !s.is_public)
        );
        assert!(symbols.iter().any(|s| s.name == "PublicInterface"
            && s.kind == SymbolKind::Interface
            && s.is_public));
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "PublicType" && s.kind == SymbolKind::Type && s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "PublicEnum" && s.kind == SymbolKind::Enum && s.is_public)
        );
    }

    #[test]
    fn test_extract_calls_named_function() {
        let content = r#"
            function helper(): number { return 1; }
            function caller(): number {
                return helper();
            }
        "#;

        let edges = extract_calls(content, &[]).unwrap();
        let direct: Vec<&CallEdge> = edges
            .iter()
            .filter(|e| e.call_kind == CallKind::Direct && e.callee_name == "helper")
            .collect();
        assert!(!direct.is_empty(), "should find a DIRECT call to helper");
        assert_eq!(direct[0].caller_name, "caller");
    }

    #[test]
    fn test_extract_calls_method() {
        let content = r#"
            function caller(): void {
                const obj = { greet() { return "hi"; } };
                obj.greet();
            }
        "#;

        let edges = extract_calls(content, &[]).unwrap();
        let method: Vec<&CallEdge> = edges
            .iter()
            .filter(|e| e.call_kind == CallKind::MethodCall && e.callee_name == "greet")
            .collect();
        assert!(!method.is_empty(), "should find a METHOD_CALL to greet");
    }

    #[test]
    fn test_extract_calls_new_expression() {
        let content = r#"
            class Service {}
            function caller(): Service {
                return new Service();
            }
        "#;

        let edges = extract_calls(content, &[]).unwrap();
        let new_edge: Vec<&CallEdge> = edges
            .iter()
            .filter(|e| e.call_kind == CallKind::MethodCall && e.callee_name == "Service")
            .collect();
        assert!(
            !new_edge.is_empty(),
            "should find a METHOD_CALL (new) for Service"
        );
        assert!(new_edge[0].evidence.contains("new"));
    }

    #[test]
    fn test_extract_calls_dynamic_callback() {
        let content = r#"
            function caller(): void {
                const cb = () => { };
                cb();
            }
        "#;

        let edges = extract_calls(content, &[]).unwrap();
        // cb() should appear; whether it's Direct or Dynamic depends on tree-sitter
        // but there should be at least one edge for the cb() invocation.
        let cb_edges: Vec<&CallEdge> = edges
            .iter()
            .filter(|e| e.callee_name == "cb" || e.callee_name.contains("cb"))
            .collect();
        assert!(!cb_edges.is_empty(), "should find a call edge for cb()");
    }
}
