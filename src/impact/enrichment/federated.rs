use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::ImpactPacket;
use miette::Result;
use tracing::warn;

pub struct FederatedProvider;

impl EnrichmentProvider for FederatedProvider {
    fn name(&self) -> &'static str {
        "Federated Intelligence Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        // Refresh federated dependencies
        // Note: calling into commands::impact for now, should be refactored later.
        if let Err(e) = crate::commands::impact::refresh_federated_dependencies(
            &context.project_root,
            packet,
            context.storage,
        ) {
            warn!("Federated discovery refresh failed: {e}");
        }

        // Cross-repo impact analysis
        if let Err(e) = crate::federated::impact::check_cross_repo_impact(packet, context.storage) {
            warn!("Federated impact analysis failed: {e}");
        }

        Ok(())
    }
}
