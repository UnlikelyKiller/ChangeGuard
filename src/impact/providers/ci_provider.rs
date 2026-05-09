use crate::config::model::Config;
use crate::impact::packet::ImpactPacket;
use crate::impact::providers::{RiskImpact, RiskProvider};
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

/// Provider that assesses risk based on changes to CI/CD pipeline configuration.
pub struct CiProvider;

impl RiskProvider for CiProvider {
    fn name(&self) -> &str {
        "CI/CD Risk Provider"
    }

    fn analyze(&self, packet: &ImpactPacket, _rules: &Rules, config: &Config) -> Result<RiskImpact> {
        let mut weight = 0;
        let mut reasons = Vec::new();

        if !config.coverage.ci_self_awareness.enabled {
            return Ok(RiskImpact::default());
        }

        let mut ci_change = packet
            .ci_config_change
            .clone()
            .or_else(|| crate::index::ci_gates::is_ci_config_changed(&packet.changes));

        if let Some(ref mut ci_change) = ci_change {
            ci_change.deploy_changed = !packet.deploy_manifest_changes.is_empty();

            // Recompute source_changed: any non-CI file has symbols/imports
            let ci_like_paths: std::collections::HashSet<String> = ci_change
                .known_ci_files
                .iter()
                .chain(ci_change.unknown_ci_files.iter())
                .chain(ci_change.pre_commit_files.iter())
                .chain(ci_change.generated_ci_files.iter())
                .cloned()
                .collect();

            let source_changed = packet.changes.iter().any(|c| {
                let p = c.path.to_string_lossy().replace('\\', "/");
                !ci_like_paths.contains(&p) && (c.symbols.is_some() || c.imports.is_some())
            });
            ci_change.source_changed = source_changed;

            // Build risk reasons with deterministic ordering by file path
            let mut ci_reasons: Vec<String> = Vec::new();

            for file in &ci_change.known_ci_files {
                ci_reasons.push(format!("CI pipeline config change: {}", file));
            }

            for file in &ci_change.pre_commit_files {
                ci_reasons.push(format!(
                    "Pre-commit hooks modified — local checks may change: {}",
                    file
                ));
            }

            for file in &ci_change.unknown_ci_files {
                ci_reasons.push(format!("Unknown CI-like file changed: {}", file));
            }

            for file in &ci_change.generated_ci_files {
                ci_reasons.push(format!("Generated CI file changed: {}", file));
            }

            ci_reasons.sort();

            // Compute category weights
            let mut ci_weight = 0u32;
            if !ci_change.known_ci_files.is_empty() {
                let known_weight = if ci_change.deploy_changed || ci_change.source_changed {
                    config.coverage.ci_self_awareness.ci_plus_source_weight
                } else {
                    config.coverage.ci_self_awareness.ci_changed_weight
                };
                ci_weight += known_weight;
            }

            if !ci_change.pre_commit_files.is_empty() {
                ci_weight += 2;
            }

            if !ci_change.unknown_ci_files.is_empty() {
                ci_weight += 1;
            }

            for reason in &ci_reasons {
                reasons.push(reason.clone());
                debug!("Risk Factor: CI self-awareness ({})", reason);
            }

            weight = ci_weight;
        }

        Ok(RiskImpact { weight, reasons })
    }
}
