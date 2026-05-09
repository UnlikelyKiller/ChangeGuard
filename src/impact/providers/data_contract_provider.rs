use crate::config::model::Config;
use crate::impact::packet::{ImpactPacket, RiskImpact};
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that assesses risk based on changes to data models and contracts.
pub struct DataContractProvider;

impl crate::impact::providers::RiskProvider for DataContractProvider {
    fn name(&self) -> &str {
        "Data Contract Provider"
    }

    fn analyze(
        &self,
        packet: &ImpactPacket,
        _rules: &Rules,
        _config: &Config,
    ) -> Result<RiskImpact> {
        let mut total_weight = 0;
        let mut reasons = Vec::new();

        // 3d. Data Contract Risk
        // Add 35 weight per file that contains data models (cap at 35 total for this category).
        // If any model has model_kind = "GENERATED", use reduced weight of 20 instead.
        let data_model_weight_full = 35;
        let data_model_weight_generated = 20;
        let mut data_model_total = 0;
        for file in &packet.changes {
            if !file.data_models.is_empty() && data_model_total == 0 {
                let has_generated = file.data_models.iter().any(|m| m.model_kind == "GENERATED");
                let weight = if has_generated {
                    data_model_weight_generated
                } else {
                    data_model_weight_full
                };
                data_model_total += weight;

                // Add a risk reason for each model
                for model in &file.data_models {
                    reasons.push(format!(
                        "Data model: {} ({})",
                        model.model_name, model.model_kind
                    ));
                    debug!(
                        "Risk Factor: Data model {} ({}) in {}",
                        model.model_name,
                        model.model_kind,
                        file.path.display()
                    );
                }
            }
        }
        total_weight += data_model_total;

        Ok(RiskImpact {
            weight: total_weight,
            reasons,
        })
    }
}
