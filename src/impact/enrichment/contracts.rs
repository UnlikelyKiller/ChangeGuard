use crate::contracts::matcher::match_changed_files;
use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::ImpactPacket;
use miette::Result;
use tracing::warn;

pub struct ContractProvider;

impl EnrichmentProvider for ContractProvider {
    fn name(&self) -> &'static str {
        "Contract Matching Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        let config = context.config;

        // Skip if no contract spec paths configured
        if config.contracts.spec_paths.is_empty() {
            return Ok(());
        }

        // Skip if no embedding model configured
        if config.local_model.embedding_model.is_empty() {
            return Ok(());
        }

        let conn = context.storage.get_connection();

        // Collect changed file paths
        let file_paths: Vec<String> = packet
            .changes
            .iter()
            .map(|c| c.path.to_string_lossy().to_string())
            .collect();

        if file_paths.is_empty() {
            return Ok(());
        }

        match match_changed_files(&config.contracts, conn, &config.local_model, &file_paths) {
            Ok(matches) => {
                packet.affected_contracts = matches;
            }
            Err(e) => {
                warn!("Contract matching failed: {e}");
                context.add_warning(format!("Contract enrichment failed: {e}"));
            }
        }

        Ok(())
    }
}
