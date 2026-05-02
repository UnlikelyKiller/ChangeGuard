use crate::index::call_graph::{CallEdge, CallKind, ResolutionStatus};
use crate::index::data_models::{ExtractedModel, ModelKind};
use crate::index::observability::{
    ErrorHandlingPattern, LogLevel, LoggingPattern, TelemetryPattern,
};
use crate::index::routes::ExtractedRoute;
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

pub fn extract_routes(content: &str, _symbols: &[Symbol]) -> Result<Vec<ExtractedRoute>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Rust content"))?;

    let mut routes = Vec::new();
    collect_rust_routes(tree.root_node(), content, &mut routes);
    Ok(routes)
}

/// HTTP methods recognized in Actix/Rocket route attributes and Axum method calls.
const HTTP_METHODS: &[&str] = &["get", "post", "put", "delete", "patch", "head", "options"];

/// Convert a lowercase method string to uppercase (e.g. "get" -> "GET").
fn method_upper(m: &str) -> String {
    m.to_ascii_uppercase()
}

fn collect_rust_routes(node: tree_sitter::Node, content: &str, routes: &mut Vec<ExtractedRoute>) {
    let kind = node.kind();

    // --- Attribute-based routes: Actix and Rocket ---
    if kind == "attribute_item" {
        let attr_text = node.utf8_text(content.as_bytes()).unwrap_or("");

        // Check for Actix-style: #[get("/path")] or #[actix_web::get("/path")]
        // Check for Rocket-style: #[rocket::get("/path")]
        for &method_str in HTTP_METHODS {
            // Actix: #[method("...")] or #[actix_web::method("...")]
            let actix_short = format!("#[{method_str}(");
            let actix_fq = format!("#[actix_web::{method_str}(");
            // Rocket: #[rocket::method("...")]
            let rocket_attr = format!("#[rocket::{method_str}(");

            if (attr_text.contains(&actix_short) || attr_text.contains(&actix_fq))
                && let Some(path) = extract_path_from_attr(attr_text)
            {
                let handler_name = find_next_function_name(node, content);
                let framework = "actix";
                let evidence = format!("#[{method_str}(\"{path}\")] on {handler_name}");
                routes.push(ExtractedRoute {
                    method: method_upper(method_str),
                    path_pattern: path,
                    handler_name,
                    framework: framework.to_string(),
                    route_source: "DECORATOR".to_string(),
                    mount_prefix: None,
                    is_dynamic: false,
                    route_confidence: 1.0,
                    evidence,
                });
            } else if attr_text.contains(&rocket_attr)
                && let Some(path) = extract_path_from_attr(attr_text)
            {
                let handler_name = find_next_function_name(node, content);
                let evidence = format!("#[rocket::{method_str}(\"{path}\")] on {handler_name}");
                routes.push(ExtractedRoute {
                    method: method_upper(method_str),
                    path_pattern: path,
                    handler_name,
                    framework: "rocket".to_string(),
                    route_source: "DECORATOR".to_string(),
                    mount_prefix: None,
                    is_dynamic: false,
                    route_confidence: 1.0,
                    evidence,
                });
            }
        }
    }

    // --- Method-chain routes: Axum .route("/path", get(handler)) ---
    if kind == "call_expression" {
        // Look for .route("path", method(handler))
        // In Rust tree-sitter, method calls like `obj.route(...)` produce a
        // `field_expression` callee, not `method_call_expression`.
        let func_node = node.child(0);
        if let Some(func) = func_node
            && (func.kind() == "method_call_expression" || func.kind() == "field_expression")
        {
            // Check if the method/field name is "route"
            let method_name = extract_method_call_name_simple(func, content);
            if method_name == "route" {
                // Arguments are in the arguments node
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "arguments" {
                        extract_axum_route(&child, content, routes);
                        break;
                    }
                }
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_rust_routes(child, content, routes);
    }
}

/// Extract the path string from an attribute text like `#[get("/path")]`.
/// Looks for the content between the first `("` and `")`.
fn extract_path_from_attr(attr_text: &str) -> Option<String> {
    // Find the opening quote after the parenthesis
    let start = attr_text.find("(\"")? + 2;
    let end = attr_text[start..].find("\")")? + start;
    Some(attr_text[start..end].to_string())
}

/// Given an attribute_item node, find the next sibling that is a function_item
/// and return its name. Returns "<unknown>" if not found.
fn find_next_function_name(node: tree_sitter::Node, content: &str) -> String {
    // attribute_item is typically a child of the source_file or a block.
    // The next sibling after the attribute_item should be the decorated item.
    if let Some(parent) = node.parent() {
        let mut cursor = parent.walk();
        let mut found_self = false;
        for child in parent.children(&mut cursor) {
            if child == node {
                found_self = true;
                continue;
            }
            if found_self && child.kind() == "function_item" {
                // Extract the function name (first identifier child)
                let mut fc = child.walk();
                for fchild in child.children(&mut fc) {
                    if fchild.kind() == "identifier" {
                        return fchild
                            .utf8_text(content.as_bytes())
                            .unwrap_or("<unknown>")
                            .to_string();
                    }
                }
            }
        }
    }
    "<unknown>".to_string()
}

/// Extract the method name from a method_call_expression node (simplified version
/// that returns the identifier text of the method field).
fn extract_method_call_name_simple(node: tree_sitter::Node, content: &str) -> String {
    let mut cursor = node.walk();
    let mut last_ident = String::new();
    for child in node.children(&mut cursor) {
        if child.kind() == "field_identifier" || child.kind() == "identifier" {
            last_ident = child
                .utf8_text(content.as_bytes())
                .unwrap_or("")
                .to_string();
        }
    }
    last_ident
}

/// Given an Axum `.route("/path", get(handler))` arguments node, extract the route.
fn extract_axum_route(
    args_node: &tree_sitter::Node,
    content: &str,
    routes: &mut Vec<ExtractedRoute>,
) {
    // Arguments node children: comma-separated expressions.
    // First arg: string literal (the path)
    // Second arg: method call like get(handler), post(handler), etc.
    let mut args = Vec::new();
    let mut cursor = args_node.walk();
    for child in args_node.children(&mut cursor) {
        if child.kind() != "," && child.kind() != "(" && child.kind() != ")" {
            args.push(child);
        }
    }

    if args.len() < 2 {
        return;
    }

    // Extract path from first argument (string literal)
    let path_node = args[0];
    let path_text = path_node.utf8_text(content.as_bytes()).unwrap_or("");
    let path = path_text
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(path_text)
        .to_string();

    if path.is_empty() {
        return;
    }

    // Extract method and handler from second argument: e.g. get(handler)
    let method_arg = args[1];
    let method_arg_text = method_arg.utf8_text(content.as_bytes()).unwrap_or("");

    for &method_str in HTTP_METHODS {
        let prefix = format!("{method_str}(");
        if method_arg_text.starts_with(&prefix) {
            // Extract handler name from inside: get(handler) -> handler
            let inner = &method_arg_text[prefix.len()..];
            let handler_name = inner.trim_end_matches(')').to_string();

            let evidence = format!(".route(\"{path}\", {method_arg_text})");
            routes.push(ExtractedRoute {
                method: method_upper(method_str),
                path_pattern: path,
                handler_name,
                framework: "axum".to_string(),
                route_source: "ROUTER_CHAIN".to_string(),
                mount_prefix: None,
                is_dynamic: false,
                route_confidence: 1.0,
                evidence,
            });
            return;
        }
    }
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

/// Directories that conventionally indicate data model definitions in Rust projects.
const RUST_MODEL_DIRS: &[&str] = &["models/", "entities/", "schema/", "domain/"];

/// Serde derive traits that indicate a data model.
const SERDE_TRAITS: &[&str] = &[
    "Serialize",
    "Deserialize",
    "serde::Serialize",
    "serde::Deserialize",
];

/// ORM derive traits that indicate a data model.
const ORM_TRAITS: &[&str] = &[
    "sqlx::FromRow",
    "FromRow",
    "diesel::Queryable",
    "Queryable",
    "diesel::Insertable",
    "Insertable",
    "diesel::Identifiable",
    "Identifiable",
    "diesel::Associations",
    "Associations",
];

pub fn extract_data_models(
    content: &str,
    file_path: &str,
    _symbols: &[Symbol],
) -> Result<Vec<ExtractedModel>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Rust content"))?;

    let mut models = Vec::new();

    // Walk the AST for struct_item nodes
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }

        if node.kind() != "struct_item" {
            continue;
        }

        // Extract struct name
        let struct_name = node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(content.as_bytes()).ok())
            .unwrap_or("")
            .to_string();

        if struct_name.is_empty() {
            continue;
        }

        // Check preceding sibling for attribute_item nodes
        let mut serde_traits_found: Vec<&str> = Vec::new();
        let mut orm_traits_found: Vec<&str> = Vec::new();
        let mut has_serde_attr = false;

        if let Some(parent) = node.parent() {
            let mut pcursor = parent.walk();
            let siblings: Vec<tree_sitter::Node> = parent.children(&mut pcursor).collect();

            // Find the index of our struct_item
            if let Some(idx) = siblings.iter().position(|s| *s == node) {
                // Check siblings before this node for attribute_item
                for i in (0..idx).rev() {
                    let sibling = siblings[i];
                    if sibling.kind() != "attribute_item" {
                        break;
                    }
                    let attr_text = sibling
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();

                    // Check for serde derives
                    for &trait_name in SERDE_TRAITS {
                        if attr_text.contains(trait_name)
                            && !serde_traits_found.contains(&trait_name)
                        {
                            serde_traits_found.push(trait_name);
                        }
                    }

                    // Check for ORM derives
                    for &trait_name in ORM_TRAITS {
                        if attr_text.contains(trait_name) && !orm_traits_found.contains(&trait_name)
                        {
                            orm_traits_found.push(trait_name);
                        }
                    }

                    // Check for #[serde(rename_all = "...")] attribute
                    if attr_text.contains("#[serde(") && attr_text.contains("rename_all") {
                        has_serde_attr = true;
                    }
                }
            }
        }

        // Determine model classification based on detected evidence
        if !orm_traits_found.is_empty() {
            let traits_str = orm_traits_found.join(", ");
            models.push(ExtractedModel {
                model_name: struct_name,
                language: "Rust".to_string(),
                model_kind: ModelKind::Struct,
                confidence: 1.0,
                evidence: format!("derive: {traits_str}"),
            });
        } else if !serde_traits_found.is_empty() {
            let traits_str = serde_traits_found.join(", ");
            models.push(ExtractedModel {
                model_name: struct_name,
                language: "Rust".to_string(),
                model_kind: ModelKind::Struct,
                confidence: 1.0,
                evidence: format!("derive: {traits_str}"),
            });
        } else if has_serde_attr {
            models.push(ExtractedModel {
                model_name: struct_name,
                language: "Rust".to_string(),
                model_kind: ModelKind::Struct,
                confidence: 0.9,
                evidence: "attr: serde(rename_all)".to_string(),
            });
        } else {
            // Directory convention check: only if no serialization derives
            let dir_match = RUST_MODEL_DIRS.iter().find(|dir| file_path.contains(*dir));
            if let Some(dir) = dir_match {
                models.push(ExtractedModel {
                    model_name: struct_name,
                    language: "Rust".to_string(),
                    model_kind: ModelKind::Struct,
                    confidence: 0.7,
                    evidence: format!("dir: {dir}"),
                });
            }
        }
    }

    Ok(models)
}

/// Logging macros and their level/framework mappings for Rust.
/// Note: macro names here do NOT include the `!` suffix; tree-sitter gives us
/// just the identifier part (e.g. "info" not "info!").
const RUST_LOG_MACROS: &[(&str, Option<LogLevel>, &str)] = &[
    ("info", Some(LogLevel::Info), "log"),
    ("warn", Some(LogLevel::Warn), "log"),
    ("error", Some(LogLevel::Error), "log"),
    ("debug", Some(LogLevel::Debug), "log"),
    ("trace", Some(LogLevel::Trace), "log"),
    ("println", Some(LogLevel::Info), "println"),
    ("eprintln", Some(LogLevel::Error), "println"),
];

pub fn extract_logging_patterns(content: &str) -> Result<Vec<LoggingPattern>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Rust content"))?;

    let mut patterns = Vec::new();
    collect_rust_logging_patterns(tree.root_node(), content, &mut patterns);

    // Cap at 1000 patterns per file
    patterns.truncate(1000);
    Ok(patterns)
}

fn collect_rust_logging_patterns(
    node: tree_sitter::Node,
    content: &str,
    patterns: &mut Vec<LoggingPattern>,
) {
    if node.kind() == "macro_invocation" {
        // A macro_invocation has children: the macro name (identifier or scoped_identifier)
        // and the macro body (token_tree or similar).
        let mut cursor = node.walk();
        let children: Vec<tree_sitter::Node> = node.children(&mut cursor).collect();

        // The first child should be the macro name
        if let Some(name_node) = children.first() {
            let full_name = name_node
                .utf8_text(content.as_bytes())
                .unwrap_or("")
                .to_string();

            // Extract the simple macro name (last segment for scoped identifiers)
            let simple_name = full_name
                .rsplit("::")
                .next()
                .unwrap_or(&full_name)
                .to_string();

            // Determine framework: check for qualified names
            let framework = if full_name.starts_with("tracing::") {
                "tracing".to_string()
            } else if full_name.starts_with("log::") {
                "log".to_string()
            } else if simple_name == "println" || simple_name == "eprintln" {
                "println".to_string()
            } else {
                // Default to "log" for unqualified logging macros
                "log".to_string()
            };

            // Find matching log level
            for &(macro_name, level, _framework) in RUST_LOG_MACROS {
                if simple_name == macro_name {
                    let line_start = node.start_position().row as i32 + 1;
                    let in_test = is_in_rust_test(node, content);
                    let evidence = node.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                    // Truncate evidence to a reasonable length
                    let evidence = if evidence.len() > 200 {
                        format!("{}...", &evidence[..197])
                    } else {
                        evidence
                    };

                    patterns.push(LoggingPattern {
                        line_start,
                        level,
                        framework: framework.clone(),
                        in_test,
                        confidence: if in_test { 0.7 } else { 1.0 },
                        evidence,
                    });
                    break;
                }
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_rust_logging_patterns(child, content, patterns);
    }
}

/// Walk up the tree to check if the node is inside a function annotated with
/// `#[test]` or `#[tokio::test]`.
fn is_in_rust_test(node: tree_sitter::Node, content: &str) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "function_item" {
            // Check if this function has a #[test] or #[tokio::test] attribute
            if has_test_attribute(parent, content) {
                return true;
            }
        }
        current = parent.parent();
    }
    false
}

pub fn extract_error_handling(content: &str) -> Result<Vec<ErrorHandlingPattern>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Rust content"))?;

    let mut patterns = Vec::new();
    collect_rust_error_handling(tree.root_node(), content, &mut patterns);

    // Cap at 1000 patterns per file
    patterns.truncate(1000);
    Ok(patterns)
}

fn collect_rust_error_handling(
    node: tree_sitter::Node,
    content: &str,
    patterns: &mut Vec<ErrorHandlingPattern>,
) {
    let kind = node.kind();

    match kind {
        "match_expression" => {
            // match expressions on Result/Option
            let line_start = node.start_position().row as i32 + 1;
            let in_test = is_in_rust_test(node, content);
            patterns.push(ErrorHandlingPattern {
                line_start,
                level: Some(LogLevel::Info),
                framework: "match_result".to_string(),
                in_test,
                confidence: if in_test { 0.7 } else { 1.0 },
                evidence: "syntactic: match expression".to_string(),
            });
        }
        "call_expression" => {
            // Check for .unwrap() or .expect() calls
            // In tree-sitter-rust, `x.unwrap()` is a call_expression where the callee
            // is a method_call_expression or field_expression.
            if let Some(callee) = node.child(0)
                && (callee.kind() == "method_call_expression"
                    || callee.kind() == "field_expression")
            {
                let method_name = extract_method_call_name_simple(callee, content);
                if method_name == "unwrap" {
                    let line_start = node.start_position().row as i32 + 1;
                    let in_test = is_in_rust_test(node, content);
                    patterns.push(ErrorHandlingPattern {
                        line_start,
                        level: Some(LogLevel::Error),
                        framework: "unwrap".to_string(),
                        in_test,
                        confidence: if in_test { 0.7 } else { 1.0 },
                        evidence: "syntactic: unwrap call".to_string(),
                    });
                } else if method_name == "expect" {
                    let line_start = node.start_position().row as i32 + 1;
                    let in_test = is_in_rust_test(node, content);
                    patterns.push(ErrorHandlingPattern {
                        line_start,
                        level: Some(LogLevel::Warn),
                        framework: "expect".to_string(),
                        in_test,
                        confidence: if in_test { 0.7 } else { 1.0 },
                        evidence: "syntactic: expect call".to_string(),
                    });
                }
            }
        }
        "try_expression" => {
            // ? operator
            let line_start = node.start_position().row as i32 + 1;
            let in_test = is_in_rust_test(node, content);
            patterns.push(ErrorHandlingPattern {
                line_start,
                level: Some(LogLevel::Info),
                framework: "try_operator".to_string(),
                in_test,
                confidence: if in_test { 0.7 } else { 1.0 },
                evidence: "syntactic: try operator".to_string(),
            });
        }
        "macro_invocation" => {
            // Check for anyhow! macro
            let mut cursor = node.walk();
            let children: Vec<tree_sitter::Node> = node.children(&mut cursor).collect();
            if let Some(name_node) = children.first() {
                let full_name = name_node
                    .utf8_text(content.as_bytes())
                    .unwrap_or("")
                    .to_string();
                let simple_name = full_name.rsplit("::").next().unwrap_or(&full_name);
                if simple_name == "anyhow" {
                    let line_start = node.start_position().row as i32 + 1;
                    let in_test = is_in_rust_test(node, content);
                    patterns.push(ErrorHandlingPattern {
                        line_start,
                        level: Some(LogLevel::Error),
                        framework: "anyhow".to_string(),
                        in_test,
                        confidence: if in_test { 0.7 } else { 1.0 },
                        evidence: "syntactic: anyhow macro".to_string(),
                    });
                }
            }
        }
        "attribute_item" => {
            // Check for #[derive(Error)] (thiserror)
            let attr_text = node.utf8_text(content.as_bytes()).unwrap_or("");
            if attr_text.contains("Error") && attr_text.contains("derive") {
                let line_start = node.start_position().row as i32 + 1;
                let in_test = is_in_rust_test(node, content);
                patterns.push(ErrorHandlingPattern {
                    line_start,
                    level: Some(LogLevel::Info),
                    framework: "thiserror".to_string(),
                    in_test,
                    confidence: if in_test { 0.7 } else { 1.0 },
                    evidence: "syntactic: derive Error".to_string(),
                });
            }
        }
        _ => {}
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_rust_error_handling(child, content, patterns);
    }
}

/// Check if a function_item node has a #[test] or #[tokio::test] attribute.
fn has_test_attribute(func_node: tree_sitter::Node, content: &str) -> bool {
    if let Some(parent) = func_node.parent() {
        // In tree-sitter-rust, attributes may appear as preceding siblings
        // or as part of an attribute_item
        let mut cursor = parent.walk();
        let siblings: Vec<tree_sitter::Node> = parent.children(&mut cursor).collect();

        if let Some(idx) = siblings.iter().position(|s| *s == func_node) {
            for i in (0..idx).rev() {
                let sibling = siblings[i];
                if sibling.kind() == "attribute_item" {
                    let attr_text = sibling.utf8_text(content.as_bytes()).unwrap_or("");
                    if attr_text.contains("#[test]") || attr_text.contains("#[tokio::test]") {
                        return true;
                    }
                } else {
                    // Attributes must be immediately before the function
                    break;
                }
            }
        }
    }
    false
}

/// Prometheus metric macros that indicate telemetry instrumentation.
const PROMETHEUS_MACROS: &[&str] = &[
    "histogram_observe",
    "histogram_observed",
    "gauge",
    "counter",
    "inc",
    "observe",
];

/// Metrics crate macros that indicate telemetry instrumentation.
const METRICS_MACROS: &[&str] = &["counter", "gauge", "histogram"];

pub fn extract_telemetry_patterns(content: &str) -> Result<Vec<TelemetryPattern>> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Rust content"))?;

    let mut patterns = Vec::new();
    collect_rust_telemetry_patterns(tree.root_node(), content, &mut patterns);

    // Also do line-based heuristic matching for telemetry.* patterns
    for (line_idx, line) in content.lines().enumerate() {
        let line_lower = line.to_ascii_lowercase();
        // Match telemetry.* or monitoring.* identifiers (but not in comments)
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("///") {
            continue;
        }
        if line_lower.contains("telemetry") || line_lower.contains("monitoring") {
            // Check it's not already matched by tree-sitter (avoid duplicates for same line)
            let line_start = (line_idx + 1) as i32;
            let already_matched = patterns.iter().any(|p| p.line_start == line_start);
            if !already_matched {
                patterns.push(TelemetryPattern {
                    line_start,
                    level: Some(LogLevel::Trace),
                    framework: "custom".to_string(),
                    in_test: false,
                    confidence: 0.7,
                    evidence: "heuristic: telemetry.* pattern match".to_string(),
                });
            }
        }
    }

    // Cap at 1000 patterns per file
    patterns.truncate(1000);
    Ok(patterns)
}

fn collect_rust_telemetry_patterns(
    node: tree_sitter::Node,
    content: &str,
    patterns: &mut Vec<TelemetryPattern>,
) {
    let kind = node.kind();

    // Check for #[instrument] or #[tracing::instrument] attributes
    if kind == "attribute_item" {
        let attr_text = node.utf8_text(content.as_bytes()).unwrap_or("");
        if attr_text.contains("#[instrument]") || attr_text.contains("#[tracing::instrument]") {
            let line_start = node.start_position().row as i32 + 1;
            let in_test = is_in_rust_test(node, content);
            patterns.push(TelemetryPattern {
                line_start,
                level: Some(LogLevel::Trace),
                framework: "tracing".to_string(),
                in_test,
                confidence: if in_test { 0.7 } else { 1.0 },
                evidence: "attribute: #[instrument]".to_string(),
            });
        } else if attr_text.contains("#[otel::instrument]") {
            let line_start = node.start_position().row as i32 + 1;
            let in_test = is_in_rust_test(node, content);
            patterns.push(TelemetryPattern {
                line_start,
                level: Some(LogLevel::Trace),
                framework: "opentelemetry".to_string(),
                in_test,
                confidence: if in_test { 0.7 } else { 1.0 },
                evidence: "attribute: #[otel::instrument]".to_string(),
            });
        }
    }

    // Check macro invocations for telemetry-related macros
    if kind == "macro_invocation" {
        let mut cursor = node.walk();
        let children: Vec<tree_sitter::Node> = node.children(&mut cursor).collect();

        if let Some(name_node) = children.first() {
            let full_name = name_node
                .utf8_text(content.as_bytes())
                .unwrap_or("")
                .to_string();

            let simple_name = full_name
                .rsplit("::")
                .next()
                .unwrap_or(&full_name)
                .to_string();

            // Check for opentelemetry:: references
            if full_name.starts_with("opentelemetry::") {
                let line_start = node.start_position().row as i32 + 1;
                let in_test = is_in_rust_test(node, content);
                patterns.push(TelemetryPattern {
                    line_start,
                    level: Some(LogLevel::Trace),
                    framework: "opentelemetry".to_string(),
                    in_test,
                    confidence: if in_test { 0.7 } else { 1.0 },
                    evidence: "call: opentelemetry::*".to_string(),
                });
            }

            // Check for prometheus:: macros
            if full_name.starts_with("prometheus::") {
                for &macro_name in PROMETHEUS_MACROS {
                    if simple_name == macro_name {
                        let line_start = node.start_position().row as i32 + 1;
                        let in_test = is_in_rust_test(node, content);
                        patterns.push(TelemetryPattern {
                            line_start,
                            level: Some(LogLevel::Trace),
                            framework: "prometheus".to_string(),
                            in_test,
                            confidence: if in_test { 0.7 } else { 1.0 },
                            evidence: format!("macro: prometheus::{}", macro_name),
                        });
                        break;
                    }
                }
            }

            // Check for metrics:: macros
            if full_name.starts_with("metrics::") {
                for &macro_name in METRICS_MACROS {
                    if simple_name == macro_name {
                        let line_start = node.start_position().row as i32 + 1;
                        let in_test = is_in_rust_test(node, content);
                        patterns.push(TelemetryPattern {
                            line_start,
                            level: Some(LogLevel::Trace),
                            framework: "metrics".to_string(),
                            in_test,
                            confidence: if in_test { 0.7 } else { 1.0 },
                            evidence: format!("macro: metrics::{}", macro_name),
                        });
                        break;
                    }
                }
            }

            // Check for unqualified prometheus macros
            if !full_name.contains("::") {
                for &macro_name in PROMETHEUS_MACROS {
                    if simple_name == macro_name {
                        let line_start = node.start_position().row as i32 + 1;
                        let in_test = is_in_rust_test(node, content);
                        patterns.push(TelemetryPattern {
                            line_start,
                            level: Some(LogLevel::Trace),
                            framework: "prometheus".to_string(),
                            in_test,
                            confidence: if in_test { 0.7 } else { 0.8 },
                            evidence: format!("macro: {}", macro_name),
                        });
                        break;
                    }
                }
                for &macro_name in METRICS_MACROS {
                    if simple_name == macro_name {
                        let line_start = node.start_position().row as i32 + 1;
                        let in_test = is_in_rust_test(node, content);
                        // Only add metrics macro if not already detected as logging
                        if !RUST_LOG_MACROS.iter().any(|(lm, _, _)| *lm == simple_name) {
                            patterns.push(TelemetryPattern {
                                line_start,
                                level: Some(LogLevel::Trace),
                                framework: "metrics".to_string(),
                                in_test,
                                confidence: if in_test { 0.7 } else { 0.8 },
                                evidence: format!("macro: {}", macro_name),
                            });
                        }
                        break;
                    }
                }
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_rust_telemetry_patterns(child, content, patterns);
    }
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

    #[test]
    fn test_extract_routes_actix() {
        let content = r#"
            use actix_web::get;

            #[get("/users")]
            async fn get_users() -> impl Responder {
                "users"
            }
        "#;

        let routes = extract_routes(content, &[]).unwrap();
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/users" && r.framework == "actix")
            .expect("should find actix GET /users route");
        assert_eq!(route.method, "GET");
        assert_eq!(route.handler_name, "get_users");
        assert_eq!(route.route_source, "DECORATOR");
        assert!(!route.is_dynamic);
        assert_eq!(route.route_confidence, 1.0);
        assert!(route.evidence.contains("#[get(\"/users\")]"));
    }

    #[test]
    fn test_extract_routes_axum() {
        let content = r#"
            use axum::{Router, routing::get};

            fn app() -> Router {
                Router::new()
                    .route("/api/users", get(list_users))
            }
        "#;

        let routes = extract_routes(content, &[]).unwrap();
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/api/users" && r.framework == "axum")
            .expect("should find axum GET /api/users route");
        assert_eq!(route.method, "GET");
        assert_eq!(route.handler_name, "list_users");
        assert_eq!(route.route_source, "ROUTER_CHAIN");
        assert!(!route.is_dynamic);
        assert_eq!(route.route_confidence, 1.0);
        assert!(route.evidence.contains(".route(\"/api/users\""));
    }

    #[test]
    fn test_extract_routes_rocket() {
        let content = r#"
            #[rocket::post("/items")]
            fn create_item() -> &'static str {
                "created"
            }
        "#;

        let routes = extract_routes(content, &[]).unwrap();
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/items" && r.framework == "rocket")
            .expect("should find rocket POST /items route");
        assert_eq!(route.method, "POST");
        assert_eq!(route.handler_name, "create_item");
        assert_eq!(route.route_source, "DECORATOR");
        assert!(!route.is_dynamic);
        assert_eq!(route.route_confidence, 1.0);
        assert!(route.evidence.contains("#[rocket::post(\"/items\")]"));
    }

    #[test]
    fn test_extract_data_models_serde() {
        let content = r#"
            use serde::{Serialize, Deserialize};

            #[derive(Serialize, Deserialize)]
            pub struct User {
                pub id: i64,
                pub name: String,
            }
        "#;

        let models = extract_data_models(content, "src/models/user.rs", &[]).unwrap();
        let user_model = models
            .iter()
            .find(|m| m.model_name == "User")
            .expect("should find User data model");
        assert_eq!(user_model.model_kind, ModelKind::Struct);
        assert!((user_model.confidence - 1.0).abs() < f64::EPSILON);
        assert!(user_model.evidence.contains("Serialize"));
        assert!(user_model.evidence.contains("Deserialize"));
    }

    #[test]
    fn test_extract_data_models_sqlx() {
        let content = r#"
            #[derive(sqlx::FromRow)]
            pub struct UserRow {
                pub id: i64,
                pub email: String,
            }
        "#;

        let models = extract_data_models(content, "src/db/user.rs", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "UserRow")
            .expect("should find UserRow data model");
        assert_eq!(model.model_kind, ModelKind::Struct);
        assert!((model.confidence - 1.0).abs() < f64::EPSILON);
        assert!(model.evidence.contains("sqlx::FromRow"));
    }

    #[test]
    fn test_extract_data_models_directory_convention() {
        let content = r#"
            pub struct Config {
                pub key: String,
                pub value: String,
            }
        "#;

        let models = extract_data_models(content, "src/models/config.rs", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "Config")
            .expect("should find Config data model via directory convention");
        assert_eq!(model.model_kind, ModelKind::Struct);
        assert!((model.confidence - 0.7).abs() < f64::EPSILON);
        assert!(model.evidence.contains("dir: models/"));
    }

    #[test]
    fn test_extract_data_models_plain_struct_not_model() {
        let content = r#"
            pub struct Helper {
                pub x: i32,
                pub y: i32,
            }
        "#;

        let models = extract_data_models(content, "src/utils/helper.rs", &[]).unwrap();
        assert!(
            models.iter().all(|m| m.model_name != "Helper"),
            "plain struct in non-model dir should NOT be a data model"
        );
    }

    #[test]
    fn test_extract_data_models_serde_rename_all() {
        let content = r#"
            #[serde(rename_all = "camelCase")]
            pub struct ApiResponse {
                pub status: String,
                pub data: serde_json::Value,
            }
        "#;

        let models = extract_data_models(content, "src/api/response.rs", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "ApiResponse")
            .expect("should find ApiResponse data model via serde attribute");
        assert_eq!(model.model_kind, ModelKind::Struct);
        assert!((model.confidence - 0.9).abs() < f64::EPSILON);
        assert!(model.evidence.contains("serde(rename_all)"));
    }

    #[test]
    fn test_extract_logging_patterns_info() {
        let content = r#"
            fn handle_request() {
                info!("processing request");
            }
        "#;

        let patterns = extract_logging_patterns(content).unwrap();
        let p = patterns
            .iter()
            .find(|p| p.level == Some(LogLevel::Info) && p.framework == "log")
            .expect("should find info! logging pattern");
        assert!(!p.in_test);
        assert!(p.evidence.contains("info!"));
    }

    #[test]
    fn test_extract_logging_patterns_tracing() {
        let content = r#"
            fn handle_request() {
                tracing::info!("processing request");
                tracing::warn!("something odd");
                tracing::error!("something broke");
            }
        "#;

        let patterns = extract_logging_patterns(content).unwrap();
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "tracing" && p.level == Some(LogLevel::Info))
        );
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "tracing" && p.level == Some(LogLevel::Warn))
        );
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "tracing" && p.level == Some(LogLevel::Error))
        );
    }

    #[test]
    fn test_extract_logging_patterns_log_crate() {
        let content = r#"
            fn handle_request() {
                log::debug!("debug info");
                log::trace!("trace info");
            }
        "#;

        let patterns = extract_logging_patterns(content).unwrap();
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "log" && p.level == Some(LogLevel::Debug))
        );
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "log" && p.level == Some(LogLevel::Trace))
        );
    }

    #[test]
    fn test_extract_logging_patterns_println() {
        let content = r#"
            fn main() {
                println!("hello world");
                eprintln!("error output");
            }
        "#;

        let patterns = extract_logging_patterns(content).unwrap();
        let println_pat = patterns
            .iter()
            .find(|p| p.level == Some(LogLevel::Info) && p.framework == "println")
            .expect("should find println! pattern");
        assert!(!println_pat.in_test);
        let eprintln_pat = patterns
            .iter()
            .find(|p| p.level == Some(LogLevel::Error) && p.framework == "println")
            .expect("should find eprintln! pattern");
        assert!(!eprintln_pat.in_test);
    }

    #[test]
    fn test_extract_logging_patterns_in_test() {
        let content = r#"
            #[test]
            fn test_something() {
                info!("test log");
            }

            fn normal_fn() {
                info!("normal log");
            }
        "#;

        let patterns = extract_logging_patterns(content).unwrap();
        let test_pattern = patterns
            .iter()
            .find(|p| p.in_test)
            .expect("should find a pattern in test");
        assert!((test_pattern.confidence - 0.7).abs() < f64::EPSILON);
        let normal_pattern = patterns
            .iter()
            .find(|p| !p.in_test)
            .expect("should find a pattern not in test");
        assert!((normal_pattern.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extract_error_handling_match_and_unwrap() {
        let content = r#"
fn handle_result() -> Result<i32, String> {
    let val = some_operation()?;
    match val {
        Ok(x) => x,
        Err(e) => 0,
    }
}

fn risky() -> i32 {
    some_operation().unwrap()
}
        "#;

        let patterns = extract_error_handling(content).unwrap();
        let match_pattern = patterns
            .iter()
            .find(|p| p.framework == "match_result")
            .expect("should find match_result pattern");
        assert_eq!(match_pattern.level, Some(LogLevel::Info));
        assert!(!match_pattern.in_test);
        assert_eq!(match_pattern.evidence, "syntactic: match expression");

        let unwrap_pattern = patterns
            .iter()
            .find(|p| p.framework == "unwrap")
            .expect("should find unwrap pattern");
        assert_eq!(unwrap_pattern.level, Some(LogLevel::Error));
        assert_eq!(unwrap_pattern.evidence, "syntactic: unwrap call");

        let try_pattern = patterns
            .iter()
            .find(|p| p.framework == "try_operator")
            .expect("should find try_operator pattern");
        assert_eq!(try_pattern.level, Some(LogLevel::Info));
        assert_eq!(try_pattern.evidence, "syntactic: try operator");
    }

    #[test]
    fn test_extract_error_handling_in_test() {
        let content = r#"
            #[test]
            fn test_error_handling() {
                let result = some_fn().unwrap();
                assert_eq!(result, 42);
            }

            fn normal_fn() -> Result<i32, Error> {
                some_op()?;
            }
        "#;

        let patterns = extract_error_handling(content).unwrap();
        let test_unwrap = patterns
            .iter()
            .find(|p| p.framework == "unwrap" && p.in_test)
            .expect("should find unwrap in test");
        assert!((test_unwrap.confidence - 0.7).abs() < f64::EPSILON);

        let normal_try = patterns
            .iter()
            .find(|p| p.framework == "try_operator" && !p.in_test)
            .expect("should find try_operator not in test");
        assert!((normal_try.confidence - 1.0).abs() < f64::EPSILON);
    }
}
