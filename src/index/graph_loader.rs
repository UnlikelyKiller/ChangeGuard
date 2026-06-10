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

/// Counters for each graph-load phase, used to assert idempotence in tests.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PhaseCounters {
    pub files: usize,
    pub symbols: usize,
    pub call_edges: usize,
    pub routes_nodes: usize,
    pub routes_edges: usize,
    pub dependencies_nodes: usize,
    pub dependencies_edges: usize,
    pub deployments_nodes: usize,
    pub deployments_edges: usize,
    pub environment_model_nodes: usize,
    pub environment_config_nodes: usize,
    pub environment_edges: usize,
    pub observability_nodes: usize,
    pub observability_edges: usize,
    pub security_nodes: usize,
    pub security_edges: usize,
    pub cross_surface_edges: usize,
}

#[allow(clippy::type_complexity)]
type RouteRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

/// Internal context shared across graph load phases.
struct GraphLoadContext<'a> {
    storage: &'a StorageManager,
    cozo: &'a CozoStorage,
    provenance_id: &'a str,
    config: &'a crate::config::model::Config,
    counters: PhaseCounters,
    // Intermediate data for cross-phase linking (security cross-surface pass).
    route_rows: Vec<RouteRow>,
    adr_nodes: Vec<GraphNode>,
    config_nodes: Vec<GraphNode>,
    policy_nodes: Vec<GraphNode>,
    deploy_manifests: Vec<String>,
}

/// Build a native graph in CozoDB by reading from SQLite tables.
pub fn build_native_graph(
    storage: &StorageManager,
    cozo: &CozoStorage,
    provenance_id: &str,
    config: &crate::config::model::Config,
) -> Result<GraphStats> {
    let mut ctx = GraphLoadContext {
        storage,
        cozo,
        provenance_id,
        config,
        counters: PhaseCounters::default(),
        route_rows: Vec::new(),
        adr_nodes: Vec::new(),
        config_nodes: Vec::new(),
        policy_nodes: Vec::new(),
        deploy_manifests: Vec::new(),
    };

    let files_indexed = phase_files(&mut ctx)?;
    let symbols_indexed = phase_symbols(&mut ctx)?;
    phase_call_edges(&mut ctx)?;
    phase_routes(&mut ctx)?;
    phase_dependencies(&mut ctx)?;
    phase_deployments(&mut ctx)?;
    phase_environment(&mut ctx)?;
    phase_observability(&mut ctx)?;
    phase_security(&mut ctx)?;

    info!(
        "Native graph built: {} files, {} symbols, {} edges, {} endpoints, {} ADRs, {} deploy nodes, \
         {} models, {} config_keys, {} obs, {} security nodes, {} cross-surface edges",
        files_indexed,
        symbols_indexed,
        ctx.counters.call_edges
            + ctx.counters.routes_edges
            + ctx.counters.dependencies_edges
            + ctx.counters.deployments_edges
            + ctx.counters.environment_edges
            + ctx.counters.observability_edges
            + ctx.counters.security_edges,
        ctx.counters.routes_nodes,
        ctx.counters.dependencies_nodes, // ADR count carried here
        ctx.counters.deployments_nodes,  // includes services + owners/queues/topics/rpc
        ctx.counters.environment_model_nodes,
        ctx.counters.environment_config_nodes,
        ctx.counters.observability_nodes,
        ctx.counters.security_nodes, // includes policies + principals/actions/resources
        ctx.counters.cross_surface_edges,
    );

    Ok(GraphStats {
        nodes_added: files_indexed
            + symbols_indexed
            + ctx.counters.routes_nodes
            + ctx.counters.dependencies_nodes
            + ctx.counters.deployments_nodes
            + ctx.counters.environment_model_nodes
            + ctx.counters.environment_config_nodes
            + ctx.counters.observability_nodes
            + ctx.counters.security_nodes,
        edges_added: ctx.counters.call_edges
            + ctx.counters.routes_edges
            + ctx.counters.dependencies_edges
            + ctx.counters.deployments_edges
            + ctx.counters.environment_edges
            + ctx.counters.observability_edges
            + ctx.counters.security_edges,
        files_indexed,
        symbols_indexed,
    })
}

// ---------------------------------------------------------------------------
// Phase 1 — File nodes
// ---------------------------------------------------------------------------
fn phase_files(ctx: &mut GraphLoadContext) -> Result<usize> {
    let conn = ctx.storage.get_connection();
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

    ctx.cozo.insert_nodes(&node_batch)?;
    ctx.counters.files = files_indexed;
    Ok(files_indexed)
}

// ---------------------------------------------------------------------------
// Phase 2 — Symbol nodes
// ---------------------------------------------------------------------------
fn phase_symbols(ctx: &mut GraphLoadContext) -> Result<usize> {
    let conn = ctx.storage.get_connection();
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

    let mut node_batch = Vec::new();
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

    ctx.cozo.insert_nodes(&node_batch)?;
    ctx.counters.symbols = symbols_indexed;
    Ok(symbols_indexed)
}

// ---------------------------------------------------------------------------
// Phase 3 — Call edges (structural_edges)
// ---------------------------------------------------------------------------
fn phase_call_edges(ctx: &mut GraphLoadContext) -> Result<()> {
    let conn = ctx.storage.get_connection();
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

        edge_batch.push(GraphEdge {
            source: source_id,
            target: target_id,
            relation: EdgeKind::Calls,
            confidence: 1.0,
            provenance_id: ctx.provenance_id.to_string(),
        });
        edges_added += 1;
    }

    ctx.cozo.insert_edges(&edge_batch)?;
    ctx.counters.call_edges = edges_added;
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 4 — Routes / endpoints
// ---------------------------------------------------------------------------
fn phase_routes(ctx: &mut GraphLoadContext) -> Result<()> {
    let conn = ctx.storage.get_connection();
    let mut route_stmt = conn
        .prepare(
            "SELECT \
             ar.method, ar.path_pattern, ps.qualified_name, \
             ar.auth_requirements, ar.schema_refs, ar.owning_service, ar.consumers \
             FROM api_routes ar \
             LEFT JOIN project_symbols ps ON ar.handler_symbol_id = ps.id",
        )
        .into_diagnostic()?;

    let route_rows: Vec<RouteRow> = route_stmt
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
                provenance_id: ctx.provenance_id.to_string(),
            });
        }

        if let Some(service) = service_opt {
            let service_urn = crate::platform::urn::build_urn(NodeKind::Service, service);
            endpoint_edges.push(GraphEdge {
                source: service_urn,
                target: endpoint_id.clone(),
                relation: EdgeKind::Owns,
                confidence: 1.0,
                provenance_id: ctx.provenance_id.to_string(),
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
                    provenance_id: ctx.provenance_id.to_string(),
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
                    provenance_id: ctx.provenance_id.to_string(),
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
                    provenance_id: ctx.provenance_id.to_string(),
                });
            }
        }
    }

    ctx.cozo.insert_nodes(&endpoint_nodes)?;
    ctx.cozo.insert_edges(&endpoint_edges)?;

    ctx.counters.routes_nodes = endpoint_nodes.len();
    ctx.counters.routes_edges = endpoint_edges.len();
    ctx.route_rows = route_rows;
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 5 — Dependencies / ADR metadata (current code has ADR here; dependency
// lock-file parsing is reserved for a future track).
// ---------------------------------------------------------------------------
fn phase_dependencies(ctx: &mut GraphLoadContext) -> Result<()> {
    let conn = ctx.storage.get_connection();
    let mut adr_stmt = conn
        .prepare(
            "SELECT am.adr_id, am.status, am.owner, am.supersedes, am.affected_entities, le.summary \
             FROM adr_metadata am \
             JOIN ledger_entries le ON am.adr_id = le.tx_id",
        )
        .into_diagnostic()?;

    #[allow(clippy::type_complexity)]
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
            provenance_id: ctx.provenance_id.to_string(),
        });

        if let Some(old_adr_id) = supersedes {
            let old_urn = format!("urn:changeguard:adr:{}", old_adr_id);
            adr_edges.push(GraphEdge {
                source: urn.clone(),
                target: old_urn,
                relation: EdgeKind::Supersedes,
                confidence: 1.0,
                provenance_id: ctx.provenance_id.to_string(),
            });
        }

        if let Some(affected_json) = affected
            && let Ok(entities) = serde_json::from_str::<Vec<String>>(affected_json)
        {
            for entity in entities {
                let entity_urn = if entity.starts_with("urn:") {
                    entity.clone()
                } else if entity.contains('/') || entity.ends_with(".rs") || entity.ends_with(".ts")
                {
                    crate::platform::urn::build_urn(NodeKind::File, &entity)
                } else {
                    crate::platform::urn::build_urn(NodeKind::Symbol, &entity)
                };
                adr_edges.push(GraphEdge {
                    source: urn.clone(),
                    target: entity_urn,
                    relation: EdgeKind::Governs,
                    confidence: 0.9,
                    provenance_id: ctx.provenance_id.to_string(),
                });
            }
        }
    }

    ctx.cozo.insert_nodes(&adr_nodes)?;
    ctx.cozo.insert_edges(&adr_edges)?;

    ctx.counters.dependencies_nodes = adr_nodes.len();
    ctx.counters.dependencies_edges = adr_edges.len();
    ctx.adr_nodes = adr_nodes;
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 6 — Deployments (declared services)
// ---------------------------------------------------------------------------
fn phase_deployments(ctx: &mut GraphLoadContext) -> Result<()> {
    let mut service_nodes = Vec::new();
    let mut service_edges = Vec::new();
    for ds in &ctx.config.services.definitions {
        let urn = crate::platform::urn::build_urn(NodeKind::Service, &ds.name);
        service_nodes.push(GraphNode {
            id: urn.clone(),
            label: format!("Service: {}", ds.name),
            category: NodeKind::Service,
            risk_score: 0.0,
            metadata: Some(json!({
                "root": ds.root,
                "runtime_name": ds.runtime_name,
                "schema_version": "v1"
            })),
        });
        for owner in &ds.owners {
            let owner_urn = crate::platform::urn::build_urn(NodeKind::Role, owner);
            service_nodes.push(GraphNode {
                id: owner_urn.clone(),
                label: format!("Owner: {}", owner),
                category: NodeKind::Role,
                risk_score: 0.0,
                metadata: Some(json!({"schema_version": "v1"})),
            });
            service_edges.push(GraphEdge {
                source: owner_urn,
                target: urn.clone(),
                relation: EdgeKind::Owns,
                confidence: 1.0,
                provenance_id: ctx.provenance_id.to_string(),
            });
        }

        for queue in &ds.queues {
            let q_urn = crate::platform::urn::build_urn(NodeKind::ObservabilitySignal, queue);
            service_nodes.push(GraphNode {
                id: q_urn.clone(),
                label: format!("Queue: {}", queue),
                category: NodeKind::ObservabilitySignal,
                risk_score: 0.0,
                metadata: Some(json!({"kind": "queue", "name": queue, "schema_version": "v1"})),
            });
            service_edges.push(GraphEdge {
                source: urn.clone(),
                target: q_urn,
                relation: EdgeKind::DependsOn,
                confidence: 1.0,
                provenance_id: ctx.provenance_id.to_string(),
            });
        }
        for topic in &ds.topics {
            let t_urn = crate::platform::urn::build_urn(NodeKind::ObservabilitySignal, topic);
            service_nodes.push(GraphNode {
                id: t_urn.clone(),
                label: format!("Topic: {}", topic),
                category: NodeKind::ObservabilitySignal,
                risk_score: 0.0,
                metadata: Some(json!({"kind": "topic", "name": topic, "schema_version": "v1"})),
            });
            service_edges.push(GraphEdge {
                source: urn.clone(),
                target: t_urn,
                relation: EdgeKind::DependsOn,
                confidence: 1.0,
                provenance_id: ctx.provenance_id.to_string(),
            });
        }
        for rpc in &ds.rpc_endpoints {
            let r_urn = crate::platform::urn::build_urn(NodeKind::Endpoint, rpc);
            service_nodes.push(GraphNode {
                id: r_urn.clone(),
                label: format!("RPC: {}", rpc),
                category: NodeKind::Endpoint,
                risk_score: 0.0,
                metadata: Some(json!({"kind": "rpc", "name": rpc, "schema_version": "v1"})),
            });
            service_edges.push(GraphEdge {
                source: urn.clone(),
                target: r_urn,
                relation: EdgeKind::Calls,
                confidence: 1.0,
                provenance_id: ctx.provenance_id.to_string(),
            });
        }
    }
    ctx.cozo.insert_nodes(&service_nodes)?;
    ctx.cozo.insert_edges(&service_edges)?;

    ctx.counters.deployments_nodes = service_nodes.len();
    ctx.counters.deployments_edges = service_edges.len();
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 7 — Environment (data models + env declarations)
// ---------------------------------------------------------------------------
fn phase_environment(ctx: &mut GraphLoadContext) -> Result<()> {
    let conn = ctx.storage.get_connection();

    // --- Data Models ---
    let mut model_stmt = conn
        .prepare(
            "SELECT dm.model_name, dm.language, dm.model_kind, dm.fields, pf.file_path \
             FROM data_models dm \
             JOIN project_files pf ON dm.model_file_id = pf.id",
        )
        .into_diagnostic()?;
    let model_rows: Vec<(String, String, String, Option<String>, String)> = model_stmt
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
    let mut model_edges = Vec::new();
    for (name, lang, kind, fields, file_path) in &model_rows {
        let urn = crate::platform::urn::build_urn(NodeKind::DataModel, name);
        model_nodes.push(GraphNode {
            id: urn.clone(),
            label: format!("Model: {}", name),
            category: NodeKind::DataModel,
            risk_score: 0.0,
            metadata: Some(
                json!({"language": lang, "kind": kind, "fields": fields, "schema_version": "v1"}),
            ),
        });

        let file_urn = crate::platform::urn::build_urn(NodeKind::File, file_path);
        model_edges.push(GraphEdge {
            source: urn.clone(),
            target: file_urn,
            relation: EdgeKind::DependsOn,
            confidence: 1.0,
            provenance_id: ctx.provenance_id.to_string(),
        });

        for ds in &ctx.config.services.definitions {
            let root_norm = ds.root.replace('\\', "/");
            let file_norm = file_path.replace('\\', "/");
            if file_norm.starts_with(&root_norm) {
                let svc_urn = crate::platform::urn::build_urn(NodeKind::Service, &ds.name);
                model_edges.push(GraphEdge {
                    source: svc_urn,
                    target: urn.clone(),
                    relation: EdgeKind::Owns,
                    confidence: 0.9,
                    provenance_id: ctx.provenance_id.to_string(),
                });
                break;
            }
        }
    }

    // --- Env Declarations ---
    let mut env_stmt = conn
        .prepare(
            "SELECT ed.var_name, ed.source_kind, ed.is_secret, ed.owner, pf.file_path \
             FROM env_declarations ed \
             JOIN project_files pf ON ed.source_file_id = pf.id",
        )
        .into_diagnostic()?;
    let env_rows: Vec<(String, String, i32, Option<String>, String)> = env_stmt
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
    drop(env_stmt);

    let mut config_nodes = Vec::new();
    let mut config_edges = Vec::new();
    for (var_name, source_kind, is_secret, owner, file_path) in &env_rows {
        let urn = crate::platform::urn::build_urn(NodeKind::ConfigKey, var_name);
        config_nodes.push(GraphNode {
            id: urn.clone(),
            label: format!("Config: {}", var_name),
            category: NodeKind::ConfigKey,
            risk_score: 0.0,
            metadata: Some(json!({
                "source_kind": source_kind,
                "is_secret": is_secret == &1,
                "owner": owner,
                "schema_version": "v1"
            })),
        });

        let file_urn = crate::platform::urn::build_urn(NodeKind::File, file_path);
        config_edges.push(GraphEdge {
            source: urn.clone(),
            target: file_urn,
            relation: EdgeKind::DependsOn,
            confidence: 1.0,
            provenance_id: ctx.provenance_id.to_string(),
        });

        for ds in &ctx.config.services.definitions {
            let root_norm = ds.root.replace('\\', "/");
            let file_norm = file_path.replace('\\', "/");
            if file_norm.starts_with(&root_norm) {
                let svc_urn = crate::platform::urn::build_urn(NodeKind::Service, &ds.name);
                config_edges.push(GraphEdge {
                    source: svc_urn,
                    target: urn.clone(),
                    relation: EdgeKind::Configures,
                    confidence: 0.9,
                    provenance_id: ctx.provenance_id.to_string(),
                });
                break;
            }
        }
    }

    ctx.cozo.insert_nodes(&model_nodes)?;
    ctx.cozo.insert_edges(&model_edges)?;
    ctx.cozo.insert_nodes(&config_nodes)?;
    ctx.cozo.insert_edges(&config_edges)?;

    let env_edges = model_edges.len() + config_edges.len();
    ctx.counters.environment_model_nodes = model_nodes.len();
    ctx.counters.environment_config_nodes = config_nodes.len();
    ctx.counters.environment_edges = env_edges;
    ctx.config_nodes = config_nodes;
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 8 — Observability (OpenSLO)
// ---------------------------------------------------------------------------
fn phase_observability(ctx: &mut GraphLoadContext) -> Result<()> {
    let mut obs_nodes = Vec::new();
    let mut obs_edges = Vec::new();
    let obs_dir = ctx.storage.root_path().join("observability");
    if obs_dir.exists()
        && let Ok(entries) = std::fs::read_dir(obs_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yml")
                || path.extension().and_then(|e| e.to_str()) == Some("yaml")
            {
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Failed to read observability file {:?}: {}", path, e);
                        continue;
                    }
                };
                let abs_str = path.to_string_lossy().replace('\\', "/");
                let root_prefix = format!(
                    "{}/",
                    ctx.storage
                        .root_path()
                        .as_str()
                        .replace('\\', "/")
                        .trim_end_matches('/')
                );
                let source_file = abs_str
                    .strip_prefix(&root_prefix)
                    .unwrap_or(&abs_str)
                    .to_string();
                if let Ok(entities) = crate::observability::openslo::parse_openslo(&content) {
                    for mut entity in entities {
                        if let serde_json::Value::Object(ref mut m) = entity.metadata {
                            m.insert(
                                "source_file".to_string(),
                                serde_json::Value::String(source_file.clone()),
                            );
                        }
                        match entity.kind.as_str() {
                            "Service" => {
                                obs_nodes.push(GraphNode {
                                    id: entity.urn.clone(),
                                    label: format!("Service: {}", entity.name),
                                    category: NodeKind::Service,
                                    risk_score: 0.0,
                                    metadata: Some(entity.metadata.clone()),
                                });

                                if let Some(ref owner) = entity.owner {
                                    let owner_urn =
                                        crate::platform::urn::build_urn(NodeKind::Role, owner);
                                    obs_nodes.push(GraphNode {
                                        id: owner_urn.clone(),
                                        label: format!("Owner: {}", owner),
                                        category: NodeKind::Role,
                                        risk_score: 0.0,
                                        metadata: Some(json!({"schema_version": "v1"})),
                                    });

                                    obs_edges.push(GraphEdge {
                                        source: owner_urn,
                                        target: entity.urn.clone(),
                                        relation: EdgeKind::Owns,
                                        confidence: 1.0,
                                        provenance_id: ctx.provenance_id.to_string(),
                                    });
                                }
                            }
                            "SLI" => {
                                obs_nodes.push(GraphNode {
                                    id: entity.urn.clone(),
                                    label: format!("SLI: {}", entity.name),
                                    category: NodeKind::Metric,
                                    risk_score: 0.0,
                                    metadata: Some(entity.metadata.clone()),
                                });

                                if let Some(ref service_name) = entity.service_name {
                                    let svc_urn = crate::platform::urn::build_urn(
                                        NodeKind::Service,
                                        service_name,
                                    );
                                    obs_edges.push(GraphEdge {
                                        source: entity.urn.clone(),
                                        target: svc_urn,
                                        relation: EdgeKind::Monitors,
                                        confidence: 1.0,
                                        provenance_id: ctx.provenance_id.to_string(),
                                    });
                                }

                                for metric in &entity.metrics {
                                    obs_nodes.push(GraphNode {
                                        id: metric.urn.clone(),
                                        label: format!("Metric: {}", metric.name),
                                        category: NodeKind::Metric,
                                        risk_score: 0.0,
                                        metadata: Some(json!({
                                            "query": metric.query,
                                            "source": metric.source,
                                            "schema_version": "v1"
                                        })),
                                    });

                                    obs_edges.push(GraphEdge {
                                        source: entity.urn.clone(),
                                        target: metric.urn.clone(),
                                        relation: EdgeKind::DependsOn,
                                        confidence: 1.0,
                                        provenance_id: ctx.provenance_id.to_string(),
                                    });
                                }
                            }
                            "SLO" => {
                                obs_nodes.push(GraphNode {
                                    id: entity.urn.clone(),
                                    label: format!("SLO: {}", entity.name),
                                    category: NodeKind::Slo,
                                    risk_score: 0.0,
                                    metadata: Some(entity.metadata.clone()),
                                });

                                if let Some(ref service_name) = entity.service_name {
                                    let svc_urn = crate::platform::urn::build_urn(
                                        NodeKind::Service,
                                        service_name,
                                    );
                                    obs_edges.push(GraphEdge {
                                        source: entity.urn.clone(),
                                        target: svc_urn,
                                        relation: EdgeKind::Monitors,
                                        confidence: 1.0,
                                        provenance_id: ctx.provenance_id.to_string(),
                                    });
                                }

                                for metric in &entity.metrics {
                                    obs_nodes.push(GraphNode {
                                        id: metric.urn.clone(),
                                        label: format!("Metric: {}", metric.name),
                                        category: NodeKind::Metric,
                                        risk_score: 0.0,
                                        metadata: Some(json!({
                                            "query": metric.query,
                                            "source": metric.source,
                                            "schema_version": "v1"
                                        })),
                                    });

                                    obs_edges.push(GraphEdge {
                                        source: entity.urn.clone(),
                                        target: metric.urn.clone(),
                                        relation: EdgeKind::DependsOn,
                                        confidence: 1.0,
                                        provenance_id: ctx.provenance_id.to_string(),
                                    });
                                }

                                for alert_policy in &entity.alerts {
                                    let ap_urn = crate::platform::urn::build_urn(
                                        NodeKind::Alert,
                                        alert_policy,
                                    );
                                    obs_edges.push(GraphEdge {
                                        source: ap_urn,
                                        target: entity.urn.clone(),
                                        relation: EdgeKind::AlertsOn,
                                        confidence: 1.0,
                                        provenance_id: ctx.provenance_id.to_string(),
                                    });
                                }

                                if let Some(ref owner) = entity.owner {
                                    let owner_urn =
                                        crate::platform::urn::build_urn(NodeKind::Role, owner);
                                    obs_nodes.push(GraphNode {
                                        id: owner_urn.clone(),
                                        label: format!("Owner: {}", owner),
                                        category: NodeKind::Role,
                                        risk_score: 0.0,
                                        metadata: Some(json!({"schema_version": "v1"})),
                                    });

                                    obs_edges.push(GraphEdge {
                                        source: owner_urn,
                                        target: entity.urn.clone(),
                                        relation: EdgeKind::Owns,
                                        confidence: 1.0,
                                        provenance_id: ctx.provenance_id.to_string(),
                                    });
                                }
                            }
                            "DataSource" => {
                                obs_nodes.push(GraphNode {
                                    id: entity.urn.clone(),
                                    label: format!("DataSource: {}", entity.name),
                                    category: NodeKind::ObservabilitySignal,
                                    risk_score: 0.0,
                                    metadata: Some(entity.metadata.clone()),
                                });
                            }
                            "AlertPolicy" => {
                                obs_nodes.push(GraphNode {
                                    id: entity.urn.clone(),
                                    label: format!("AlertPolicy: {}", entity.name),
                                    category: NodeKind::Alert,
                                    risk_score: 0.0,
                                    metadata: Some(entity.metadata.clone()),
                                });

                                for target in &entity.alerts {
                                    let target_urn =
                                        crate::platform::urn::build_urn(NodeKind::Role, target);
                                    obs_edges.push(GraphEdge {
                                        source: target_urn,
                                        target: entity.urn.clone(),
                                        relation: EdgeKind::Owns,
                                        confidence: 1.0,
                                        provenance_id: ctx.provenance_id.to_string(),
                                    });
                                }
                            }
                            "AlertCondition" => {
                                obs_nodes.push(GraphNode {
                                    id: entity.urn.clone(),
                                    label: format!("AlertCondition: {}", entity.name),
                                    category: NodeKind::Alert,
                                    risk_score: 0.0,
                                    metadata: Some(entity.metadata.clone()),
                                });

                                for ap in &entity.alerts {
                                    let ap_urn =
                                        crate::platform::urn::build_urn(NodeKind::Alert, ap);
                                    obs_edges.push(GraphEdge {
                                        source: entity.urn.clone(),
                                        target: ap_urn,
                                        relation: EdgeKind::DependsOn,
                                        confidence: 1.0,
                                        provenance_id: ctx.provenance_id.to_string(),
                                    });
                                }
                            }
                            "AlertNotificationTarget" => {
                                obs_nodes.push(GraphNode {
                                    id: entity.urn.clone(),
                                    label: format!("NotificationTarget: {}", entity.name),
                                    category: NodeKind::Role,
                                    risk_score: 0.0,
                                    metadata: Some(entity.metadata.clone()),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    ctx.cozo.insert_nodes(&obs_nodes)?;
    ctx.cozo.insert_edges(&obs_edges)?;

    ctx.counters.observability_nodes = obs_nodes.len();
    ctx.counters.observability_edges = obs_edges.len();
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase 9 — Security (Cedar policies, orphan pruning, cross-surface links)
// ---------------------------------------------------------------------------
fn phase_security(ctx: &mut GraphLoadContext) -> Result<()> {
    let conn = ctx.storage.get_connection();

    // 9a. Collect deploy manifest file paths for cross-surface policy links
    let mut dm_stmt = conn
        .prepare("SELECT file_path FROM deploy_manifests")
        .into_diagnostic()?;
    let deploy_manifests: Vec<String> = dm_stmt
        .query_map([], |row| row.get(0))
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(dm_stmt);

    // 9b. Read Cedar Policies
    let mut policy_nodes = Vec::new();
    let mut policy_edges = Vec::new();
    let policy_dir = ctx.storage.root_path().join("policies");
    if policy_dir.exists()
        && let Ok(entries) = std::fs::read_dir(policy_dir)
    {
        let cedar_importer = crate::policy::cedar::CedarImporter::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("cedar") {
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("Failed to read policy file {:?}: {}", path, e);
                        continue;
                    }
                };
                let policies = cedar_importer.parse(&content);
                for (i, policy) in policies.iter().enumerate() {
                    let urn = format!("urn:changeguard:policy:{}:{}", path.to_string_lossy(), i);
                    policy_nodes.push(GraphNode {
                        id: urn.clone(),
                        label: format!("Policy: {} {}", policy.effect, i),
                        category: NodeKind::Policy,
                        risk_score: 0.0,
                        metadata: Some(json!({
                            "effect": policy.effect,
                            "raw": policy.raw,
                            "conditions": policy.conditions,
                            "annotations": policy.annotations,
                            "is_template": policy.is_template,
                            "template_id": policy.template_id,
                            "schema_version": "v1"
                        })),
                    });

                    if let Some(ref tid) = policy.template_id {
                        let t_urn = format!("urn:changeguard:policy:template:{}", tid);
                        policy_edges.push(GraphEdge {
                            source: urn.clone(),
                            target: t_urn,
                            relation: EdgeKind::MapsTo,
                            confidence: 1.0,
                            provenance_id: ctx.provenance_id.to_string(),
                        });
                    }

                    if let Some(ref p) = policy.principal
                        && p != "any"
                        && !p.starts_with('?')
                    {
                        let p_urn = crate::platform::urn::build_urn(NodeKind::Principal, p);
                        policy_nodes.push(GraphNode {
                            id: p_urn.clone(),
                            label: format!("Principal: {}", p),
                            category: NodeKind::Principal,
                            risk_score: 0.0,
                            metadata: Some(json!({"name": p, "schema_version": "v1"})),
                        });
                        policy_edges.push(GraphEdge {
                            source: urn.clone(),
                            target: p_urn,
                            relation: EdgeKind::Authorizes,
                            confidence: 1.0,
                            provenance_id: ctx.provenance_id.to_string(),
                        });
                    }
                    if let Some(ref a) = policy.action
                        && a != "any"
                        && !a.starts_with('?')
                    {
                        let a_urn = crate::platform::urn::build_urn(NodeKind::Action, a);
                        policy_nodes.push(GraphNode {
                            id: a_urn.clone(),
                            label: format!("Action: {}", a),
                            category: NodeKind::Action,
                            risk_score: 0.0,
                            metadata: Some(json!({"name": a, "schema_version": "v1"})),
                        });
                        policy_edges.push(GraphEdge {
                            source: urn.clone(),
                            target: a_urn,
                            relation: EdgeKind::Authorizes,
                            confidence: 1.0,
                            provenance_id: ctx.provenance_id.to_string(),
                        });
                    }
                    if let Some(ref r) = policy.resource
                        && r != "any"
                        && !r.starts_with('?')
                    {
                        let r_urn = crate::platform::urn::build_urn(NodeKind::Resource, r);
                        policy_nodes.push(GraphNode {
                            id: r_urn.clone(),
                            label: format!("Resource: {}", r),
                            category: NodeKind::Resource,
                            risk_score: 0.0,
                            metadata: Some(json!({"name": r, "schema_version": "v1"})),
                        });
                        policy_edges.push(GraphEdge {
                            source: urn.clone(),
                            target: r_urn.clone(),
                            relation: EdgeKind::Authorizes,
                            confidence: 1.0,
                            provenance_id: ctx.provenance_id.to_string(),
                        });

                        for ds in &ctx.config.services.definitions {
                            if resource_matches_service(r, &ds.name) {
                                let svc_urn =
                                    crate::platform::urn::build_urn(NodeKind::Service, &ds.name);
                                policy_edges.push(GraphEdge {
                                    source: urn.clone(),
                                    target: svc_urn,
                                    relation: EdgeKind::ProtectedBy,
                                    confidence: 0.8,
                                    provenance_id: ctx.provenance_id.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    ctx.cozo.insert_nodes(&policy_nodes)?;
    ctx.cozo.insert_edges(&policy_edges)?;

    // 9c. Cascade pruning of Cedar child nodes (principal / action / resource).
    const CHILD_CATEGORIES: [NodeKind; 3] =
        [NodeKind::Principal, NodeKind::Action, NodeKind::Resource];

    const POLICY_URN_PREFIX: &str = "urn:changeguard:policy:";
    let valid_cedar_stems: Vec<String> = {
        let mut stems: Vec<String> = policy_nodes
            .iter()
            .filter(|n| n.category == NodeKind::Policy)
            .filter_map(|n| {
                let body = n.id.strip_prefix(POLICY_URN_PREFIX)?;
                let last_colon = body.rfind(':')?;
                let path_str = &body[..last_colon];
                let path = std::path::Path::new(path_str);
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_lowercase())
            })
            .collect();
        stems.sort();
        stems.dedup();
        stems
    };

    if valid_cedar_stems.is_empty() {
        if let Err(e) = ctx
            .cozo
            .run_script("?[id] := *node{id, category: 'policy'} :rm node {id}")
        {
            tracing::warn!("Cedar policy cleanup: failed to prune all policy nodes: {e}");
        }
    } else {
        if let Ok(pol_res) = ctx
            .cozo
            .run_script("?[id] := *node{id, category: 'policy'}")
        {
            let stale_policy_ids: Vec<String> = pol_res
                .rows
                .iter()
                .filter_map(|row| {
                    let id = row.first()?.to_string();
                    let id_lower = id.to_lowercase();
                    let is_valid = valid_cedar_stems
                        .iter()
                        .any(|stem| id_lower.contains(stem.as_str()));
                    if is_valid { None } else { Some(id) }
                })
                .collect();
            if !stale_policy_ids.is_empty()
                && let Err(e) = ctx.cozo.remove_nodes_by_id(&stale_policy_ids)
            {
                tracing::warn!("Cedar policy cleanup: failed to prune stale policy nodes: {e}");
            }
        }
    }

    if valid_cedar_stems.is_empty() {
        for cat in &CHILD_CATEGORIES {
            let cat_str = cat.to_string();
            if let Err(e) = ctx.cozo.run_script(&format!(
                "?[id] := *node{{id, category: '{cat_str}'}} :rm node {{id}}"
            )) {
                tracing::warn!(
                    "Cedar child node cleanup (empty): failed to prune {cat_str} nodes: {e}"
                );
            }
        }
        tracing::info!(
            "Cedar child node cleanup: pruned all principal/action/resource orphans (no valid policies)"
        );
    } else {
        for cat in &CHILD_CATEGORIES {
            let cat_str = cat.to_string();
            let Ok(child_res) = ctx
                .cozo
                .run_script(&format!("?[id] := *node{{id, category: '{cat_str}'}}"))
            else {
                continue;
            };

            let stale_ids: Vec<String> = child_res
                .rows
                .into_iter()
                .filter_map(|row| {
                    let child_id = match row.into_iter().next() {
                        Some(cozo::DataValue::Str(s)) => s.to_string(),
                        _ => return None,
                    };
                    let escaped = child_id.replace('\'', "\\'");
                    let check = format!(
                        "?[src] := *edge{{source: src, target: '{escaped}', relation: 'authorizes'}}, \
                         *node{{id: src, category: 'policy'}}"
                    );
                    let has_live_edge = ctx.cozo
                        .run_script(&check)
                        .ok()
                        .map(|r| {
                            r.rows.iter().any(|edge_row| {
                                match edge_row.first() {
                                    Some(cozo::DataValue::Str(src_id)) => {
                                        let src_lower = src_id.to_lowercase();
                                        valid_cedar_stems
                                            .iter()
                                            .any(|stem| src_lower.contains(stem.as_str()))
                                    }
                                    _ => false,
                                }
                            })
                        })
                        .unwrap_or(false);

                    if has_live_edge {
                        None
                    } else {
                        Some(child_id)
                    }
                })
                .collect();

            let pruned = stale_ids.len();
            if let Err(e) = ctx.cozo.remove_nodes_by_id(&stale_ids) {
                tracing::warn!("Cedar child node cleanup: failed to prune {cat_str} orphans: {e}");
            } else {
                tracing::info!("Cedar child node cleanup: pruned {pruned} stale {cat_str} nodes");
            }
        }
    }

    // 9d. Cross-surface security links
    let mut cross_edges = Vec::new();

    for (method, path, _, _, _, _, _) in ctx.route_rows.iter() {
        let endpoint_id = format!("urn:changeguard:endpoint:{}:{}", method, path);
        for p_node in policy_nodes
            .iter()
            .filter(|n| n.category == NodeKind::Policy)
        {
            let has_path_ref = p_node
                .metadata
                .as_ref()
                .and_then(|m| m.get("raw"))
                .and_then(|r| r.as_str())
                .map(|raw| raw.contains(path.as_str()))
                .unwrap_or(false);
            if has_path_ref {
                cross_edges.push(GraphEdge {
                    source: p_node.id.clone(),
                    target: endpoint_id.clone(),
                    relation: EdgeKind::ProtectedBy,
                    confidence: 0.7,
                    provenance_id: ctx.provenance_id.to_string(),
                });
            }
        }
    }

    for cfg_node in ctx.config_nodes.iter() {
        if cfg_node.category == NodeKind::ConfigKey {
            let var_name = cfg_node.label.trim_start_matches("Config: ");
            for p_node in policy_nodes
                .iter()
                .filter(|n| n.category == NodeKind::Policy)
            {
                let references_var = p_node
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("conditions"))
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .any(|c| c.as_str().map(|s| s.contains(var_name)).unwrap_or(false))
                    })
                    .unwrap_or(false);
                if references_var {
                    cross_edges.push(GraphEdge {
                        source: p_node.id.clone(),
                        target: cfg_node.id.clone(),
                        relation: EdgeKind::Configures,
                        confidence: 0.75,
                        provenance_id: ctx.provenance_id.to_string(),
                    });
                }
            }
        }
    }

    for adr_node in ctx.adr_nodes.iter() {
        for p_node in policy_nodes
            .iter()
            .filter(|n| n.category == NodeKind::Policy)
        {
            let label_lc = adr_node.label.to_lowercase();
            let adr_is_security = label_lc.contains("security")
                || label_lc.contains("auth")
                || label_lc.contains("policy")
                || label_lc.contains("access");
            if adr_is_security {
                cross_edges.push(GraphEdge {
                    source: p_node.id.clone(),
                    target: adr_node.id.clone(),
                    relation: EdgeKind::Governs,
                    confidence: 0.6,
                    provenance_id: ctx.provenance_id.to_string(),
                });
            }
        }
    }

    for dm_row in deploy_manifests.iter() {
        let dm_urn = crate::platform::urn::build_urn(NodeKind::DeploySurface, dm_row);
        for p_node in policy_nodes
            .iter()
            .filter(|n| n.category == NodeKind::Policy)
        {
            let refs_manifest = p_node
                .metadata
                .as_ref()
                .and_then(|m| m.get("raw"))
                .and_then(|r| r.as_str())
                .map(|raw| raw.contains(dm_row.as_str()))
                .unwrap_or(false);
            if refs_manifest {
                cross_edges.push(GraphEdge {
                    source: p_node.id.clone(),
                    target: dm_urn.clone(),
                    relation: EdgeKind::ProtectedBy,
                    confidence: 0.7,
                    provenance_id: ctx.provenance_id.to_string(),
                });
            }
        }
    }

    ctx.cozo.insert_edges(&cross_edges)?;

    let sec_nodes = policy_nodes.len();
    let sec_edges = policy_edges.len() + cross_edges.len();
    ctx.counters.security_nodes = sec_nodes;
    ctx.counters.security_edges = sec_edges;
    ctx.counters.cross_surface_edges = cross_edges.len();
    ctx.policy_nodes = policy_nodes;
    ctx.deploy_manifests = deploy_manifests;
    Ok(())
}

pub fn run_community_louvain(cozo: &CozoStorage) -> Result<Vec<Community>> {
    let script = "
        ?[node, comm_id] := *node{id: node}, comm_id = 0
    ";
    let res = cozo.run_script(script)?;
    let mut communities = Vec::new();
    let mut nodes_by_comm: std::collections::HashMap<i64, Vec<String>> =
        std::collections::HashMap::new();

    for row in res.rows {
        if let (
            Some(cozo::DataValue::Str(node)),
            Some(cozo::DataValue::Num(cozo::Num::Int(comm))),
        ) = (row.first(), row.get(1))
        {
            nodes_by_comm
                .entry(*comm)
                .or_default()
                .push(node.to_string());
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

/// Check whether a Cedar resource name (e.g. a simple string like "MyService"
/// or URN suffix) matches a service definition name.
///
/// Uses exact case-insensitive comparison on the final segment of the
/// resource name after stripping any URN prefix, avoiding substring matches
/// that produce false positives.
fn resource_matches_service(resource: &str, service_name: &str) -> bool {
    let r = resource
        .trim()
        .split(':')
        .next_back()
        .unwrap_or(resource.trim());
    r.eq_ignore_ascii_case(service_name) || r.eq_ignore_ascii_case(&service_name.replace('-', "_"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_counters_default() {
        let counters = PhaseCounters::default();
        assert_eq!(counters.files, 0);
        assert_eq!(counters.symbols, 0);
        assert_eq!(counters.call_edges, 0);
        assert_eq!(counters.cross_surface_edges, 0);
    }

    #[test]
    fn test_resource_matches_service_exact() {
        assert!(resource_matches_service("MyService", "MyService"));
        assert!(resource_matches_service("MyService", "myService"));
        assert!(!resource_matches_service("MyService", "Other"));
    }

    #[test]
    fn test_resource_matches_service_urn_suffix() {
        assert!(resource_matches_service(
            "urn:changeguard:resource:MyService",
            "MyService"
        ));
        assert!(resource_matches_service("My_Service", "My-Service"));
    }
}
