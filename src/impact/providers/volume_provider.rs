use crate::config::model::Config;
use crate::impact::packet::ImpactPacket;
use crate::impact::providers::{RiskImpact, RiskProvider};
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that assesses risk based on the volume of changed files and symbols.
pub struct VolumeProvider;

impl RiskProvider for VolumeProvider {
    fn name(&self) -> &str {
        "Change Volume Provider"
    }

    fn analyze(
        &self,
        packet: &ImpactPacket,
        _rules: &Rules,
        _config: &Config,
    ) -> Result<RiskImpact> {
        let mut weight = 0;
        let mut reasons = Vec::new();

        // 1. File Volume
        if packet.changes.len() > 5 {
            let file_weight = 20;
            weight += file_weight;
            reasons.push(format!(
                "High volume of changed files: {}",
                packet.changes.len()
            ));
            debug!("Risk Factor: High file volume +{}", file_weight);
        }

        // 2. Symbol Volume
        let total_symbols: usize = packet
            .changes
            .iter()
            .map(|f| f.symbols.as_ref().map(|s| s.len()).unwrap_or(0))
            .sum();

        if total_symbols > 20 {
            let symbol_weight = 20;
            weight += symbol_weight;
            reasons.push(format!("High volume of changed symbols: {}", total_symbols));
            debug!("Risk Factor: High symbol volume +{}", symbol_weight);
        }

        Ok(RiskImpact { weight, reasons })
    }
}
