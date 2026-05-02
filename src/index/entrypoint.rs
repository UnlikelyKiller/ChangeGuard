use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::index::symbols::Symbol;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum EntrypointKind {
    Entrypoint,
    Handler,
    PublicApi,
    Test,
    #[default]
    Internal,
}


impl EntrypointKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntrypointKind::Entrypoint => "ENTRYPOINT",
            EntrypointKind::Handler => "HANDLER",
            EntrypointKind::PublicApi => "PUBLIC_API",
            EntrypointKind::Test => "TEST",
            EntrypointKind::Internal => "INTERNAL",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ENTRYPOINT" => Some(EntrypointKind::Entrypoint),
            "HANDLER" => Some(EntrypointKind::Handler),
            "PUBLIC_API" => Some(EntrypointKind::PublicApi),
            "TEST" => Some(EntrypointKind::Test),
            "INTERNAL" => Some(EntrypointKind::Internal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntrypointStats {
    pub entrypoints: usize,
    pub handlers: usize,
    pub public_apis: usize,
    pub tests: usize,
    pub internal: usize,
}

/// Result of classifying a single symbol's entrypoint kind.
pub struct SymbolClassification {
    pub symbol_name: String,
    pub kind: EntrypointKind,
    pub confidence: f64,
    pub evidence: String,
}

/// Detect entry points in Rust source code.
pub fn detect_rust_entrypoints(content: &str, symbols: &[Symbol]) -> Vec<SymbolClassification> {
    let mut results = Vec::new();
    let mut has_entrypoint = false;
    let mut has_handler = false;

    // First pass: detect attributes and main function via tree-sitter
    let attr_map = parse_rust_attributes(content);

    for symbol in symbols {
        if symbol.kind != crate::index::symbols::SymbolKind::Function {
            continue;
        }

        // Check for #[test] or #[tokio::test]
        if attr_map.get(&symbol.name).is_some_and(|attrs| {
            attrs.iter().any(|a| a == "test" || a == "tokio::test")
        }) {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Test,
                confidence: 1.0,
                evidence: "Rust test function".to_string(),
            });
            continue;
        }

        // Check for #[tokio::main] or #[actix_web::main] — these are ENTRYPOINT
        if attr_map.get(&symbol.name).is_some_and(|attrs| {
            attrs.iter().any(|a| a == "tokio::main" || a == "actix_web::main")
        }) {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Entrypoint,
                confidence: 1.0,
                evidence: "Async entry point attribute".to_string(),
            });
            has_entrypoint = true;
            continue;
        }

        // Check for HTTP handler attributes
        if attr_map.get(&symbol.name).is_some_and(|attrs| {
            attrs.iter().any(|a| is_rust_handler_attr(a))
        }) {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Handler,
                confidence: 1.0,
                evidence: "HTTP handler attribute".to_string(),
            });
            has_handler = true;
            continue;
        }

        // Check for fn main()
        if symbol.name == "main" && !symbol.is_public {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Entrypoint,
                confidence: 1.0,
                evidence: "Rust main function".to_string(),
            });
            has_entrypoint = true;
            continue;
        }

        // Public function — will be classified later based on whether entrypoints exist
        if symbol.is_public {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::PublicApi,
                confidence: 0.7,
                evidence: "Public function".to_string(),
            });
            continue;
        }

        // Internal function
        results.push(SymbolClassification {
            symbol_name: symbol.name.clone(),
            kind: EntrypointKind::Internal,
            confidence: 1.0,
            evidence: "Internal function".to_string(),
        });
    }

    // If no entrypoints or handlers found, all PublicApi classifications are correct.
    // If entrypoints exist, PublicApi should stay as-is (they are public API surface).
    // If no entrypoints and no handlers, all public fns should be PUBLIC_API with
    // updated evidence.
    if !has_entrypoint && !has_handler {
        for r in &mut results {
            if r.kind == EntrypointKind::PublicApi {
                r.evidence = "no entry point found; all public symbols labeled as public API"
                    .to_string();
            }
        }
    }

    results
}

fn is_rust_handler_attr(attr: &str) -> bool {
    let handler_prefixes = [
        "actix_web::get",
        "actix_web::post",
        "actix_web::put",
        "actix_web::delete",
        "actix_web::patch",
        "axum::routing::get",
        "axum::routing::post",
        "axum::routing::put",
        "axum::routing::delete",
        "axum::routing::patch",
        "rocket::get",
        "rocket::post",
        "rocket::put",
        "rocket::delete",
        "rocket::patch",
    ];
    handler_prefixes.iter().any(|p| attr.starts_with(p))
}

/// Parse Rust source to extract a map of function name -> list of attribute paths.
fn parse_rust_attributes(content: &str) -> HashMap<String, Vec<String>> {
    use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    if parser.set_language(&language.into()).is_err() {
        return HashMap::new();
    }

    let tree = match parser.parse(content, None) {
        Some(t) => t,
        None => return HashMap::new(),
    };

    // Query for function_item nodes and their preceding attributes
    let query_str = r#"
        (function_item
          name: (identifier) @func_name)
        (attribute_item) @attr
    "#;

    let query = match Query::new(&language.into(), query_str) {
        Ok(q) => q,
        Err(_) => return HashMap::new(),
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

    let mut attr_map: HashMap<String, Vec<String>> = HashMap::new();

    // Walk the tree manually to find function_items with their attribute items
    fn walk_tree(
        node: tree_sitter::Node,
        content: &str,
        attr_map: &mut HashMap<String, Vec<String>>,
    ) {
        let mut pending_attrs: Vec<String> = Vec::new();

        for i in 0..node.child_count() {
            let child = node.child(i as u32).unwrap();
            match child.kind() {
                "attribute_item" => {
                    if let Ok(attr_text) = child.utf8_text(content.as_bytes()) {
                        // Extract attribute path from #[attr_path] or #[attr_path::nested]
                        let path = extract_rust_attr_path(attr_text);
                        pending_attrs.push(path);
                    }
                }
                "inner_attribute_item" => {
                    // Inner attributes (#![]) don't apply to functions, skip
                }
                "function_item" => {
                    // Get function name
                    for j in 0..child.child_count() {
                        let fc = child.child(j as u32).unwrap();
                        if fc.kind() == "identifier"
                            && let Ok(name) = fc.utf8_text(content.as_bytes()) {
                                attr_map.insert(name.to_string(), pending_attrs.clone());
                            }
                    }
                    pending_attrs.clear();
                    // Also recurse into function body for nested functions
                    walk_tree(child, content, attr_map);
                }
                _ => {
                    // Reset pending attrs for non-attribute, non-function siblings
                    // unless it's a block that might contain both
                    if child.kind() != "declaration_list"
                        && child.kind() != "block"
                        && child.kind() != "mod_item"
                    {
                        pending_attrs.clear();
                    }
                    walk_tree(child, content, attr_map);
                }
            }
        }
    }

    walk_tree(tree.root_node(), content, &mut attr_map);

    // Also consume the query cursor to avoid unused variable warning
    while matches.next().is_some() {}

    attr_map
}

fn extract_rust_attr_path(attr_text: &str) -> String {
    // #[attr_path(args)] or #[attr::path(args)]
    // Strip #[ and trailing ] and any parenthesized arguments
    let trimmed = attr_text.trim().trim_start_matches("#[").trim_end_matches(']');

    // Find the ( and take everything before it
    if let Some(paren_pos) = trimmed.find('(') {
        trimmed[..paren_pos].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Detect entry points in TypeScript/JavaScript source code.
pub fn detect_typescript_entrypoints(
    content: &str,
    symbols: &[Symbol],
    file_path: &str,
) -> Vec<SymbolClassification> {
    let mut results = Vec::new();
    let file_name = file_path
        .rsplit('/')
        .next()
        .unwrap_or(file_path)
        .to_string();

    let entry_file_names = ["main.ts", "main.tsx", "main.js", "main.jsx",
                           "index.ts", "index.tsx", "index.js", "index.jsx",
                           "server.ts", "server.tsx", "server.js", "server.jsx",
                           "app.ts", "app.tsx", "app.js", "app.jsx"];
    let is_entry_file = entry_file_names.contains(&file_name.as_str());

    // Parse route handlers and test blocks via tree-sitter
    let handler_routes = parse_typescript_handlers(content);
    let test_names = parse_typescript_tests(content);
    let has_default_export = content.contains("export default");

    for symbol in symbols {
        // Check if this is a test symbol
        if test_names.contains(&symbol.name) {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Test,
                confidence: 0.9,
                evidence: "TypeScript test function".to_string(),
            });
            continue;
        }

        // Check if this is a route handler
        if let Some(route) = handler_routes.get(&symbol.name) {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Handler,
                confidence: 0.9,
                evidence: format!("Route handler: {}", route),
            });
            continue;
        }

        // Check if this is an entry point (from entry file name or default export)
        if is_entry_file && symbol.is_public {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Entrypoint,
                confidence: 0.8,
                evidence: format!("Entry point file: {}", file_name),
            });
            continue;
        }

        if has_default_export && symbol.is_public {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Entrypoint,
                confidence: 0.7,
                evidence: "Default export".to_string(),
            });
            continue;
        }

        // Public function in non-entry file
        if symbol.is_public {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::PublicApi,
                confidence: 0.7,
                evidence: "Public export".to_string(),
            });
            continue;
        }

        results.push(SymbolClassification {
            symbol_name: symbol.name.clone(),
            kind: EntrypointKind::Internal,
            confidence: 1.0,
            evidence: "Internal symbol".to_string(),
        });
    }

    results
}

fn parse_typescript_handlers(content: &str) -> HashMap<String, String> {
    use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

    let mut handlers: HashMap<String, String> = HashMap::new();
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    if parser.set_language(&language.into()).is_err() {
        return handlers;
    }

    let tree = match parser.parse(content, None) {
        Some(t) => t,
        None => return handlers,
    };

    // Look for call expressions like app.get("/path", handler) or router.post(...)
    let query_str = r#"
        (call_expression
          function: (member_expression
            object: (identifier) @obj
            property: (property_identifier) @method)
          arguments: (arguments . (string) @route)) @call
    "#;

    let query = match Query::new(&language.into(), query_str) {
        Ok(q) => q,
        Err(_) => return handlers,
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

    let http_methods = ["get", "post", "put", "delete", "patch"];
    let handler_objects = ["app", "router"];

    while let Some(m) = matches.next() {
        let mut obj_name = String::new();
        let mut method_name = String::new();
        let mut route = String::new();

        for capture in m.captures {
            let name = query.capture_names()[capture.index as usize];
            match name {
                "obj" => {
                    obj_name = capture.node.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                }
                "method" => {
                    method_name = capture.node.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                }
                "route" => {
                    route = capture.node.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                }
                _ => {}
            }
        }

        if handler_objects.contains(&obj_name.as_str())
            && http_methods.contains(&method_name.as_str())
        {
            let handler_name = format!("{} {}", method_name.to_uppercase(), route);
            handlers.insert(format!("{}_{}", obj_name, method_name), handler_name);
        }
    }

    handlers
}

fn parse_typescript_tests(content: &str) -> Vec<String> {
    use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

    let mut test_names = Vec::new();
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    if parser.set_language(&language.into()).is_err() {
        return test_names;
    }

    let tree = match parser.parse(content, None) {
        Some(t) => t,
        None => return test_names,
    };

    // Find describe() and it() and test() calls
    let query_str = r#"
        (call_expression
          function: (identifier) @func_name
          arguments: (arguments (string) @test_desc)) @call
    "#;

    let query = match Query::new(&language.into(), query_str) {
        Ok(q) => q,
        Err(_) => return test_names,
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

    let test_funcs = ["describe", "it", "test"];

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let name = query.capture_names()[capture.index as usize];
            if name == "func_name" {
                let func_name = capture.node.utf8_text(content.as_bytes()).unwrap_or("");
                if test_funcs.contains(&func_name) {
                    test_names.push(func_name.to_string());
                }
            }
        }
    }

    test_names
}

/// Detect entry points in Python source code.
pub fn detect_python_entrypoints(
    content: &str,
    symbols: &[Symbol],
    file_path: &str,
) -> Vec<SymbolClassification> {
    let mut results = Vec::new();
    let file_name = file_path
        .rsplit('/')
        .next()
        .unwrap_or(file_path)
        .to_string();

    let has_main_block = content.contains("__name__")
        && content.contains("__main__");
    let has_fastapi = content.contains("FastAPI(") || content.contains("FastAPI (");
    let has_flask = content.contains("Flask(") || content.contains("Flask (");

    // Parse decorators via tree-sitter
    let decorator_map = parse_python_decorators(content);

    for symbol in symbols {
        // Check if function is a test
        if symbol.name.starts_with("test_") || symbol.name.ends_with("_test") {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Test,
                confidence: 0.9,
                evidence: "Python test function".to_string(),
            });
            continue;
        }

        // Check for handler decorators
        if let Some(decorators) = decorator_map.get(&symbol.name)
            && decorators.iter().any(|d| is_python_handler_decorator(d)) {
                results.push(SymbolClassification {
                    symbol_name: symbol.name.clone(),
                    kind: EntrypointKind::Handler,
                    confidence: 0.9,
                    evidence: "HTTP handler decorator".to_string(),
                });
                continue;
            }

        // Check for entrypoint indicators
        if has_main_block && symbol.name == "main" {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Entrypoint,
                confidence: 0.9,
                evidence: "Python __main__ block with main()".to_string(),
            });
            continue;
        }

        // FastAPI/Flask app creation
        if (has_fastapi || has_flask) && symbol.is_public {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::Entrypoint,
                confidence: 0.8,
                evidence: if has_fastapi {
                    "FastAPI application".to_string()
                } else {
                    "Flask application".to_string()
                },
            });
            continue;
        }

        // Public function in a file with __main__ block
        if symbol.is_public && has_main_block {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::PublicApi,
                confidence: 0.7,
                evidence: "Public function in module with __main__".to_string(),
            });
            continue;
        }

        // Public function without entry point context
        if symbol.is_public {
            results.push(SymbolClassification {
                symbol_name: symbol.name.clone(),
                kind: EntrypointKind::PublicApi,
                confidence: 0.7,
                evidence: "Public function".to_string(),
            });
            continue;
        }

        results.push(SymbolClassification {
            symbol_name: symbol.name.clone(),
            kind: EntrypointKind::Internal,
            confidence: 1.0,
            evidence: "Internal function".to_string(),
        });
    }

    // If __main__ block exists but no main() function was found,
    // add a module-level entrypoint marker
    if has_main_block && !results.iter().any(|r| r.kind == EntrypointKind::Entrypoint) {
        results.push(SymbolClassification {
            symbol_name: format!("{}.__main__", file_name.trim_end_matches(".py")),
            kind: EntrypointKind::Entrypoint,
            confidence: 0.8,
            evidence: "Python __main__ block".to_string(),
        });
    }

    // If FastAPI/Flask app exists but no handlers found, mark public functions as public API
    if (has_fastapi || has_flask)
        && !results.iter().any(|r| r.kind == EntrypointKind::Handler)
    {
        for r in &mut results {
            if r.kind == EntrypointKind::PublicApi {
                r.evidence = if has_fastapi {
                    "FastAPI public endpoint".to_string()
                } else {
                    "Flask public endpoint".to_string()
                };
            }
        }
    }

    results
}

fn is_python_handler_decorator(decorator: &str) -> bool {
    let patterns = [
        "@app.get",
        "@app.post",
        "@app.put",
        "@app.delete",
        "@app.patch",
        "@app.route",
        "@router.get",
        "@router.post",
        "@router.put",
        "@router.delete",
        "@router.patch",
    ];
    patterns.iter().any(|p| decorator.starts_with(p))
}

fn parse_python_decorators(content: &str) -> HashMap<String, Vec<String>> {
    use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

    let mut dec_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE;
    if parser.set_language(&language.into()).is_err() {
        return dec_map;
    }

    let tree = match parser.parse(content, None) {
        Some(t) => t,
        None => return dec_map,
    };

    // Find decorated definitions
    let query_str = r#"
        (decorated_definition
          (decorator) @deco
          definition: (function_definition name: (identifier) @func_name)) @def
    "#;

    let query = match Query::new(&language.into(), query_str) {
        Ok(q) => q,
        Err(_) => return dec_map,
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

    while let Some(m) = matches.next() {
        let mut func_name = String::new();
        let mut decorator = String::new();

        for capture in m.captures {
            let name = query.capture_names()[capture.index as usize];
            match name {
                "func_name" => {
                    func_name = capture
                        .node
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();
                }
                "deco" => {
                    decorator = capture
                        .node
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();
                }
                _ => {}
            }
        }

        if !func_name.is_empty() && !decorator.is_empty() {
            dec_map
                .entry(func_name)
                .or_default()
                .push(decorator);
        }
    }

    dec_map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::symbols::{Symbol, SymbolKind};

    #[test]
    fn test_entrypoint_kind_serialization() {
        assert_eq!(
            serde_json::to_string(&EntrypointKind::Entrypoint).unwrap(),
            "\"ENTRYPOINT\""
        );
        assert_eq!(
            serde_json::to_string(&EntrypointKind::PublicApi).unwrap(),
            "\"PUBLIC_API\""
        );
        let deserialized: EntrypointKind =
            serde_json::from_str("\"HANDLER\"").unwrap();
        assert_eq!(deserialized, EntrypointKind::Handler);
    }

    #[test]
    fn test_entrypoint_kind_default() {
        assert_eq!(EntrypointKind::default(), EntrypointKind::Internal);
    }

    #[test]
    fn test_rust_entrypoint_main() {
        let content = r#"
fn main() {
    println!("hello");
}
"#;
        let symbols = vec![Symbol {
            name: "main".to_string(),
            kind: SymbolKind::Function,
            is_public: false,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: None,
            byte_start: None,
            byte_end: None,
        entrypoint_kind: None,
        }];
        let results = detect_rust_entrypoints(content, &symbols);
        assert!(results.iter().any(|r| r.kind == EntrypointKind::Entrypoint && r.symbol_name == "main"));
    }

    #[test]
    fn test_rust_entrypoint_test() {
        let content = r#"
#[test]
fn test_foo() {
    assert_eq!(1, 1);
}
"#;
        let symbols = vec![Symbol {
            name: "test_foo".to_string(),
            kind: SymbolKind::Function,
            is_public: false,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: None,
            byte_start: None,
            byte_end: None,
        entrypoint_kind: None,
        }];
        let results = detect_rust_entrypoints(content, &symbols);
        assert!(results.iter().any(|r| r.kind == EntrypointKind::Test && r.symbol_name == "test_foo"));
    }

    #[test]
    fn test_rust_entrypoint_public_api_library() {
        let content = r#"
pub fn my_lib_fn() -> i32 { 42 }
pub fn another_pub() -> bool { true }
"#;
        let symbols = vec![
            Symbol {
                name: "my_lib_fn".to_string(),
                kind: SymbolKind::Function,
                is_public: true,
                cognitive_complexity: None,
                cyclomatic_complexity: None,
                line_start: None,
                line_end: None,
                qualified_name: None,
                byte_start: None,
                byte_end: None,
        entrypoint_kind: None,
            },
            Symbol {
                name: "another_pub".to_string(),
                kind: SymbolKind::Function,
                is_public: true,
                cognitive_complexity: None,
                cyclomatic_complexity: None,
                line_start: None,
                line_end: None,
                qualified_name: None,
                byte_start: None,
                byte_end: None,
        entrypoint_kind: None,
            },
        ];
        let results = detect_rust_entrypoints(content, &symbols);
        let pub_apis: Vec<_> = results
            .iter()
            .filter(|r| r.kind == EntrypointKind::PublicApi)
            .collect();
        assert_eq!(pub_apis.len(), 2);
        assert!(pub_apis.iter().any(|r| r.evidence.contains("no entry point found")));
    }

    #[test]
    fn test_rust_entrypoint_handler() {
        let content = r#"
#[actix_web::get("/users")]
async fn get_users() -> HttpResponse {
    HttpResponse::Ok().finish()
}
"#;
        let symbols = vec![Symbol {
            name: "get_users".to_string(),
            kind: SymbolKind::Function,
            is_public: true,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: None,
            byte_start: None,
            byte_end: None,
        entrypoint_kind: None,
        }];
        let results = detect_rust_entrypoints(content, &symbols);
        assert!(results.iter().any(|r| r.kind == EntrypointKind::Handler && r.symbol_name == "get_users"));
    }

    #[test]
    fn test_typescript_entry_file() {
        let content = r#"
export function handler() { }
"#;
        let symbols = vec![Symbol {
            name: "handler".to_string(),
            kind: SymbolKind::Function,
            is_public: true,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: None,
            byte_start: None,
            byte_end: None,
        entrypoint_kind: None,
        }];
        let results = detect_typescript_entrypoints(content, &symbols, "server.ts");
        assert!(results.iter().any(|r| r.kind == EntrypointKind::Entrypoint));
    }

    #[test]
    fn test_python_main_block() {
        let content = r#"
def run():
    pass

if __name__ == "__main__":
    run()
"#;
        let symbols = vec![Symbol {
            name: "run".to_string(),
            kind: SymbolKind::Function,
            is_public: false,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: None,
            byte_start: None,
            byte_end: None,
        entrypoint_kind: None,
        }];
        let results = detect_python_entrypoints(content, &symbols, "main.py");
        assert!(results.iter().any(|r| r.kind == EntrypointKind::Entrypoint));
    }

    #[test]
    fn test_python_test_function() {
        let symbols = vec![Symbol {
            name: "test_something".to_string(),
            kind: SymbolKind::Function,
            is_public: false,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: None,
            byte_start: None,
            byte_end: None,
        entrypoint_kind: None,
        }];
        let results = detect_python_entrypoints("", &symbols, "test_module.py");
        assert!(results.iter().any(|r| r.kind == EntrypointKind::Test && r.symbol_name == "test_something"));
    }

    #[test]
    fn test_extract_rust_attr_path() {
        assert_eq!(extract_rust_attr_path("#[test]"), "test");
        assert_eq!(extract_rust_attr_path("#[tokio::test]"), "tokio::test");
        assert_eq!(extract_rust_attr_path("#[actix_web::get(\"/users\")]"), "actix_web::get");
        assert_eq!(extract_rust_attr_path("#[derive(Debug)]"), "derive");
    }
}