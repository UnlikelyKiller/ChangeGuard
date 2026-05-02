use crate::index::call_graph::{CallEdge, CallKind, ResolutionStatus};
use crate::index::data_models::{ExtractedModel, ModelKind};
use crate::index::routes::ExtractedRoute;
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

/// HTTP method names used as FastAPI decorator methods.
const PY_HTTP_METHODS: &[&str] = &["get", "post", "put", "delete", "patch", "head", "options"];

pub fn extract_routes(content: &str, _symbols: &[Symbol]) -> Result<Vec<ExtractedRoute>> {
    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Python content"))?;

    // Detect FastAPI router objects and Flask app/blueprint objects
    let fastapi_routers = detect_fastapi_routers(tree.root_node(), content);
    let flask_objects = detect_flask_objects(content);

    let mut routes = Vec::new();
    collect_py_routes(
        tree.root_node(),
        content,
        &fastapi_routers,
        &flask_objects,
        &mut routes,
    );
    Ok(routes)
}

/// Detect variable names assigned from APIRouter() calls (FastAPI).
fn detect_fastapi_routers(root: tree_sitter::Node, content: &str) -> Vec<String> {
    let mut routers = Vec::new();
    // Common defaults: app, router, api_router
    routers.push("app".to_string());
    routers.push("router".to_string());
    routers.push("api_router".to_string());

    // Walk AST looking for assignments like: router = APIRouter()
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "assignment" {
            let mut cursor = node.walk();
            let children: Vec<tree_sitter::Node> = node.children(&mut cursor).collect();
            // assignment: left = right
            // children[0] is the left-hand side (identifier), children[1] is "=",
            // children[2] is the right-hand side (call)
            let lhs = children.first();
            let rhs = children.get(2);
            if let (Some(lhs_node), Some(rhs_node)) = (lhs, rhs)
                && lhs_node.kind() == "identifier"
                && rhs_node.kind() == "call"
            {
                let rhs_text = rhs_node.utf8_text(content.as_bytes()).unwrap_or("");
                if rhs_text.starts_with("APIRouter") {
                    let name = lhs_node
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    if !routers.contains(&name) {
                        routers.push(name);
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }

    routers
}

/// Detect variable names that are Flask app/blueprint objects by scanning content.
fn detect_flask_objects(content: &str) -> Vec<String> {
    let mut objects = Vec::new();
    // Common defaults: app, bp, blueprint
    objects.push("app".to_string());
    objects.push("bp".to_string());
    objects.push("blueprint".to_string());

    // Look for assignments like: app = Flask(__name__) or bp = Blueprint("name", __name__)
    for line in content.lines() {
        let line = line.trim();
        if (line.contains("Flask(") || line.contains("Blueprint("))
            && line.contains('=')
            && let Some(lhs) = line.split('=').next()
        {
            let name = lhs.trim().to_string();
            if !name.is_empty() && !objects.contains(&name) {
                objects.push(name);
            }
        }
    }

    objects
}

fn collect_py_routes(
    node: tree_sitter::Node,
    content: &str,
    fastapi_routers: &[String],
    flask_objects: &[String],
    routes: &mut Vec<ExtractedRoute>,
) {
    let kind = node.kind();

    // In Python tree-sitter, decorators are children of decorated_definition.
    // Each decorator node corresponds to a @... line.
    if kind == "decorator" {
        let decorator_text = node.utf8_text(content.as_bytes()).unwrap_or("");

        // --- FastAPI: @router.get("/path") or @app.post("/path") ---
        // Pattern: @{varname}.{method}("path")
        for router_name in fastapi_routers {
            for &method_str in PY_HTTP_METHODS {
                let pattern = format!("@{router_name}.{method_str}(");
                if decorator_text.contains(&pattern)
                    && let Some(path) = extract_py_decorator_path(decorator_text)
                {
                    // Find the decorated function name from the parent decorated_definition
                    let handler_name = find_py_decorated_function_name(node, content);
                    let evidence =
                        format!("@{router_name}.{method_str}(\"{path}\") on {handler_name}");
                    routes.push(ExtractedRoute {
                        method: method_str.to_ascii_uppercase(),
                        path_pattern: path,
                        handler_name,
                        framework: "fastapi".to_string(),
                        route_source: "DECORATOR".to_string(),
                        mount_prefix: None,
                        is_dynamic: false,
                        route_confidence: 1.0,
                        evidence,
                    });
                }
            }
        }

        // --- Flask: @app.route("/path") or @bp.route("/path", methods=["POST"]) ---
        for obj_name in flask_objects {
            let route_pattern = format!("@{obj_name}.route(");
            if decorator_text.contains(&route_pattern)
                && let Some(path) = extract_py_decorator_path(decorator_text)
            {
                let handler_name = find_py_decorated_function_name(node, content);
                // Determine HTTP method: default GET, check for methods=["POST"] etc.
                let method = extract_flask_method_from_decorator(decorator_text);
                let evidence = format!("@{obj_name}.route(\"{path}\") on {handler_name}");
                routes.push(ExtractedRoute {
                    method,
                    path_pattern: path,
                    handler_name,
                    framework: "flask".to_string(),
                    route_source: "DECORATOR".to_string(),
                    mount_prefix: None,
                    is_dynamic: false,
                    route_confidence: 1.0,
                    evidence,
                });
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_py_routes(child, content, fastapi_routers, flask_objects, routes);
    }
}

/// Extract the path string from a Python decorator text like `@router.get("/items/{item_id}")`.
/// Looks for the content between the first `("` and `")`.
fn extract_py_decorator_path(decorator_text: &str) -> Option<String> {
    let start = decorator_text.find("(\"")? + 2;
    let end = decorator_text[start..]
        .find("\")")
        .or_else(|| decorator_text[start..].find('"'))?
        + start;
    Some(decorator_text[start..end].to_string())
}

/// Given a decorator node, find the decorated function name by looking at the
/// parent (decorated_definition) and then its function_definition child.
fn find_py_decorated_function_name(node: tree_sitter::Node, content: &str) -> String {
    // decorator nodes are children of decorated_definition.
    // The decorated_definition also has a function_definition child.
    if let Some(parent) = node.parent()
        && parent.kind() == "decorated_definition"
    {
        let mut cursor = parent.walk();
        for child in parent.children(&mut cursor) {
            if child.kind() == "function_definition" {
                // The function name is the first identifier child
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

/// Extract the HTTP method from a Flask @app.route() decorator.
/// Looks for methods=["POST"] etc. Defaults to GET.
fn extract_flask_method_from_decorator(decorator_text: &str) -> String {
    // Look for methods=["POST"] or methods=['POST']
    if let Some(idx) = decorator_text.find("methods=") {
        let after = &decorator_text[idx + 8..];
        let after = after.trim_start();
        // Look for the method string inside the list
        if let Some(bracket_start) = after.find('[') {
            let rest = &after[bracket_start + 1..];
            // Extract the string content
            for quote in &['"', '\''] {
                if let Some(q_start) = rest.find(*quote) {
                    let inner = &rest[q_start + 1..];
                    if let Some(q_end) = inner.find(*quote) {
                        let method = &inner[..q_end];
                        return method.to_ascii_uppercase();
                    }
                }
            }
        }
    }
    "GET".to_string()
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

/// Directories/filenames that conventionally indicate Python data models.
const PY_MODEL_DIRS: &[&str] = &["models/", "entities/", "domain/"];
const PY_MODEL_FILES: &[&str] = &["models.py"];

pub fn extract_data_models(
    content: &str,
    file_path: &str,
    _symbols: &[Symbol],
) -> Result<Vec<ExtractedModel>> {
    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE;
    parser.set_language(&language.into()).into_diagnostic()?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| miette::miette!("Failed to parse Python content"))?;

    let mut models = Vec::new();
    collect_py_data_models(tree.root_node(), content, file_path, &mut models);
    Ok(models)
}

fn collect_py_data_models(
    node: tree_sitter::Node,
    content: &str,
    file_path: &str,
    models: &mut Vec<ExtractedModel>,
) {
    let kind = node.kind();

    if kind == "class_definition" {
        let class_name = node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(content.as_bytes()).ok())
            .unwrap_or("")
            .to_string();

        if !class_name.is_empty() {
            // Check base classes in argument_list
            let mut base_classes: Vec<String> = Vec::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "argument_list" {
                    let mut acursor = child.walk();
                    for arg in child.children(&mut acursor) {
                        let arg_text = arg.utf8_text(content.as_bytes()).unwrap_or("").to_string();
                        base_classes.push(arg_text);
                    }
                }
            }

            // Check for @dataclass decorator
            let mut has_dataclass = false;
            if let Some(parent) = node.parent()
                && parent.kind() == "decorated_definition"
            {
                let mut pcursor = parent.walk();
                for sibling in parent.children(&mut pcursor) {
                    if sibling.kind() == "decorator" {
                        let dec_text = sibling
                            .utf8_text(content.as_bytes())
                            .unwrap_or("")
                            .to_string();
                        if dec_text.contains("@dataclass") {
                            has_dataclass = true;
                        }
                    }
                }
            }

            // Check base classes against known model bases
            let mut found_model = false;
            for base in &base_classes {
                // Pydantic: BaseModel
                if base == "BaseModel" {
                    models.push(ExtractedModel {
                        model_name: class_name.clone(),
                        language: "Python".to_string(),
                        model_kind: ModelKind::Class,
                        confidence: 1.0,
                        evidence: "base: BaseModel".to_string(),
                    });
                    found_model = true;
                    break;
                }
                // SQLAlchemy: Base
                if base == "Base" {
                    models.push(ExtractedModel {
                        model_name: class_name.clone(),
                        language: "Python".to_string(),
                        model_kind: ModelKind::Class,
                        confidence: 1.0,
                        evidence: "base: Base".to_string(),
                    });
                    found_model = true;
                    break;
                }
                // Flask-SQLAlchemy: db.Model
                if base == "db.Model" {
                    models.push(ExtractedModel {
                        model_name: class_name.clone(),
                        language: "Python".to_string(),
                        model_kind: ModelKind::Class,
                        confidence: 1.0,
                        evidence: "base: db.Model".to_string(),
                    });
                    found_model = true;
                    break;
                }
                // Django: models.Model
                if base == "models.Model" {
                    models.push(ExtractedModel {
                        model_name: class_name.clone(),
                        language: "Python".to_string(),
                        model_kind: ModelKind::Class,
                        confidence: 1.0,
                        evidence: "base: models.Model".to_string(),
                    });
                    found_model = true;
                    break;
                }
            }

            // dataclass in models directory/file
            if !found_model && has_dataclass {
                let in_model_dir = PY_MODEL_DIRS.iter().any(|dir| file_path.contains(dir));
                let in_model_file = PY_MODEL_FILES.iter().any(|f| file_path.ends_with(f));
                if in_model_dir || in_model_file {
                    let dir_match = PY_MODEL_DIRS
                        .iter()
                        .find(|dir| file_path.contains(*dir))
                        .unwrap_or(&"models/");
                    models.push(ExtractedModel {
                        model_name: class_name.clone(),
                        language: "Python".to_string(),
                        model_kind: ModelKind::Class,
                        confidence: 0.7,
                        evidence: format!("dir: {dir_match}"),
                    });
                    found_model = true;
                }
            }

            // Directory convention: classes in models.py or models/ package
            if !found_model {
                let in_model_dir = PY_MODEL_DIRS.iter().any(|dir| file_path.contains(dir));
                let in_model_file = PY_MODEL_FILES.iter().any(|f| file_path.ends_with(f));
                if in_model_dir || in_model_file {
                    let dir_match = PY_MODEL_DIRS
                        .iter()
                        .find(|dir| file_path.contains(*dir))
                        .copied()
                        .unwrap_or("models.py");
                    models.push(ExtractedModel {
                        model_name: class_name.clone(),
                        language: "Python".to_string(),
                        model_kind: ModelKind::Class,
                        confidence: 0.7,
                        evidence: format!("dir: {dir_match}"),
                    });
                }
            }
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_py_data_models(child, content, file_path, models);
    }
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

    #[test]
    fn test_extract_routes_fastapi() {
        let content = r#"
from fastapi import APIRouter

router = APIRouter()

@router.get("/items/{item_id}")
def read_item(item_id: int):
    pass
"#;

        let routes = extract_routes(content, &[]).unwrap();
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/items/{item_id}" && r.framework == "fastapi")
            .expect("should find fastapi GET /items/{item_id} route");
        assert_eq!(route.method, "GET");
        assert_eq!(route.handler_name, "read_item");
        assert_eq!(route.route_source, "DECORATOR");
        assert!(!route.is_dynamic);
        assert_eq!(route.route_confidence, 1.0);
    }

    #[test]
    fn test_extract_routes_flask_get() {
        let content = r#"
from flask import Flask

app = Flask(__name__)

@app.route("/users")
def get_users():
    return "users"
"#;

        let routes = extract_routes(content, &[]).unwrap();
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/users" && r.framework == "flask")
            .expect("should find flask GET /users route");
        assert_eq!(route.method, "GET");
        assert_eq!(route.handler_name, "get_users");
        assert_eq!(route.route_source, "DECORATOR");
        assert!(!route.is_dynamic);
        assert_eq!(route.route_confidence, 1.0);
    }

    #[test]
    fn test_extract_routes_flask_post() {
        let content = r#"
from flask import Flask

app = Flask(__name__)

@app.route("/items", methods=["POST"])
def create_item():
    return "created"
"#;

        let routes = extract_routes(content, &[]).unwrap();
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/items" && r.framework == "flask")
            .expect("should find flask POST /items route");
        assert_eq!(route.method, "POST");
        assert_eq!(route.handler_name, "create_item");
        assert_eq!(route.route_source, "DECORATOR");
        assert!(!route.is_dynamic);
        assert_eq!(route.route_confidence, 1.0);
    }

    #[test]
    fn test_extract_data_models_pydantic() {
        let content = r#"
from pydantic import BaseModel

class User(BaseModel):
    id: int
    name: str
    email: str
"#;

        let models = extract_data_models(content, "src/models/user.py", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "User")
            .expect("should find User data model via Pydantic BaseModel");
        assert_eq!(model.model_kind, ModelKind::Class);
        assert!((model.confidence - 1.0).abs() < f64::EPSILON);
        assert!(model.evidence.contains("base: BaseModel"));
    }

    #[test]
    fn test_extract_data_models_sqlalchemy() {
        let content = r#"
from sqlalchemy.orm import Base

class User(Base):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True)
"#;

        let models = extract_data_models(content, "src/db/user.py", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "User")
            .expect("should find User data model via SQLAlchemy Base");
        assert_eq!(model.model_kind, ModelKind::Class);
        assert!((model.confidence - 1.0).abs() < f64::EPSILON);
        assert!(model.evidence.contains("base: Base"));
    }

    #[test]
    fn test_extract_data_models_django() {
        let content = r#"
from django.db import models

class User(models.Model):
    name = models.CharField(max_length=100)
    email = models.EmailField()
"#;

        let models = extract_data_models(content, "src/models.py", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "User")
            .expect("should find User data model via Django models.Model");
        assert_eq!(model.model_kind, ModelKind::Class);
        assert!((model.confidence - 1.0).abs() < f64::EPSILON);
        assert!(model.evidence.contains("base: models.Model"));
    }

    #[test]
    fn test_extract_data_models_flask_sqlalchemy() {
        let content = r#"
from flask_sqlalchemy import SQLAlchemy

db = SQLAlchemy()

class User(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(100))
"#;

        let models = extract_data_models(content, "src/app/models.py", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "User")
            .expect("should find User data model via Flask-SQLAlchemy db.Model");
        assert_eq!(model.model_kind, ModelKind::Class);
        assert!((model.confidence - 1.0).abs() < f64::EPSILON);
        assert!(model.evidence.contains("base: db.Model"));
    }

    #[test]
    fn test_extract_data_models_dataclass_in_models() {
        let content = r#"
from dataclasses import dataclass

@dataclass
class UserDTO:
    id: int
    name: str
"#;

        let models = extract_data_models(content, "src/models/dto.py", &[]).unwrap();
        let model = models
            .iter()
            .find(|m| m.model_name == "UserDTO")
            .expect("should find UserDTO data model via dataclass in models dir");
        assert_eq!(model.model_kind, ModelKind::Class);
        assert!((model.confidence - 0.7).abs() < f64::EPSILON);
        assert!(model.evidence.contains("dir: models/"));
    }

    #[test]
    fn test_extract_data_models_plain_class_not_model() {
        let content = r#"
class Helper:
    def __init__(self, x: int):
        self.x = x

    def process(self):
        pass
"#;

        let models = extract_data_models(content, "src/utils/helper.py", &[]).unwrap();
        assert!(
            models.iter().all(|m| m.model_name != "Helper"),
            "plain class in non-model dir should NOT be a data model"
        );
    }
}
