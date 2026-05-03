use crate::coverage::dataflow::compute_data_flow_coupling;
use crate::coverage::sdk::detect_sdk_changes;
use crate::coverage::traces::{detect_trace_config_changes, detect_trace_env_vars};
use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{ApiRoute, DataModel, ImpactPacket};
use crate::index::call_graph::{CallEdge, CallGraph, CallKind, ResolutionStatus};
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

pub struct CoverageProvider;

impl EnrichmentProvider for CoverageProvider {
    fn name(&self) -> &'static str {
        "Engineering Coverage Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        let config = &context.config.coverage;
        if !config.enabled {
            return Ok(());
        }

        // 1. Trace Config Drift
        if config.traces.enabled {
            packet.trace_config_drift =
                detect_trace_config_changes(&packet.changes, &config.traces.config_patterns);
            packet.trace_env_vars = detect_trace_env_vars(
                &packet.env_var_deps,
                &config.traces.env_var_patterns,
                &config.traces.exclude_env_patterns,
            );
        }

        // 2. SDK Dependencies Delta
        if config.sdk.enabled {
            packet.sdk_dependencies_delta = Some(detect_sdk_changes(
                &packet.changes,
                &config.sdk.patterns,
                &context.project_root,
            ));
        }

        // 3. Data Flow Enrichment
        if config.data_flow.enabled {
            self.enrich_data_flow(context, packet)?;
        }

        Ok(())
    }
}

impl CoverageProvider {
    fn enrich_data_flow(
        &self,
        context: &EnrichmentContext,
        packet: &mut ImpactPacket,
    ) -> Result<()> {
        let conn = context.storage.get_connection();

        // 1. Load all routes
        let mut route_stmt = conn
            .prepare("SELECT method, path_pattern, handler_symbol_name, framework, route_source, mount_prefix, is_dynamic, route_confidence, evidence FROM api_routes")
            .into_diagnostic()?;
        let routes: Vec<ApiRoute> = route_stmt
            .query_map([], |row| {
                Ok(ApiRoute {
                    method: row.get(0)?,
                    path_pattern: row.get(1)?,
                    handler_symbol_name: row.get(2)?,
                    framework: row.get(3)?,
                    route_source: row.get(4)?,
                    mount_prefix: row.get(5)?,
                    is_dynamic: row.get::<_, i32>(6)? != 0,
                    route_confidence: row.get(7)?,
                    evidence: row.get(8)?,
                })
            })
            .into_diagnostic()?
            .collect::<rusqlite::Result<Vec<_>>>()
            .into_diagnostic()?;

        // 2. Load Call Graph Edges (full graph for data flow)
        let mut edge_stmt = conn
            .prepare(
                "SELECT COALESCE(ps_caller.qualified_name, ps_caller.symbol_name), \
                    pf_caller.file_path, \
                    COALESCE(ps_callee.qualified_name, ps_callee.symbol_name), \
                    pf_callee.file_path, \
                    se.call_kind \
             FROM structural_edges se \
             JOIN project_symbols ps_caller ON se.caller_symbol_id = ps_caller.id \
             JOIN project_files pf_caller ON se.caller_file_id = pf_caller.id \
             JOIN project_symbols ps_callee ON se.callee_symbol_id = ps_callee.id \
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
                    call_kind: match row.get::<_, String>(4)?.as_str() {
                        "METHOD_CALL" => CallKind::MethodCall,
                        "TRAIT_DISPATCH" => CallKind::TraitDispatch,
                        "DYNAMIC" => CallKind::Dynamic,
                        "EXTERNAL" => CallKind::External,
                        _ => CallKind::Direct,
                    },
                    resolution_status: ResolutionStatus::Resolved,
                    confidence: 1.0,
                    evidence: "".to_string(),
                })
            })
            .into_diagnostic()?
            .collect::<rusqlite::Result<Vec<_>>>()
            .into_diagnostic()?;

        let call_graph = CallGraph { edges };

        // 3. Load all Data Models
        let mut model_stmt = conn
            .prepare("SELECT model_name, model_kind, confidence, evidence FROM data_models")
            .into_diagnostic()?;
        let data_models: Vec<DataModel> = model_stmt
            .query_map([], |row| {
                Ok(DataModel {
                    model_name: row.get(0)?,
                    model_kind: row.get(1)?,
                    confidence: row.get(2)?,
                    evidence: row.get(3)?,
                })
            })
            .into_diagnostic()?
            .collect::<rusqlite::Result<Vec<_>>>()
            .into_diagnostic()?;

        // 4. Enumerate Call Chains
        let chains = call_graph.enumerate_call_chains(
            &routes,
            context.config.coverage.data_flow.chain_depth_max as usize,
        );

        // 5. Compute Coupling
        packet.data_flow_matches = compute_data_flow_coupling(
            &chains,
            &packet.changes,
            &data_models,
            0.2,
            &context.project_root,
        );

        Ok(())
    }
}
