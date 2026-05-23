use crate::index::routes::ExtractedRoute;
use crate::index::symbols::Symbol;
use miette::{IntoDiagnostic, Result};
use tree_sitter::{Node, Parser};

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

const HTTP_METHODS: &[&str] = &["get", "post", "put", "delete", "patch", "head", "options"];

fn method_upper(m: &str) -> String {
    m.to_ascii_uppercase()
}

fn collect_rust_routes(node: Node, content: &str, routes: &mut Vec<ExtractedRoute>) {
    let kind = node.kind();

    // --- Attribute-based routes: Actix and Rocket ---
    if kind == "attribute_item" {
        let attr_text = node.utf8_text(content.as_bytes()).unwrap_or("");

        for &method in HTTP_METHODS {
            let actix_short = format!("#[{}(", method);
            let actix_fq = format!("#[actix_web::{}(", method);
            let rocket_attr = format!("#[rocket::{}(", method);

            if (attr_text.contains(&actix_short) || attr_text.contains(&actix_fq))
                && let Some(path) = extract_path_from_attr(attr_text)
            {
                let handler_name = find_next_function_name(node, content);
                let evidence = format!("#[{}(\"{}\")] on {}", method, path, handler_name);
                routes.push(ExtractedRoute {
                    method: method_upper(method),
                    path_pattern: path,
                    handler_name,
                    framework: "Actix".to_string(),
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
                let evidence = format!("#[rocket::{}(\"{}\")] on {}", method, path, handler_name);
                routes.push(ExtractedRoute {
                    method: method_upper(method),
                    path_pattern: path,
                    handler_name,
                    framework: "Rocket".to_string(),
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
        let func_node = node.child(0);
        if let Some(func) = func_node
            && (func.kind() == "method_call_expression" || func.kind() == "field_expression")
        {
            let method_name = extract_method_call_name_simple(func, content);
            if method_name == "route" {
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

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_rust_routes(child, content, routes);
    }
}

fn extract_path_from_attr(attr_text: &str) -> Option<String> {
    let start = attr_text.find("(\"")? + 2;
    let end = attr_text[start..].find("\")")? + start;
    Some(attr_text[start..end].to_string())
}

fn find_next_function_name(node: Node, content: &str) -> String {
    if let Some(parent) = node.parent() {
        let mut cursor = parent.walk();
        let mut found_self = false;
        for child in parent.children(&mut cursor) {
            if child == node {
                found_self = true;
                continue;
            }
            if found_self {
                if child.kind() == "function_item" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        return name_node
                            .utf8_text(content.as_bytes())
                            .unwrap_or("")
                            .to_string();
                    }
                }
                // Skip unrelated nodes like line comments but stop if we hit another significant item
                if child.kind() != "line_comment"
                    && child.kind() != "block_comment"
                    && child.kind() != "attribute_item"
                {
                    break;
                }
            }
        }
    }
    "<unknown>".to_string()
}

fn extract_method_call_name_simple(node: Node, content: &str) -> String {
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

fn extract_axum_route(node: &Node, content: &str, routes: &mut Vec<ExtractedRoute>) {
    let mut cursor = node.walk();
    let args: Vec<Node> = node.children(&mut cursor).collect();
    if args.len() < 3 {
        return;
    }

    let path_arg = args.iter().find(|n| n.kind() == "string_literal");
    let path = match path_arg {
        Some(p) => {
            let t = p.utf8_text(content.as_bytes()).unwrap_or("");
            t.trim_matches('"').to_string()
        }
        None => return,
    };

    for arg in args {
        if arg.kind() == "call_expression" {
            let text = arg.utf8_text(content.as_bytes()).unwrap_or("");
            for &method in HTTP_METHODS {
                if text.starts_with(method) || text.contains(&format!("::{}", method)) {
                    let handler = find_axum_handler(&arg, content);
                    routes.push(ExtractedRoute {
                        method: method_upper(method),
                        path_pattern: path.clone(),
                        handler_name: handler.clone(),
                        framework: "Axum".to_string(),
                        route_source: "METHOD_CHAIN".to_string(),
                        mount_prefix: None,
                        is_dynamic: path.contains(':'),
                        route_confidence: 0.9,
                        evidence: format!("{}(\"{}\") -> {}", method, path, handler),
                    });
                }
            }
        }
    }
}

fn find_axum_handler(node: &Node, content: &str) -> String {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "arguments" {
            let mut arg_cursor = child.walk();
            for arg in child.children(&mut arg_cursor) {
                if arg.kind() == "identifier" {
                    return arg.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                }
            }
        }
    }
    "unknown".to_string()
}
