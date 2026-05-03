use crate::coverage::services::compute_cross_service_edges;
use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{ImpactPacket, Service, ServiceMapDelta};
use crate::index::call_graph::{CallEdge, CallGraph, CallKind, ResolutionStatus};
use miette::{IntoDiagnostic, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::info;

pub struct ServiceProvider;

impl EnrichmentProvider for ServiceProvider {
    fn name(&self) -> &'static str {
        "Service Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        if !context.storage.table_exists_and_has_data("project_files")? {
            info!("Skipping service enrichment: project_files table is empty or missing.");
            return Ok(());
        }

        let conn = context.storage.get_connection();

        // 1. Detect affected services by checking service_name of changed files
        let mut affected_services_set = HashSet::new();
        for change in &packet.changes {
            let path_to_check = change.old_path.as_ref().unwrap_or(&change.path);
            let path_str = path_to_check.to_string_lossy().to_string();
            let service_name: Option<String> = conn
                .query_row(
                    "SELECT service_name FROM project_files WHERE file_path = ?1",
                    [path_str],
                    |row| row.get(0),
                )
                .unwrap_or(None);

            if let Some(name) = service_name {
                affected_services_set.insert(name);
            }
        }

        if affected_services_set.is_empty() {
            info!("No services affected by these changes.");
            return Ok(());
        }

        // 2. Load All Known Services
        let mut service_stmt = conn
            .prepare(
                "SELECT DISTINCT service_name FROM project_files WHERE service_name IS NOT NULL",
            )
            .into_diagnostic()?;
        let service_names: Vec<String> = service_stmt
            .query_map([], |row| row.get(0))
            .into_diagnostic()?
            .collect::<rusqlite::Result<Vec<_>>>()
            .into_diagnostic()?;

        let total_services = service_names.len();
        let mut services = Vec::new();

        for name in &service_names {
            // Load routes
            let mut route_stmt = conn
                .prepare(
                    "SELECT ar.path_pattern 
                 FROM api_routes ar 
                 JOIN project_files pf ON ar.handler_file_id = pf.id 
                 WHERE pf.service_name = ?1",
                )
                .into_diagnostic()?;
            let routes = route_stmt
                .query_map([name], |row| row.get(0))
                .into_diagnostic()?
                .collect::<rusqlite::Result<Vec<String>>>()
                .into_diagnostic()?;

            // Load data models
            let mut model_stmt = conn
                .prepare(
                    "SELECT dm.model_name 
                 FROM data_models dm 
                 JOIN project_files pf ON dm.model_file_id = pf.id 
                 WHERE pf.service_name = ?1",
                )
                .into_diagnostic()?;
            let data_models = model_stmt
                .query_map([name], |row| row.get(0))
                .into_diagnostic()?
                .collect::<rusqlite::Result<Vec<String>>>()
                .into_diagnostic()?;

            // Load directory (take first file's parent as heuristic)
            let directory: String = conn
                .query_row(
                    "SELECT file_path FROM project_files WHERE service_name = ?1 LIMIT 1",
                    [name],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| ".".to_string());
            let directory = Path::new(&directory)
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf();

            services.push(Service {
                name: name.clone(),
                directory,
                routes,
                data_models,
            });
        }

        // 3. Load Call Graph Edges
        let mut edge_stmt = conn
            .prepare(
                "SELECT COALESCE(ps_caller.qualified_name, ps_caller.symbol_name), \
                    pf_caller.file_path, \
                    COALESCE(ps_callee.qualified_name, ps_callee.symbol_name), \
                    pf_callee.file_path
             FROM structural_edges se
             JOIN project_symbols ps_caller ON se.caller_symbol_id = ps_caller.id
             JOIN project_files pf_caller ON se.caller_file_id = pf_caller.id
             JOIN project_symbols ps_callee ON se.callee_symbol_id = ps_callee.id
             JOIN project_files pf_callee ON se.callee_file_id = pf_callee.id",
            )
            .into_diagnostic()?;

        let edges = edge_stmt
            .query_map([], |row| {
                Ok(CallEdge {
                    caller_name: row.get(0)?,
                    caller_file: PathBuf::from(row.get::<_, String>(1)?),
                    callee_name: row.get(2)?,
                    callee_file: Some(PathBuf::from(row.get::<_, String>(3)?)),
                    call_kind: CallKind::Direct,
                    resolution_status: ResolutionStatus::Resolved,
                    confidence: 1.0,
                    evidence: "".to_string(),
                })
            })
            .into_diagnostic()?
            .collect::<rusqlite::Result<Vec<_>>>()
            .into_diagnostic()?;

        let call_graph = CallGraph { edges };
        let cross_service_edges = compute_cross_service_edges(&services, &call_graph);

        let mut affected_services: Vec<String> = affected_services_set.into_iter().collect();
        affected_services.sort();

        packet.service_map_delta = Some(ServiceMapDelta {
            services,
            affected_services,
            cross_service_edges,
            total_services,
        });

        Ok(())
    }
}
