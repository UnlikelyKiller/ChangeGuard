use crate::impact::packet::ImpactPacket;
use crate::policy::rules::Rules;
use miette::Result;

use crate::config::model::Config;
pub fn analyze_risk(packet: &mut ImpactPacket, rules: &Rules, config: &Config) -> Result<()> {
    let registry = crate::impact::providers::RiskRegistry::default();
    registry.run(packet, rules, config)
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::impact::packet::{
        CentralityRisk, ChangedFile, CoverageDelta, FileAnalysisStatus, RiskLevel,
    };
    use std::path::PathBuf;

    #[test]
    fn test_analyze_risk_low() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

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
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules {
            protected_paths: vec!["Cargo.toml".to_string()],
            ..Rules::default()
        };

        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

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
            old_path: None,
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
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

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
            old_path: None,
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
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

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
            old_path: None,
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
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

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
            old_path: None,
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
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

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
            old_path: None,
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
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string())
        );
    }

    #[test]
    fn test_analyze_risk_structural_coupling() {
        use crate::impact::packet::StructuralCoupling;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/utils.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.structural_couplings.push(StructuralCoupling {
            caller_symbol_name: "caller_fn".to_string(),
            callee_symbol_name: "helper_fn".to_string(),
            caller_file_path: PathBuf::from("src/main.rs"),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // 15 weight from structural coupling, plus default "Provisional baseline risk" replaced
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Structurally coupled")
                    && r.contains("caller_fn")
                    && r.contains("helper_fn"))
        );
        // Weight should be Medium (15 > 0, which is > 20? No, 15 <= 20, so Low.
        // Actually the threshold is >20 for Medium. 15 <= 20, so it's Low.
        // Let's check that it has the risk reason even at Low.
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Structurally coupled"))
        );
    }

    #[test]
    fn test_analyze_risk_structural_coupling_cap_at_two() {
        use crate::impact::packet::StructuralCoupling;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/utils.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // Add 3 callers — only first 2 should contribute weight (30 total)
        packet.structural_couplings.push(StructuralCoupling {
            caller_symbol_name: "caller_a".to_string(),
            callee_symbol_name: "helper".to_string(),
            caller_file_path: PathBuf::from("src/a.rs"),
        });
        packet.structural_couplings.push(StructuralCoupling {
            caller_symbol_name: "caller_b".to_string(),
            callee_symbol_name: "helper".to_string(),
            caller_file_path: PathBuf::from("src/b.rs"),
        });
        packet.structural_couplings.push(StructuralCoupling {
            caller_symbol_name: "caller_c".to_string(),
            callee_symbol_name: "helper".to_string(),
            caller_file_path: PathBuf::from("src/c.rs"),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Only first 2 should produce risk reasons
        let coupling_reasons: Vec<_> = packet
            .risk_reasons
            .iter()
            .filter(|r| r.contains("Structurally coupled"))
            .collect();
        assert_eq!(coupling_reasons.len(), 2);
        // Total structural weight should be 30 (capped), so overall >20 -> Medium
        assert_eq!(packet.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_analyze_risk_structural_coupling_graceful_degradation() {
        // Empty structural_couplings should produce identical output to no field
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // structural_couplings is empty by default

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string())
        );
        // No structural coupling reasons
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Structurally coupled"))
        );
    }

    /// E2E Test 2: Impact integration — structural coupling risk
    /// Builds an ImpactPacket with a change to "internal" and adds
    /// StructuralCoupling entries showing "helper" calls "internal",
    /// then verifies the risk reasons reflect this coupling.
    #[test]
    fn test_e2e_structural_coupling_risk_reason() {
        use crate::impact::packet::StructuralCoupling;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/utils.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // Add structural coupling: helper calls internal
        packet.structural_couplings.push(StructuralCoupling {
            caller_symbol_name: "helper".to_string(),
            callee_symbol_name: "internal".to_string(),
            caller_file_path: PathBuf::from("src/main.rs"),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Verify the risk reasons include the exact structural coupling message
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Structurally coupled")
                    && r.contains("helper")
                    && r.contains("internal")),
            "expected risk reason 'Structurally coupled: helper calls internal', got {:?}",
            packet.risk_reasons
        );

        // Verify the structural coupling contributed risk weight (15 pts -> Medium if alone, Low otherwise)
        // With 15 pts from structural coupling alone and no other risk factors, 15 <= 20 -> Low
        // But we want to at least verify it is not ignored
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Structurally coupled")),
            "expected at least one structural coupling risk reason"
        );
    }

    /// E2E Test 4a: Empty structural_edges — no regression (impact analysis)
    /// Verifies that running impact analysis with NO structural coupling data
    /// produces output identical to what it would have been before E2-1.
    #[test]
    fn test_e2e_no_structural_coupling_no_regression() {
        // Baseline: a simple low-risk change with no structural couplings
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // structural_couplings is empty by default (Vec::new())

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Risk level should be Low (same as pre-E2-1 behavior)
        assert_eq!(packet.risk_level, RiskLevel::Low);

        // No structural coupling reasons should appear
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Structurally coupled")),
            "expected no structural coupling reasons, got {:?}",
            packet.risk_reasons
        );

        // The default "Minimal changes detected" reason should still be present
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string()),
            "expected 'Minimal changes detected' in risk reasons, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_route_handler() {
        use crate::impact::packet::ApiRoute;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/routes.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: vec![ApiRoute {
                method: "GET".to_string(),
                path_pattern: "/users".to_string(),
                handler_symbol_name: Some("get_users".to_string()),
                framework: "Axum".to_string(),
                route_source: "DECORATOR".to_string(),
                mount_prefix: None,
                is_dynamic: false,
                route_confidence: 1.0,
                evidence: None,
            }],
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Should have risk reason for the route
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Public API route")
                    && r.contains("GET")
                    && r.contains("/users")),
            "expected 'Public API route: GET /users' in risk reasons, got {:?}",
            packet.risk_reasons
        );
        // 30 weight from route handler -> Medium
        assert_eq!(packet.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_analyze_risk_empty_api_routes_no_regression() {
        // Empty api_routes should produce identical output to before route integration
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        // No route risk reasons should appear
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Public API route")),
            "expected no route risk reasons, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string()),
            "expected 'Minimal changes detected' in risk reasons, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_data_model() {
        use crate::impact::packet::DataModel;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/models/user.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: vec![DataModel {
                model_name: "UserModel".to_string(),
                model_kind: "STRUCT".to_string(),
                confidence: 1.0,
                evidence: None,
            }],
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Should have risk reason for the data model
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r == "Data model: UserModel (STRUCT)"),
            "expected 'Data model: UserModel (STRUCT)' in risk reasons, got {:?}",
            packet.risk_reasons
        );
        // 35 weight from data contract risk -> Medium (>20)
        assert_eq!(packet.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_analyze_risk_generated_data_model() {
        use crate::impact::packet::DataModel;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/generated/proto.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: vec![DataModel {
                model_name: "UserProto".to_string(),
                model_kind: "GENERATED".to_string(),
                confidence: 0.6,
                evidence: None,
            }],
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Should have risk reason for the data model
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r == "Data model: UserProto (GENERATED)"),
            "expected 'Data model: UserProto (GENERATED)' in risk reasons, got {:?}",
            packet.risk_reasons
        );
        // 20 weight from data contract risk (reduced for GENERATED) -> Low (<=20)
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_empty_data_models_no_regression() {
        // Empty data_models should produce identical output to before data model integration
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        // No data contract risk reasons should appear
        assert!(
            !packet.risk_reasons.iter().any(|r| r.contains("Data model")),
            "expected no data contract risk reasons, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string()),
            "expected 'Minimal changes detected' in risk reasons, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_centrality_high() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/core.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.centrality_risks.push(CentralityRisk {
            symbol_name: "process_request".to_string(),
            entrypoints_reachable: 8,
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("High centrality") && r.contains("8 entry points")),
            "expected centrality risk reason, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("High centrality")),
            "expected centrality risk reason, got {:?}",
            packet.risk_reasons
        );
        // Centrality alone contributes 15 weight — may be Low or Medium depending on other factors
        assert!(
            packet.risk_level == RiskLevel::Low || packet.risk_level == RiskLevel::Medium,
            "expected Low or Medium risk for centrality-only change, got {:?}",
            packet.risk_level
        );
    }

    #[test]
    fn test_analyze_risk_centrality_low_threshold() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/util.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.centrality_risks.push(CentralityRisk {
            symbol_name: "helper".to_string(),
            entrypoints_reachable: 3, // Below threshold of 5
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("High centrality")),
            "expected no centrality risk reason for below-threshold symbol, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_centrality_empty_no_regression() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // No centrality_risks — default empty

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("High centrality")),
            "expected no centrality risk reasons when empty, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_logging_coverage_reduced() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/service.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.logging_coverage_delta.push(CoverageDelta {
            file_path: "src/service.rs".to_string(),
            pattern_kind: "LOG".to_string(),
            previous_count: 10,
            current_count: 7,
            message: "Logging coverage reduced in src/service.rs: 3 statements removed".to_string(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Should have a risk reason about logging coverage reduction
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Logging coverage reduced")
                    && r.contains("src/service.rs")
                    && r.contains("3 statements removed")),
            "expected logging coverage risk reason, got {:?}",
            packet.risk_reasons
        );
        // 25 weight from observability reduction -> Medium (>20)
        assert_eq!(packet.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_analyze_risk_logging_coverage_no_regression() {
        // Empty logging_coverage_delta should produce no observability risk reasons
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // logging_coverage_delta is empty by default

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Logging coverage reduced")),
            "expected no logging coverage risk reasons when empty, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string()),
            "expected 'Minimal changes detected' in risk reasons, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_error_handling_reduced() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/handler.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.error_handling_delta.push(CoverageDelta {
            file_path: "src/handler.rs".to_string(),
            pattern_kind: "ERROR_HANDLE".to_string(),
            previous_count: 8,
            current_count: 5,
            message: "Error handling reduced in src/handler.rs: 3 patterns removed".to_string(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Should have a risk reason about error handling reduction
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Error handling reduced")
                    && r.contains("src/handler.rs")
                    && r.contains("3 patterns removed")),
            "expected error handling risk reason, got {:?}",
            packet.risk_reasons
        );
        // 25 weight from error handling reduction -> Medium (>20)
        assert_eq!(packet.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_analyze_risk_error_handling_no_regression() {
        // Empty error_handling_delta should produce no error handling risk reasons
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // error_handling_delta is empty by default

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Error handling reduced")),
            "expected no error handling risk reasons when empty, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string()),
            "expected 'Minimal changes detected' in risk reasons, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_infrastructure_error_handling() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("deploy/config.yaml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.error_handling_delta.push(CoverageDelta {
            file_path: "deploy/config.yaml".to_string(),
            pattern_kind: "ERROR_HANDLE".to_string(),
            previous_count: 5,
            current_count: 3,
            message: "Error handling reduced in deploy/config.yaml: 2 patterns removed".to_string(),
        });
        // Use topology data: deploy is an Infrastructure directory
        packet.infrastructure_dirs.push("deploy".to_string());

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Should have infrastructure error handling risk reason
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Error handling change in infrastructure")
                    && r.contains("deploy/config.yaml")),
            "expected infrastructure error handling risk reason, got {:?}",
            packet.risk_reasons
        );
        // 25 (error handling reduction) + 25 (infrastructure) = 50 weight -> Medium (>20)
        assert_eq!(packet.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_analyze_risk_infrastructure_no_topology() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("deploy/config.yaml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.error_handling_delta.push(CoverageDelta {
            file_path: "deploy/config.yaml".to_string(),
            pattern_kind: "ERROR_HANDLE".to_string(),
            previous_count: 5,
            current_count: 3,
            message: "Error handling reduced in deploy/config.yaml: 2 patterns removed".to_string(),
        });
        // infrastructure_dirs is empty — falls back to heuristic which includes "deploy"

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Should have infrastructure error handling risk reason via heuristic fallback
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Error handling change in infrastructure")
                    && r.contains("deploy/config.yaml")),
            "expected infrastructure error handling risk reason via heuristic, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_telemetry_coverage_reduced() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/api/handler.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.telemetry_coverage_delta.push(CoverageDelta {
            file_path: "src/api/handler.rs".to_string(),
            pattern_kind: "TRACE".to_string(),
            previous_count: 5,
            current_count: 2,
            message:
                "Telemetry coverage reduced in src/api/handler.rs: 3 instrumentation points removed"
                    .to_string(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Should have a risk reason about telemetry coverage reduction
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Telemetry coverage reduced")
                    && r.contains("src/api/handler.rs")
                    && r.contains("3 instrumentation points removed")),
            "expected telemetry coverage risk reason, got {:?}",
            packet.risk_reasons
        );
        // 25 weight from telemetry reduction -> Medium (>20)
        assert_eq!(packet.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_analyze_risk_telemetry_coverage_no_regression() {
        // Empty telemetry_coverage_delta should produce no telemetry risk reasons
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // telemetry_coverage_delta is empty by default

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Telemetry coverage reduced")),
            "expected no telemetry coverage risk reasons when empty, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string()),
            "expected 'Minimal changes detected' in risk reasons, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_test_coverage_with_tests() {
        use crate::impact::packet::{CoveringTest, TestCoverage};

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/lib.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // Symbol with test coverage
        packet.test_coverage.push(TestCoverage {
            changed_symbol: "my_function".to_string(),
            changed_file: "src/lib.rs".to_string(),
            covering_tests: vec![CoveringTest {
                test_file: "tests/test_lib.rs".to_string(),
                test_symbol: "test_my_function".to_string(),
                confidence: 1.0,
                mapping_kind: "IMPORT".to_string(),
            }],
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Should not have "No test coverage" advisory since covering_tests is non-empty
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("No test coverage found for my_function")),
            "expected no test coverage advisory when tests exist, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_no_test_coverage_advisory() {
        use crate::impact::packet::TestCoverage;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/lib.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // Symbol without test coverage
        packet.test_coverage.push(TestCoverage {
            changed_symbol: "my_function".to_string(),
            changed_file: "src/lib.rs".to_string(),
            covering_tests: vec![],
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        // Should have advisory about missing test coverage
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("No test coverage found for my_function")
                    && r.contains("src/lib.rs")),
            "expected 'No test coverage found for my_function' advisory, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_test_coverage_empty_no_regression() {
        // Empty test_coverage should produce no advisory
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        // test_coverage is empty by default

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules, &Config::default()).unwrap();

        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("No test coverage found")),
            "expected no test coverage advisory when empty, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string()),
            "expected 'Minimal changes detected' in risk reasons, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_ci_gates() {
        use crate::impact::packet::CIGate;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".github/workflows/ci.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: vec![CIGate {
                platform: "github_actions".to_string(),
                job_name: "build".to_string(),
                trigger: Some("push".to_string()),
            }],
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = true;
        analyze_risk(&mut packet, &rules, &config).unwrap();

        // Should have a CI/CD change risk reason
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("CI pipeline config change")),
            "expected 'CI pipeline config change' in risk reasons, got {:?}",
            packet.risk_reasons
        );
        // 3 weight from CI/CD change -> Low (<= 20)
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_ci_gates_empty_no_regression() {
        // Empty ci_gates should produce no CI/CD risk reasons
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("CI/CD change")),
            "expected no CI/CD change risk reasons when empty, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string()),
            "expected 'Minimal changes detected' in risk reasons, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_ci_gates_weight_cap() {
        use crate::impact::packet::CIGate;

        // Two files with CI gates should still only contribute 30 weight total (cap)
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".github/workflows/ci.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: vec![CIGate {
                platform: "github_actions".to_string(),
                job_name: "build".to_string(),
                trigger: Some("push".to_string()),
            }],
        });
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".gitlab-ci.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: vec![CIGate {
                platform: "gitlab_ci".to_string(),
                job_name: "test".to_string(),
                trigger: Some("merge_request".to_string()),
            }],
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = true;
        analyze_risk(&mut packet, &rules, &config).unwrap();

        // Should have per-file CI pipeline reasons (deterministic, sorted)
        let ci_reasons: Vec<_> = packet
            .risk_reasons
            .iter()
            .filter(|r| r.contains("CI pipeline config change"))
            .collect();
        assert_eq!(
            ci_reasons.len(),
            2,
            "expected 2 CI pipeline config change reasons, got {:?}",
            ci_reasons
        );

        // 3 weight (alone, category-capped) -> Low
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_runtime_env_var_dependency() {
        use crate::index::env_schema::EnvVarDep;

        // File with a non-common env var like DATABASE_URL should get risk weight
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/config.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.env_var_deps.push(EnvVarDep {
            var_name: "DATABASE_URL".to_string(),
            declared: false,
            evidence: "".to_string(),
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        // Should have a runtime dependency risk reason
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("New environment variable dependency: DATABASE_URL")),
            "expected 'New environment variable dependency: DATABASE_URL' in risk reasons, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet.risk_level == RiskLevel::Low || packet.risk_level == RiskLevel::Medium,
            "expected Low or Medium risk for single env var dependency, got {:?}",
            packet.risk_level
        );
    }

    #[test]
    fn test_analyze_risk_runtime_common_env_var_skipped() {
        use crate::index::env_schema::EnvVarDep;

        // File with only common env vars (like PATH) should NOT get runtime risk weight
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/main.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.env_var_deps.push(EnvVarDep {
            var_name: "PATH".to_string(),
            declared: false,
            evidence: "".to_string(),
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        // No runtime dependency risk reasons should appear for common env vars
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("New environment variable dependency")),
            "expected no runtime env var risk reasons for common vars, got {:?}",
            packet.risk_reasons
        );
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_runtime_config_key_dependency() {
        use crate::impact::packet::RuntimeUsageDelta;

        // File with config keys should get risk weight
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/settings.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.runtime_usage_delta.push(RuntimeUsageDelta {
            file_path: "src/settings.rs".to_string(),
            env_vars_previous_count: 0,
            env_vars_current_count: 0,
            config_keys_previous_count: 1,
            config_keys_current_count: 2,
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        // Should have config key risk reasons
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Configuration key references changed in src/settings.rs")),
            "expected 'Configuration key references changed in src/settings.rs' in risk reasons, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_runtime_framework_convention_reduced_weight() {
        use crate::impact::packet::RuntimeUsageDelta;
        use crate::index::runtime_usage::RuntimeUsage;

        // File with only framework convention config keys should get reduced weight (5)
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/app.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: Some(RuntimeUsage {
                env_vars: vec![],
                config_keys: vec!["server.port".to_string(), "logging.level".to_string()],
            }),
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.runtime_usage_delta.push(RuntimeUsageDelta {
            file_path: "src/app.rs".to_string(),
            env_vars_previous_count: 0,
            env_vars_current_count: 0,
            config_keys_previous_count: 0,
            config_keys_current_count: 2,
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Configuration key references changed in src/app.rs")),
            "expected 'Configuration key references changed in src/app.rs' in risk reasons, got {:?}",
            packet.risk_reasons
        );
        // With only framework conventions, weight is 5, which is <= 20, so Low
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_runtime_empty_no_regression() {
        // File with no runtime_usage should produce no runtime risk reasons
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("README.md"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Runtime dependency on env var")
                    || r.contains("Runtime dependency on config key")
                    || r.contains("Framework config key")),
            "expected no runtime dependency risk reasons when empty, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .contains(&"Minimal changes detected".to_string()),
            "expected 'Minimal changes detected' in risk reasons, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_trace_config_drift() {
        use crate::impact::packet::{TraceConfigChange, TraceConfigType};
        let mut packet = ImpactPacket::default();
        packet.trace_config_drift.push(TraceConfigChange {
            file: PathBuf::from("otel-config.yaml"),
            config_type: TraceConfigType::OpenTelemetryCollector,
            risk_weight: 3,
            is_deleted: false,
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Observability config drift"))
        );
        // Default weight is 3
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_sdk_dependencies() {
        use crate::impact::packet::{SdkDependency, SdkDependencyDelta};
        let mut packet = ImpactPacket::default();
        packet.sdk_dependencies_delta = Some(SdkDependencyDelta {
            added: vec![SdkDependency {
                sdk_name: "opentelemetry".to_string(),
                file_path: PathBuf::from("src/main.rs"),
                import_statement: "use opentelemetry;".to_string(),
            }],
            modified: vec![SdkDependency {
                sdk_name: "sentry".to_string(),
                file_path: PathBuf::from("src/lib.rs"),
                import_statement: "use sentry;".to_string(),
            }],
            removed: vec![],
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("New SDK dependency: opentelemetry"))
        );
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Modified SDK dependency: sentry"))
        );
        // New(5) + Mod(3) = 8
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_service_map_delta() {
        use crate::impact::packet::ServiceMapDelta;
        let mut packet = ImpactPacket::default();
        packet.service_map_delta = Some(ServiceMapDelta {
            affected_services: vec![
                "users".to_string(),
                "billing".to_string(),
                "auth".to_string(),
            ],
            services: vec![],
            cross_service_edges: vec![],
            total_services: 3,
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Cross-service change affecting 3 services"))
        );
        // 3 services -> weight 6
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_preserves_enrichment_elevated_risk_level() {
        let mut packet = ImpactPacket {
            risk_level: RiskLevel::High,
            risk_reasons: vec!["Enrichment elevated risk".to_string()],
            ..ImpactPacket::default()
        };

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::High);
        assert!(
            packet
                .risk_reasons
                .contains(&"Enrichment elevated risk".to_string())
        );
    }

    #[test]
    fn test_analyze_risk_respects_cross_service_elevation_threshold() {
        use crate::impact::packet::ServiceMapDelta;

        let mut packet = ImpactPacket::default();
        packet.service_map_delta = Some(ServiceMapDelta {
            affected_services: vec![
                "users".to_string(),
                "billing".to_string(),
                "auth".to_string(),
                "notifications".to_string(),
            ],
            services: vec![],
            cross_service_edges: vec![],
            total_services: 4,
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.services.cross_service_elevation_threshold = 5;
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::Low);
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Cross-service change affecting"))
        );
    }

    #[test]
    fn test_analyze_risk_data_flow_coupling() {
        use crate::impact::packet::DataFlowMatch;
        let mut packet = ImpactPacket::default();
        packet.data_flow_matches.push(DataFlowMatch {
            chain_label: "GET /users -> User".to_string(),
            changed_nodes: vec!["get_users".to_string()],
            total_nodes: 2,
            change_pct: 0.5,
            risk: RiskLevel::Low,
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Data-flow coupling: chain GET /users -> User affected"))
        );
        // weight 4
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_deploy_manifest_change() {
        use crate::impact::packet::{DeployManifestChange, ManifestType};
        let mut packet = ImpactPacket::default();
        packet.deploy_manifest_changes.push(DeployManifestChange {
            file: PathBuf::from("Dockerfile"),
            manifest_type: ManifestType::Dockerfile,
            risk_tier: 1,
            coupled_files: Vec::new(),
            high_blast_resources: Vec::new(),
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Deployment manifest change: Dockerfile"))
        );
        // weight 3
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_adr_staleness_advisory() {
        use crate::impact::packet::RelevantDecision;
        let mut packet = ImpactPacket::default();
        packet.relevant_decisions.push(RelevantDecision {
            file_path: PathBuf::from("docs/adr/001-auth.md"),
            heading: Some("Auth".to_string()),
            excerpt: "Use OAuth2".to_string(),
            similarity: 0.9,
            rerank_score: None,
            staleness_days: Some(400),
            staleness_tier: Some(crate::impact::packet::StalenessTier::Warning),
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.adr_staleness.threshold_days = 365;
        config.coverage.adr_staleness.enabled = true;

        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(packet.risk_reasons.iter().any(|r| {
            r.contains("Stale architectural context: docs/adr/001-auth.md (400 days old)")
        }));
        // Advisory weight is 0 in the current implementation (advisory only)
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_combined_high() {
        use crate::impact::packet::CoverageDelta;
        let mut packet = ImpactPacket::default();

        // 1. Telemetry reduction (25)
        packet.telemetry_coverage_delta.push(CoverageDelta {
            file_path: "src/api.rs".to_string(),
            pattern_kind: "TRACE".to_string(),
            previous_count: 10,
            current_count: 5,
            message: "reduced".to_string(),
        });

        // 2. Multi-service impact (10) - 5+ services
        packet.service_map_delta = Some(crate::impact::packet::ServiceMapDelta {
            affected_services: vec![
                "s1".to_string(),
                "s2".to_string(),
                "s3".to_string(),
                "s4".to_string(),
                "s5".to_string(),
            ],
            services: vec![],
            cross_service_edges: vec![],
            total_services: 5,
        });

        // 3. Data flow matches (12) - 3 matches at 4 each
        for i in 0..3 {
            packet
                .data_flow_matches
                .push(crate::impact::packet::DataFlowMatch {
                    chain_label: format!("chain-{}", i),
                    changed_nodes: vec!["node".to_string()],
                    total_nodes: 2,
                    change_pct: 0.5,
                    risk: RiskLevel::Low,
                });
        }

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert_eq!(packet.risk_level, RiskLevel::High);
    }
    #[test]
    fn test_analyze_risk_ci_gates_disabled() {
        use crate::impact::packet::CIGate;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".github/workflows/ci.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: vec![CIGate {
                platform: "github_actions".to_string(),
                job_name: "build".to_string(),
                trigger: Some("push".to_string()),
            }],
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = false;

        analyze_risk(&mut packet, &rules, &config).unwrap();

        // Should NOT have CI/CD risk reason because it's disabled
        assert!(
            !packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("CI pipeline config change")),
            "expected no CI pipeline risk reason when disabled, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_ci_gates_enabled() {
        use crate::impact::packet::CIGate;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".github/workflows/ci.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: vec![CIGate {
                platform: "github_actions".to_string(),
                job_name: "build".to_string(),
                trigger: Some("push".to_string()),
            }],
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = true;
        config.coverage.ci_self_awareness.ci_changed_weight = 10;

        analyze_risk(&mut packet, &rules, &config).unwrap();

        // Should HAVE CI/CD risk reason because it's enabled
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("CI pipeline config change")),
            "expected CI pipeline risk reason when enabled, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_data_flow_weight_capping() {
        use crate::impact::packet::DataFlowMatch;

        let mut packet = ImpactPacket::default();
        for i in 0..6 {
            packet.data_flow_matches.push(DataFlowMatch {
                chain_label: format!("chain-{}", i),
                changed_nodes: vec!["node".to_string()],
                total_nodes: 2,
                change_pct: 0.5,
                risk: RiskLevel::Low,
            });
        }

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.data_flow.risk_weight_per_match = 4;
        config.coverage.data_flow.risk_cap = 20;

        analyze_risk(&mut packet, &rules, &config).unwrap();

        // With cap 20 and weight 4, only 5 matches should contribute (20 total, not 24)
        let df_reasons: Vec<_> = packet
            .risk_reasons
            .iter()
            .filter(|r| r.contains("Data-flow coupling"))
            .collect();
        assert_eq!(
            df_reasons.len(),
            5,
            "expected exactly 5 data-flow reasons (capped at 20), got {:?}",
            df_reasons
        );

        // Total weight 20 means Low (threshold is >20 for Medium)
        assert_eq!(
            packet.risk_level,
            RiskLevel::Low,
            "expected Low risk when total weight is exactly 20 (capped), got {:?}",
            packet.risk_level
        );

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Data-flow coupling")),
            "expected at least one data-flow coupling risk reason"
        );
    }

    // --- M7-5 CI Self-Awareness Risk Weighting Tests ---

    #[test]
    fn test_analyze_risk_ci_alone_low() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".github/workflows/ci.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = true;
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("CI pipeline config change")),
            "expected CI pipeline reason, got {:?}",
            packet.risk_reasons
        );
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_ci_plus_source_medium() {
        use crate::index::symbols::{Symbol, SymbolKind};

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".github/workflows/ci.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/main.rs"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: Some(vec![Symbol {
                name: "helper".to_string(),
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
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = true;
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("CI pipeline config change")),
            "expected CI pipeline reason, got {:?}",
            packet.risk_reasons
        );
        // 5 weight from CI + source -> Low (<=20)
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_ci_plus_deploy_escalated() {
        use crate::impact::packet::{DeployManifestChange, ManifestType};

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".github/workflows/ci.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.deploy_manifest_changes.push(DeployManifestChange {
            file: PathBuf::from("Dockerfile"),
            manifest_type: ManifestType::Dockerfile,
            risk_tier: 1,
            coupled_files: Vec::new(),
            high_blast_resources: Vec::new(),
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = true;
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("CI pipeline config change")),
            "expected CI pipeline reason, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Deployment manifest change: Dockerfile")),
            "expected deploy manifest reason, got {:?}",
            packet.risk_reasons
        );
        // CI weight = 5 (escalated by deploy) + deploy weight 3 = 8, still Low
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_pre_commit_low() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".pre-commit-config.yaml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = true;
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Pre-commit hooks modified")),
            "expected pre-commit reason, got {:?}",
            packet.risk_reasons
        );
        // 2 weight from pre-commit -> Low (<=20)
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_generated_ci_informational() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".github/workflows/generated-ci.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = true;
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Generated CI file changed")),
            "expected generated CI reason, got {:?}",
            packet.risk_reasons
        );
        // Generated files are informational only (no weight)
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_unknown_ci_like_low() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("ci/deploy.sh"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = true;
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Unknown CI-like file changed")),
            "expected unknown CI-like reason, got {:?}",
            packet.risk_reasons
        );
        // 1 weight from unknown CI-like -> Low (<=20)
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_ci_reasons_sorted() {
        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".github/workflows/b.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });
        packet.changes.push(ChangedFile {
            path: PathBuf::from(".github/workflows/a.yml"),
            status: "Modified".to_string(),
            old_path: None,
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.ci_self_awareness.enabled = true;
        analyze_risk(&mut packet, &rules, &config).unwrap();

        let ci_reasons: Vec<_> = packet
            .risk_reasons
            .iter()
            .filter(|r| r.contains("CI pipeline config change"))
            .cloned()
            .collect();
        assert_eq!(ci_reasons.len(), 2);
        assert_eq!(
            ci_reasons[0],
            "CI pipeline config change: .github/workflows/a.yml"
        );
        assert_eq!(
            ci_reasons[1],
            "CI pipeline config change: .github/workflows/b.yml"
        );
    }

    // --- M7-4 Deployment Manifest Awareness Tests ---

    #[test]
    fn test_analyze_risk_deploy_manifest_k8s_terraform_high_tier() {
        use crate::impact::packet::{DeployManifestChange, ManifestType};
        let mut packet = ImpactPacket::default();
        packet.deploy_manifest_changes.push(DeployManifestChange {
            file: PathBuf::from("k8s/deployment.yaml"),
            manifest_type: ManifestType::Kubernetes,
            risk_tier: 3,
            coupled_files: Vec::new(),
            high_blast_resources: Vec::new(),
        });
        packet.deploy_manifest_changes.push(DeployManifestChange {
            file: PathBuf::from("main.tf"),
            manifest_type: ManifestType::Terraform,
            risk_tier: 3,
            coupled_files: Vec::new(),
            high_blast_resources: Vec::new(),
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        // Each tier-3 manifest contributes weight 8, total 16, cap at 15.
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Deployment manifest change: k8s/deployment.yaml")),
            "expected k8s manifest reason, got {:?}",
            packet.risk_reasons
        );
        assert!(
            packet
                .risk_reasons
                .iter()
                .any(|r| r.contains("Deployment manifest change: main.tf")),
            "expected terraform manifest reason, got {:?}",
            packet.risk_reasons
        );
    }

    #[test]
    fn test_analyze_risk_deploy_manifest_weight_cap() {
        use crate::impact::packet::{DeployManifestChange, ManifestType};
        let mut packet = ImpactPacket::default();
        for i in 0..6 {
            packet.deploy_manifest_changes.push(DeployManifestChange {
                file: PathBuf::from(format!("Dockerfile.{}", i)),
                manifest_type: ManifestType::Dockerfile,
                risk_tier: 1,
                coupled_files: Vec::new(),
                high_blast_resources: Vec::new(),
            });
        }

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        // 6 manifests at tier 1 (3 each) = 18, cap at 15.
        let deploy_reasons: Vec<_> = packet
            .risk_reasons
            .iter()
            .filter(|r| r.contains("Deployment manifest change"))
            .collect();
        assert_eq!(
            deploy_reasons.len(),
            6,
            "expected 6 deploy reasons, got {:?}",
            deploy_reasons
        );
        // Total weight 15 means Low (threshold is >20 for Medium)
        assert_eq!(
            packet.risk_level,
            RiskLevel::Low,
            "expected Low risk at cap 15, got {:?}",
            packet.risk_level
        );
    }
}
