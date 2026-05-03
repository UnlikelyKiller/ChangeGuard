use crate::impact::packet::{ImpactPacket, RiskLevel};
use crate::policy::protected_paths::ProtectedPathChecker;
use crate::policy::rules::Rules;
use miette::Result;
use std::sync::LazyLock;
use tracing::debug;

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

use crate::config::model::Config;
pub fn analyze_risk(packet: &mut ImpactPacket, rules: &Rules, config: &Config) -> Result<()> {
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

    // 3j. Test Coverage Advisory (informational, not risk weight)
    // For each TestCoverage entry with empty covering_tests, add an advisory.
    for coverage in &packet.test_coverage {
        if coverage.covering_tests.is_empty() {
            reasons.push(format!(
                "No test coverage found for {} ({})",
                coverage.changed_symbol, coverage.changed_file
            ));
            debug!(
                "Advisory: No test coverage for {} in {}",
                coverage.changed_symbol, coverage.changed_file
            );
        }
    }

    // 3i. Telemetry Reduction Risk
    // Each file with reduced telemetry coverage contributes 25 points, capped at 25 total.
    let telemetry_weight_per_file = 25;
    let telemetry_weight_cap = 25;
    let mut telemetry_total = 0;
    for delta in &packet.telemetry_coverage_delta {
        if delta.current_count < delta.previous_count
            && telemetry_total + telemetry_weight_per_file <= telemetry_weight_cap
        {
            telemetry_total += telemetry_weight_per_file;
            let reduction = delta.previous_count - delta.current_count;
            reasons.push(format!(
                "Telemetry coverage reduced in {}: {} instrumentation points removed",
                delta.file_path, reduction
            ));
            debug!(
                "Risk Factor: Telemetry coverage reduced ({}) +{}",
                delta.file_path, telemetry_weight_per_file
            );
        }
    }
    total_weight += telemetry_total;

    // 4. M7 Engineering Coverage Risks
    
    // 4a. Trace Config Drift
    let trace_config_weight_per_file = config.coverage.traces.risk_weight_per_config_file;
    let trace_config_weight_cap = config.coverage.traces.risk_cap;
    let mut trace_config_total = 0;
    for change in &packet.trace_config_drift {
        if trace_config_total + trace_config_weight_per_file <= trace_config_weight_cap {
            trace_config_total += trace_config_weight_per_file;
            reasons.push(format!("Observability config drift: {:?}", change.file));
            debug!("Risk Factor: Trace config drift ({:?}) +{}", change.file, trace_config_weight_per_file);
        }
    }
    total_weight += trace_config_total;

    // 4b. Trace Env Var Changes
    let trace_env_weight_per_var = config.coverage.traces.risk_weight_per_env_var;
    let trace_env_weight_cap = config.coverage.traces.risk_cap; // Note: using same cap as config drift for now or check config
    let mut trace_env_total = 0;
    for change in &packet.trace_env_vars {
        if trace_env_total + trace_env_weight_per_var <= trace_env_weight_cap {
            trace_env_total += trace_env_weight_per_var;
            reasons.push(format!("Observability env var change: {}", change.var_name));
            debug!("Risk Factor: Trace env var change ({}) +{}", change.var_name, trace_env_weight_per_var);
        }
    }
    total_weight += trace_env_total;

    // 4c. SDK Dependency Changes
    if let Some(ref delta) = packet.sdk_dependencies_delta {
        let sdk_new_weight = config.coverage.sdk.risk_weight_new;
        let sdk_new_cap = config.coverage.sdk.risk_cap;
        let mut sdk_new_total = 0;
        for sdk in &delta.added {
            if sdk_new_total + sdk_new_weight <= sdk_new_cap {
                sdk_new_total += sdk_new_weight;
                reasons.push(format!("New SDK dependency: {}", sdk.sdk_name));
                debug!("Risk Factor: New SDK ({}) +{}", sdk.sdk_name, sdk_new_weight);
            }
        }
        total_weight += sdk_new_total;

        let sdk_mod_weight = config.coverage.sdk.risk_weight_modified;
        let sdk_mod_cap = config.coverage.sdk.risk_cap;
        let mut sdk_mod_total = 0;
        for sdk in &delta.modified {
            if sdk_mod_total + sdk_mod_weight <= sdk_mod_cap {
                sdk_mod_total += sdk_mod_weight;
                reasons.push(format!("Modified SDK dependency: {}", sdk.sdk_name));
                debug!("Risk Factor: Modified SDK ({}) +{}", sdk.sdk_name, sdk_mod_weight);
            }
        }
        total_weight += sdk_mod_total;
    }

    // 4d. Cross-Service Impact (Service-Map)
    if let Some(ref delta) = packet.service_map_delta {
        let count = delta.affected_services.len();
        let svc_weight = if count >= 5 {
            config.coverage.services.risk_weight_5plus
        } else if count >= 3 {
            config.coverage.services.risk_weight_3to4
        } else if count == 2 {
            config.coverage.services.risk_weight_2svcs
        } else {
            0
        };
        if svc_weight > 0 {
            total_weight += svc_weight;
            reasons.push(format!("Cross-service change affecting {} services", count));
            debug!("Risk Factor: Cross-service impact ({} svcs) +{}", count, svc_weight);
        }
    }

    // 4e. Data-Flow Coupling
    let data_flow_weight_per_match = config.coverage.data_flow.risk_weight_per_match;
    let data_flow_weight_cap = config.coverage.data_flow.risk_cap;
    let mut data_flow_total = 0;
    for m in &packet.data_flow_matches {
        if data_flow_total + data_flow_weight_per_match <= data_flow_weight_cap {
            data_flow_total += data_flow_weight_per_match;
            reasons.push(format!("Data-flow coupling: chain {} affected ({:.0}% change)", m.chain_label, m.change_pct * 100.0));
            debug!("Risk Factor: Data-flow match ({}) +{}", m.chain_label, data_flow_weight_per_match);
        }
    }
    total_weight += data_flow_total;

    // 4f. Deploy Manifest Changes
    let deploy_weight_per_manifest = config.coverage.deploy.risk_weight_per_manifest;
    let deploy_weight_cap = config.coverage.deploy.risk_cap;
    let mut deploy_total = 0;
    for change in &packet.deploy_manifest_changes {
        if deploy_total + deploy_weight_per_manifest <= deploy_weight_cap {
            deploy_total += deploy_weight_per_manifest;
            reasons.push(format!("Deploy manifest changed: {:?}", change.file));
            debug!("Risk Factor: Deploy manifest changed ({:?}) +{}", change.file, deploy_weight_per_manifest);
        }
    }
    total_weight += deploy_total;

    // 4g. CI Self-Awareness
    let ci_config_changed = packet.changes.iter().any(|c| !c.ci_gates.is_empty());
    let source_changed = packet.changes.iter().any(|c| c.symbols.is_some() || c.imports.is_some());
    if ci_config_changed && config.coverage.ci_self_awareness.enabled {
        let ci_weight = if source_changed { config.coverage.ci_self_awareness.ci_plus_source_weight } else { config.coverage.ci_self_awareness.ci_changed_weight };
        total_weight += ci_weight;
        reasons.push(format!("CI pipeline config change{}", if source_changed { " + source code" } else { "" }));
        debug!("Risk Factor: CI self-awareness (source={}) +{}", source_changed, ci_weight);
    }

    // 4h. ADR Staleness Advisory
    if config.coverage.adr_staleness.enabled {
        let threshold = config.coverage.adr_staleness.threshold_days;
        for decision in &packet.relevant_decisions {
            if let Some(days) = decision.staleness_days {
                if days > threshold {
                    reasons.push(format!("Stale architectural context: {} ({} days old)", decision.file_path.display(), days));
                    debug!("Advisory: Stale ADR ({}) {} days", decision.file_path.display(), days);
                }
            }
        }
    }


    // 3m. Runtime/Config Dependency Risk
    // Category Cap: 25 points
    let mut runtime_config_total = 0;
    let runtime_config_cap = 25;

    // 1. New env var dependencies (+20)
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

    // 2. Env var reference changes (+10) and Config key reference changes (+10 or +5)
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
            let mut weight = 10;

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
                    weight = 5;
                }
            }

            reasons.push(format!(
                "Configuration key references changed in {}",
                delta.file_path
            ));
            if runtime_config_total + weight <= runtime_config_cap {
                runtime_config_total += weight;
            } else if runtime_config_total < runtime_config_cap {
                runtime_config_total = runtime_config_cap;
            }
        }
    }

    total_weight += runtime_config_total;

    // 4. Scoring
    packet.risk_level = if total_weight > 60 {
        RiskLevel::High
    } else if total_weight > 20 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };

    if reasons.is_empty() && packet.risk_reasons.is_empty() {
        packet.risk_reasons.push("Minimal changes detected".to_string());
    } else {
        packet.risk_reasons.extend(reasons);
    }

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

        // Should have one CI pipeline reason (it's not per-file anymore, but global)
        let ci_reasons: Vec<_> = packet
            .risk_reasons
            .iter()
            .filter(|r| r.contains("CI pipeline config change"))
            .collect();
        assert_eq!(
            ci_reasons.len(),
            1,
            "expected 1 CI pipeline config change reason, got {:?}",
            ci_reasons
        );

        // 3 weight (alone) -> Low
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

        assert!(packet.risk_reasons.iter().any(|r| r.contains("Observability config drift")));
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

        assert!(packet.risk_reasons.iter().any(|r| r.contains("New SDK dependency: opentelemetry")));
        assert!(packet.risk_reasons.iter().any(|r| r.contains("Modified SDK dependency: sentry")));
        // New(5) + Mod(3) = 8
        assert_eq!(packet.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_risk_service_map_delta() {
        use crate::impact::packet::ServiceMapDelta;
        let mut packet = ImpactPacket::default();
        packet.service_map_delta = Some(ServiceMapDelta {
            affected_services: vec!["users".to_string(), "billing".to_string(), "auth".to_string()],
            services: vec![],
            cross_service_edges: vec![],
            total_services: 3,
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(packet.risk_reasons.iter().any(|r| r.contains("Cross-service change affecting 3 services")));
        // 3 services -> weight 6
        assert_eq!(packet.risk_level, RiskLevel::Low);
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

        assert!(packet.risk_reasons.iter().any(|r| r.contains("Data-flow coupling: chain GET /users -> User affected")));
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
            risk_weight: 3,
            is_deleted: false,
        });

        let rules = Rules::default();
        let config = Config::default();
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(packet.risk_reasons.iter().any(|r| r.contains("Deployment manifest change: Dockerfile")));
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
        });

        let rules = Rules::default();
        let mut config = Config::default();
        config.coverage.adr_staleness.threshold_days = 365;
        config.coverage.adr_staleness.enabled = true;
        
        analyze_risk(&mut packet, &rules, &config).unwrap();

        assert!(packet.risk_reasons.iter().any(|r| r.contains("Stale architectural context: docs/adr/001-auth.md (400 days old)")));
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
            affected_services: vec!["s1".to_string(), "s2".to_string(), "s3".to_string(), "s4".to_string(), "s5".to_string()],
            services: vec![],
            cross_service_edges: vec![],
            total_services: 5,
        });
        
        // 3. Data flow matches (12) - 3 matches at 4 each
        for i in 0..3 {
            packet.data_flow_matches.push(crate::impact::packet::DataFlowMatch {
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
}
