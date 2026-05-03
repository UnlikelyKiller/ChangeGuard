use crate::impact::analysis::analyze_risk;
use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::ImpactPacket;
use crate::policy::load::load_rules;
use crate::state::layout::Layout;
use miette::Result;
use tracing::{info, warn};

pub struct RiskProvider;

impl EnrichmentProvider for RiskProvider {
    fn name(&self) -> &'static str {
        "Risk Analysis Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        info!("Performing risk analysis...");

        let layout = Layout::new(context.project_root.to_string_lossy().as_ref());
        let rules = match load_rules(&layout) {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to load rules: {e}");
                context.add_warning(format!("Risk analysis skipped: could not load rules: {e}"));
                return Ok(());
            }
        };

        analyze_risk(packet, &rules, context.config)?;

        Ok(())
    }
}
