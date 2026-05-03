use crate::commands::CommandError;
use crate::exec::ExecutionResult;
use crate::output::human::{print_verify_plan, print_verify_result};
use crate::platform::process_policy::ProcessPolicy;
use crate::policy::load::load_rules;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::verify::plan::{VerificationPlan, VerificationStep, build_plan, build_plan_from_config};
use crate::verify::results::{VerificationReport, VerificationResult, write_verify_report};
use crate::verify::runner::{execute_step, prepare_manual_step, prepare_rule_step};
use crate::verify::timeouts::{DEFAULT_AUTO_TIMEOUT_SECS, manual_timeout};
use chrono::Utc;
use miette::Result;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub fn execute_verify(
    command_str: Option<String>,
    timeout_secs: u64,
    no_predict: bool,
) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let manual_requested = command_str.is_some();

    let mut current_warnings = Vec::new();
    let mut saved_packet: Option<crate::impact::packet::ImpactPacket> = None;
    let mut saved_config: Option<crate::config::model::Config> = None;
    let (plan, steps) = match command_str {
        Some(cmd) => (None, vec![manual_step(cmd, manual_timeout(timeout_secs))]),
        None => {
            let config = crate::config::load::load_config(&layout).unwrap_or_else(|e| {
                let warning = format!("Config load failed: {e}. Using defaults.");
                warn!("{warning}");
                current_warnings.push(warning);
                crate::config::model::Config::default()
            });
            saved_config = Some(config.clone());

            // Priority 2: config-defined verify steps
            if let Some(config_plan) = build_plan_from_config(&config.verify) {
                print_verify_plan(&config_plan);
                (Some(config_plan.clone()), config_plan.steps)
            } else {
                // Priority 3: predictive mode (existing logic)
                let db_path = layout.state_subdir().join("ledger.db");
                let storage = match StorageManager::init(db_path.as_std_path()) {
                    Ok(storage) => Some(storage),
                    Err(err) => {
                        let warning = format!(
                            "Prediction disabled: failed to initialize SQLite storage: {err}"
                        );
                        warn!("{warning}");
                        current_warnings.push(warning);
                        None
                    }
                };
                let mut packet = match storage.as_ref() {
                    Some(storage) => match storage.get_latest_packet() {
                        Ok(packet) => packet,
                        Err(err) => {
                            let warning =
                                format!("Prediction disabled: failed to load latest packet: {err}");
                            warn!("{warning}");
                            current_warnings.push(warning);
                            None
                        }
                    },
                    None => None,
                };

                let rules = load_rules(&layout)?;
                let prediction = if no_predict {
                    crate::verify::predict::PredictionResult::default()
                } else {
                    if let Some(packet) = &mut packet {
                        recompute_temporal_if_missing(
                            packet,
                            &current_dir,
                            &layout,
                            &mut current_warnings,
                        );
                    }

                    match &packet {
                        Some(p) => {
                            let history = match storage.as_ref() {
                                Some(storage) => match storage.get_all_packets() {
                                    Ok(history) => history,
                                    Err(err) => {
                                        let warning = format!(
                                            "Historical prediction degraded: failed to load packet history: {err}"
                                        );
                                        warn!("{warning}");
                                        current_warnings.push(warning);
                                        Vec::new()
                                    }
                                },
                                None => Vec::new(),
                            };

                            let current_imports = match scan_current_imports(&current_dir) {
                                Ok(imports) => imports,
                                Err(err) => {
                                    let warning = format!(
                                        "Current structural prediction degraded: failed to scan repository imports: {err}"
                                    );
                                    warn!("{warning}");
                                    current_warnings.push(warning);
                                    BTreeMap::new()
                                }
                            };

                            let call_data = match storage.as_ref() {
                                Some(storage) => {
                                    fetch_structural_call_data(p, storage, &mut current_warnings)
                                }
                                None => crate::verify::predict::StructuralCallData::default(),
                            };

                            let test_mapping_data = match storage.as_ref() {
                                Some(storage) => {
                                    fetch_test_mapping_data(p, storage, &mut current_warnings)
                                }
                                None => crate::verify::predict::TestMappingData::default(),
                            };

                            crate::verify::predict::Predictor::predict_with_test_mappings(
                                p,
                                &history,
                                &current_imports,
                                &call_data,
                                &test_mapping_data,
                            )
                        }
                        None => crate::verify::predict::PredictionResult::default(),
                    }
                };

                for warning in &prediction.warnings {
                    warn!("{}", warning);
                    current_warnings.push(warning.clone());
                }

                let plan = match &packet {
                    Some(packet) => build_plan(packet, &rules, &prediction.files),
                    None => VerificationPlan {
                        steps: vec![manual_step(
                            "cargo test -j 1 -- --test-threads=1".to_string(),
                            DEFAULT_AUTO_TIMEOUT_SECS,
                        )],
                    },
                };
                print_verify_plan(&plan);
                let steps = plan.steps.clone();
                saved_packet = packet.clone();
                (Some(plan), steps)
            }
        }
    };

    let mut persisted_results = Vec::new();
    let mut final_error: Option<miette::Report> = None;
    let policy = ProcessPolicy::default();

    for step in &steps {
        let prepared = if manual_requested {
            prepare_manual_step(step)
        } else {
            prepare_rule_step(step)
        };
        info!(
            "Running verification command via {:?}: {}",
            prepared.execution_mode, prepared.display_command
        );
        let result = execute_step(&prepared, &policy)?;
        print_verify_result(&prepared.display_command, step.timeout_secs, &result);

        persisted_results.push(to_report_result(&prepared.display_command, &result));

        if result.exit_code != 0 && final_error.is_none() {
            final_error = Some(
                CommandError::Verify(format!("Process exited with code {}", result.exit_code))
                    .into(),
            );
        }
    }

    let report = VerificationReport::new(plan.clone(), persisted_results.clone())
        .with_warnings(current_warnings);
    write_verify_report(&layout, &report)?;
    persist_verify_report(&layout, &report);

    record_semantic_test_outcomes(&layout, &saved_config, &saved_packet, &persisted_results);

    if let Some(error) = final_error {
        Err(error)
    } else {
        Ok(())
    }
}

fn record_semantic_test_outcomes(
    layout: &Layout,
    config: &Option<crate::config::model::Config>,
    packet: &Option<crate::impact::packet::ImpactPacket>,
    results: &[VerificationResult],
) {
    let (Some(config), Some(packet)) = (config, packet) else {
        return;
    };

    let db_path = layout.state_subdir().join("ledger.db");
    let Ok(storage) = StorageManager::init(db_path.as_std_path()) else {
        warn!("Failed to open storage for semantic test outcome recording");
        return;
    };

    let diff_text = crate::verify::semantic_predictor::build_diff_text(packet);
    let diff_summary: String = diff_text.chars().take(200).collect();
    let commit_hash = packet.head_hash.clone().unwrap_or_default();

    let outcomes: Vec<crate::verify::semantic_predictor::TestOutcome> = results
        .iter()
        .map(|r| crate::verify::semantic_predictor::TestOutcome {
            test_name: r.command.clone(),
            test_file: r.command.clone(),
            commit_hash: commit_hash.clone(),
            status: if r.exit_code == 0 {
                crate::verify::semantic_predictor::TestStatus::Passed
            } else {
                crate::verify::semantic_predictor::TestStatus::Failed
            },
            duration_ms: r.duration_ms,
            diff_summary: diff_summary.clone(),
        })
        .collect();

    if let Err(e) = crate::verify::semantic_predictor::record_test_outcomes(
        storage.get_connection(),
        &config.local_model,
        &outcomes,
        &diff_text,
    ) {
        warn!("Failed to record test outcomes for semantic prediction: {e}");
    }
}

fn manual_step(command: String, timeout_secs: u64) -> VerificationStep {
    VerificationStep {
        description: "Manually requested verification command".to_string(),
        command,
        timeout_secs,
    }
}

fn to_report_result(command: &str, result: &ExecutionResult) -> VerificationResult {
    VerificationResult {
        command: command.to_string(),
        exit_code: result.exit_code,
        duration_ms: result.duration.as_millis() as u64,
        stdout_summary: truncate_summary(&result.stdout),
        stderr_summary: truncate_summary(&result.stderr),
        truncated: result.truncated,
        timestamp: Utc::now().to_rfc3339(),
    }
}

fn truncate_summary(output: &str) -> String {
    output.chars().take(500).collect()
}

fn persist_verify_report(layout: &Layout, report: &VerificationReport) {
    let db_path = layout.state_subdir().join("ledger.db");
    let Ok(storage) = StorageManager::init(db_path.as_std_path()) else {
        warn!("Could not initialize SQLite for verification report persistence");
        return;
    };

    let plan_json = report
        .plan
        .as_ref()
        .and_then(|plan| serde_json::to_string(plan).ok());

    let Ok(run_id) =
        storage.save_verification_run(&report.timestamp, plan_json.as_deref(), report.overall_pass)
    else {
        warn!("Failed to persist verification run metadata");
        return;
    };

    for result in &report.results {
        if let Err(err) = storage.save_verification_result(
            run_id,
            &result.command,
            result.exit_code,
            result.duration_ms,
            result.truncated,
        ) {
            warn!("Failed to persist verification result: {err}");
        }
    }
}

fn recompute_temporal_if_missing(
    packet: &mut crate::impact::packet::ImpactPacket,
    current_dir: &Path,
    layout: &Layout,
    warnings: &mut Vec<String>,
) {
    if !packet.temporal_couplings.is_empty() || packet.changes.is_empty() {
        return;
    }

    let repo = match crate::git::repo::open_repo(current_dir) {
        Ok(repo) => repo,
        Err(err) => {
            let warning = format!("Temporal prediction degraded: failed to open repository: {err}");
            warn!("{warning}");
            warnings.push(warning);
            return;
        }
    };

    let config = match crate::config::load::load_config(layout) {
        Ok(config) => config,
        Err(err) => {
            let warning = format!("Temporal prediction degraded: failed to load config: {err}");
            warn!("{warning}");
            warnings.push(warning);
            return;
        }
    };

    let provider = crate::impact::temporal::GixHistoryProvider::new(&repo);
    let engine = crate::impact::temporal::TemporalEngine::new(provider, config.temporal);

    match engine.calculate_couplings() {
        Ok(couplings) => {
            packet.temporal_couplings = couplings;
        }
        Err(err) => {
            let warning = format!("Temporal prediction degraded: {err}");
            warn!("{warning}");
            warnings.push(warning);
        }
    }
}

fn fetch_structural_call_data(
    packet: &crate::impact::packet::ImpactPacket,
    storage: &StorageManager,
    _warnings: &mut Vec<String>,
) -> crate::verify::predict::StructuralCallData {
    use rusqlite::OptionalExtension;

    let conn = storage.get_connection();

    // Check if structural_edges table exists and has data
    let has_edges: Option<i64> = match conn
        .query_row("SELECT count(*) FROM structural_edges LIMIT 1", [], |row| {
            row.get::<_, i64>(0)
        })
        .optional()
    {
        Ok(Some(count)) if count > 0 => Some(count),
        Ok(_) => None, // Table exists but is empty
        Err(_) => {
            // Table doesn't exist — graceful degradation
            return crate::verify::predict::StructuralCallData::default();
        }
    };

    if has_edges.is_none() {
        return crate::verify::predict::StructuralCallData::default();
    }

    // Collect changed symbol names
    let changed_symbols: Vec<String> = packet
        .changes
        .iter()
        .filter_map(|f| f.symbols.as_ref())
        .flat_map(|symbols| symbols.iter().map(|s| s.name.clone()))
        .collect();

    if changed_symbols.is_empty() {
        return crate::verify::predict::StructuralCallData::default();
    }

    let mut callers = Vec::new();

    for callee_name in &changed_symbols {
        // Resolved edges
        if let Ok(mut stmt) = conn.prepare(
            "SELECT pf_caller.file_path, ps_caller.symbol_name
             FROM structural_edges se
             JOIN project_symbols ps_caller ON se.caller_symbol_id = ps_caller.id
             JOIN project_files pf_caller ON se.caller_file_id = pf_caller.id
             JOIN project_symbols ps_callee ON se.callee_symbol_id = ps_callee.id
             WHERE ps_callee.symbol_name = ?1
             AND se.callee_symbol_id IS NOT NULL",
        ) && let Ok(rows) = stmt.query_map([callee_name], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                callers.push((PathBuf::from(row.0), row.1, callee_name.clone()));
            }
        }

        // Unresolved edges
        if let Ok(mut stmt) = conn.prepare(
            "SELECT pf_caller.file_path, ps_caller.symbol_name
             FROM structural_edges se
             JOIN project_symbols ps_caller ON se.caller_symbol_id = ps_caller.id
             JOIN project_files pf_caller ON se.caller_file_id = pf_caller.id
             WHERE se.unresolved_callee = ?1",
        ) && let Ok(rows) = stmt.query_map([callee_name], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                // Deduplicate with resolved edges
                let already_exists = callers.iter().any(|(path, sym, callee)| {
                    path == row.0.as_str() && sym == &row.1 && callee == callee_name
                });
                if !already_exists {
                    callers.push((PathBuf::from(&row.0), row.1, callee_name.clone()));
                }
            }
        }
    }

    if callers.is_empty() {
        return crate::verify::predict::StructuralCallData::default();
    }

    crate::verify::predict::StructuralCallData { callers }
}

fn scan_current_imports(
    root: &Path,
) -> Result<BTreeMap<PathBuf, crate::index::references::ImportExport>> {
    let mut imports = BTreeMap::new();
    scan_imports_recursive(root, root, &mut imports)?;
    Ok(imports)
}

fn scan_imports_recursive(
    root: &Path,
    dir: &Path,
    imports: &mut BTreeMap<PathBuf, crate::index::references::ImportExport>,
) -> Result<()> {
    for entry in fs::read_dir(dir)
        .map_err(|err| miette::miette!("failed to read directory {}: {err}", dir.display()))?
    {
        let entry =
            entry.map_err(|err| miette::miette!("failed to read directory entry: {err}"))?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if path.is_dir() {
            if matches!(file_name.as_ref(), ".git" | ".changeguard" | "target") {
                continue;
            }
            scan_imports_recursive(root, &path, imports)?;
            continue;
        }

        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if crate::index::languages::Language::from_extension(extension).is_none() {
            continue;
        }

        let source = fs::read_to_string(&path).map_err(|err| {
            miette::miette!("failed to read source file {}: {err}", path.display())
        })?;
        let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        if let Some(import_export) =
            crate::index::references::extract_import_export(&relative, &source).map_err(|err| {
                miette::miette!("failed to parse imports for {}: {err}", relative.display())
            })?
        {
            imports.insert(relative, import_export);
        }
    }

    Ok(())
}

fn fetch_test_mapping_data(
    packet: &crate::impact::packet::ImpactPacket,
    storage: &StorageManager,
    _warnings: &mut Vec<String>,
) -> crate::verify::predict::TestMappingData {
    use rusqlite::OptionalExtension;
    use std::collections::BTreeMap;

    let conn = storage.get_connection();

    // Gracefully skip if test_mapping table doesn't exist or is empty
    let has_mappings: Option<i64> = match conn
        .query_row("SELECT count(*) FROM test_mapping LIMIT 1", [], |row| {
            row.get::<_, i64>(0)
        })
        .optional()
    {
        Ok(Some(count)) if count > 0 => Some(count),
        Ok(_) => None, // Table exists but is empty
        Err(_) => return crate::verify::predict::TestMappingData::default(), // Table doesn't exist
    };

    if has_mappings.is_none() {
        return crate::verify::predict::TestMappingData::default();
    }

    // Collect changed symbol names
    let changed_symbols: Vec<String> = packet
        .changes
        .iter()
        .filter_map(|f| f.symbols.as_ref())
        .flat_map(|symbols| symbols.iter().map(|s| s.name.clone()))
        .collect();

    if changed_symbols.is_empty() {
        return crate::verify::predict::TestMappingData::default();
    }

    // For each changed symbol, find test files that cover it
    let mut mappings: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for symbol_name in &changed_symbols {
        // Query test_mapping joined with project_symbols and project_files
        // to find test files that cover this symbol
        if let Ok(mut stmt) = conn.prepare(
            "SELECT DISTINCT pf_test.file_path, ps_test.symbol_name
             FROM test_mapping tm
             JOIN project_symbols ps_test ON tm.test_symbol_id = ps_test.id
             JOIN project_files pf_test ON tm.test_file_id = pf_test.id
             JOIN project_symbols ps_tested ON tm.tested_symbol_id = ps_tested.id
             WHERE ps_tested.symbol_name = ?1",
        ) && let Ok(rows) = stmt.query_map([symbol_name], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                mappings.entry(row.0).or_default().insert(row.1);
            }
        }
    }

    crate::verify::predict::TestMappingData { mappings }
}
