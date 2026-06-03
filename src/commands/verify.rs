use crate::output::human::print_verify_plan;
use crate::output::verification::VerificationReporter;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::verify::engine::{VerificationContext, VerifyEngine};
use crate::verify::plan::{VerificationStep, build_plan, build_plan_from_config};
use crate::verify::predictor::OutcomePredictor;
use crate::verify::suggestions::{
    generate_health_suggestions, generate_suggestions, query_ledger_status,
};
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
        println!("Ledger is empty. No signatures to verify.");
        return Ok(());
    }

    println!(
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
                    println!(
                        "  [{}] TX {} signed by {}",
                        "VALID".green(),
                        &entry.tx_id[..8],
                        &pub_key[..8]
                    );
                    valid_count += 1;
                } else {
                    println!(
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
                    println!(
                        "  [{}] TX {} has no signature — treating as verification failure.",
                        "UNSIGNED".yellow(),
                        &entry.tx_id[..8]
                    );
                    invalid_count += 1;
                    all_valid = false;
                } else {
                    println!(
                        "  [{}] TX {} has no signature (signing not required, skipping).",
                        "SKIP".yellow(),
                        &entry.tx_id[..8]
                    );
                    skipped_count += 1;
                }
            }
        }
    }

    println!(
        "\nSignature verification summary: {} valid, {} invalid, {} skipped.",
        valid_count.green(),
        if invalid_count > 0 { invalid_count.red().to_string() } else { invalid_count.to_string() },
        skipped_count.yellow()
    );

    if all_valid {
        println!(
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

    // 3. Build Plan
    let (plan, steps) = match command_str {
        Some(cmd) => (None, vec![manual_step(cmd, manual_timeout(timeout_secs))]),
        None => {
            if let Some(config_plan) = build_plan_from_config(&config.verify) {
                print_verify_plan(&config_plan);
                (Some(config_plan.clone()), config_plan.steps)
            } else {
                let prediction = OutcomePredictor::predict(&mut ctx)?;
                let rules = crate::policy::load::load_rules(&layout)?;

                let mut plan = match &ctx.packet {
                    Some(packet) => build_plan(packet, &rules, &prediction.files),
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

    // Dry Run early exit
    if dry_run {
        println!(
            "{}",
            "Dry run mode: verification plan displayed above. No commands were executed.".yellow()
        );
        return Ok(());
    }

    // Health check validation path
    if health {
        println!("{}", "Verification Health Check".bold().green());
        let mut all_ok = true;
        for step in &steps {
            let exe = extract_executable(&step.command);
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

        let ledger_status = query_ledger_status(&layout);
        let suggestions = generate_health_suggestions(&ledger_status);
        if !suggestions.is_empty() {
            println!("\n{}", "Suggestions:".bold());
            for sugg in suggestions {
                println!("  - {}: {}", sugg.description, sugg.command);
            }
        }

        if all_ok {
            return Ok(());
        } else {
            return Err(miette::miette!(
                "Verification health check failed: some executables are missing."
            ));
        }
    }

    // 4. Execute
    // Explicitly release the database connection and close locks before running verification commands.
    // This prevents deadlock/lock contention when cargo test runs child ChangeGuard commands.
    if let Some(storage) = ctx.storage.take() {
        let _ = storage.shutdown();
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
        crate::verify::plan::VerificationPlan {
            steps: vec![VerificationStep {
                description: "Default fallback verification".to_string(),
                command: "cargo test -j 1 -- --test-threads=1".to_string(),
                timeout_secs: DEFAULT_AUTO_TIMEOUT_SECS,
            }],
        }
    }
}
