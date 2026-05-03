use crate::git::repo::open_repo;
use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::hotspots::calculate_hotspots;
use crate::impact::packet::ImpactPacket;
use crate::impact::temporal::GixHistoryProvider;
use miette::Result;
use tracing::{info, warn};

pub struct HotspotProvider;

impl EnrichmentProvider for HotspotProvider {
    fn name(&self) -> &'static str {
        "Hotspot Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        info!("Calculating hotspots...");

        let repo = open_repo(&context.project_root).map_err(|e| {
            miette::miette!("Failed to open repo for hotspot analysis: {}", e)
        })?;

        let history_provider = GixHistoryProvider::new(&repo);

        match calculate_hotspots(
            context.storage,
            &history_provider,
            context.config.hotspots.max_commits,
            context.config.hotspots.limit,
            context.config.temporal.all_parents,
            None,
            None,
        ) {
            Ok(hotspots) => {
                packet.hotspots = hotspots;
            }
            Err(e) => {
                warn!("Hotspot analysis failed: {e}");
                context.add_warning(format!("Hotspot analysis failed: {e}"));
            }
        }

        Ok(())
    }
}
