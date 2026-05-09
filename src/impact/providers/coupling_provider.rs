use crate::config::model::Config;
use crate::impact::packet::{ImpactPacket, RiskImpact};
use crate::impact::providers::RiskProvider;
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that analyzes structural and data-flow coupling between components and services.
pub struct CouplingProvider;

impl RiskProvider for CouplingProvider {
    fn name(&self) -> &str {
        "Coupling Provider"
    }

    fn analyze(
        &self,
        packet: &ImpactPacket,
        _rules: &Rules,
        config: &Config,
    ) -> Result<RiskImpact> {
        let mut total_weight = 0;
        let mut reasons = Vec::new();

        // 3b. Structural Coupling Risk
        // Max 30 total weight for this category (cap at 2 callers contributing 15 each).
        let structural_weight_cap = 30;
        let weight_per_caller = 15;
        let mut structural_weight = 0;
        for (callers_counted, coupling) in packet.structural_couplings.iter().enumerate() {
            if callers_counted >= 2 {
                break;
            }
            if structural_weight + weight_per_caller > structural_weight_cap {
                // Cap at the max
                let remaining = structural_weight_cap - structural_weight;
                if remaining > 0 {
                    structural_weight += remaining;
                }
                break;
            }
            structural_weight += weight_per_caller;
            reasons.push(format!(
                "Structurally coupled: {} calls {}",
                coupling.caller_symbol_name, coupling.callee_symbol_name
            ));
            debug!(
                "Risk Factor: Structurally coupled ({} calls {}) +{}",
                coupling.caller_symbol_name, coupling.callee_symbol_name, weight_per_caller
            );
        }
        total_weight += structural_weight;

        // 4d. Cross-Service Impact (Service-Map)
        if let Some(ref delta) = packet.service_map_delta {
            let count = delta.affected_services.len();
            let threshold = config.coverage.services.cross_service_elevation_threshold as usize;
            let svc_weight = if count < threshold {
                0
            } else if count >= 5 {
                config.coverage.services.risk_weight_5plus
            } else if count >= 3 {
                config.coverage.services.risk_weight_3to4
            } else if count == 2 {
                config.coverage.services.risk_weight_2svcs
            } else {
                0
            };
            if svc_weight > 0 {
                total_weight += svc_weight;
                reasons.push(format!("Cross-service change affecting {} services", count));
                debug!(
                    "Risk Factor: Cross-service impact ({} svcs) +{}",
                    count, svc_weight
                );
            }
        }

        // 4e. Data-Flow Coupling
        let data_flow_weight_per_match = config.coverage.data_flow.risk_weight_per_match;
        let data_flow_weight_cap = config.coverage.data_flow.risk_cap;
        let mut data_flow_total = 0;
        for m in &packet.data_flow_matches {
            if data_flow_total + data_flow_weight_per_match <= data_flow_weight_cap {
                data_flow_total += data_flow_weight_per_match;
                reasons.push(format!(
                    "Data-flow coupling: chain {} affected ({:.0}% change)",
                    m.chain_label,
                    m.change_pct * 100.0
                ));
                debug!(
                    "Risk Factor: Data-flow match ({}) +{}",
                    m.chain_label, data_flow_weight_per_match
                );
            }
        }
        total_weight += data_flow_total;

        Ok(RiskImpact {
            weight: total_weight,
            reasons,
        })
    }
}
