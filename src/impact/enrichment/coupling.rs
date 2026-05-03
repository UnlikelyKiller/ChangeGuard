use crate::git::repo::open_repo;
use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{ImpactPacket, StructuralCoupling};
use crate::impact::temporal::{GixHistoryProvider, TemporalEngine};
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;
use tracing::{info, warn};

pub struct CouplingProvider;

impl EnrichmentProvider for CouplingProvider {
    fn name(&self) -> &'static str {
        "Coupling Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        // 1. Structural Couplings (from DB)
        self.enrich_structural(context, packet)?;

        // 2. Temporal Couplings (from Git history)
        self.enrich_temporal(context, packet)?;

        Ok(())
    }
}

impl CouplingProvider {
    fn enrich_structural(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        if !context.storage.table_exists_and_has_data("structural_edges")? {
            info!("Skipping structural coupling enrichment: structural_edges table is empty or missing.");
            return Ok(());
        }

        let conn = context.storage.get_connection();
        
        // Collect changed symbol names
        let changed_symbols: Vec<String> = packet
            .changes
            .iter()
            .filter_map(|f| f.symbols.as_ref())
            .flat_map(|symbols| symbols.iter().map(|s| s.name.clone()))
            .collect();

        if changed_symbols.is_empty() {
            return Ok(());
        }

        for callee_name in &changed_symbols {
            let mut stmt = conn
                .prepare(
                    "SELECT ps_caller.symbol_name, pf_caller.file_path
                     FROM structural_edges se
                     JOIN project_symbols ps_caller ON se.caller_symbol_id = ps_caller.id
                     JOIN project_files pf_caller ON se.caller_file_id = pf_caller.id
                     JOIN project_symbols ps_callee ON se.callee_symbol_id = ps_callee.id
                     WHERE ps_callee.symbol_name = ?1
                     AND se.callee_symbol_id IS NOT NULL",
                )
                .into_diagnostic()?;

            let edges = stmt
                .query_map([callee_name], |row| {
                    Ok(StructuralCoupling {
                        caller_symbol_name: row.get(0)?,
                        callee_symbol_name: callee_name.clone(),
                        caller_file_path: PathBuf::from(row.get::<_, String>(1)?),
                    })
                })
                .into_diagnostic()?;

            for edge in edges {
                packet.structural_couplings.push(edge.into_diagnostic()?);
            }
        }

        // Deduplicate structural couplings
        packet.structural_couplings.sort_unstable();
        packet.structural_couplings.dedup();

        Ok(())
    }

    fn enrich_temporal(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        info!("Running temporal coupling analysis...");
        
        let repo = open_repo(&context.project_root).map_err(|e| {
            miette::miette!("Failed to open repo for temporal analysis: {}", e)
        })?;

        let history_provider = GixHistoryProvider::new(&repo);
        let temporal_engine = TemporalEngine::new(
            history_provider,
            context.config.temporal.clone(),
        );

        match temporal_engine.calculate_couplings() {
            Ok(couplings) => {
                packet.temporal_couplings = couplings;
            }
            Err(e) => {
                warn!("Temporal analysis failed: {e}");
                context.add_warning(format!("Temporal analysis failed: {e}"));
            }
        }

        Ok(())
    }
}
