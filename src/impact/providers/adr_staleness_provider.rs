use crate::config::model::Config;
use crate::impact::packet::ImpactPacket;
use crate::impact::providers::{RiskImpact, RiskProvider};
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that checks for stale Architectural Decision Records (ADRs).
pub struct ADRStalenessProvider;

impl RiskProvider for ADRStalenessProvider {
    fn name(&self) -> &str {
        "ADR Staleness Provider"
    }

    fn analyze(
        &self,
        packet: &ImpactPacket,
        _rules: &Rules,
        config: &Config,
    ) -> Result<RiskImpact> {
        let weight = 0;
        let mut reasons = Vec::new();

        if config.coverage.adr_staleness.enabled {
            let threshold = config.coverage.adr_staleness.threshold_days;
            for decision in &packet.relevant_decisions {
                if let Some(days) = decision.staleness_days
                    && days > threshold
                {
                    reasons.push(format!(
                        "Stale architectural context: {} ({} days old)",
                        decision.file_path.display(),
                        days
                    ));
                    debug!(
                        "Advisory: Stale ADR ({}) {} days",
                        decision.file_path.display(),
                        days
                    );
                }
            }
        }

        Ok(RiskImpact { weight, reasons })
    }
}
