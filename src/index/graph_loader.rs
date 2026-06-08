use crate::state::graph_kinds::{EdgeKind, NodeKind};
use crate::state::storage::StorageManager;
use crate::state::storage_cozo::{CozoStorage, GraphEdge, GraphNode};
use miette::{IntoDiagnostic, Result};
use serde_json::json;
use tracing::info;

#[derive(Debug, Clone)]
pub struct GraphStats {
    pub nodes_added: usize,
    pub edges_added: usize,
    pub files_indexed: usize,
    pub symbols_indexed: usize,
}

#[derive(Debug, Clone)]
pub struct Community {
    pub id: usize,
    pub node_ids: Vec<String>,
    pub size: usize,
}

/// Build a native graph in CozoDB by reading from SQLite tables.
pub fn build_native_graph(
    storage: &StorageManager,
    cozo: &CozoStorage,
    provenance_id: &str,
    config: &crate::config::model::Config,
) -> Result<GraphStats> {
    let conn = storage.get_connection();

    // --- 1. Read project_files → file nodes ---
    let mut file_stmt = conn
        .prepare("SELECT file_path, language FROM project_files WHERE parse_status != 'DELETED'")
        .into_diagnostic()?;

    let file_rows: Vec<(String, Option<String>)> = file_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(file_stmt);

    let mut node_batch = Vec::new();
    let mut files_indexed = 0usize;
    for (file_path, language) in &file_rows {
        let metadata = json!({ "language": language, "schema_version": "v1" });
        let id = crate::platform::urn::build_urn(NodeKind::File, file_path);
        node_batch.push(GraphNode {
            id,
            label: file_path.clone(),
            category: NodeKind::File,
            risk_score: 0.0,
            metadata: Some(metadata),
        });
        files_indexed += 1;
    }

    // --- 2. Read project_symbols → symbol nodes ---
    let mut sym_stmt = conn
        .prepare("SELECT qualified_name, symbol_name, symbol_kind FROM project_symbols")
        .into_diagnostic()?;

    let sym_rows: Vec<(String, String, String)> = sym_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(sym_stmt);

    let mut symbols_indexed = 0usize;
    for (qualified_name, symbol_name, symbol_kind) in &sym_rows {
        let metadata = json!({ "kind": symbol_kind, "schema_version": "v1" });
        let id = crate::platform::urn::build_urn(NodeKind::Symbol, qualified_name);
        node_batch.push(GraphNode {
            id,
            label: symbol_name.clone(),
            category: NodeKind::Symbol,
            risk_score: 0.0,
            metadata: Some(metadata),
        });
        symbols_indexed += 1;
    }

    cozo.insert_nodes(&node_batch)?;

    // --- 3. Read structural_edges → edge relations ---
    let mut edge_stmt = conn
        .prepare(
            "SELECT \
             ps_caller.qualified_name, \
             COALESCE(ps_callee.qualified_name, se.unresolved_callee), \
             se.call_kind \
             FROM structural_edges se \
             JOIN project_symbols ps_caller ON se.caller_symbol_id = ps_caller.id \
             LEFT JOIN project_symbols ps_callee ON se.callee_symbol_id = ps_callee.id",
        )
        .into_diagnostic()?;

    let edge_rows: Vec<(String, Option<String>, String)> = edge_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(edge_stmt);

    let mut edge_batch = Vec::new();
    let mut edges_added = 0usize;
    for (source, target_opt, _call_kind) in &edge_rows {
        let target = match target_opt {
            Some(t) => t.as_str(),
            None => continue,
        };
        let source_id = crate::platform::urn::build_urn(NodeKind::Symbol, &source);
        let target_id = crate::platform::urn::build_urn(NodeKind::Symbol, target);

        edge_batch.push(GraphEdge {
            source: source_id,
            target: target_id,
            relation: EdgeKind::Calls,
            confidence: 1.0,
            provenance_id: provenance_id.to_string(),
        });
        edges_added += 1;
    }

    cozo.insert_edges(&edge_batch)?;

    // --- 4. Read api_routes → endpoint nodes and edges ---
    let mut route_stmt = conn
        .prepare(
            "SELECT \
             ar.method, ar.path_pattern, ps.qualified_name, \
             ar.auth_requirements, ar.schema_refs, ar.owning_service, ar.consumers \
             FROM api_routes ar \
             LEFT JOIN project_symbols ps ON ar.handler_symbol_id = ps.id",
        )
        .into_diagnostic()?;

    #[allow(clippy::type_complexity)]
    let route_rows: Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = route_stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(route_stmt);

    let mut endpoint_nodes = Vec::new();
    let mut endpoint_edges = Vec::new();

    for (method, path, qn_opt, auth_json, schema_json, service_opt, consumers_json) in &route_rows {
        let endpoint_id = format!("urn:changeguard:endpoint:{}:{}", method, path);
        let metadata = json!({
            "method": method,
            "path": path,
            "schema_version": "v1",
            "auth": auth_json,
            "schemas": schema_json,
        });

        endpoint_nodes.push(GraphNode {
            id: endpoint_id.clone(),
            label: format!("{} {}", method, path),
            category: NodeKind::Endpoint,
            risk_score: 0.0,
            metadata: Some(metadata),
        });

        if let Some(qn) = qn_opt {
            let handler_urn = crate::platform::urn::build_urn(NodeKind::Symbol, qn);
            endpoint_edges.push(GraphEdge {
                source: handler_urn,
                target: endpoint_id.clone(),
                relation: EdgeKind::Handles,
                confidence: 1.0,
                provenance_id: provenance_id.to_string(),
            });
        }

        if let Some(service) = service_opt {
            let service_urn = crate::platform::urn::build_urn(NodeKind::Service, service);
            endpoint_edges.push(GraphEdge {
                source: service_urn,
                target: endpoint_id.clone(),
                relation: EdgeKind::Owns,
                confidence: 1.0,
                provenance_id: provenance_id.to_string(),
            });
        }

        if let Some(auth_reqs_raw) = auth_json
            && let Ok(auth_reqs) = serde_json::from_str::<Vec<String>>(auth_reqs_raw)
        {
            for auth in auth_reqs {
                let auth_urn = crate::platform::urn::build_urn(NodeKind::SecurityBoundary, &auth);
                endpoint_edges.push(GraphEdge {
                    source: endpoint_id.clone(),
                    target: auth_urn,
                    relation: EdgeKind::Authenticates,
                    confidence: 1.0,
                    provenance_id: provenance_id.to_string(),
                });
            }
        }

        if let Some(consumers_raw) = consumers_json
            && let Ok(consumers) = serde_json::from_str::<Vec<String>>(consumers_raw)
        {
            for consumer in consumers {
                let consumer_urn = if consumer.starts_with("urn:") {
                    consumer
                } else {
                    crate::platform::urn::build_urn(NodeKind::Service, &consumer)
                };
                endpoint_edges.push(GraphEdge {
                    source: consumer_urn,
                    target: endpoint_id.clone(),
                    relation: EdgeKind::Consumes,
                    confidence: 1.0,
                    provenance_id: provenance_id.to_string(),
                });
            }
        }

        if let Some(schemas_raw) = schema_json
            && let Ok(schemas) = serde_json::from_str::<Vec<String>>(schemas_raw)
        {
            for schema in schemas {
                let schema_urn = crate::platform::urn::build_urn(NodeKind::DataModel, &schema);
                endpoint_edges.push(GraphEdge {
                    source: endpoint_id.clone(),
                    target: schema_urn,
                    relation: EdgeKind::Handles,
                    confidence: 1.0,
                    provenance_id: provenance_id.to_string(),
                });
            }
        }
    }

    cozo.insert_nodes(&endpoint_nodes)?;
    cozo.insert_edges(&endpoint_edges)?;

    // --- 5. Read adr_metadata → ADR nodes and links ---
    let mut adr_stmt = conn
        .prepare(
            "SELECT am.adr_id, am.status, am.owner, am.supersedes, am.affected_entities, le.summary \
             FROM adr_metadata am \
             JOIN ledger_entries le ON am.adr_id = le.tx_id",
        )
        .into_diagnostic()?;

    let adr_rows: Vec<(String, String, Option<String>, Option<String>, Option<String>, String)> = adr_stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(adr_stmt);

    let mut adr_nodes = Vec::new();
    let mut adr_edges = Vec::new();

    for (adr_id, status, owner, supersedes, _affected, summary) in &adr_rows {
        let urn = format!("urn:changeguard:adr:{}", adr_id);
        adr_nodes.push(GraphNode {
            id: urn.clone(),
            label: format!("ADR: {}", summary),
            category: NodeKind::Adr,
            risk_score: 0.0,
            metadata: Some(json!({"status": status, "owner": owner, "schema_version": "v1"})),
        });

        let tx_urn = format!("urn:changeguard:transaction:{}", adr_id);
        adr_edges.push(GraphEdge {
            source: urn.clone(),
            target: tx_urn,
            relation: EdgeKind::Governs,
            confidence: 1.0,
            provenance_id: provenance_id.to_string(),
        });

        if let Some(old_adr_id) = supersedes {
            let old_urn = format!("urn:changeguard:adr:{}", old_adr_id);
            adr_edges.push(GraphEdge { source: urn.clone(), target: old_urn, relation: EdgeKind::Supersedes, confidence: 1.0, provenance_id: provenance_id.to_string() });
        }
    }

    cozo.insert_nodes(&adr_nodes)?;
    cozo.insert_edges(&adr_edges)?;

    // --- 6. Read declared services ---
    let mut service_nodes = Vec::new();
    let service_edges: Vec<GraphEdge> = Vec::new();
    for ds in &config.services.definitions {
        let urn = crate::platform::urn::build_urn(NodeKind::Service, &ds.name);
        service_nodes.push(GraphNode {
            id: urn.clone(),
            label: format!("Service: {}", ds.name),
            category: NodeKind::Service,
            risk_score: 0.0,
            metadata: Some(json!({"root": ds.root, "schema_version": "v1"})),
        });
    }
    cozo.insert_nodes(&service_nodes)?;

    // --- 7. Read Data Models ---
    let mut model_stmt = conn.prepare("SELECT model_name, language, model_kind, fields FROM data_models").into_diagnostic()?;
    let model_rows: Vec<(String, String, String, Option<String>)> = model_stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))).into_diagnostic()?.collect::<Result<Vec<_>, _>>().into_diagnostic()?;
    drop(model_stmt);

    let mut model_nodes = Vec::new();
    for (name, lang, kind, fields) in &model_rows {
        let urn = crate::platform::urn::build_urn(NodeKind::DataModel, name);
        model_nodes.push(GraphNode {
            id: urn,
            label: format!("Model: {}", name),
            category: NodeKind::DataModel,
            risk_score: 0.0,
            metadata: Some(json!({"language": lang, "kind": kind, "fields": fields, "schema_version": "v1"})),
        });
    }
    cozo.insert_nodes(&model_nodes)?;

    // --- 8. Read OpenSLO YAMLs ---
    let mut obs_nodes = Vec::new();
    let obs_edges: Vec<GraphEdge> = Vec::new();
    let obs_dir = storage.root_path().join("observability");
    if obs_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(obs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("yml") || path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                    let content = std::fs::read_to_string(&path).unwrap_or_default();
                    if let Ok(slos) = crate::observability::openslo::parse_openslo(&content) {
                        for slo in slos {
                            obs_nodes.push(GraphNode {
                                id: slo.urn.clone(),
                                label: format!("SLO: {}", slo.name),
                                category: NodeKind::Slo,
                                risk_score: 0.0,
                                metadata: Some(slo.metadata),
                            });
                        }
                    }
                }
            }
        }
    }
    cozo.insert_nodes(&obs_nodes)?;

    // --- 9. Read Cedar Policies ---
    let mut policy_nodes = Vec::new();
    let policy_edges: Vec<GraphEdge> = Vec::new();
    let policy_dir = storage.root_path().join("policies");
    if policy_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(policy_dir) {
            let cedar_importer = crate::policy::cedar::CedarImporter::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("cedar") {
                    let content = std::fs::read_to_string(&path).unwrap_or_default();
                    let policies = cedar_importer.parse(&content);
                    for (i, policy) in policies.iter().enumerate() {
                        let urn = format!("urn:changeguard:policy:{}:{}", path.to_string_lossy(), i);
                        policy_nodes.push(GraphNode {
                            id: urn,
                            label: format!("Policy: {} {}", policy.effect, i),
                            category: NodeKind::Policy,
                            risk_score: 0.0,
                            metadata: Some(json!({"effect": policy.effect, "raw": policy.raw})),
                        });
                    }
                }
            }
        }
    }
    cozo.insert_nodes(&policy_nodes)?;

    info!(
        "Native graph built: {} files, {} symbols, {} edges, {} endpoints, {} ADRs, {} services, {} models, {} obs, {} policies",
        files_indexed, symbols_indexed, edges_added + endpoint_edges.len() + adr_edges.len() + obs_edges.len() + policy_edges.len(),
        endpoint_nodes.len(), adr_nodes.len(), service_nodes.len(), model_nodes.len(), obs_nodes.len(), policy_nodes.len()
    );

    Ok(GraphStats {
        nodes_added: files_indexed + symbols_indexed + endpoint_nodes.len() + adr_nodes.len() + service_nodes.len() + model_nodes.len() + obs_nodes.len() + policy_nodes.len(),
        edges_added: edges_added + endpoint_edges.len() + adr_edges.len() + service_edges.len() + obs_edges.len() + policy_edges.len(),
        files_indexed,
        symbols_indexed,
    })
}

pub fn run_community_louvain(cozo: &CozoStorage) -> Result<Vec<Community>> {
    let script = "
        ?[node, comm_id] := *node{id: node}, comm_id = 0
    ";
    let res = cozo.run_script(script)?;
    let mut communities = Vec::new();
    let mut nodes_by_comm: std::collections::HashMap<i64, Vec<String>> = std::collections::HashMap::new();

    for row in res.rows {
        if let (Some(cozo::DataValue::Str(node)), Some(cozo::DataValue::Num(cozo::Num::Int(comm)))) = (row.first(), row.get(1)) {
            nodes_by_comm.entry(*comm).or_default().push(node.to_string());
        }
    }

    for (id, nodes) in nodes_by_comm {
        communities.push(Community {
            id: id as usize,
            size: nodes.len(),
            node_ids: nodes,
        });
    }

    Ok(communities)
}

