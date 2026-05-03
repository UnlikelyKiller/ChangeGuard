use crate::impact::packet::{ApiRoute, DataModel, Service};
use crate::index::call_graph::CallGraph;
use crate::index::topology::DirectoryClassification;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DataModelSource {
    pub model: DataModel,
    pub source_path: String,
}

pub struct DirectoryTopology {
    pub classifications: Vec<DirectoryClassification>,
}

/// Infer service boundaries from routes, call graph, and directory topology.
pub fn infer_services(
    routes: &[ApiRoute],
    data_models: &[DataModelSource],
    call_graph: &CallGraph,
    topology: &DirectoryTopology,
) -> Vec<Service> {
    if routes.is_empty() && call_graph.edges.is_empty() {
        return Vec::new();
    }

    let mut service_groups: std::collections::HashMap<PathBuf, (String, Vec<String>, Vec<String>)> =
        std::collections::HashMap::new();

    // 0. Pre-populate services from explicit ServiceRoot topology
    for class in &topology.classifications {
        if class.role == crate::index::topology::DirectoryRole::ServiceRoot {
            let path = PathBuf::from(&class.dir_path);
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "root".to_string());
            service_groups.insert(path, (name, Vec::new(), Vec::new()));
        }
    }

    // 1. Group routes by their service root directory
    for route in routes {
        let (root_dir, name) = find_best_service_root(Path::new(&route.route_source), topology);
        let entry = service_groups
            .entry(root_dir)
            .or_insert_with(|| (name, Vec::new(), Vec::new()));
        if let Some(handler) = &route.handler_symbol_name
            && !entry.1.contains(handler)
        {
            entry.1.push(handler.clone());
        }
    }

    // 1.5 Detect "worker" services using CallGraph (background logic with no routes)
    for edge in &call_graph.edges {
        let (root_dir, name) = find_best_service_root(&edge.caller_file, topology);
        let entry = service_groups
            .entry(root_dir)
            .or_insert_with(|| (name, Vec::new(), Vec::new()));

        // Add caller as a "route" (logical entrypoint) if not already present
        if !entry.1.contains(&edge.caller_name) {
            entry.1.push(edge.caller_name.clone());
        }
    }

    // 2. Associate data models with services
    for dm_source in data_models {
        let (dm_root_dir, _) = find_best_service_root(Path::new(&dm_source.source_path), topology);
        if let Some(entry) = service_groups.get_mut(&dm_root_dir)
            && !entry.2.contains(&dm_source.model.model_name)
        {
            entry.2.push(dm_source.model.model_name.clone());
        }
    }

    // 3. Map groups to Service structs
    let mut services: Vec<Service> = Vec::new();
    let mut unnamed_count = 0;

    let mut sorted_keys: Vec<_> = service_groups.keys().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        let (name, routes, dms) = &service_groups[key];
        let mut final_name = name.clone();

        if final_name.is_empty() {
            if let Some(pkg_name) = find_package_name(key) {
                final_name = pkg_name;
            } else {
                unnamed_count += 1;
                final_name = format!("unnamed-service-{}", unnamed_count);
            }
        }

        services.push(Service {
            name: final_name,
            directory: key.clone(),
            routes: routes.clone(),
            data_models: dms.clone(),
        });
    }

    services.sort_by(|a, b| a.name.cmp(&b.name));
    services
}

fn get_root_dir(path: &Path) -> PathBuf {
    let dir = if path.extension().is_some() {
        path.parent().unwrap_or(Path::new("")).to_path_buf()
    } else {
        path.to_path_buf()
    };

    let components: Vec<_> = dir.components().collect();
    if components.len() > 3 {
        components.iter().take(3).collect()
    } else {
        dir
    }
}

fn find_best_service_root(path: &Path, topology: &DirectoryTopology) -> (PathBuf, String) {
    // 1. Check topology for any prefix that is a ServiceRoot (deepest first)
    let mut current = if path.extension().is_some() {
        path.parent().unwrap_or(Path::new("")).to_path_buf()
    } else {
        path.to_path_buf()
    };

    loop {
        let current_str = current.to_string_lossy().replace('\\', "/");
        if let Some(_class) = topology.classifications.iter().find(|c| {
            (c.dir_path == current_str || (current_str.is_empty() && c.dir_path == "."))
                && c.role == crate::index::topology::DirectoryRole::ServiceRoot
        }) {
            let name = current
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "root".to_string());
            return (current, name);
        }

        if let Some(parent) = current.parent() {
            if parent == current || current == Path::new("") {
                break;
            }
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    // 2. Fallback to default heuristic
    get_service_root_and_name(&path.to_string_lossy())
}

fn get_service_root_and_name(source_path: &str) -> (PathBuf, String) {
    let path = PathBuf::from(source_path);
    let root_dir = get_root_dir(&path);

    // Rule 3: If directory is just "src", name it "src"
    if root_dir == Path::new("src") {
        return (root_dir, "src".to_string());
    }

    // Rule 1: Last component of the directory path
    if let Some(last) = root_dir.file_name() {
        let name = last.to_string_lossy().to_string();
        if !name.is_empty() && name != "." && name != ".." {
            return (root_dir, name);
        }
    }

    (root_dir, "".to_string())
}

fn find_package_name(dir: &Path) -> Option<String> {
    let mut current = dir;
    loop {
        for filename in &["Cargo.toml", "package.json"] {
            let p = current.join(filename);
            if let Ok(content) = std::fs::read_to_string(&p) {
                if *filename == "Cargo.toml" {
                    if let Ok(value) = toml::from_str::<toml::Value>(&content)
                        && let Some(name) = value
                            .get("package")
                            .and_then(|v| v.get("name"))
                            .and_then(|v| v.as_str())
                        {
                            return Some(name.to_string());
                        }
                } else if *filename == "package.json"
                    && let Ok(value) = serde_json::from_str::<serde_json::Value>(&content)
                        && let Some(name) = value.get("name").and_then(|v| v.as_str()) {
                            return Some(name.to_string());
                        }
            }
        }

        if current.join("__init__.py").exists()
            && let Some(name) = current.file_name().and_then(|n| n.to_str()) {
                return Some(name.to_string());
            }

        if let Some(parent) = current.parent() {
            if parent == current || current == Path::new("") {
                break;
            }
            current = parent;
        } else {
            break;
        }
    }
    None
}

/// Compute cross-service dependency edges from call graph.
pub fn compute_cross_service_edges(
    services: &[Service],
    call_graph: &CallGraph,
) -> Vec<(String, String, usize)> {
    let mut symbol_to_service: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for service in services {
        for route in &service.routes {
            symbol_to_service.insert(route.clone(), service.name.clone());
        }
        for model in &service.data_models {
            symbol_to_service.insert(model.clone(), service.name.clone());
        }
    }

    let mut edges: std::collections::HashMap<(String, String), usize> =
        std::collections::HashMap::new();
    for edge in &call_graph.edges {
        if let (Some(caller_svc), Some(callee_svc)) = (
            symbol_to_service.get(&edge.caller_name),
            symbol_to_service.get(&edge.callee_name),
        )
            && caller_svc != callee_svc {
                *edges
                    .entry((caller_svc.clone(), callee_svc.clone()))
                    .or_insert(0) += 1;
            }
    }

    let mut result: Vec<_> = edges
        .into_iter()
        .map(|((a, b), count)| (a, b, count))
        .collect();

    result.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::call_graph::{CallEdge, CallKind, ResolutionStatus};
    use crate::index::topology::DirectoryRole;

    #[test]
    fn test_infer_services_directory_naming() {
        let routes = vec![ApiRoute {
            method: "GET".to_string(),
            path_pattern: "/users".to_string(),
            handler_symbol_name: Some("get_users".to_string()),
            framework: "actix-web".to_string(),
            route_source: "src/api/users/mod.rs".to_string(),
            mount_prefix: None,
            is_dynamic: false,
            route_confidence: 1.0,
            evidence: None,
        }];
        let topology = DirectoryTopology {
            classifications: vec![DirectoryClassification {
                dir_path: "src/api/users".to_string(),
                role: DirectoryRole::Source,
                confidence: 1.0,
                evidence: "test".to_string(),
            }],
        };
        let call_graph = CallGraph { edges: Vec::new() };

        let services = infer_services(&routes, &[], &call_graph, &topology);
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "users");
    }

    #[test]
    fn test_infer_services_flat_repo() {
        let routes = vec![ApiRoute {
            method: "GET".to_string(),
            path_pattern: "/".to_string(),
            handler_symbol_name: Some("index".to_string()),
            framework: "actix-web".to_string(),
            route_source: "src/handler.rs".to_string(),
            mount_prefix: None,
            is_dynamic: false,
            route_confidence: 1.0,
            evidence: None,
        }];
        let topology = DirectoryTopology {
            classifications: vec![DirectoryClassification {
                dir_path: "src".to_string(),
                role: DirectoryRole::Source,
                confidence: 1.0,
                evidence: "test".to_string(),
            }],
        };
        let call_graph = CallGraph { edges: Vec::new() };

        let services = infer_services(&routes, &[], &call_graph, &topology);
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "src");
    }

    #[test]
    fn test_infer_services_empty() {
        let services = infer_services(
            &[],
            &[],
            &CallGraph { edges: Vec::new() },
            &DirectoryTopology {
                classifications: Vec::new(),
            },
        );
        assert!(services.is_empty());
    }

    #[test]
    fn test_infer_services_monorepo_depth_cap() {
        let routes = vec![ApiRoute {
            method: "GET".to_string(),
            path_pattern: "/auth".to_string(),
            handler_symbol_name: Some("login".to_string()),
            framework: "actix-web".to_string(),
            route_source: "src/api/users/auth/mod.rs".to_string(),
            mount_prefix: None,
            is_dynamic: false,
            route_confidence: 1.0,
            evidence: None,
        }];
        let topology = DirectoryTopology {
            classifications: vec![],
        };
        let call_graph = CallGraph { edges: Vec::new() };

        let services = infer_services(&routes, &[], &call_graph, &topology);
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "users");
        assert_eq!(services[0].directory, PathBuf::from("src/api/users"));
    }

    #[test]
    fn test_infer_services_with_data_models() {
        let routes = vec![ApiRoute {
            method: "GET".to_string(),
            path_pattern: "/users".to_string(),
            handler_symbol_name: Some("get_users".to_string()),
            framework: "actix-web".to_string(),
            route_source: "src/api/users/handlers.rs".to_string(),
            mount_prefix: None,
            is_dynamic: false,
            route_confidence: 1.0,
            evidence: None,
        }];
        let data_models = vec![DataModelSource {
            model: DataModel {
                model_name: "User".to_string(),
                model_kind: "STRUCT".to_string(),
                confidence: 1.0,
                evidence: None,
            },
            source_path: "src/api/users/models.rs".to_string(),
        }];
        let topology = DirectoryTopology {
            classifications: vec![],
        };
        let call_graph = CallGraph { edges: Vec::new() };

        let services = infer_services(&routes, &data_models, &call_graph, &topology);
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].data_models, vec!["User".to_string()]);
    }

    #[test]
    fn test_infer_services_package_name_fallback() {
        use std::fs;
        let temp = tempfile::tempdir().unwrap();
        let repo_dir = temp.path();
        fs::write(repo_dir.join("package.json"), r#"{"name": "from-json"}"#).unwrap();

        let routes = vec![ApiRoute {
            method: "GET".to_string(),
            path_pattern: "/".to_string(),
            handler_symbol_name: Some("h".to_string()),
            framework: "f".to_string(),
            route_source: "handler.rs".to_string(), // In root, so dir is ""
            mount_prefix: None,
            is_dynamic: false,
            route_confidence: 1.0,
            evidence: None,
        }];

        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(repo_dir).unwrap();

        let topology = DirectoryTopology {
            classifications: vec![],
        };
        let call_graph = CallGraph { edges: Vec::new() };

        let services = infer_services(&routes, &[], &call_graph, &topology);

        std::env::set_current_dir(old_cwd).unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "from-json");
    }

    #[test]
    fn test_compute_cross_service_edges() {
        let services = vec![
            Service {
                name: "users".to_string(),
                routes: vec!["get_users".to_string()],
                data_models: vec![],
                directory: PathBuf::from("src/api/users"),
            },
            Service {
                name: "billing".to_string(),
                routes: vec!["charge".to_string()],
                data_models: vec![],
                directory: PathBuf::from("src/api/billing"),
            },
        ];

        let call_graph = CallGraph {
            edges: vec![CallEdge {
                caller_name: "get_users".to_string(),
                caller_file: PathBuf::from("src/api/users/handlers.rs"),
                callee_name: "charge".to_string(),
                callee_file: Some(PathBuf::from("src/api/billing/handlers.rs")),
                call_kind: CallKind::Direct,
                resolution_status: ResolutionStatus::Resolved,
                confidence: 1.0,
                evidence: "test".to_string(),
            }],
        };

        let edges = compute_cross_service_edges(&services, &call_graph);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0], ("users".to_string(), "billing".to_string(), 1));
    }

    #[test]
    fn test_compute_cross_service_edges_collapsed() {
        let services = vec![
            Service {
                name: "users".to_string(),
                routes: vec!["get_users".to_string(), "create_user".to_string()],
                data_models: vec![],
                directory: PathBuf::from("src/api/users"),
            },
            Service {
                name: "billing".to_string(),
                routes: vec!["charge".to_string()],
                data_models: vec![],
                directory: PathBuf::from("src/api/billing"),
            },
        ];

        let call_graph = CallGraph {
            edges: vec![
                CallEdge {
                    caller_name: "get_users".to_string(),
                    caller_file: PathBuf::from("src/api/users/handlers.rs"),
                    callee_name: "charge".to_string(),
                    callee_file: Some(PathBuf::from("src/api/billing/handlers.rs")),
                    call_kind: CallKind::Direct,
                    resolution_status: ResolutionStatus::Resolved,
                    confidence: 1.0,
                    evidence: "test".to_string(),
                },
                CallEdge {
                    caller_name: "create_user".to_string(),
                    caller_file: PathBuf::from("src/api/users/handlers.rs"),
                    callee_name: "charge".to_string(),
                    callee_file: Some(PathBuf::from("src/api/billing/handlers.rs")),
                    call_kind: CallKind::Direct,
                    resolution_status: ResolutionStatus::Resolved,
                    confidence: 1.0,
                    evidence: "test".to_string(),
                },
            ],
        };

        let edges = compute_cross_service_edges(&services, &call_graph);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0], ("users".to_string(), "billing".to_string(), 2));
    }
}
