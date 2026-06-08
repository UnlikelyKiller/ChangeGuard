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
        node_batch.push(json!([
            id,
            file_path.as_str(),
            NodeKind::File.to_string(),
            0.0,
            metadata
        ]));
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
        node_batch.push(json!([
            id,
            symbol_name.as_str(),
            NodeKind::Symbol.to_string(),
            0.0,
            metadata
        ]));
        symbols_indexed += 1;
    }

    if !node_batch.is_empty() {
        let script = "?[id, label, category, risk_score, metadata] <- $batch :put node";
        let mut params = std::collections::BTreeMap::new();
        params.insert(
            "batch".to_string(),
            cozo::DataValue::from(serde_json::Value::Array(node_batch)),
        );
        cozo.run_script_with_params(script, params, cozo::ScriptMutability::Mutable)?;
    }

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
        let source_id = crate::platform::urn::build_urn(NodeKind::Symbol, source);
        let target_id = crate::platform::urn::build_urn(NodeKind::Symbol, target);

        edge_batch.push(json!([
            source_id,
            target_id,
            EdgeKind::Calls.to_string(),
            1.0,
            provenance_id
        ]));
        edges_added += 1;
    }

    if !edge_batch.is_empty() {
        let script = "?[source, target, relation, confidence, provenance_id] <- $batch :put edge";
        let mut params = std::collections::BTreeMap::new();
        params.insert(
            "batch".to_string(),
            cozo::DataValue::from(serde_json::Value::Array(edge_batch)),
        );
        cozo.run_script_with_params(script, params, cozo::ScriptMutability::Mutable)?;
    }

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

        // Handler -> Endpoint
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

        // Service -> Endpoint
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

        // Auth -> Endpoint
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

        // Consumers -> Endpoint
        if let Some(consumers_raw) = consumers_json
            && let Ok(consumers) = serde_json::from_str::<Vec<String>>(consumers_raw)
        {
            for consumer in consumers {
                // Try to guess kind: if it looks like a URN, use it, otherwise assume Service
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

        // Schemas -> Endpoint
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

    let adr_rows: Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
    )> = adr_stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(adr_stmt);

    let mut adr_nodes = Vec::new();
    let mut adr_edges = Vec::new();

    for (adr_id, status, owner, supersedes, affected, summary) in &adr_rows {
        let urn = format!("urn:changeguard:adr:{}", adr_id);
        let metadata = json!({
            "status": status,
            "owner": owner,
            "schema_version": "v1",
        });

        adr_nodes.push(GraphNode {
            id: urn.clone(),
            label: format!("ADR: {}", summary),
            category: NodeKind::Adr,
            risk_score: 0.0,
            metadata: Some(metadata),
        });

        // ADR -> Ledger Transaction
        let tx_urn = format!("urn:changeguard:transaction:{}", adr_id);
        adr_edges.push(GraphEdge {
            source: urn.clone(),
            target: tx_urn,
            relation: EdgeKind::Governs,
            confidence: 1.0,
            provenance_id: provenance_id.to_string(),
        });

        // Supersession
        if let Some(old_adr_id) = supersedes {
            let old_urn = format!("urn:changeguard:adr:{}", old_adr_id);
            adr_edges.push(GraphEdge {
                source: urn.clone(),
                target: old_urn,
                relation: EdgeKind::Supersedes,
                confidence: 1.0,
                provenance_id: provenance_id.to_string(),
            });
        }

        // Affected Entities
        if let Some(affected_raw) = affected {
            if let Ok(entities) = serde_json::from_str::<Vec<String>>(affected_raw) {
                for entity in entities {
                    // Try to guess kind, or use as is if URN
                    let target_urn = if entity.starts_with("urn:") {
                        entity
                    } else {
                        // Assume it's a file or symbol
                        if entity.contains('/') || entity.ends_with(".rs") {
                            crate::platform::urn::build_urn(NodeKind::File, &entity)
                        } else {
                            crate::platform::urn::build_urn(NodeKind::Symbol, &entity)
                        }
                    };
                    adr_edges.push(GraphEdge {
                        source: urn.clone(),
                        target: target_urn,
                        relation: EdgeKind::Governs,
                        confidence: 0.9,
                        provenance_id: provenance_id.to_string(),
                    });
                }
            }
        }
    }

    cozo.insert_nodes(&adr_nodes)?;
    cozo.insert_edges(&adr_edges)?;

    // --- 6. Read declared services → Service nodes and topology links ---
    let mut service_nodes = Vec::new();
    let mut service_edges = Vec::new();

    for ds in &config.services.definitions {
        let urn = crate::platform::urn::build_urn(NodeKind::Service, &ds.name);
        let metadata = json!({
            "root": ds.root,
            "runtime_name": ds.runtime_name,
            "schema_version": "v1",
        });

        service_nodes.push(GraphNode {
            id: urn.clone(),
            label: format!("Service: {}", ds.name),
            category: NodeKind::Service,
            risk_score: 0.0,
            metadata: Some(metadata),
        });

        // Service -> Owners
        for owner in &ds.owners {
            let owner_urn = crate::platform::urn::build_urn(NodeKind::Symbol, owner); // Heuristic: owners are people/teams modeled as symbols or just strings
            service_edges.push(GraphEdge {
                source: owner_urn,
                target: urn.clone(),
                relation: EdgeKind::Owns,
                confidence: 1.0,
                provenance_id: provenance_id.to_string(),
            });
        }

        // Service -> Queues (Emits)
        for queue in &ds.queues {
            let queue_urn = format!("urn:changeguard:queue:{}", queue);
            service_edges.push(GraphEdge {
                source: urn.clone(),
                target: queue_urn.clone(),
                relation: EdgeKind::Emits,
                confidence: 1.0,
                provenance_id: provenance_id.to_string(),
            });
            // Ensure queue node exists
            service_nodes.push(GraphNode {
                id: queue_urn,
                label: format!("Queue: {}", queue),
                category: NodeKind::Service, // Or a new Queue kind? Using Service for now.
                risk_score: 0.0,
                metadata: None,
            });
        }

        // Service -> Topics (Emits)
        for topic in &ds.topics {
            let topic_urn = format!("urn:changeguard:topic:{}", topic);
            service_edges.push(GraphEdge {
                source: urn.clone(),
                target: topic_urn.clone(),
                relation: EdgeKind::Emits,
                confidence: 1.0,
                provenance_id: provenance_id.to_string(),
            });
            // Ensure topic node exists
            service_nodes.push(GraphNode {
                id: topic_urn,
                label: format!("Topic: {}", topic),
                category: NodeKind::Service,
                risk_score: 0.0,
                metadata: None,
            });
        }
    }

    cozo.insert_nodes(&service_nodes)?;
    cozo.insert_edges(&service_edges)?;

    // --- 7. Read data_models → DataModel nodes ---
    let mut model_stmt = conn
        .prepare("SELECT model_name, language, model_kind, confidence, evidence FROM data_models")
        .into_diagnostic()?;

    let model_rows: Vec<(String, String, String, f64, Option<String>)> = model_stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(model_stmt);

    let mut model_nodes = Vec::new();
    for (name, lang, kind, conf, evidence) in &model_rows {
        let urn = crate::platform::urn::build_urn(NodeKind::DataModel, name);
        let metadata = json!({
            "language": lang,
            "kind": kind,
            "confidence": conf,
            "evidence": evidence,
            "schema_version": "v1",
        });

        model_nodes.push(GraphNode {
            id: urn,
            label: format!("Model: {}", name),
            category: NodeKind::DataModel,
            risk_score: 0.0,
            metadata: Some(metadata),
        });
    }
    cozo.insert_nodes(&model_nodes)?;

    // --- 8. Read OpenSLO YAMLs → SLO/Metric nodes and edges ---
    let mut obs_nodes = Vec::new();
    let mut obs_edges = Vec::new();

    let mut yaml_files = Vec::new();
    let obs_dir = storage.root_path().join("observability");
    if obs_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(obs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "yml" || ext == "yaml" {
                        yaml_files.push(path);
                    }
                }
            }
        }
    }

    for path in yaml_files {
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

                if let Some(svc) = slo.service_name {
                    let svc_urn = crate::platform::urn::build_urn(NodeKind::Service, &svc);
                    obs_edges.push(GraphEdge {
                        source: slo.urn.clone(),
                        target: svc_urn,
                        relation: EdgeKind::Monitors,
                        confidence: 1.0,
                        provenance_id: provenance_id.to_string(),
                    });
                }

                for metric in slo.metrics {
                    obs_nodes.push(GraphNode {
                        id: metric.urn.clone(),
                        label: format!("Metric: {}", metric.name),
                        category: NodeKind::Metric,
                        risk_score: 0.0,
                        metadata: Some(json!({"query": metric.query, "source": metric.source})),
                    });

                    obs_edges.push(GraphEdge {
                        source: slo.urn.clone(),
                        target: metric.urn,
                        relation: EdgeKind::DependsOn,
                        confidence: 1.0,
                        provenance_id: provenance_id.to_string(),
                    });
                }
            }
        }
    }

    cozo.insert_nodes(&obs_nodes)?;
    cozo.insert_edges(&obs_edges)?;

    info!(
        "Native graph built: {} files, {} symbols, {} edges, {} endpoints, {} ADRs, {} services, {} models, {} observability",
        files_indexed,
        symbols_indexed,
        edges_added + endpoint_edges.len() + adr_edges.len() + service_edges.len() + obs_edges.len(),
        endpoint_nodes.len(),
        adr_nodes.len(),
        service_nodes.len(),
        model_nodes.len(),
        obs_nodes.len()
    );

    Ok(GraphStats {
        nodes_added: files_indexed + symbols_indexed + endpoint_nodes.len() + adr_nodes.len() + service_nodes.len() + model_nodes.len() + obs_nodes.len(),
        edges_added: edges_added + endpoint_edges.len() + adr_edges.len() + service_edges.len() + obs_edges.len(),
        files_indexed,
        symbols_indexed,
    })
}

/// Run Louvain community detection on the CozoDB graph and group results.
/// Note: CozoDB 0.7 does not ship with Leiden; Louvain is the closest available algorithm.
pub fn run_community_louvain(cozo: &CozoStorage) -> Result<Vec<Community>> {
    let raw = cozo.run_community_louvain()?;
    let mut groups: std::collections::HashMap<i64, Vec<String>> = std::collections::HashMap::new();
    for (node, comm) in raw {
        groups.entry(comm).or_default().push(node);
    }

    let mut communities: Vec<Community> = groups
        .into_iter()
        .enumerate()
        .map(|(id, (_, node_ids))| {
            let size = node_ids.len();
            Community { id, node_ids, size }
        })
        .collect();

    communities.sort_by_key(|a| a.id);
    Ok(communities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::storage_cozo::CozoStorage;
    use std::path::PathBuf;

    fn in_memory_storage_with_cozo() -> (StorageManager, CozoStorage) {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let mut conn = conn;
        crate::state::migrations::get_migrations()
            .to_latest(&mut conn)
            .unwrap();
        let storage = StorageManager::init_from_conn(conn);
        let cozo = CozoStorage::new(&PathBuf::from("")).unwrap();
        (storage, cozo)
    }

    #[test]
    fn test_build_native_graph_populates_nodes_and_edges() {
        let (storage, cozo) = in_memory_storage_with_cozo();
        let conn = storage.get_connection();

        // Insert project_files
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) \
             VALUES ('src/main.rs', 'Rust', 'hash1', 100, 'OK', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let file_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) \
             VALUES ('src/lib.rs', 'Rust', 'hash2', 200, 'OK', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let file_id2 = conn.last_insert_rowid();

        // Insert project_symbols
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at) \
             VALUES (?1, 'crate::main', 'main', 'Function', '2026-01-01T00:00:00Z')",
            [file_id],
        )
        .unwrap();
        let sym_main = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at) \
             VALUES (?1, 'crate::helper', 'helper', 'Function', '2026-01-01T00:00:00Z')",
            [file_id2],
        )
        .unwrap();
        let sym_helper = conn.last_insert_rowid();

        // Insert structural_edges
        conn.execute(
            "INSERT INTO structural_edges (caller_symbol_id, caller_file_id, callee_symbol_id, callee_file_id, call_kind, resolution_status) \
             VALUES (?1, ?2, ?3, ?4, 'DIRECT', 'RESOLVED')",
            [sym_main, file_id, sym_helper, file_id2],
        )
        .unwrap();

        let stats = build_native_graph(&storage, &cozo, "test_provenance").unwrap();

        // Verify stats
        assert_eq!(stats.files_indexed, 2);
        assert_eq!(stats.symbols_indexed, 2);
        assert_eq!(stats.edges_added, 1);
        assert_eq!(stats.nodes_added, 4);

        // Verify CozoDB nodes
        let res = cozo.run_script("?[id] := *node{id: id}").unwrap();
        let ids: Vec<String> = res
            .rows
            .iter()
            .filter_map(|row| match row.first() {
                Some(cozo::DataValue::Str(s)) => Some(s.to_string()),
                _ => None,
            })
            .collect();
        assert!(ids.contains(&crate::platform::urn::build_urn(NodeKind::File, "src/main.rs")));
        assert!(ids.contains(&crate::platform::urn::build_urn(NodeKind::Symbol, "crate::main")));
        assert!(ids.contains(&crate::platform::urn::build_urn(NodeKind::Symbol, "crate::helper")));

        // Verify CozoDB edges
        let res = cozo
            .run_script("?[source, target] := *edge{source: source, target: target}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        if let (Some(cozo::DataValue::Str(src)), Some(cozo::DataValue::Str(tgt))) =
            (res.rows[0].first(), res.rows[0].get(1))
        {
            assert_eq!(src.as_str(), &crate::platform::urn::build_urn(NodeKind::Symbol, "crate::main"));
            assert_eq!(tgt.as_str(), &crate::platform::urn::build_urn(NodeKind::Symbol, "crate::helper"));
        } else {
            panic!("Expected string edge endpoints");
        }
    }

    #[test]
    fn test_run_community_louvain_finds_communities() {
        let cozo = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Two disconnected clusters
        cozo.run_script(
            "?[id, label, category, risk_score, metadata] <- [
                ['a1', 'A1', 'code', 0.0, {}],
                ['a2', 'A2', 'code', 0.0, {}],
                ['b1', 'B1', 'code', 0.0, {}],
                ['b2', 'B2', 'code', 0.0, {}]
            ] :put node",
        )
        .unwrap();

        cozo.run_script(
            "?[source, target, relation, confidence, provenance_id] <- [
                ['a1', 'a2', 'calls', 1.0, 'tx1'],
                ['b1', 'b2', 'calls', 1.0, 'tx1']
            ] :put edge",
        )
        .unwrap();

        let communities = run_community_louvain(&cozo).unwrap();
        assert!(!communities.is_empty());

        let distinct_ids: std::collections::HashSet<usize> =
            communities.iter().map(|c| c.id).collect();
        assert!(
            distinct_ids.len() >= 2,
            "Expected at least 2 communities, got {:?}",
            distinct_ids.len()
        );

        let total_nodes: usize = communities.iter().map(|c| c.size).sum();
        assert_eq!(total_nodes, 4);
    }

    #[test]
    fn test_graph_stats_counts_correct() {
        let stats = GraphStats {
            nodes_added: 10,
            edges_added: 5,
            files_indexed: 3,
            symbols_indexed: 7,
        };
        assert_eq!(stats.nodes_added, 10);
        assert_eq!(stats.edges_added, 5);
        assert_eq!(stats.files_indexed, 3);
        assert_eq!(stats.symbols_indexed, 7);
        assert_eq!(
            stats.nodes_added,
            stats.files_indexed + stats.symbols_indexed
        );
    }
}
