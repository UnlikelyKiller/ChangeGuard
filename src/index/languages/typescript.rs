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

/// HTTP method names used as Express/Fastify method calls.
const TS_HTTP_METHODS: &[&str] = &["get", "post", "put", "delete", "patch"];

/// Common variable names for Express app/router objects.
const EXPRESS_APP_NAMES: &[&str] = &["app", "router", "Router"];
/// Common variable names for Fastify instances.
const FASTIFY_APP_NAMES: &[&str] = &["fastify", "app"];

pub fn extract_routes(content: &str, _symbols: &[Symbol]) -> Result<Vec<ExtractedRoute>> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse TypeScript content"))?;

    let is_fastify = detect_fastify(content);
    let mut routes = Vec::new();
    collect_ts_routes(tree.root_node(), content, is_fastify, &mut routes);
    Ok(routes)
}

/// Detect whether the file is likely Fastify (vs Express) by scanning for
/// import statements or variable assignments referencing fastify.
fn detect_fastify(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    // Check for fastify imports or factory calls
    lower.contains("fastify")
        || lower.contains("fastify(")
        || lower.contains("require('fastify')")
        || lower.contains("from 'fastify'")
}

fn collect_ts_routes(
    node: tree_sitter::Node,
    content: &str,
    is_fastify: bool,
    routes: &mut Vec<ExtractedRoute>,
) {
    let kind = node.kind();

    // Look for call_expression where the callee is a member_expression like
    // app.get("/path", handler) or router.post("/path", handler)
    if kind == "call_expression" {
        let callee_node = node.child(0);
        if let Some(callee) = callee_node
            && callee.kind() == "member_expression"
        {
            let method_name = extract_ts_member_name(callee, content);
            let obj_name = extract_ts_object_name(callee, content);

            if TS_HTTP_METHODS.contains(&method_name.as_str()) {
                // Check if the object name is a known app/router variable
                let is_express_obj = EXPRESS_APP_NAMES.contains(&obj_name.as_str());
                let is_fastify_obj = FASTIFY_APP_NAMES.contains(&obj_name.as_str());

                if is_express_obj || is_fastify_obj {
                    // Find the arguments child node
                    let mut cursor = node.walk();
                    let args_node = node.children(&mut cursor).find(|c| c.kind() == "arguments");

                    if let Some(args_node) = args_node {
                        // Collect actual argument expressions (skip punctuation)
                        let mut cursor2 = args_node.walk();
                        let args: Vec<tree_sitter::Node> = args_node
                            .children(&mut cursor2)
                            .filter(|c| c.kind() != "," && c.kind() != "(" && c.kind() != ")")
                            .collect();

                        if args.len() >= 2 {
                            // First argument should be the path (string)
                            let path_node = args[0];
                            let path_text = path_node.utf8_text(content.as_bytes()).unwrap_or("");
                            let path = extract_ts_string_literal(path_text);

                            // Second argument is the handler
                            let handler_node = args[1];
                            let handler_text =
                                handler_node.utf8_text(content.as_bytes()).unwrap_or("");

                            let (handler_name, is_dynamic, route_confidence) =
                                if handler_node.kind() == "identifier" {
                                    (handler_text.to_string(), false, 1.0)
                                } else {
                                    // Inline function / arrow function / callback
                                    ("anonymous".to_string(), true, 0.5)
                                };

                            let framework = if is_fastify && is_fastify_obj {
                                "fastify"
                            } else {
                                "express"
                            };

                            let evidence =
                                format!("{obj_name}.{method_name}(\"{path}\", {handler_text})");
                            routes.push(ExtractedRoute {
                                method: method_name.to_ascii_uppercase(),
                                path_pattern: path,
                                handler_name,
                                framework: framework.to_string(),
                                route_source: "APP_METHOD".to_string(),
                                mount_prefix: None,
                                is_dynamic,
                                route_confidence,
                                evidence,
                            });
                        }
                    }
                }
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_ts_routes(child, content, is_fastify, routes);
    }
}

/// Extract the object name from a member_expression (e.g. app.get -> "app").
fn extract_ts_object_name(node: tree_sitter::Node, content: &str) -> String {
    let mut cursor = node.walk();
    // The first child is typically the object
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" || child.kind() == "member_expression" {
            // For simple identifiers, return the name.
            // For nested member expressions (e.g. this.app), take the last identifier.
            if child.kind() == "identifier" {
                return child
                    .utf8_text(content.as_bytes())
                    .unwrap_or("")
                    .to_string();
            }
        }
    }
    String::new()
}

/// Strip surrounding quotes from a string literal, returning the inner content.
fn extract_ts_string_literal(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('\'') && s.ends_with('\''))
        || (s.starts_with('`') && s.ends_with('`'))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
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

/// Directories that conventionally indicate data model definitions in TypeScript projects.
const TS_MODEL_DIRS: &[&str] = &["models/", "types/", "schemas/", "interfaces/"];

pub fn extract_data_models(
    content: &str,
    file_path: &str,
    _symbols: &[Symbol],
) -> Result<Vec<ExtractedModel>> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse TypeScript content"))?;

    let mut models = Vec::new();
    collect_ts_data_models(tree.root_node(), content, file_path, &mut models);
    Ok(models)
}

fn collect_ts_data_models(
    node: tree_sitter::Node,
    content: &str,
    file_path: &str,
    models: &mut Vec<ExtractedModel>,
) {
    let kind = node.kind();

    // --- TypeORM: class with @Entity decorator ---
    if kind == "decorator" {
        let decorator_text = node.utf8_text(content.as_bytes()).unwrap_or("");

        if decorator_text.contains("@Entity") {
            // The decorated class is a sibling of the decorator under the parent
            if let Some(parent) = node.parent() {
                let mut cursor = parent.walk();
                for child in parent.children(&mut cursor) {
                    if child.kind() == "class_declaration" {
                        let class_name = child
                            .child_by_field_name("name")
                            .and_then(|n| n.utf8_text(content.as_bytes()).ok())
                            .unwrap_or("")
                            .to_string();
                        if !class_name.is_empty() {
                            models.push(ExtractedModel {
                                model_name: class_name,
                                language: "TypeScript".to_string(),
                                model_kind: ModelKind::Class,
                                confidence: 1.0,
                                evidence: "decorator: @Entity".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // --- Classes that extend Model (Sequelize, Objection) ---
    if kind == "class_declaration" {
        let class_name = node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(content.as_bytes()).ok())
            .unwrap_or("")
            .to_string();

        if !class_name.is_empty() {
            // Check for extends clause
            let class_text = node.utf8_text(content.as_bytes()).unwrap_or("");
            if class_text.contains("extends Model") {
                models.push(ExtractedModel {
                    model_name: class_name,
                    language: "TypeScript".to_string(),
                    model_kind: ModelKind::Class,
                    confidence: 0.9,
                    evidence: "extends: Model".to_string(),
                });
            }
        }
    }

    // --- Directory convention: interfaces and type aliases in model directories ---
    if kind == "interface_declaration" || kind == "type_alias_declaration" {
        let name = node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(content.as_bytes()).ok())
            .unwrap_or("")
            .to_string();

        if !name.is_empty() {
            let dir_match = TS_MODEL_DIRS.iter().find(|dir| file_path.contains(*dir));
            if let Some(dir) = dir_match {
                // Only add if not already detected by a higher-confidence rule
                if !models.iter().any(|m| m.model_name == name) {
                    let model_kind = if kind == "interface_declaration" {
                        ModelKind::Interface
                    } else {
                        ModelKind::Schema
                    };
                    models.push(ExtractedModel {
                        model_name: name,
                        language: "TypeScript".to_string(),
                        model_kind,
                        confidence: 0.7,
                        evidence: format!("dir: {dir}"),
                    });
                }
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_ts_data_models(child, content, file_path, models);
    }
}

/// Console method names and their log level mappings.
const CONSOLE_METHODS: &[(&str, LogLevel)] = &[
    ("log", LogLevel::Info),
    ("warn", LogLevel::Warn),
    ("error", LogLevel::Error),
    ("info", LogLevel::Info),
    ("debug", LogLevel::Debug),
];

/// Logger/winston method names and their log level mappings.
const LOGGER_METHODS: &[(&str, LogLevel)] = &[
    ("info", LogLevel::Info),
    ("warn", LogLevel::Warn),
    ("error", LogLevel::Error),
    ("debug", LogLevel::Debug),
];

/// Winston log method mapping (includes "log" -> Info).
const WINSTON_METHODS: &[(&str, LogLevel)] = &[
    ("info", LogLevel::Info),
    ("warn", LogLevel::Warn),
    ("error", LogLevel::Error),
    ("debug", LogLevel::Debug),
    ("log", LogLevel::Info),
];

pub fn extract_logging_patterns(content: &str) -> Result<Vec<LoggingPattern>> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse TypeScript content"))?;

    let mut patterns = Vec::new();
    collect_ts_logging_patterns(tree.root_node(), content, &mut patterns);

    // Cap at 1000 patterns per file
    patterns.truncate(1000);
    Ok(patterns)
}

fn collect_ts_logging_patterns(
    node: tree_sitter::Node,
    content: &str,
    patterns: &mut Vec<LoggingPattern>,
) {
    if node.kind() == "call_expression" {
        let callee_node = node.child(0);
        if let Some(callee) = callee_node
            && callee.kind() == "member_expression"
        {
            let obj_name = extract_ts_object_name(callee, content);
            let method_name = extract_ts_member_name(callee, content);

            // Check console.* methods
            if obj_name == "console" {
                for &(method, level) in CONSOLE_METHODS {
                    if method_name == method {
                        let line_start = node.start_position().row as i32 + 1;
                        let in_test = is_in_ts_test(node, content);
                        let evidence = node.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                        let evidence = if evidence.len() > 200 {
                            format!("{}...", &evidence[..197])
                        } else {
                            evidence
                        };

                        patterns.push(LoggingPattern {
                            line_start,
                            level: Some(level),
                            framework: "console".to_string(),
                            in_test,
                            confidence: if in_test { 0.7 } else { 1.0 },
                            evidence,
                        });
                        break;
                    }
                }
            }
            // Check logger.* methods
            else if obj_name == "logger" || obj_name == "Logger" {
                for &(method, level) in LOGGER_METHODS {
                    if method_name == method {
                        let line_start = node.start_position().row as i32 + 1;
                        let in_test = is_in_ts_test(node, content);
                        let evidence = node.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                        let evidence = if evidence.len() > 200 {
                            format!("{}...", &evidence[..197])
                        } else {
                            evidence
                        };

                        patterns.push(LoggingPattern {
                            line_start,
                            level: Some(level),
                            framework: "logger".to_string(),
                            in_test,
                            confidence: if in_test { 0.7 } else { 1.0 },
                            evidence,
                        });
                        break;
                    }
                }
            }
            // Check winston.* methods
            else if obj_name == "winston" {
                for &(method, level) in WINSTON_METHODS {
                    if method_name == method {
                        let line_start = node.start_position().row as i32 + 1;
                        let in_test = is_in_ts_test(node, content);
                        let evidence = node.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                        let evidence = if evidence.len() > 200 {
                            format!("{}...", &evidence[..197])
                        } else {
                            evidence
                        };

                        patterns.push(LoggingPattern {
                            line_start,
                            level: Some(level),
                            framework: "winston".to_string(),
                            in_test,
                            confidence: if in_test { 0.7 } else { 1.0 },
                            evidence,
                        });
                        break;
                    }
                }
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_ts_logging_patterns(child, content, patterns);
    }
}

/// Walk up the tree to check if the node is inside a test block
/// (describe, it, or test call).
fn is_in_ts_test(node: tree_sitter::Node, content: &str) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "call_expression" {
            // Check if the callee is describe, it, or test
            if let Some(callee) = parent.child(0) {
                let callee_text = callee.utf8_text(content.as_bytes()).unwrap_or("");
                if callee_text.starts_with("describe")
                    || callee_text.starts_with("it(")
                    || callee_text.starts_with("test(")
                    || callee_text == "describe"
                    || callee_text == "it"
                    || callee_text == "test"
                {
                    return true;
                }
                // Also handle member expressions like describe.skip, it.only
                if callee.kind() == "member_expression" {
                    let obj = extract_ts_object_name(callee, content);
                    if obj == "describe" || obj == "it" || obj == "test" {
                        return true;
                    }
                }
            }
        }
        current = parent.parent();
    }
    false
}

pub fn extract_error_handling(content: &str) -> Result<Vec<ErrorHandlingPattern>> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse TypeScript content"))?;

    let mut patterns = Vec::new();
    collect_ts_error_handling(tree.root_node(), content, &mut patterns);

    // Cap at 1000 patterns per file
    patterns.truncate(1000);
    Ok(patterns)
}

fn collect_ts_error_handling(
    node: tree_sitter::Node,
    content: &str,
    patterns: &mut Vec<ErrorHandlingPattern>,
) {
    let kind = node.kind();

    match kind {
        "try_statement" => {
            // try/catch/finally blocks
            let line_start = node.start_position().row as i32 + 1;
            let in_test = is_in_ts_test(node, content);
            patterns.push(ErrorHandlingPattern {
                line_start,
                level: Some(LogLevel::Info),
                framework: "try_catch".to_string(),
                in_test,
                confidence: if in_test { 0.7 } else { 1.0 },
                evidence: "syntactic: try/catch block".to_string(),
            });
        }
        "throw_statement" => {
            let line_start = node.start_position().row as i32 + 1;
            let in_test = is_in_ts_test(node, content);
            patterns.push(ErrorHandlingPattern {
                line_start,
                level: Some(LogLevel::Warn),
                framework: "throw".to_string(),
                in_test,
                confidence: if in_test { 0.7 } else { 1.0 },
                evidence: "syntactic: throw statement".to_string(),
            });
        }
        "call_expression" => {
            // Check for .catch() calls and Promise.reject
            let callee_node = node.child(0);
            if let Some(callee) = callee_node
                && callee.kind() == "member_expression"
            {
                let method_name = extract_ts_member_name(callee, content);
                if method_name == "catch" {
                    let line_start = node.start_position().row as i32 + 1;
                    let in_test = is_in_ts_test(node, content);
                    patterns.push(ErrorHandlingPattern {
                        line_start,
                        level: Some(LogLevel::Info),
                        framework: "promise_catch".to_string(),
                        in_test,
                        confidence: if in_test { 0.7 } else { 1.0 },
                        evidence: "syntactic: .catch() call".to_string(),
                    });
                }
                // Check for Promise.reject
                let obj_name = extract_ts_object_name(callee, content);
                if obj_name == "Promise" && method_name == "reject" {
                    let line_start = node.start_position().row as i32 + 1;
                    let in_test = is_in_ts_test(node, content);
                    patterns.push(ErrorHandlingPattern {
                        line_start,
                        level: Some(LogLevel::Warn),
                        framework: "promise_reject".to_string(),
                        in_test,
                        confidence: if in_test { 0.7 } else { 1.0 },
                        evidence: "syntactic: Promise.reject".to_string(),
                    });
                }
            }
        }
        _ => {}
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_ts_error_handling(child, content, patterns);
    }
}

pub fn extract_telemetry_patterns(content: &str) -> Result<Vec<TelemetryPattern>> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse TypeScript content"))?;

    let mut patterns = Vec::new();
    collect_ts_telemetry_patterns(tree.root_node(), content, &mut patterns);

    // Also do line-based heuristic matching for telemetry.* patterns
    for (line_idx, line) in content.lines().enumerate() {
        let line_lower = line.to_ascii_lowercase();
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }
        if line_lower.contains("telemetry") || line_lower.contains("monitoring") {
            let line_start = (line_idx + 1) as i32;
            let already_matched = patterns.iter().any(|p| p.line_start == line_start);
            if !already_matched {
                patterns.push(TelemetryPattern {
                    line_start,
                    level: Some(LogLevel::Trace),
                    framework: "custom".to_string(),
                    in_test: is_in_ts_test_from_line(line),
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

/// Simple line-level heuristic to detect if a line is inside a test block.
fn is_in_ts_test_from_line(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    lower.contains("describe(")
        || lower.contains("it(")
        || lower.contains("test(")
        || lower.contains("describe.skip(")
        || lower.contains("it.skip(")
}

fn collect_ts_telemetry_patterns(
    node: tree_sitter::Node,
    content: &str,
    patterns: &mut Vec<TelemetryPattern>,
) {
    let kind = node.kind();

    // Check for @Trace() decorator
    if kind == "decorator" {
        let decorator_text = node.utf8_text(content.as_bytes()).unwrap_or("");
        if decorator_text.contains("@Trace") || decorator_text.contains("@trace") {
            let line_start = node.start_position().row as i32 + 1;
            let in_test = is_in_ts_test(node, content);
            patterns.push(TelemetryPattern {
                line_start,
                level: Some(LogLevel::Trace),
                framework: "opentelemetry".to_string(),
                in_test,
                confidence: if in_test { 0.7 } else { 1.0 },
                evidence: "decorator: @Trace()".to_string(),
            });
        }
    }

    // Check call expressions for opentelemetry imports and prom-client usage
    if kind == "call_expression" {
        let callee_node = node.child(0);
        if let Some(callee) = callee_node {
            let call_text = callee.utf8_text(content.as_bytes()).unwrap_or("");

            // Check for new Counter/Histogram/Gauge from prom-client
            if call_text.contains("Counter")
                || call_text.contains("Histogram")
                || call_text.contains("Gauge")
                || call_text.contains("Summary")
            {
                // Only match if it looks like prom-client usage (new expression or member access)
                let full_text = node.utf8_text(content.as_bytes()).unwrap_or("");
                if full_text.contains("prom") || full_text.contains("Prom") {
                    let line_start = node.start_position().row as i32 + 1;
                    let in_test = is_in_ts_test(node, content);
                    patterns.push(TelemetryPattern {
                        line_start,
                        level: Some(LogLevel::Trace),
                        framework: "prom-client".to_string(),
                        in_test,
                        confidence: if in_test { 0.7 } else { 1.0 },
                        evidence: format!("call: {}", truncate_str(full_text, 200)),
                    });
                }
            }
        }
    }

    // Check import statements for opentelemetry
    if kind == "import_statement" {
        let import_text = node.utf8_text(content.as_bytes()).unwrap_or("");
        if import_text.contains("opentelemetry") || import_text.contains("@opentelemetry") {
            let line_start = node.start_position().row as i32 + 1;
            let in_test = is_in_ts_test(node, content);
            patterns.push(TelemetryPattern {
                line_start,
                level: Some(LogLevel::Trace),
                framework: "opentelemetry".to_string(),
                in_test,
                confidence: if in_test { 0.7 } else { 1.0 },
                evidence: "import: opentelemetry".to_string(),
            });
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_ts_telemetry_patterns(child, content, patterns);
    }
}

/// Truncate a string to a maximum length, adding "..." if truncated.
fn truncate_str(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len.saturating_sub(3)]
    }
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

    #[test]
    fn test_extract_routes_express_get() {
        let content = r#"
            import express from 'express';
            const app = express();

            app.get("/api/users", getUsers);
        "#;

        let routes = extract_routes(content, &[]).unwrap();
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/api/users" && r.framework == "express")
            .expect("should find express GET /api/users route");
        assert_eq!(route.method, "GET");
        assert_eq!(route.handler_name, "getUsers");
        assert_eq!(route.route_source, "APP_METHOD");
        assert!(!route.is_dynamic);
        assert_eq!(route.route_confidence, 1.0);
    }

    #[test]
    fn test_extract_routes_express_router_post() {
        let content = r#"
            import { Router } from 'express';
            const router = Router();

            router.post("/items", createItem);
        "#;

        let routes = extract_routes(content, &[]).unwrap();
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/items" && r.framework == "express")
            .expect("should find express POST /items route");
        assert_eq!(route.method, "POST");
        assert_eq!(route.handler_name, "createItem");
        assert_eq!(route.route_source, "APP_METHOD");
    }

    #[test]
    fn test_extract_routes_fastify_get() {
        let content = r#"
            import Fastify from 'fastify';
            const fastify = Fastify();

            fastify.get("/health", healthCheck);
        "#;

        let routes = extract_routes(content, &[]).unwrap();
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/health" && r.framework == "fastify")
            .expect("should find fastify GET /health route");
        assert_eq!(route.method, "GET");
        assert_eq!(route.handler_name, "healthCheck");
        assert_eq!(route.route_source, "APP_METHOD");
    }

    #[test]
    fn test_extract_data_models_interface_in_models_dir() {
        let content = r#"
            export interface User {
                id: number;
                name: string;
                email: string;
            }
        "#;

        let models = extract_data_models(content, "src/models/user.ts", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "User")
            .expect("should find User data model via directory convention");
        assert_eq!(model.model_kind, ModelKind::Interface);
        assert!((model.confidence - 0.7).abs() < f64::EPSILON);
        assert!(model.evidence.contains("dir: models/"));
    }

    #[test]
    fn test_extract_data_models_entity_decorator() {
        let content = r#"
            @Entity("users")
            export class User {
                @PrimaryGeneratedColumn()
                id: number;

                @Column()
                name: string;
            }
        "#;

        let models = extract_data_models(content, "src/entities/user.entity.ts", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "User")
            .expect("should find User data model via @Entity decorator");
        assert_eq!(model.model_kind, ModelKind::Class);
        assert!((model.confidence - 1.0).abs() < f64::EPSILON);
        assert!(model.evidence.contains("decorator: @Entity"));
    }

    #[test]
    fn test_extract_data_models_extends_model() {
        let content = r#"
            export class User extends Model<User> {
                declare id: number;
                declare name: string;
            }
        "#;

        let models = extract_data_models(content, "src/db/user.model.ts", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "User")
            .expect("should find User data model via extends Model");
        assert_eq!(model.model_kind, ModelKind::Class);
        assert!((model.confidence - 0.9).abs() < f64::EPSILON);
        assert!(model.evidence.contains("extends: Model"));
    }

    #[test]
    fn test_extract_data_models_interface_not_in_model_dir() {
        let content = r#"
            export interface ConfigOptions {
                debug: boolean;
                port: number;
            }
        "#;

        let models = extract_data_models(content, "src/config/options.ts", &[]).unwrap();
        assert!(
            models.iter().all(|m| m.model_name != "ConfigOptions"),
            "interface in non-model dir should NOT be a data model"
        );
    }

    #[test]
    fn test_extract_logging_patterns_console() {
        let content = r#"
            function main() {
                console.log("hello");
                console.warn("warning");
                console.error("error");
                console.info("info");
                console.debug("debug");
            }
        "#;

        let patterns = extract_logging_patterns(content).unwrap();
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "console" && p.level == Some(LogLevel::Info))
        );
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "console" && p.level == Some(LogLevel::Warn))
        );
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "console" && p.level == Some(LogLevel::Error))
        );
    }

    #[test]
    fn test_extract_logging_patterns_logger() {
        let content = r#"
            function handleRequest() {
                logger.info("request received");
                logger.error("request failed");
            }
        "#;

        let patterns = extract_logging_patterns(content).unwrap();
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "logger" && p.level == Some(LogLevel::Info))
        );
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "logger" && p.level == Some(LogLevel::Error))
        );
    }

    #[test]
    fn test_extract_logging_patterns_winston() {
        let content = r#"
            function processItem() {
                winston.info("processing");
                winston.log("general log");
            }
        "#;

        let patterns = extract_logging_patterns(content).unwrap();
        assert!(
            patterns
                .iter()
                .any(|p| p.framework == "winston" && p.level == Some(LogLevel::Info))
        );
        assert!(patterns.iter().any(|p| p.framework == "winston"
            && p.level == Some(LogLevel::Info)
            && p.evidence.contains("winston.log")));
    }

    #[test]
    fn test_extract_logging_patterns_in_test() {
        let content = r#"
            describe("my suite", () => {
                it("should work", () => {
                    console.log("test output");
                });
            });
        "#;

        let patterns = extract_logging_patterns(content).unwrap();
        let test_pattern = patterns
            .iter()
            .find(|p| p.framework == "console")
            .expect("should find console pattern");
        assert!(test_pattern.in_test);
    }

    #[test]
    fn test_extract_error_handling_try_catch_and_throw() {
        let content = r#"
            function handleData() {
                try {
                    const result = fetchData();
                    return result;
                } catch (e) {
                    throw new Error("failed");
                }
            }
        "#;

        let patterns = extract_error_handling(content).unwrap();
        let try_catch = patterns
            .iter()
            .find(|p| p.framework == "try_catch")
            .expect("should find try_catch pattern");
        assert_eq!(try_catch.level, Some(LogLevel::Info));
        assert!(!try_catch.in_test);
        assert_eq!(try_catch.evidence, "syntactic: try/catch block");

        let throw_pattern = patterns
            .iter()
            .find(|p| p.framework == "throw")
            .expect("should find throw pattern");
        assert_eq!(throw_pattern.level, Some(LogLevel::Warn));
        assert_eq!(throw_pattern.evidence, "syntactic: throw statement");
    }

    #[test]
    fn test_extract_error_handling_in_test() {
        let content = r#"
            describe("error handling", () => {
                it("should catch errors", () => {
                    try {
                        doSomething();
                    } catch (e) {
                        expect(e).toBeDefined();
                    }
                });
            });

            function normalFn() {
                throw new Error("bad");
            }
        "#;

        let patterns = extract_error_handling(content).unwrap();
        let test_try = patterns
            .iter()
            .find(|p| p.framework == "try_catch" && p.in_test)
            .expect("should find try_catch in test");
        assert!((test_try.confidence - 0.7).abs() < f64::EPSILON);

        let normal_throw = patterns
            .iter()
            .find(|p| p.framework == "throw" && !p.in_test)
            .expect("should find throw not in test");
        assert!((normal_throw.confidence - 1.0).abs() < f64::EPSILON);
    }
}
