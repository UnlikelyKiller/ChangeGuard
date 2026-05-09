use crate::config::model::Config;
use crate::impact::packet::{ImpactPacket, RiskImpact};
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that analyzes risk from infrastructure changes, including deployment manifests
/// and error handling within infrastructure-related directories.
pub struct InfraProvider;

impl crate::impact::providers::RiskProvider for InfraProvider {
    fn name(&self) -> &str {
        "Infrastructure Provider"
    }

    fn analyze(&self, packet: &ImpactPacket, _rules: &Rules, config: &Config) -> Result<RiskImpact> {
        let mut total_weight = 0;
        let mut reasons = Vec::new();

        // 3h. Infrastructure Error Handling Risk
        // Changed files in Infrastructure directories that also have error_handling_delta entries
        // contribute 25 weight per file, capped at 25 total.
        let infra_error_weight_per_file = 25;
        let infra_error_weight_cap = 25;
        let mut infra_error_total = 0;

        // Collect file paths from error_handling_delta for lookup
        let error_handling_files: std::collections::HashSet<&str> = packet
            .error_handling_delta
            .iter()
            .map(|d| d.file_path.as_str())
            .collect();

        if !error_handling_files.is_empty() {
            // Determine infrastructure directories: use topology data if available, else heuristic
            let infra_dirs: Vec<&str> = if packet.infrastructure_dirs.is_empty() {
                vec![".github/workflows", "infra", "deploy", "terraform", "k8s"]
            } else {
                packet
                    .infrastructure_dirs
                    .iter()
                    .map(|s| s.as_str())
                    .collect()
            };

            for change in &packet.changes {
                if infra_error_total + infra_error_weight_per_file > infra_error_weight_cap {
                    break;
                }
                let path_str = change.path.to_string_lossy();
                let path_str_ref = path_str.as_ref();

                // Check if this file is in an infrastructure directory
                let is_infra = infra_dirs.iter().any(|dir| {
                    path_str_ref.starts_with(dir)
                        && (path_str_ref.len() == dir.len()
                            || path_str_ref.chars().nth(dir.len()) == Some('/')
                            || path_str_ref.chars().nth(dir.len()) == Some('\\'))
                });

                // Check if this file has an error handling delta
                let has_error_handling_delta = error_handling_files.contains(path_str_ref);

                if is_infra && has_error_handling_delta {
                    infra_error_total += infra_error_weight_per_file;
                    reasons.push(format!(
                        "Error handling change in infrastructure: {}",
                        path_str_ref
                    ));
                    debug!(
                        "Risk Factor: Error handling change in infrastructure ({}) +{}",
                        path_str_ref, infra_error_weight_per_file
                    );
                }
            }
        }
        total_weight += infra_error_total;

        // 4f. Deploy Manifest Changes
        let deploy_weight_cap = config.coverage.deploy.risk_cap;
        let mut deploy_total = 0u32;
        for change in &packet.deploy_manifest_changes {
            let weight = match change.risk_tier {
                1 => 3u32,
                2 => 5u32,
                _ => 8u32,
            };
            let added = if deploy_total + weight <= deploy_weight_cap {
                weight
            } else {
                deploy_weight_cap.saturating_sub(deploy_total)
            };
            deploy_total += added;

            let mut reason = format!("Deployment manifest change: {}", change.file.display());
            if change.risk_tier > 1 {
                reason.push_str(&format!(" [tier {}]", change.risk_tier));
            }
            if !change.coupled_files.is_empty() {
                reason.push_str(&format!(" coupled: {:?}", change.coupled_files));
            }
            if !change.high_blast_resources.is_empty() {
                reason.push_str(&format!(" high-blast: {:?}", change.high_blast_resources));
            }
            reasons.push(reason);
            debug!(
                "Risk Factor: Deploy manifest changed ({:?}) tier={} +{}",
                change.file, change.risk_tier, added
            );
        }
        total_weight += deploy_total;

        Ok(RiskImpact {
            weight: total_weight,
            reasons,
        })
    }
}
