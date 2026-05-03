use crate::coverage::deploy::detect_deploy_manifest_changes;
use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::ImpactPacket;
use miette::Result;

pub struct DeployProvider;

impl EnrichmentProvider for DeployProvider {
    fn name(&self) -> &'static str {
        "Deployment Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        let config = &context.config.coverage;
        if !config.enabled || !config.deploy.enabled {
            return Ok(());
        }

        packet.deploy_manifest_changes =
            detect_deploy_manifest_changes(&packet.changes, &config.deploy.patterns);

        Ok(())
    }
}
