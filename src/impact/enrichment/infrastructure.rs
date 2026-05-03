use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::ImpactPacket;
use miette::{IntoDiagnostic, Result};
use tracing::info;

pub struct InfrastructureProvider;

impl EnrichmentProvider for InfrastructureProvider {
    fn name(&self) -> &'static str {
        "Infrastructure Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        if !context.storage.table_exists_and_has_data("project_topology")? {
            info!("Skipping infrastructure enrichment: project_topology table is empty or missing.");
            return Ok(());
        }

        let conn = context.storage.get_connection();
        
        let mut stmt = conn
            .prepare("SELECT directory_path FROM project_topology WHERE role = 'Infrastructure'")
            .into_diagnostic()?;

        let dirs = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .into_diagnostic()?;

        for dir in dirs {
            packet.infrastructure_dirs.push(dir.into_diagnostic()?);
        }

        Ok(())
    }
}
