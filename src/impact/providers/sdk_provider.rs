use crate::config::model::Config;
use crate::impact::packet::{ImpactPacket, RiskImpact};
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that assesses risk based on changes to SDK dependencies.
pub struct SdkProvider;

impl crate::impact::providers::RiskProvider for SdkProvider {
    fn name(&self) -> &str {
        "SDK Provider"
    }

    fn analyze(&self, packet: &ImpactPacket, _rules: &Rules, config: &Config) -> Result<RiskImpact> {
        let mut total_weight = 0;
        let mut reasons = Vec::new();

        // 4c. SDK Dependency Changes
        if let Some(ref delta) = packet.sdk_dependencies_delta {
            let sdk_new_weight = config.coverage.sdk.risk_weight_new;
            let sdk_new_cap = config.coverage.sdk.risk_cap;
            let mut sdk_new_total = 0;
            for sdk in &delta.added {
                if sdk_new_total + sdk_new_weight <= sdk_new_cap {
                    sdk_new_total += sdk_new_weight;
                    reasons.push(format!("New SDK dependency: {}", sdk.sdk_name));
                    debug!(
                        "Risk Factor: New SDK ({}) +{}",
                        sdk.sdk_name, sdk_new_weight
                    );
                }
            }
            total_weight += sdk_new_total;

            let sdk_mod_weight = config.coverage.sdk.risk_weight_modified;
            let sdk_mod_cap = config.coverage.sdk.risk_cap;
            let mut sdk_mod_total = 0;
            for sdk in &delta.modified {
                if sdk_mod_total + sdk_mod_weight <= sdk_mod_cap {
                    sdk_mod_total += sdk_mod_weight;
                    reasons.push(format!("Modified SDK dependency: {}", sdk.sdk_name));
                    debug!(
                        "Risk Factor: Modified SDK ({}) +{}",
                        sdk.sdk_name, sdk_mod_weight
                    );
                }
            }
            total_weight += sdk_mod_total;
        }

        Ok(RiskImpact {
            weight: total_weight,
            reasons,
        })
    }
}
