use crate::config::model::Config;
use crate::impact::packet::{ImpactPacket, RiskImpact};
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that assesses risk based on symbol centrality (reachability from entry points).
pub struct CentralityProvider;

impl crate::impact::providers::RiskProvider for CentralityProvider {
    fn name(&self) -> &str {
        "Centrality Provider"
    }

    fn analyze(
        &self,
        packet: &ImpactPacket,
        _rules: &Rules,
        _config: &Config,
    ) -> Result<RiskImpact> {
        let mut total_weight = 0;
        let mut reasons = Vec::new();

        // 3e. Centrality Risk
        // Symbols reachable from >5 entry points contribute up to 15 points within
        // the Historical Hotspot category (max 30 points).
        let centrality_threshold = 5;
        let centrality_weight = 15;
        let centrality_weight_cap = 15;
        let mut centrality_total = 0;
        // Centrality risk is applied via pre-populated data on symbols.
        // See populate_centrality_risks in commands/impact.rs.
        for risk in &packet.centrality_risks {
            if centrality_total + centrality_weight <= centrality_weight_cap
                && risk.entrypoints_reachable > centrality_threshold
            {
                centrality_total += centrality_weight;
                reasons.push(format!(
                    "High centrality: {} reachable from {} entry points",
                    risk.symbol_name, risk.entrypoints_reachable
                ));
                debug!(
                    "Risk Factor: High centrality ({} reachable from {} entry points) +{}",
                    risk.symbol_name, risk.entrypoints_reachable, centrality_weight
                );
            }
        }
        total_weight += centrality_total;

        Ok(RiskImpact {
            weight: total_weight,
            reasons,
        })
    }
}
