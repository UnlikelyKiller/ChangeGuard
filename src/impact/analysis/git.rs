use crate::config::model::Config;
use crate::impact::packet::{ImpactPacket, RiskImpact};
use crate::impact::analysis::ImpactProvider;
use crate::policy::protected_paths::ProtectedPathChecker;
use crate::policy::rules::Rules;
use miette::Result;

/// Provider that analyzes basic Git-level risk: path-based rules and change volume.
pub struct GitImpactProvider;

impl ImpactProvider for GitImpactProvider {
    fn name(&self) -> &'static str {
        "Git Impact Provider"
    }

    fn analyze(&self, packet: &ImpactPacket, rules: &Rules, config: &Config) -> Result<RiskImpact> {
        let mut total_weight = 0;
        let mut reasons = Vec::new();

        // 1. Path-based risk (from rules.toml)
        let checker = ProtectedPathChecker::new(rules)?;
        for change in &packet.changes {
            let path_str = change.path.to_string_lossy();
            if checker.is_protected(&path_str) {
                reasons.push(format!("Protected path hit: {}", path_str));
                total_weight += 70; // Original high weight for protected paths
            }
        }

        // 2. Volume-based risk
        let file_count = packet.changes.len();
        if file_count >= 10 {
            reasons.push(format!("High volume: {} files changed", file_count));
            total_weight += 20;
        } else if file_count >= 5 {
            reasons.push(format!("Moderate volume: {} files changed", file_count));
            total_weight += 10;
        }

        // 3. CI Self-Awareness Risk
        if config.coverage.ci_self_awareness.enabled {
            let mut ci_change = packet
                .ci_config_change
                .clone()
                .or_else(|| crate::index::ci_gates::is_ci_config_changed(&packet.changes));

            if let Some(ref mut ci_change) = ci_change {
                ci_change.deploy_changed = !packet.deploy_manifest_changes.is_empty();

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

                let mut ci_reasons: Vec<String> = Vec::new();
                for file in &ci_change.known_ci_files {
                    ci_reasons.push(format!("CI pipeline config change: {}", file));
                }
                for file in &ci_change.pre_commit_files {
                    ci_reasons.push(format!("Pre-commit hooks modified: {}", file));
                }
                for file in &ci_change.unknown_ci_files {
                    ci_reasons.push(format!("Unknown CI-like file changed: {}", file));
                }
                for file in &ci_change.generated_ci_files {
                    ci_reasons.push(format!("Generated CI file changed: {}", file));
                }
                ci_reasons.sort();

                let mut ci_weight = 0u32;
                if !ci_change.known_ci_files.is_empty() {
                    ci_weight += if ci_change.deploy_changed || ci_change.source_changed {
                        config.coverage.ci_self_awareness.ci_plus_source_weight
                    } else {
                        config.coverage.ci_self_awareness.ci_changed_weight
                    };
                }
                if !ci_change.pre_commit_files.is_empty() { ci_weight += 2; }
                if !ci_change.unknown_ci_files.is_empty() { ci_weight += 1; }

                total_weight += ci_weight;
                reasons.extend(ci_reasons);
            }
        }

        Ok(RiskImpact {
            weight: total_weight,
            reasons,
        })
    }
}
