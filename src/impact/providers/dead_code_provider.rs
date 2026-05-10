use crate::config::model::Config;
use crate::impact::packet::{ImpactPacket, RiskImpact};
use crate::impact::providers::RiskProvider;
use crate::policy::rules::Rules;
use miette::Result;

pub struct DeadCodeProvider;

impl RiskProvider for DeadCodeProvider {
    fn name(&self) -> &str {
        "DeadCode"
    }

    fn analyze(
        &self,
        packet: &ImpactPacket,
        _rules: &Rules,
        config: &Config,
    ) -> Result<RiskImpact> {
        let mut impact = RiskImpact {
            weight: 0,
            reasons: Vec::new(),
        };

        if !config.dead_code.enabled {
            return Ok(impact);
        }

        for finding in &packet.dead_code_findings {
            if finding.confidence >= config.dead_code.confidence_threshold {
                let reason = format!(
                    "Advisory: changed symbol '{}' in {} is likely dead code (confidence: {:.0}%)",
                    finding.symbol_name,
                    finding.file_path.display(),
                    finding.confidence * 100.0
                );
                impact.reasons.push(reason);
            }
        }

        Ok(impact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::{ConfidenceFactor, DeadCodeFinding};
    use std::path::PathBuf;

    #[test]
    fn test_provider_emits_advisory() {
        let packet = ImpactPacket {
            dead_code_findings: vec![DeadCodeFinding {
                symbol_name: "unused_fn".to_string(),
                file_path: PathBuf::from("src/lib.rs"),
                confidence: 0.92,
                factors: vec![ConfidenceFactor::NoTestCoverage],
                recommendation: "Remove".to_string(),
            }],
            ..ImpactPacket::default()
        };
        let rules = Rules::default();
        let mut config = Config::default();
        config.dead_code.enabled = true;
        config.dead_code.confidence_threshold = 0.75;

        let provider = DeadCodeProvider;
        let impact = provider.analyze(&packet, &rules, &config).unwrap();
        assert_eq!(impact.weight, 0);
        assert_eq!(impact.reasons.len(), 1);
        assert!(impact.reasons[0].contains("likely dead code"));
        assert!(impact.reasons[0].contains("unused_fn"));
    }

    #[test]
    fn test_provider_weight_is_zero() {
        let packet = ImpactPacket {
            dead_code_findings: vec![DeadCodeFinding {
                symbol_name: "unused_fn".to_string(),
                file_path: PathBuf::from("src/lib.rs"),
                confidence: 0.92,
                factors: vec![ConfidenceFactor::NoTestCoverage],
                recommendation: "Remove".to_string(),
            }],
            ..ImpactPacket::default()
        };
        let rules = Rules::default();
        let mut config = Config::default();
        config.dead_code.enabled = true;

        let provider = DeadCodeProvider;
        let impact = provider.analyze(&packet, &rules, &config).unwrap();
        assert_eq!(impact.weight, 0);
    }

    #[test]
    fn test_provider_skipped_when_disabled() {
        let packet = ImpactPacket {
            dead_code_findings: vec![DeadCodeFinding {
                symbol_name: "unused_fn".to_string(),
                file_path: PathBuf::from("src/lib.rs"),
                confidence: 0.92,
                factors: vec![ConfidenceFactor::NoTestCoverage],
                recommendation: "Remove".to_string(),
            }],
            ..ImpactPacket::default()
        };
        let rules = Rules::default();
        let config = Config::default(); // dead_code.enabled = false

        let provider = DeadCodeProvider;
        let impact = provider.analyze(&packet, &rules, &config).unwrap();
        assert!(impact.reasons.is_empty());
    }

    #[test]
    fn test_provider_respects_threshold() {
        let packet = ImpactPacket {
            dead_code_findings: vec![DeadCodeFinding {
                symbol_name: "maybe_dead".to_string(),
                file_path: PathBuf::from("src/lib.rs"),
                confidence: 0.5,
                factors: vec![ConfidenceFactor::NoTestCoverage],
                recommendation: "Review".to_string(),
            }],
            ..ImpactPacket::default()
        };
        let rules = Rules::default();
        let mut config = Config::default();
        config.dead_code.enabled = true;
        config.dead_code.confidence_threshold = 0.75;

        let provider = DeadCodeProvider;
        let impact = provider.analyze(&packet, &rules, &config).unwrap();
        assert!(impact.reasons.is_empty());
    }
}
