use crate::impact::enrichment::{EnrichmentContext, EnrichmentProvider};
use crate::impact::packet::{ImpactPacket, RuntimeUsageDelta};
use crate::index::runtime_usage::extract_runtime_usage;
use miette::Result;
use std::process::Command;

pub struct RuntimeUsageProvider;

impl EnrichmentProvider for RuntimeUsageProvider {
    fn name(&self) -> &'static str {
        "Runtime Usage Enrichment Provider"
    }

    fn enrich(&self, context: &EnrichmentContext, packet: &mut ImpactPacket) -> Result<()> {
        let mut deltas = Vec::new();

        for change in &packet.changes {
            let current_env_vars = change
                .runtime_usage
                .as_ref()
                .map(|u| u.env_vars.len())
                .unwrap_or(0);
            let current_config_keys = change
                .runtime_usage
                .as_ref()
                .map(|u| u.config_keys.len())
                .unwrap_or(0);

            let path_str = change.path.to_string_lossy().replace('\\', "/");

            let output = Command::new("git")
                .args(["show", &format!("HEAD:{}", path_str)])
                .current_dir(&context.project_root)
                .output();

            let mut previous_env_vars = 0;
            let mut previous_config_keys = 0;

            if let Some(output) = output.ok().filter(|o| o.status.success()) {
                let prev_content = String::from_utf8_lossy(&output.stdout);
                if let Some(prev_usage) = extract_runtime_usage(&change.path, &prev_content) {
                    previous_env_vars = prev_usage.env_vars.len();
                    previous_config_keys = prev_usage.config_keys.len();
                }
            }

            if current_env_vars != previous_env_vars || current_config_keys != previous_config_keys
            {
                deltas.push(RuntimeUsageDelta {
                    file_path: change.path.to_string_lossy().to_string(),
                    env_vars_previous_count: previous_env_vars,
                    env_vars_current_count: current_env_vars,
                    config_keys_previous_count: previous_config_keys,
                    config_keys_current_count: current_config_keys,
                });
            }
        }

        packet.runtime_usage_delta = deltas;
        Ok(())
    }
}
