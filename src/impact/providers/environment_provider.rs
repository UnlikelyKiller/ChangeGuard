use crate::config::model::Config;
use crate::impact::packet::ImpactPacket;
use crate::impact::providers::{RiskImpact, RiskProvider};
use crate::policy::rules::Rules;
use miette::Result;
use std::sync::LazyLock;

/// Common env vars that are too ubiquitous to be meaningful risk indicators.
static COMMON_ENV_VARS: LazyLock<[&str; 17]> = LazyLock::new(|| {
    [
        "PATH",
        "HOME",
        "USER",
        "LANG",
        "SHELL",
        "TERM",
        "PWD",
        "EDITOR",
        "VISUAL",
        "HOSTNAME",
        "TMPDIR",
        "TEMP",
        "TMP",
        "SYSTEMROOT",
        "COMSPEC",
        "PROCESSOR_ARCHITECTURE",
        "OS",
    ]
});

/// Framework convention config keys that receive reduced weight because they
/// are standard boilerplate rather than meaningful runtime dependencies.
static FRAMEWORK_CONVENTION_CONFIG_KEYS: LazyLock<[&str; 8]> = LazyLock::new(|| {
    [
        "server.port",
        "server.host",
        "logging.level",
        "logging.level.*",
        "log.level",
        "debug",
        "env",
        "NODE_ENV",
    ]
});

/// Provider that assesses risk based on environment variable dependencies and runtime usage changes.
pub struct EnvironmentProvider;

impl RiskProvider for EnvironmentProvider {
    fn name(&self) -> &str {
        "Environment & Runtime Provider"
    }

    fn analyze(
        &self,
        packet: &ImpactPacket,
        _rules: &Rules,
        _config: &Config,
    ) -> Result<RiskImpact> {
        let mut weight = 0;
        let mut reasons = Vec::new();

        let runtime_config_cap = 25;
        let mut runtime_config_total = 0;

        // 1. New environment variable dependencies
        for dep in &packet.env_var_deps {
            if !COMMON_ENV_VARS.contains(&dep.var_name.as_str()) {
                reasons.push(format!(
                    "New environment variable dependency: {}",
                    dep.var_name
                ));
                if runtime_config_total + 20 <= runtime_config_cap {
                    runtime_config_total += 20;
                } else if runtime_config_total < runtime_config_cap {
                    runtime_config_total = runtime_config_cap;
                }
            }
        }

        // 2. Env var reference changes and Config key reference changes
        for delta in &packet.runtime_usage_delta {
            // Env var changes
            if delta.env_vars_current_count != delta.env_vars_previous_count {
                reasons.push(format!(
                    "Environment variable references changed in {}",
                    delta.file_path
                ));
                if runtime_config_total + 10 <= runtime_config_cap {
                    runtime_config_total += 10;
                } else if runtime_config_total < runtime_config_cap {
                    runtime_config_total = runtime_config_cap;
                }
            }

            // Config key changes
            if delta.config_keys_current_count != delta.config_keys_previous_count {
                let mut config_weight = 10;

                if let Some(usage) = packet
                    .changes
                    .iter()
                    .find(|c| c.path.to_string_lossy() == delta.file_path)
                    .and_then(|c| c.runtime_usage.as_ref())
                    .filter(|u| !u.config_keys.is_empty())
                {
                    let has_only_framework = usage
                        .config_keys
                        .iter()
                        .all(|k| FRAMEWORK_CONVENTION_CONFIG_KEYS.contains(&k.as_str()));
                    if has_only_framework {
                        config_weight = 5;
                    }
                }

                reasons.push(format!(
                    "Configuration key references changed in {}",
                    delta.file_path
                ));

                if runtime_config_total + config_weight <= runtime_config_cap {
                    runtime_config_total += config_weight;
                } else if runtime_config_total < runtime_config_cap {
                    runtime_config_total = runtime_config_cap;
                }
            }
        }

        weight += runtime_config_total;

        Ok(RiskImpact { weight, reasons })
    }
}
