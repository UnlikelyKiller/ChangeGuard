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

    // 3. Symbol Visibility & Entrypoint Risk
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

                // Entrypoint-based risk (API Surface category, max 35 points)
                if let Some(ref kind) = symbol.entrypoint_kind {
                    match kind.as_str() {
                        "ENTRYPOINT" => {
                            let weight = 35;
                            total_weight += weight;
                            reasons.push(format!(
                                "Entry point changed: {} ({})",
                                symbol.name,
                                file.path.display()
                            ));
                            debug!(
                                "Risk Factor: Entry point changed ({}) +{}",
                                symbol.name, weight
                            );
                        }
                        "HANDLER" => {
                            let weight = 30;
                            total_weight += weight;
                            reasons.push(format!(
                                "Handler changed: {} ({})",
                                symbol.name,
                                file.path.display()
                            ));
                            debug!(
                                "Risk Factor: Handler changed ({}) +{}",
                                symbol.name, weight
                            );
                        }
                        "PUBLIC_API" => {
                            let weight = 20;
                            total_weight += weight;
                            reasons.push(format!(
                                "Public API changed: {} ({})",
                                symbol.name,
                                file.path.display()
                            ));
                            debug!(
                                "Risk Factor: Public API changed ({}) +{}",
                                symbol.name, weight
                            );
                        }
                        // TEST — no additional weight for test entry points
                        _ => {}
                    }
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

    #[test]
    fn test_analyze_risk_entrypoint() {
        use crate::index::symbols::{Symbol, SymbolKind};

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/main.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: Some(vec![Symbol {
                name: "main".to_string(),
                kind: SymbolKind::Function,
                is_public: false,
                cognitive_complexity: None,
                cyclomatic_complexity: None,
                line_start: None,
                line_end: None,
                qualified_name: None,
                byte_start: None,
                byte_end: None,
                entrypoint_kind: Some("ENTRYPOINT".to_string()),
            }]),
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Medium);
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Entry point changed"))
        );
    }

    #[test]
    fn test_analyze_risk_handler() {
        use crate::index::symbols::{Symbol, SymbolKind};

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/handlers.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: Some(vec![Symbol {
                name: "get_users".to_string(),
                kind: SymbolKind::Function,
                is_public: true,
                cognitive_complexity: None,
                cyclomatic_complexity: None,
                line_start: None,
                line_end: None,
                qualified_name: None,
                byte_start: None,
                byte_end: None,
                entrypoint_kind: Some("HANDLER".to_string()),
            }]),
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Handler changed"))
        );
    }

    #[test]
    fn test_analyze_risk_public_api() {
        use crate::index::symbols::{Symbol, SymbolKind};

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/lib.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: Some(vec![Symbol {
                name: "public_fn".to_string(),
                kind: SymbolKind::Function,
                is_public: true,
                cognitive_complexity: None,
                cyclomatic_complexity: None,
                line_start: None,
                line_end: None,
                qualified_name: None,
                byte_start: None,
                byte_end: None,
                entrypoint_kind: Some("PUBLIC_API".to_string()),
            }]),
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Public API changed"))
        );
    }

    #[test]
    fn test_analyze_risk_test_no_extra_weight() {
        use crate::index::symbols::{Symbol, SymbolKind};

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/lib.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: Some(vec![Symbol {
                name: "test_foo".to_string(),
                kind: SymbolKind::Function,
                is_public: false,
                cognitive_complexity: None,
                cyclomatic_complexity: None,
                line_start: None,
                line_end: None,
                qualified_name: None,
                byte_start: None,
                byte_end: None,
                entrypoint_kind: Some("TEST".to_string()),
            }]),
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

        // TEST entry points get no additional risk weight
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_no_entrypoint_graceful_degradation() {
        use crate::index::symbols::{Symbol, SymbolKind};

        // Symbols without entrypoint_kind (None) should still work
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/lib.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: Some(vec![Symbol {
                name: "some_fn".to_string(),
                kind: SymbolKind::Function,
                is_public: false,
                cognitive_complexity: None,
                cyclomatic_complexity: None,
                line_start: None,
                line_end: None,
                qualified_name: None,
                byte_start: None,
                byte_end: None,
                entrypoint_kind: None,
            }]),
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
}
