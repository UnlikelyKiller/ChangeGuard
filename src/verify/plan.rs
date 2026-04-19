use crate::impact::packet::ImpactPacket;
use crate::policy::rules::Rules;
use crate::verify::timeouts::DEFAULT_AUTO_TIMEOUT_SECS;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationStep {
    pub command: String,
    pub timeout_secs: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationPlan {
    pub steps: Vec<VerificationStep>,
}

const DEFAULT_COMMAND: &str = "cargo test -j 1 -- --test-threads=1";

pub fn build_plan(packet: &ImpactPacket, rules: &Rules) -> VerificationPlan {
    let mut commands: Vec<String> = Vec::new();

    // Merge global required_verifications
    for cmd in &rules.global.required_verifications {
        commands.push(cmd.clone());
    }

    // Merge path-specific required_verifications from matching PathRule entries
    for override_rule in &rules.overrides {
        let glob = match globset::Glob::new(&override_rule.pattern) {
            Ok(g) => g,
            Err(_) => continue,
        };
        let compiled = match globset::GlobSet::builder().add(glob).build() {
            Ok(s) => s,
            Err(_) => continue,
        };

        let matches_any = packet.changes.iter().any(|f| compiled.is_match(&f.path));
        if matches_any {
            for cmd in &override_rule.required_verifications {
                commands.push(cmd.clone());
            }
        }
    }

    // Deduplicate by exact command string
    commands.sort_unstable();
    commands.dedup();

    // Build steps
    let steps: Vec<VerificationStep> = if commands.is_empty() {
        vec![VerificationStep {
            command: DEFAULT_COMMAND.to_string(),
            timeout_secs: DEFAULT_AUTO_TIMEOUT_SECS,
            description: "Default: run project tests".to_string(),
        }]
    } else {
        commands
            .into_iter()
            .map(|cmd| VerificationStep {
                command: cmd.clone(),
                timeout_secs: DEFAULT_AUTO_TIMEOUT_SECS,
                description: format!("From rules: {}", cmd),
            })
            .collect()
    };

    VerificationPlan { steps }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::{ChangedFile, FileAnalysisStatus, ImpactPacket};
    use crate::policy::mode::Mode;
    use crate::policy::rules::{GlobalRules, PathRule, Rules};
    use std::path::PathBuf;

    fn empty_packet() -> ImpactPacket {
        ImpactPacket {
            changes: vec![ChangedFile {
                path: PathBuf::from("src/main.rs"),
                status: "Modified".to_string(),
                is_staged: false,
                symbols: None,
                imports: None,
                runtime_usage: None,
                analysis_status: FileAnalysisStatus::default(),
                analysis_warnings: Vec::new(),
            }],
            ..ImpactPacket::default()
        }
    }

    #[test]
    fn test_build_plan_default_when_no_rules() {
        let packet = empty_packet();
        let rules = Rules::default();
        let plan = build_plan(&packet, &rules);

        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].command, DEFAULT_COMMAND);
    }

    #[test]
    fn test_build_plan_with_global_verifications() {
        let packet = empty_packet();
        let rules = Rules {
            global: GlobalRules {
                mode: Mode::Analyze,
                required_verifications: vec!["cargo test".to_string(), "cargo clippy".to_string()],
            },
            overrides: Vec::new(),
            protected_paths: Vec::new(),
        };

        let plan = build_plan(&packet, &rules);

        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].command, "cargo clippy");
        assert_eq!(plan.steps[1].command, "cargo test");
    }

    #[test]
    fn test_build_plan_deduplicates() {
        let packet = empty_packet();
        let rules = Rules {
            global: GlobalRules {
                mode: Mode::Analyze,
                required_verifications: vec!["cargo test".to_string()],
            },
            overrides: vec![PathRule {
                pattern: "*.rs".to_string(),
                mode: None,
                required_verifications: vec!["cargo test".to_string()],
            }],
            protected_paths: Vec::new(),
        };

        let plan = build_plan(&packet, &rules);

        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].command, "cargo test");
    }

    #[test]
    fn test_build_plan_path_rule_matching() {
        let packet = empty_packet(); // src/main.rs matches *.rs
        let rules = Rules {
            global: GlobalRules {
                mode: Mode::Analyze,
                required_verifications: vec!["cargo test".to_string()],
            },
            overrides: vec![PathRule {
                pattern: "*.rs".to_string(),
                mode: None,
                required_verifications: vec!["cargo clippy".to_string()],
            }],
            protected_paths: Vec::new(),
        };

        let plan = build_plan(&packet, &rules);

        assert_eq!(plan.steps.len(), 2);
        assert!(plan.steps.iter().any(|s| s.command == "cargo clippy"));
        assert!(plan.steps.iter().any(|s| s.command == "cargo test"));
    }

    #[test]
    fn test_build_plan_path_rule_no_match() {
        let packet = empty_packet(); // src/main.rs
        let rules = Rules {
            global: GlobalRules {
                mode: Mode::Analyze,
                required_verifications: vec![],
            },
            overrides: vec![PathRule {
                pattern: "*.py".to_string(),
                mode: None,
                required_verifications: vec!["pytest".to_string()],
            }],
            protected_paths: Vec::new(),
        };

        let plan = build_plan(&packet, &rules);

        // No match, falls back to default
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].command, DEFAULT_COMMAND);
    }

    #[test]
    fn test_build_plan_deterministic() {
        let packet = empty_packet();
        let rules = Rules {
            global: GlobalRules {
                mode: Mode::Analyze,
                required_verifications: vec!["z_cmd".to_string(), "a_cmd".to_string()],
            },
            overrides: Vec::new(),
            protected_paths: Vec::new(),
        };

        let plan1 = build_plan(&packet, &rules);
        let plan2 = build_plan(&packet, &rules);

        assert_eq!(plan1, plan2);
        // Sorted alphabetically
        assert_eq!(plan1.steps[0].command, "a_cmd");
        assert_eq!(plan1.steps[1].command, "z_cmd");
    }

    #[test]
    fn test_build_plan_empty_changes_no_path_match() {
        let packet = ImpactPacket {
            changes: vec![],
            ..ImpactPacket::default()
        };

        let rules = Rules {
            global: GlobalRules {
                mode: Mode::Analyze,
                required_verifications: vec!["cargo test".to_string()],
            },
            overrides: vec![PathRule {
                pattern: "*.rs".to_string(),
                mode: None,
                required_verifications: vec!["cargo clippy".to_string()],
            }],
            protected_paths: Vec::new(),
        };

        let plan = build_plan(&packet, &rules);

        // Global is included, path rule doesn't match empty changes
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].command, "cargo test");
    }
}
