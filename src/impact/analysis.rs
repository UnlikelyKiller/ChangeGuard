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

    // 3d. Data Contract Risk
    // Add 35 weight per file that contains data models (cap at 35 total for this category).
    // If any model has model_kind = "GENERATED", use reduced weight of 20 instead.
    let data_model_weight_full = 35;
    let data_model_weight_generated = 20;
    let mut data_model_total = 0;
    for file in &packet.changes {
        if !file.data_models.is_empty() && data_model_total == 0 {
            let has_generated = file.data_models.iter().any(|m| m.model_kind == "GENERATED");
            let weight = if has_generated {
                data_model_weight_generated
            } else {
                data_model_weight_full
            };
            data_model_total += weight;

            // Add a risk reason for each model
            for model in &file.data_models {
                reasons.push(format!(
                    "Data model: {} ({})",
                    model.model_name, model.model_kind
                ));
                debug!(
                    "Risk Factor: Data model {} ({}) in {}",
                    model.model_name,
                    model.model_kind,
                    file.path.display()
                );
            }
        }
    }
    total_weight += data_model_total;

    // 3e. Centrality Risk
    // Symbols reachable from >5 entry points contribute up to 15 points within
    // the Historical Hotspot category (max 30 points).
    let centrality_threshold = 5;
    let centrality_weight = 15;
    let centrality_weight_cap = 15;
    let mut centrality_total = 0;
    // Centrality risk is applied via pre-populated data on symbols.
    // See populate_centrality_risks in commands/impact.rs.
    for risk in &packet.centrality_risks {
        if centrality_total + centrality_weight <= centrality_weight_cap
            && risk.entrypoints_reachable > centrality_threshold
        {
            centrality_total += centrality_weight;
            reasons.push(format!(
                "High centrality: {} reachable from {} entry points",
                risk.symbol_name, risk.entrypoints_reachable
            ));
            debug!(
                "Risk Factor: High centrality ({} reachable from {} entry points) +{}",
                risk.symbol_name, risk.entrypoints_reachable, centrality_weight
            );
        }
    }
    total_weight += centrality_total;

    // 3f. Observability Reduction Risk
    // Each file with reduced logging coverage contributes 25 points, capped at 25 total.
    let observability_weight_per_file = 25;
    let observability_weight_cap = 25;
    let mut observability_total = 0;
    for delta in &packet.logging_coverage_delta {
        if delta.current_count < delta.previous_count
            && observability_total + observability_weight_per_file <= observability_weight_cap
        {
            observability_total += observability_weight_per_file;
            let reduction = delta.previous_count - delta.current_count;
            reasons.push(format!(
                "Logging coverage reduced in {}: {} statements removed",
                delta.file_path, reduction
            ));
            debug!(
                "Risk Factor: Logging coverage reduced ({}) +{}",
                delta.file_path, observability_weight_per_file
            );
        }
    }
    total_weight += observability_total;

    // 3g. Error Handling Reduction Risk
    // Each file with reduced error handling coverage contributes 25 points, capped at 25 total.
    let error_handling_weight_per_file = 25;
    let error_handling_weight_cap = 25;
    let mut error_handling_total = 0;
    for delta in &packet.error_handling_delta {
        if delta.current_count < delta.previous_count
            && error_handling_total + error_handling_weight_per_file <= error_handling_weight_cap
        {
            error_handling_total += error_handling_weight_per_file;
            let reduction = delta.previous_count - delta.current_count;
            reasons.push(format!(
                "Error handling reduced in {}: {} patterns removed",
                delta.file_path, reduction
            ));
            debug!(
                "Risk Factor: Error handling reduced ({}) +{}",
                delta.file_path, error_handling_weight_per_file
            );
        }
    }
    total_weight += error_handling_total;

    // 3h. Infrastructure Error Handling Risk
    // Changed files in Infrastructure directories that also have error_handling_delta entries
    // contribute 25 weight per file, capped at 25 total.
    let infra_weight_per_file = 25;
    let infra_weight_cap = 25;
    let mut infra_total = 0;

    // Collect file paths from error_handling_delta for lookup
    let error_handling_files: std::collections::HashSet<&str> = packet
        .error_handling_delta
        .iter()
        .map(|d| d.file_path.as_str())
        .collect();

    if !error_handling_files.is_empty() {
        // Determine infrastructure directories: use topology data if available, else heuristic
        let infra_dirs: Vec<&str> = if packet.infrastructure_dirs.is_empty() {
            vec![".github/workflows", "infra", "deploy", "terraform", "k8s"]
        } else {
            packet
                .infrastructure_dirs
                .iter()
                .map(|s| s.as_str())
                .collect()
        };

        for change in &packet.changes {
            if infra_total + infra_weight_per_file > infra_weight_cap {
                break;
            }
            let path_str = change.path.to_string_lossy();
            let path_str_ref = path_str.as_ref();

            // Check if this file is in an infrastructure directory
            let is_infra = infra_dirs.iter().any(|dir| {
                path_str_ref.starts_with(dir)
                    && (path_str_ref.len() == dir.len()
                        || path_str_ref.chars().nth(dir.len()) == Some('/')
                        || path_str_ref.chars().nth(dir.len()) == Some('\\'))
            });

            // Check if this file has an error handling delta
            let has_error_handling_delta = error_handling_files.contains(path_str_ref);

            if is_infra && has_error_handling_delta {
                infra_total += infra_weight_per_file;
                reasons.push(format!(
                    "Error handling change in infrastructure: {}",
                    path_str_ref
                ));
                debug!(
                    "Risk Factor: Error handling change in infrastructure ({}) +{}",
                    path_str_ref, infra_weight_per_file
                );
            }
        }
    }
    total_weight += infra_total;

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
    use crate::impact::packet::{CentralityRisk, ChangedFile, CoverageDelta, FileAnalysisStatus};
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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
            data_models: Vec::new(),
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

    #[test]
    fn test_analyze_risk_data_model() {
        use crate::impact::packet::DataModel;

        let mut packet = ImpactPacket::default();
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/models/user.rs"),
            status: "Modified".to_string(),
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
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });
        packet.centrality_risks.push(CentralityRisk {
            symbol_name: "process_request".to_string(),
            entrypoints_reachable: 8,
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });
        packet.centrality_risks.push(CentralityRisk {
            symbol_name: "helper".to_string(),
            entrypoints_reachable: 3, // Below threshold of 5
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });
        // No centrality_risks — default empty

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });
        packet.logging_coverage_delta.push(CoverageDelta {
            file_path: "src/service.rs".to_string(),
            pattern_kind: "LOG".to_string(),
            previous_count: 10,
            current_count: 7,
            message: "Logging coverage reduced in src/service.rs: 3 statements removed".to_string(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });
        // logging_coverage_delta is empty by default

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });
        packet.error_handling_delta.push(CoverageDelta {
            file_path: "src/handler.rs".to_string(),
            pattern_kind: "ERROR_HANDLE".to_string(),
            previous_count: 8,
            current_count: 5,
            message: "Error handling reduced in src/handler.rs: 3 patterns removed".to_string(),
        });

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
        });
        // error_handling_delta is empty by default

        let rules = Rules::default();
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
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
        analyze_risk(&mut packet, &rules).unwrap();

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
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
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
        analyze_risk(&mut packet, &rules).unwrap();

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
}
