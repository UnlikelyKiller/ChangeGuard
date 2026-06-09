use crate::output::human::print_verify_plan;
use crate::output::verification::VerificationReporter;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::verify::engine::{VerificationContext, VerifyEngine};
use crate::verify::plan::{VerificationStep, build_plan, build_plan_from_config};
use crate::verify::predictor::OutcomePredictor;
use crate::verify::suggestions::{generate_suggestions, query_ledger_status};
use crate::verify::timeouts::{DEFAULT_AUTO_TIMEOUT_SECS, manual_timeout};
use miette::Result;
use owo_colors::OwoColorize;
use std::env;
use tracing::{info, warn};

pub fn verify_ledger_signatures(layout: &Layout) -> Result<()> {
    let db_path = layout.state_subdir().join("ledger.db");
    let mut storage = StorageManager::init(db_path.as_std_path())?;
    let db = crate::ledger::db::LedgerDb::new(storage.get_connection_mut());

    // Load config to determine whether signing is required.
    let config = crate::config::load::load_config(layout).unwrap_or_default();
    let signing_required = config.intent.require_signing;

    let entries = db
        .get_all_committed_ledger_entries()
        .map_err(|e| miette::miette!("Failed to read ledger entries: {}", e))?;

    if entries.is_empty() {
        eprintln!("Ledger is empty. No signatures to verify.");
        return Ok(());
    }

    eprintln!(
        "Verifying signatures for {} ledger entries (require_signing={})...",
        entries.len(),
        signing_required
    );
    let mut all_valid = true;
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut skipped_count = 0;

    for entry in &entries {
        match (&entry.signature, &entry.public_key) {
            (Some(sig), Some(pub_key)) => {
                let valid = crate::ledger::crypto::verify_signature(
                    &entry.tx_id,
                    &entry.category.to_string(),
                    &entry.summary,
                    &entry.reason,
                    &entry.committed_at,
                    sig,
                    pub_key,
                );
                if valid {
                    eprintln!(
                        "  [{}] TX {} signed by {}",
                        "VALID".green(),
                        &entry.tx_id[..8],
                        &pub_key[..8]
                    );
                    valid_count += 1;
                } else {
                    eprintln!(
                        "  [{}] TX {} signature verification FAILED!",
                        "INVALID".red(),
                        &entry.tx_id[..8]
                    );
                    invalid_count += 1;
                    all_valid = false;
                }
            }
            _ => {
                if signing_required {
                    eprintln!(
                        "  [{}] TX {} has no signature — treating as verification failure.",
                        "UNSIGNED".yellow(),
                        &entry.tx_id[..8]
                    );
                    invalid_count += 1;
                    all_valid = false;
                } else {
                    eprintln!(
                        "  [{}] TX {} has no signature (signing not required, skipping).",
                        "SKIP".yellow(),
                        &entry.tx_id[..8]
                    );
                    skipped_count += 1;
                }
            }
        }
    }

    eprintln!(
        "\nSignature verification summary: {} valid, {} invalid, {} skipped.",
        valid_count.green(),
        if invalid_count > 0 {
            invalid_count.red().to_string()
        } else {
            invalid_count.to_string()
        },
        skipped_count.yellow()
    );

    if all_valid {
        eprintln!(
            "{}",
            "All signature validations passed successfully!"
                .green()
                .bold()
        );
        Ok(())
    } else {
        Err(miette::miette!(
            "Ledger signature verification failed: {} entries have invalid or missing signatures.",
            invalid_count
        ))
    }
}

pub fn execute_verify(
    command_str: Option<String>,
    timeout_secs: u64,
    no_predict: bool,
    explain: bool,
    entity: Option<String>,
    health: bool,
    dry_run: bool,
) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let manual_requested = command_str.is_some();

    // 1. Initialize Context
    let config = crate::config::load::load_config(&layout).unwrap_or_else(|e| {
        warn!("Config load failed: {e}. Using defaults.");
        crate::config::model::Config::default()
    });

    let mut ctx = VerificationContext::new(
        layout.clone(),
        current_dir.clone(),
        config.clone(),
        no_predict,
        explain,
        health,
    );

    // 2. Load Storage and Packet
    ctx.storage = match StorageManager::open_read_only(&layout.root) {
        Ok(storage) => Some(storage),
        Err(err) => {
            if !no_predict {
                let warning =
                    format!("Prediction disabled: failed to initialize SQLite storage: {err}");
                warn!("{warning}");
                ctx.add_warning(warning);
            }
            None
        }
    };

    if let Some(storage) = &ctx.storage {
        ctx.packet = match storage.get_latest_packet() {
            Ok(packet) => packet,
            Err(err) => {
                if !no_predict {
                    let warning =
                        format!("Prediction disabled: failed to load latest packet: {err}");
                    warn!("{warning}");
                    ctx.add_warning(warning);
                }
                None
            }
        };
    }

    // Health mode early exit — skip OutcomePredictor::predict and full plan building
    if health {
        return execute_verify_health(&layout, &config);
    }

    // 3. Build Plan
    let (plan, steps) = match command_str {
        Some(ref cmd) => (
            None,
            vec![manual_step(cmd.clone(), manual_timeout(timeout_secs))],
        ),
        None => {
            if let Some(config_plan) = build_plan_from_config(&config.verify) {
                print_verify_plan(&config_plan);
                (Some(config_plan.clone()), config_plan.steps)
            } else {
                let prediction = OutcomePredictor::predict(&mut ctx)?;
                let rules = crate::policy::load::load_rules(&layout)?;

                let mut plan = match &ctx.packet {
                    Some(packet) => build_plan(
                        packet,
                        &rules,
                        &prediction.files,
                        config.verify.prefer_nextest,
                    ),
                    None => VerificationPlan::default_manual(),
                };

                // Apply probabilistic ordering if storage is available
                if let Some(stg) = &ctx.storage
                    && let Ok(dataset) =
                        crate::verify::probability::extract_dataset(stg.get_connection())
                {
                    let probs = crate::verify::probability::calculate_probabilities(&dataset);
                    plan.apply_probability_ordering(&probs);
                    info!(
                        "Probabilistic verification ordering applied ({} active models).",
                        probs.len()
                    );
                }

                print_verify_plan(&plan);
                let steps = plan.steps.clone();
                (Some(plan), steps)
            }
        }
    };

    // Entity-scoped explanation: show tests mapped to the entity and relevant steps.
    if explain && entity.is_some() {
        let target = entity.as_deref().unwrap_or("");
        println!(
            "\n{}",
            format!("Verification explanation for entity: {}", target)
                .bold()
                .cyan()
        );

        if let Some(storage) = &ctx.storage {
            let conn = storage.get_connection();
            let mapped: Vec<String> = conn
                .prepare(
                    "SELECT DISTINCT tm.test_name FROM test_mappings tm \
                     JOIN project_files pf ON tm.source_file_id = pf.id \
                     WHERE pf.file_path LIKE ?1 OR pf.file_path = ?1",
                )
                .and_then(|mut s| {
                    s.query_map([format!("%{}%", target)], |row| row.get(0))
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                })
                .unwrap_or_default();

            if mapped.is_empty() {
                println!(
                    "  No test mappings found for '{}'. Run `changeguard index --incremental` to refresh.",
                    target
                );
            } else {
                println!("  Mapped tests ({}):", mapped.len());
                for t in &mapped {
                    println!("    • {}", t);
                }
            }
        }

        let relevant: Vec<_> = steps
            .iter()
            .filter(|s| {
                let cmd = s.command.to_lowercase();
                let t = target.to_lowercase();
                cmd.contains(&t) || cmd.contains("test") || cmd.contains("check")
            })
            .collect();
        println!(
            "\n  Verification steps relevant to this entity ({}):",
            relevant.len()
        );
        for s in &relevant {
            println!("    • {} (timeout: {}s)", s.command, s.timeout_secs);
        }
        println!();
    }

    // Dry Run early exit with compressed output
    if dry_run {
        // For manual commands, print the steps derived from the CLI arg
        if manual_requested {
            println!("{}", "Verification Plan".bold().green());
            println!(
                "  • {} (timeout: {}s)",
                command_str.as_deref().unwrap_or(""),
                timeout_secs
            );
            println!();
        }

        // Group predicted impacts by source for compressed output
        let verbose = std::env::var("VERBOSE_DRY_RUN").is_ok();
        let predicted: Vec<&VerificationStep> = steps
            .iter()
            .filter(|s| s.description.starts_with("Predicted impact"))
            .collect();
        let other: Vec<&VerificationStep> = steps
            .iter()
            .filter(|s| !s.description.starts_with("Predicted impact"))
            .collect();

        // Print non-predicted steps (rules, config)
        if !other.is_empty() {
            println!("{}", "Verification Steps:".bold().cyan());
            for step in &other {
                println!("  • {} (timeout: {}s)", step.command, step.timeout_secs);
            }
        }

        // Print compressed predicted impacts
        if !predicted.is_empty() {
            println!(
                "\n{}",
                "Predicted Impacts (grouped by source):".bold().cyan()
            );
            let mut groups: std::collections::BTreeMap<String, Vec<String>> =
                std::collections::BTreeMap::new();
            for step in &predicted {
                // Extract group name from "Predicted impact (GroupName) on path"
                let desc = &step.description;
                if let Some(start) = desc.find('(')
                    && let Some(end) = desc.find(')')
                {
                    let group = desc[start + 1..end].to_string();
                    let path = desc[end + 5..].to_string(); // ") on " = 5 chars
                    groups.entry(group).or_default().push(path);
                }
            }

            for (source, paths) in &groups {
                println!(
                    "  {}",
                    format!("Source: {} — {} items", source, paths.len()).bold()
                );
                let show = if verbose {
                    paths.len()
                } else {
                    std::cmp::min(5, paths.len())
                };
                for path in paths.iter().take(show) {
                    println!("    • {}", path);
                }
                if !verbose && paths.len() > 5 {
                    println!(
                        "    ... and {} more (set VERBOSE_DRY_RUN=1 for full list)",
                        paths.len() - 5
                    );
                }
            }
        }

        println!(
            "\n{}",
            "Dry run mode: verification plan displayed above. No commands were executed.".yellow()
        );
        return Ok(());
    }

    // 4. Execute
    // Explicitly release the database connection and close locks before running verification commands.
    // This prevents deadlock/lock contention when cargo test runs child ChangeGuard commands.
    if let Some(storage) = ctx.storage.take() {
        let _ = storage.shutdown();
    }

    // Show progress indicator before verification execution
    if !ctx.no_predict {
        let num_steps = steps.len();
        if num_steps > 0 {
            eprintln!("Running {} verification step(s)...", num_steps);
        }
    }

    let mut report = VerifyEngine::execute(&mut ctx, plan, &steps, manual_requested)?;

    // 5. Generate Suggestions
    let ledger_status = query_ledger_status(&layout);
    let suggestions = generate_suggestions(&report, &ledger_status);

    report = report.with_suggested_actions(suggestions);

    // 6. Final Reporting & IPC
    VerificationReporter::report(&ctx, &report);

    // Push results to AI-Brains
    let bridge_outcomes = report
        .results
        .iter()
        .map(|res| crate::bridge::model::BridgeVerifyOutcome {
            success: res.exit_code == 0,
            command: res.command.clone(),
            error_snippet: if res.exit_code != 0 {
                let err = if !res.stderr_summary.is_empty() {
                    &res.stderr_summary
                } else {
                    &res.stdout_summary
                };
                Some(err.chars().take(200).collect::<String>())
            } else {
                None
            },
        })
        .collect();
    crate::bridge::notify::push_verify_results(bridge_outcomes);

    if report.overall_pass {
        Ok(())
    } else {
        Err(miette::miette!("Verification failed"))
    }
}

/// Fast health check that only probes executable availability and basic ledger
/// state, skipping OutcomePredictor::predict and full plan building entirely.
/// Returns within a bounded time (<5s on normal machines).
fn execute_verify_health(layout: &Layout, config: &crate::config::model::Config) -> Result<()> {
    println!("{}", "Verification Health Check".bold().green());
    eprintln!("Checking verification dependencies...");
    let mut all_ok = true;

    if let Some(config_plan) = build_plan_from_config(&config.verify) {
        for step in &config_plan.steps {
            let exe = extract_executable(&step.command);
            eprintln!("  Checking {}...", exe);
            let exists = check_executable_exists(exe);
            if exists {
                println!(
                    "  [{}] Command '{}' is available.",
                    "OK".green(),
                    step.command
                );
            } else {
                println!(
                    "  [{}] Executable '{}' for command '{}' NOT found on PATH.",
                    "FAILED".red(),
                    exe,
                    step.command
                );
                all_ok = false;
            }
        }
    } else {
        // Auto-detect common tools
        let common_tools = ["cargo", "cargo-nextest", "python", "npm"];
        for tool in &common_tools {
            eprintln!("  Checking {}...", tool);
            let exists = check_executable_exists(tool);
            if exists {
                println!("  [{}] {} is available.", "OK".green(), tool);
            } else {
                println!("  [{}] {} not found.", "-".dimmed(), tool);
            }
        }
    }

    // Check ledger health (bounded query)
    eprintln!("  Checking ledger state...");
    let ledger_status = query_ledger_status(layout);
    if ledger_status.unaudited_count > 0 || ledger_status.has_stale_pending {
        println!(
            "  [{}] Ledger: {} unaudited, stale pending: {}",
            "NOTE".yellow(),
            ledger_status.unaudited_count,
            ledger_status.has_stale_pending
        );
    } else if ledger_status.no_impact_report {
        println!(
            "  [{}] No impact report found. Run 'changeguard scan --impact' after making changes.",
            "NOTE".yellow()
        );
    } else {
        println!("  [{}] Ledger is clean.", "OK".green());
    }

    // Show runner selection info
    let has_nextest = check_executable_exists("cargo-nextest");
    let prefer_nextest = has_nextest && config.verify.prefer_nextest.unwrap_or(false);
    println!(
        "  [{}] Runner: {} (nextest {})",
        "OK".green(),
        if prefer_nextest {
            "cargo nextest"
        } else {
            "cargo test"
        },
        if has_nextest {
            "available"
        } else {
            "not available"
        }
    );

    if all_ok {
        println!(
            "\n{}",
            "All verification dependencies are available.".green()
        );
        Ok(())
    } else {
        Err(miette::miette!(
            "Verification health check failed: some executables are missing."
        ))
    }
}

fn extract_executable(command: &str) -> &str {
    // Skip leading `KEY=value` tokens to reach the actual executable.
    // e.g. `CARGO_TERM_COLOR=always cargo test` -> `cargo`
    let exe_token = command
        .split_whitespace()
        .find(|tok| !tok.contains('='))
        .unwrap_or("");
    // Strip surrounding quotes from the token if present.
    exe_token
        .trim_start_matches(['\"', '\''])
        .trim_end_matches(['\"', '\''])
}

fn check_executable_exists(name: &str) -> bool {
    let path = std::path::Path::new(name);
    if path.is_absolute() || path.components().count() > 1 {
        return path.exists();
    }
    if let Ok(path_env) = std::env::var("PATH") {
        let paths = std::env::split_paths(&path_env);
        for p in paths {
            let exe_path = p.join(name);
            #[cfg(target_os = "windows")]
            {
                for ext in &["", ".exe", ".cmd", ".bat"] {
                    let full_path = if ext.is_empty() {
                        exe_path.clone()
                    } else {
                        let mut s = exe_path.to_string_lossy().to_string();
                        s.push_str(ext);
                        std::path::PathBuf::from(s)
                    };
                    if full_path.is_file() {
                        return true;
                    }
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                if exe_path.is_file() {
                    return true;
                }
            }
        }
    }
    false
}

fn manual_step(command: String, timeout_secs: u64) -> VerificationStep {
    VerificationStep {
        description: "Manually requested verification command".to_string(),
        command,
        timeout_secs,
    }
}

struct VerificationPlan;
impl VerificationPlan {
    fn default_manual() -> crate::verify::plan::VerificationPlan {
        let cmd = crate::verify::plan::resolve_default_test_command(None);
        crate::verify::plan::VerificationPlan {
            steps: vec![VerificationStep {
                description: "Default fallback verification".to_string(),
                command: cmd,
                timeout_secs: DEFAULT_AUTO_TIMEOUT_SECS,
            }],
        }
    }
}
