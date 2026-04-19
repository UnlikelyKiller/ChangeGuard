use crate::impact::packet::{ImpactPacket, RiskLevel};
use crate::policy::protected_paths::ProtectedPathChecker;
use crate::policy::rules::Rules;
use miette::Result;
use tracing::debug;

pub fn analyze_risk(packet: &mut ImpactPacket, rules: &Rules) -> Result<()> {
    let mut total_weight = 0;
    let mut reasons = Vec::new();

    // 1. Protected Paths
    let checker = ProtectedPathChecker::new(rules)?;
    for change in &packet.changes {
        let path_str = change.path.to_string_lossy();
        if checker.is_protected(&path_str) {
            let weight = 70; // Automatic High
            total_weight += weight;
            reasons.push(format!("Protected path hit: {}", path_str));
            debug!("Risk Factor: Protected path hit ({}) +{}", path_str, weight);
        }
    }

    // 2. Change Volume
    if packet.changes.len() > 5 {
        let weight = 20;
        total_weight += weight;
        reasons.push(format!(
            "High volume of changed files: {}",
            packet.changes.len()
        ));
        debug!("Risk Factor: High file volume +{}", weight);
    }

    let total_symbols: usize = packet
        .changes
        .iter()
        .map(|f| f.symbols.as_ref().map(|s| s.len()).unwrap_or(0))
        .sum();

    if total_symbols > 20 {
        let weight = 20;
        total_weight += weight;
        reasons.push(format!("High volume of changed symbols: {}", total_symbols));
        debug!("Risk Factor: High symbol volume +{}", weight);
    }

    // 3. Symbol Visibility
    for file in &packet.changes {
        if let Some(symbols) = &file.symbols {
            for symbol in symbols {
                if symbol.is_public {
                    let weight = 30;
                    total_weight += weight;
                    reasons.push(format!(
                        "Public symbol modified: {} ({})",
                        symbol.name,
                        file.path.display()
                    ));
                    debug!(
                        "Risk Factor: Public symbol modified ({}) +{}",
                        symbol.name, weight
                    );
                }
            }
        }
    }

    // 4. Scoring
    packet.risk_level = if total_weight > 60 {
        RiskLevel::High
    } else if total_weight > 20 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };

    if reasons.is_empty() {
        reasons.push("Minimal changes detected".to_string());
    }

    packet.risk_reasons = reasons;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::{ChangedFile, FileAnalysisStatus};
    use std::path::PathBuf;

    #[test]
    fn test_analyze_risk_low() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string())
        );
    }

    #[test]
    fn test_analyze_risk_protected_path() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("Cargo.toml"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
        });

        let rules = Rules {
            protected_paths: vec!["Cargo.toml".to_string()],
            ..Rules::default()
        };

        analyze_risk(&mut packet, &rules).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::High);
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Protected path hit"))
        );
    }
}
