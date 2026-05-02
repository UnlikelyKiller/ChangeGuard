use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::git::{ChangeType, RepoSnapshot};
use crate::impact::packet::{
    AnalysisStatus, ChangedFile, FileAnalysisStatus, ImpactPacket, StructuralCoupling,
};
use crate::index::languages::{Language, parse_symbols};
use crate::index::metrics::ComplexityScorer;
use crate::index::references::extract_import_export;
use crate::index::runtime_usage::extract_runtime_usage;
use crate::output::diagnostics::{success_marker, warning_marker};
use crate::output::human::print_impact_summary;
use crate::state::layout::Layout;
use crate::state::reports::write_impact_report;
use crate::util::clock::SystemClock;
use indicatif::{ProgressBar, ProgressStyle};
use miette::Result;
use owo_colors::OwoColorize;
use std::env;
use std::fs;
use std::path::Path;

struct AnalysisOutcome {
    symbols: Option<Vec<crate::index::symbols::Symbol>>,
    imports: Option<crate::index::references::ImportExport>,
    runtime_usage: Option<crate::index::runtime_usage::RuntimeUsage>,
    analysis_status: FileAnalysisStatus,
    analysis_warnings: Vec<String>,
}

pub fn execute_impact(all_parents: bool, summary: bool) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

    let repo = open_repo(&current_dir)?;
    let (head_hash, branch_name) = get_head_info(&repo)?;
    let changes = get_repo_status(&repo)?;

    let is_clean = changes.is_empty();

    let snapshot = RepoSnapshot {
        head_hash,
        branch_name,
        is_clean,
        changes,
    };

    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let mut packet = map_snapshot_to_packet(snapshot, &current_dir)?;

    // Load main config for temporal analysis
    let mut config = crate::config::load::load_config(&layout).unwrap_or_else(|e| {
        tracing::warn!("Failed to load config: {e}. Using defaults.");
        println!(
            "{} Could not load config. Using default temporal analysis settings.",
            warning_marker()
        );
        crate::config::model::Config::default()
    });

    // CLI override
    if all_parents {
        config.temporal.all_parents = true;
    }

    // Run temporal coupling analysis
    let history_provider = crate::impact::temporal::GixHistoryProvider::new(&repo);
    let temporal_engine = crate::impact::temporal::TemporalEngine::new(
        history_provider.clone(),
        config.temporal.clone(),
    );
    match temporal_engine.calculate_couplings() {
        Ok(couplings) => {
            packet.temporal_couplings = couplings;
        }
        Err(e) => {
            tracing::warn!("Temporal analysis failed: {e}");
            println!("{} Temporal analysis skipped: {e}", warning_marker());
        }
    }

    // Load rules and perform risk analysis
    match crate::policy::load::load_rules(&layout) {
        Ok(rules) => {
            if let Err(e) = crate::impact::analysis::analyze_risk(&mut packet, &rules) {
                tracing::warn!("Risk analysis failed: {e}");
                println!(
                    "{} Risk analysis failed. Impact report written without risk scoring.",
                    warning_marker()
                );
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load rules: {e}");
            println!(
                "{} Could not load rules. Impact report written without risk scoring.",
                warning_marker()
            );
        }
    }

    // Finalize and redact BEFORE persisting anywhere
    packet.finalize();
    let redactions = crate::impact::redact::redact_secrets(&mut packet);
    if !redactions.is_empty() {
        tracing::info!("Redacted {} secret(s) from impact packet", redactions.len());
    }

    // Persist to SQLite and run federated analysis
    let db_path = layout.state_subdir().join("ledger.db");
    match crate::state::storage::StorageManager::init(db_path.as_std_path()) {
        Ok(storage) => {
            if let Err(e) = refresh_federated_dependencies(&current_dir, &packet, &storage) {
                tracing::warn!("Federated discovery refresh failed: {e}");
                println!("{} Federated discovery skipped: {e}", warning_marker());
            }

            // Federated Intelligence
            if let Err(e) = crate::federated::impact::check_cross_repo_impact(&mut packet, &storage)
            {
                tracing::warn!("Federated impact analysis failed: {e}");
            }

            // Hotspot Analysis
            match crate::impact::hotspots::calculate_hotspots(
                &storage,
                &history_provider,
                config.hotspots.max_commits,
                config.hotspots.limit,
                config.temporal.all_parents,
                None, // No directory filter in impact command
                None, // No language filter in impact command
            ) {
                Ok(hotspots) => {
                    packet.hotspots = hotspots;
                }
                Err(e) => {
                    tracing::warn!("Hotspot analysis failed: {e}");
                    println!("{} Hotspot analysis skipped: {e}", warning_marker());
                }
            }

            // Structural Coupling Analysis (from call graph)
            if let Err(e) = populate_structural_couplings(&mut packet, &storage) {
                tracing::warn!("Structural coupling analysis failed: {e}");
                // Graceful degradation: packet.structural_couplings stays empty
            }

            if let Err(e) = storage.save_packet(&packet) {
                tracing::warn!("SQLite save failed: {e}");
                println!(
                    "{} Impact report saved to disk but SQLite ledger was not updated. The 'ask' command may not find this report.",
                    warning_marker()
                );
            }
        }
        Err(e) => {
            tracing::warn!("SQLite init failed: {e}");
            println!(
                "{} Could not initialize SQLite. Impact report saved to disk but not persisted to database.",
                warning_marker()
            );
        }
    }

    write_impact_report(&layout, &packet)?;

    if summary {
        crate::output::human::print_impact_brief(&packet);
    } else {
        print_impact_summary(&packet);
    }

    println!(
        "\n{} Wrote impact report to {}",
        success_marker(),
        ".changeguard/reports/latest-impact.json".cyan()
    );

    Ok(())
}

fn refresh_federated_dependencies(
    current_dir: &Path,
    packet: &ImpactPacket,
    storage: &crate::state::storage::StorageManager,
) -> Result<()> {
    let utf8_current_dir = camino::Utf8PathBuf::from_path_buf(current_dir.to_path_buf())
        .map_err(|_| miette::miette!("Invalid UTF-8 path in current directory"))?;
    let scanner = crate::federated::scanner::FederatedScanner::new(utf8_current_dir);
    let (siblings, warnings) = scanner.scan_siblings()?;

    for warning in warnings {
        tracing::warn!("Federated discovery warning: {warning}");
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    for (path, schema) in siblings {
        crate::federated::storage::update_federated_link(
            storage.get_connection(),
            &schema.repo_name,
            path.as_str(),
            &timestamp,
        )?;
        crate::federated::storage::clear_federated_dependencies(
            storage.get_connection(),
            &schema.repo_name,
        )?;
        for (local_symbol, sibling_symbol) in
            scanner.discover_dependencies(packet, &schema.repo_name, &schema)?
        {
            crate::federated::storage::save_federated_dependencies(
                storage.get_connection(),
                &schema.repo_name,
                &local_symbol,
                &sibling_symbol,
            )?;
        }
    }

    Ok(())
}

fn map_snapshot_to_packet(snapshot: RepoSnapshot, base_dir: &Path) -> Result<ImpactPacket> {
    let mut packet = ImpactPacket {
        head_hash: snapshot.head_hash,
        branch_name: snapshot.branch_name,
        ..ImpactPacket::with_clock(&SystemClock)
    };

    let pb = ProgressBar::new(snapshot.changes.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );
    pb.set_message("Extracting symbols...");

    packet.changes = snapshot
        .changes
        .into_iter()
        .map(|c| {
            pb.set_message(format!("Extracting symbols from {}", c.path.display()));
            let status = match c.change_type {
                ChangeType::Added => "Added".to_string(),
                ChangeType::Modified => "Modified".to_string(),
                ChangeType::Deleted => "Deleted".to_string(),
                ChangeType::Renamed { .. } => "Renamed".to_string(),
            };

            let outcome = if matches!(c.change_type, ChangeType::Added | ChangeType::Modified) {
                analyze_changed_file(&c.path, base_dir)
            } else {
                AnalysisOutcome {
                    symbols: None,
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                }
            };

            pb.inc(1);
            ChangedFile {
                path: c.path,
                status,
                is_staged: c.is_staged,
                symbols: outcome.symbols,
                imports: outcome.imports,
                runtime_usage: outcome.runtime_usage,
                analysis_status: outcome.analysis_status,
                analysis_warnings: outcome.analysis_warnings,
            }
        })
        .collect();

    pb.finish_with_message("Symbol extraction complete.");
    Ok(packet)
}

fn analyze_changed_file(relative_path: &Path, base_dir: &Path) -> AnalysisOutcome {
    let full_path = base_dir.join(relative_path);
    let mut warnings = Vec::new();
    let mut status = FileAnalysisStatus::default();

    let Some(extension) = relative_path.extension().and_then(|ext| ext.to_str()) else {
        status.symbols = AnalysisStatus::Unsupported;
        status.imports = AnalysisStatus::Unsupported;
        status.runtime_usage = AnalysisStatus::Unsupported;
        warnings.push(format!(
            "{relative_path:?}: analysis unsupported for files without an extension"
        ));
        return AnalysisOutcome {
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: status,
            analysis_warnings: warnings,
        };
    };

    let supported = matches!(extension, "rs" | "ts" | "tsx" | "js" | "jsx" | "py");
    if !supported {
        status.symbols = AnalysisStatus::Unsupported;
        status.imports = AnalysisStatus::Unsupported;
        status.runtime_usage = AnalysisStatus::Unsupported;
        warnings.push(format!(
            "{}: analysis unsupported for extension .{}",
            relative_path.display(),
            extension
        ));
        return AnalysisOutcome {
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: status,
            analysis_warnings: warnings,
        };
    }

    let content = match fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(err) => {
            status.symbols = AnalysisStatus::ReadFailed;
            status.imports = AnalysisStatus::ReadFailed;
            status.runtime_usage = AnalysisStatus::ReadFailed;
            warnings.push(format!(
                "{}: failed to read file: {}",
                relative_path.display(),
                err
            ));
            return AnalysisOutcome {
                symbols: None,
                imports: None,
                runtime_usage: None,
                analysis_status: status,
                analysis_warnings: warnings,
            };
        }
    };

    let mut symbols = match parse_symbols(relative_path, &content) {
        Ok(symbols) => {
            status.symbols = AnalysisStatus::Ok;
            symbols
        }
        Err(err) => {
            status.symbols = AnalysisStatus::ExtractionFailed;
            warnings.push(format!(
                "{}: symbol extraction failed: {}",
                relative_path.display(),
                err
            ));
            None
        }
    };

    // Integrate Complexity Scoring
    if let (Some(syms), Some(lang)) = (&mut symbols, Language::from_extension(extension)) {
        let scorer = crate::index::metrics::NativeComplexityScorer::new();
        if let Some(path) = camino::Utf8Path::from_path(relative_path) {
            match scorer.score_file(path, &content, lang) {
                Ok(file_complexity) => {
                    for sym in syms {
                        if let Some(symbol_complexity) = file_complexity
                            .functions
                            .iter()
                            .find(|f| f.name == sym.name)
                        {
                            sym.cognitive_complexity = Some(symbol_complexity.cognitive as i32);
                            sym.cyclomatic_complexity = Some(symbol_complexity.cyclomatic as i32);
                        }
                    }
                }
                Err(e) => {
                    warnings.push(format!(
                        "{}: complexity scoring failed: {e}",
                        relative_path.display()
                    ));
                }
            }
        } else {
            warnings.push(format!(
                "{}: complexity scoring skipped: path is not valid UTF-8",
                relative_path.display()
            ));
        }
    }

    let imports = match extract_import_export(relative_path, &content) {
        Ok(imports) => {
            status.imports = AnalysisStatus::Ok;
            imports
        }
        Err(err) => {
            status.imports = AnalysisStatus::ExtractionFailed;
            warnings.push(format!(
                "{}: import/export extraction failed: {}",
                relative_path.display(),
                err
            ));
            None
        }
    };

    status.runtime_usage = AnalysisStatus::Ok;
    let runtime_usage = extract_runtime_usage(relative_path, &content);

    AnalysisOutcome {
        symbols,
        imports,
        runtime_usage,
        analysis_status: status,
        analysis_warnings: warnings,
    }
}

fn populate_structural_couplings(
    packet: &mut ImpactPacket,
    storage: &crate::state::storage::StorageManager,
) -> miette::Result<()> {
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
        Ok(_) => None,           // Table exists but is empty
        Err(_) => return Ok(()), // Table doesn't exist — graceful skip
    };

    if has_edges.is_none() {
        return Ok(()); // Table empty or doesn't exist — graceful skip
    }

    // Collect changed symbol names
    let changed_symbols: Vec<String> = packet
        .changes
        .iter()
        .filter_map(|f| f.symbols.as_ref())
        .flat_map(|symbols| symbols.iter().map(|s| s.name.clone()))
        .collect();

    if changed_symbols.is_empty() {
        return Ok(());
    }

    // For each changed symbol, query structural_edges for callers
    for callee_name in &changed_symbols {
        // Query resolved edges: callee_symbol_id matches a project_symbols row
        let mut stmt = conn
            .prepare(
                "SELECT se.caller_symbol_id, ps_caller.symbol_name, pf_caller.file_path
                 FROM structural_edges se
                 JOIN project_symbols ps_caller ON se.caller_symbol_id = ps_caller.id
                 JOIN project_files pf_caller ON se.caller_file_id = pf_caller.id
                 JOIN project_symbols ps_callee ON se.callee_symbol_id = ps_callee.id
                 WHERE ps_callee.symbol_name = ?1
                 AND se.callee_symbol_id IS NOT NULL",
            )
            .map_err(|e| miette::miette!("Failed to prepare structural edges query: {e}"))?;

        let rows: Vec<(String, String)> = stmt
            .query_map([callee_name], |row| {
                Ok((
                    row.get::<_, String>(1)?, // caller symbol name
                    row.get::<_, String>(2)?, // caller file path
                ))
            })
            .map_err(|e| miette::miette!("Failed to query structural edges: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| miette::miette!("Failed to collect structural edges rows: {e}"))?;

        for (caller_name, caller_file) in rows {
            packet.structural_couplings.push(StructuralCoupling {
                caller_symbol_name: caller_name,
                callee_symbol_name: callee_name.clone(),
                caller_file_path: std::path::PathBuf::from(caller_file),
            });
        }

        drop(stmt);

        // Query unresolved edges: unresolved_callee matches the symbol name
        let mut unresolved_stmt = conn
            .prepare(
                "SELECT se.caller_symbol_id, ps_caller.symbol_name, pf_caller.file_path
                 FROM structural_edges se
                 JOIN project_symbols ps_caller ON se.caller_symbol_id = ps_caller.id
                 JOIN project_files pf_caller ON se.caller_file_id = pf_caller.id
                 WHERE se.unresolved_callee = ?1",
            )
            .map_err(|e| miette::miette!("Failed to prepare unresolved edges query: {e}"))?;

        let unresolved_rows: Vec<(String, String)> = unresolved_stmt
            .query_map([callee_name], |row| {
                Ok((
                    row.get::<_, String>(1)?, // caller symbol name
                    row.get::<_, String>(2)?, // caller file path
                ))
            })
            .map_err(|e| miette::miette!("Failed to query unresolved edges: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| miette::miette!("Failed to collect unresolved edges rows: {e}"))?;

        for (caller_name, caller_file) in unresolved_rows {
            // Avoid duplicates with resolved edges
            let already_exists = packet.structural_couplings.iter().any(|c| {
                c.caller_symbol_name == caller_name
                    && c.callee_symbol_name == *callee_name
                    && c.caller_file_path == caller_file
            });
            if !already_exists {
                packet.structural_couplings.push(StructuralCoupling {
                    caller_symbol_name: caller_name,
                    callee_symbol_name: callee_name.clone(),
                    caller_file_path: std::path::PathBuf::from(caller_file),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn analyze_changed_file_marks_unsupported_extensions() {
        let tmp = tempdir().unwrap();
        let path = Path::new("notes.txt");

        let outcome = analyze_changed_file(path, tmp.path());

        assert_eq!(outcome.analysis_status.symbols, AnalysisStatus::Unsupported);
        assert_eq!(outcome.analysis_status.imports, AnalysisStatus::Unsupported);
        assert_eq!(
            outcome.analysis_status.runtime_usage,
            AnalysisStatus::Unsupported
        );
        assert_eq!(outcome.analysis_warnings.len(), 1);
        assert!(outcome.analysis_warnings[0].contains("unsupported"));
    }

    #[test]
    fn analyze_changed_file_marks_read_failures() {
        let tmp = tempdir().unwrap();
        let path = Path::new("missing.rs");

        let outcome = analyze_changed_file(path, tmp.path());

        assert_eq!(outcome.analysis_status.symbols, AnalysisStatus::ReadFailed);
        assert_eq!(outcome.analysis_status.imports, AnalysisStatus::ReadFailed);
        assert_eq!(
            outcome.analysis_status.runtime_usage,
            AnalysisStatus::ReadFailed
        );
        assert_eq!(outcome.analysis_warnings.len(), 1);
        assert!(outcome.analysis_warnings[0].contains("failed to read file"));
    }
}
