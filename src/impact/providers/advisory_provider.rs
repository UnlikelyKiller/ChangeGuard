use crate::config::model::Config;
use crate::impact::packet::{ImpactPacket, RiskImpact};
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that adds informational advisories (e.g., missing test coverage) without adding risk weight.
pub struct AdvisoryProvider;

impl crate::impact::providers::RiskProvider for AdvisoryProvider {
    fn name(&self) -> &str {
        "Advisory Provider"
    }

    fn analyze(&self, packet: &ImpactPacket, _rules: &Rules, _config: &Config) -> Result<RiskImpact> {
        let mut reasons = Vec::new();

        // 3j. Test Coverage Advisory (informational, not risk weight)
        // For each TestCoverage entry with empty covering_tests, add an advisory.
        for coverage in &packet.test_coverage {
            if coverage.covering_tests.is_empty() {
                reasons.push(format!(
                    "No test coverage found for {} ({})",
                    coverage.changed_symbol, coverage.changed_file
                ));
                debug!(
                    "Advisory: No test coverage for {} in {}",
                    coverage.changed_symbol, coverage.changed_file
                );
            }
        }

        Ok(RiskImpact {
            weight: 0,
            reasons,
        })
    }
}
