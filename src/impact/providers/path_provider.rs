use crate::config::model::Config;
use crate::impact::packet::ImpactPacket;
use crate::impact::providers::{RiskImpact, RiskProvider};
use crate::policy::protected_paths::ProtectedPathChecker;
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that checks if any changed files are in protected paths.
pub struct PathProvider;

impl RiskProvider for PathProvider {
    fn name(&self) -> &str {
        "Protected Path Provider"
    }

    fn analyze(&self, packet: &ImpactPacket, rules: &Rules, _config: &Config) -> Result<RiskImpact> {
        let mut weight = 0;
        let mut reasons = Vec::new();

        let checker = ProtectedPathChecker::new(rules)?;
        for change in &packet.changes {
            let path_str = change.path.to_string_lossy();
            if checker.is_protected(&path_str) {
                let path_weight = 70; // Automatic High
                weight += path_weight;
                reasons.push(format!("Protected path hit: {}", path_str));
                debug!("Risk Factor: Protected path hit ({}) +{}", path_str, path_weight);
            }
        }

        Ok(RiskImpact { weight, reasons })
    }
}
