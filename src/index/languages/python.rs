use crate::index::call_graph::{CallEdge, CallKind, ResolutionStatus};
use crate::index::symbols::{Symbol, SymbolKind};
use miette::{IntoDiagnostic, Result};
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

pub fn extract_symbols(content: &str) -> Result<Option<Vec<Symbol>>> {
    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Python content"))?;

    let query_str = r#"
        (function_definition name: (identifier) @name)
        (class_definition name: (identifier) @name)
    "#;

    let query = Query::new(&language.into(), query_str).into_diagnostic()?;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

    let mut symbols = Vec::new();

    while let Some(m) = matches.next() {
        let mut name = String::new();
        let mut kind = SymbolKind::Function;

        for capture in m.captures {
            let capture_name = query.capture_names()[capture.index as usize];
            if capture_name == "name" {
                name = capture
                    .node
                    .utf8_text(content.as_bytes())
                    .into_diagnostic()?
                    .to_string();

                if let Some(parent) = capture.node.parent() {
                    match parent.kind() {
                        "function_definition" => kind = SymbolKind::Function,
                        "class_definition" => kind = SymbolKind::Class,
                        _ => {}
                    }
                }
            }
        }

        if !name.is_empty() {
            // In Python, leading underscore usually means private
            let is_public = !name.starts_with('_');

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
    let language = tree_sitter_python::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Python content"))?;

    let mut edges = Vec::new();
    collect_py_call_edges(tree.root_node(), content, &mut edges);
    Ok(edges)
}

fn collect_py_call_edges(node: tree_sitter::Node, content: &str, edges: &mut Vec<CallEdge>) {
    let kind = node.kind();

    if kind == "call" {
        let caller_name = find_py_enclosing_function(node, content);
        // In Python tree-sitter, a call node's first child is the function being called.
        let callee_node = node.child(0);
        if let Some(callee) = callee_node {
            match callee.kind() {
                "identifier" => {
                    let name = callee
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    if !name.is_empty() {
                        // Check if this is a known dynamic-dispatch pattern like getattr()
                        let call_kind = if name == "getattr" {
                            CallKind::Dynamic
                        } else {
                            CallKind::Direct
                        };
                        let resolution_status = if call_kind == CallKind::Dynamic {
                            ResolutionStatus::Unresolved
                        } else {
                            ResolutionStatus::Resolved
                        };
                        let confidence = call_kind.default_confidence();
                        let evidence = format!("call_expr:{name}()");
                        edges.push(CallEdge {
                            caller_name,
                            callee_name: name,
                            call_kind,
                            resolution_status,
                            confidence,
                            evidence,
                        });
                    }
                }
                "attribute" => {
                    // e.g. obj.method() -- attribute node in Python tree-sitter
                    let callee_name = extract_py_attribute_name(callee, content);
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
                    // Unrecognized pattern (e.g. subscript, lambda invocation)
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
        collect_py_call_edges(child, content, edges);
    }
}

/// Extract the attribute name from a Python attribute node (e.g. obj.method -> "method").
fn extract_py_attribute_name(node: tree_sitter::Node, content: &str) -> String {
    let mut cursor = node.walk();
    let mut last_ident = String::new();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            last_ident = child
                .utf8_text(content.as_bytes())
                .unwrap_or("")
                .to_string();
        }
    }
    last_ident
}

/// Walk up the tree to find the nearest enclosing function_definition and return its name.
fn find_py_enclosing_function(node: tree_sitter::Node, content: &str) -> String {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "function_definition" {
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
    fn test_extract_python_symbols() {
        let content = r#"
def public_fn():
    pass

def _private_fn():
    pass

class PublicClass:
    pass

class _PrivateClass:
    pass
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
                .any(|s| s.name == "_private_fn" && s.kind == SymbolKind::Function && !s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "PublicClass" && s.kind == SymbolKind::Class && s.is_public)
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "_PrivateClass" && s.kind == SymbolKind::Class && !s.is_public)
        );
    }

    #[test]
    fn test_extract_calls_function() {
        let content = r#"
def helper():
    return 42

def caller():
    return helper()
"#;

        let edges = extract_calls(content, &[]).unwrap();
        let direct: Vec<&CallEdge> = edges
            .iter()
            .filter(|e| e.call_kind == CallKind::Direct && e.callee_name == "helper")
            .collect();
        assert!(!direct.is_empty(), "should find a DIRECT call to helper");
        assert_eq!(direct[0].caller_name, "caller");
        assert_eq!(direct[0].resolution_status, ResolutionStatus::Resolved);
    }

    #[test]
    fn test_extract_calls_method() {
        let content = r#"
class Service:
    def process(self):
        pass

def caller():
    s = Service()
    s.process()
"#;

        let edges = extract_calls(content, &[]).unwrap();
        let method: Vec<&CallEdge> = edges
            .iter()
            .filter(|e| e.call_kind == CallKind::MethodCall && e.callee_name == "process")
            .collect();
        assert!(!method.is_empty(), "should find a METHOD_CALL to process");
    }

    #[test]
    fn test_extract_calls_dynamic_dispatch() {
        let content = r#"
def caller():
    fn = getattr(obj, "method_name")
    fn()
"#;

        let edges = extract_calls(content, &[]).unwrap();
        let getattr_edge: Vec<&CallEdge> = edges
            .iter()
            .filter(|e| e.callee_name == "getattr" && e.call_kind == CallKind::Dynamic)
            .collect();
        assert!(
            !getattr_edge.is_empty(),
            "should find a DYNAMIC call to getattr"
        );
    }
}
