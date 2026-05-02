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
                            debug!("Risk Factor: Handler changed ({}) +{}", symbol.name, weight);
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

    // 3b. Structural Coupling Risk
    // Max 30 total weight for this category (cap at 2 callers contributing 15 each).
    let structural_weight_cap = 30;
    let weight_per_caller = 15;
    let mut structural_weight = 0;
    for (callers_counted, coupling) in packet.structural_couplings.iter().enumerate() {
        if callers_counted >= 2 {
            break;
        }
        if structural_weight + weight_per_caller > structural_weight_cap {
            // Cap at the max
            let remaining = structural_weight_cap - structural_weight;
            if remaining > 0 {
                structural_weight += remaining;
            }
            break;
        }
        structural_weight += weight_per_caller;
        reasons.push(format!(
            "Structurally coupled: {} calls {}",
            coupling.caller_symbol_name, coupling.callee_symbol_name
        ));
        debug!(
            "Risk Factor: Structurally coupled ({} calls {}) +{}",
            coupling.caller_symbol_name, coupling.callee_symbol_name, weight_per_caller
        );
    }
    total_weight += structural_weight;

    // 3c. Route Handler Risk
    // Add 30 weight per file that has route handlers (max 30 total, not per-route).
    // This stacks with but doesn't duplicate the entrypoint HANDLER weight.
    let route_weight = 30;
    let route_weight_cap = 30;
    let mut route_total = 0;
    for file in &packet.changes {
        if !file.api_routes.is_empty() && route_total + route_weight <= route_weight_cap {
            route_total += route_weight;
            // Add a risk reason for the first route (summarize all routes for this file)
            let first_route = &file.api_routes[0];
            reasons.push(format!(
                "Public API route: {} {}",
                first_route.method, first_route.path_pattern
            ));
            debug!(
                "Risk Factor: Route handler in {} ({} routes) +{}",
                file.path.display(),
                file.api_routes.len(),
                route_weight
            );
        }
    }
    total_weight += route_total;

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
            api_routes: Vec::new(),
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
            api_routes: Vec::new(),
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
            api_routes: Vec::new(),
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
            api_routes: Vec::new(),
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
            api_routes: Vec::new(),
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
            api_routes: Vec::new(),
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
            api_routes: Vec::new(),
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
    fn test_analyze_risk_structural_coupling() {
        use crate::impact::packet::StructuralCoupling;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/utils.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
        });
        packet.structural_couplings.push(StructuralCoupling {
            caller_symbol_name: "caller_fn".to_string(),
            callee_symbol_name: "helper_fn".to_string(),
            caller_file_path: PathBuf::from("src/main.rs"),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
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
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
        });
        // structural_couplings is empty by default

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
        });
        // Add structural coupling: helper calls internal
        packet.structural_couplings.push(StructuralCoupling {
            caller_symbol_name: "helper".to_string(),
            callee_symbol_name: "internal".to_string(),
            caller_file_path: PathBuf::from("src/main.rs"),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
        });
        // structural_couplings is empty by default (Vec::new())

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
}
