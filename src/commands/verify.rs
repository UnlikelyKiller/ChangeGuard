use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::verify::engine::{VerificationContext, VerifyEngine};
use crate::verify::plan::{build_plan, build_plan_from_config, VerificationStep};
use crate::verify::predictor::OutcomePredictor;
use crate::verify::suggestions::{generate_health_suggestions, generate_suggestions, query_ledger_status};
use crate::verify::timeouts::{DEFAULT_AUTO_TIMEOUT_SECS, manual_timeout};
use crate::output::human::print_verify_plan;
use crate::output::verification::VerificationReporter;
use miette::Result;
use tracing::{info, warn};
use std::env;

pub fn execute_verify(
    command_str: Option<String>,
    timeout_secs: u64,
    no_predict: bool,
    explain: bool,
    health: bool,
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
                let warning = format!("Prediction disabled: failed to initialize SQLite storage: {err}");
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
                    let warning = format!("Prediction disabled: failed to load latest packet: {err}");
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
                if let Some(stg) = &ctx.storage {
                    if let Ok(dataset) = crate::verify::probability::extract_dataset(stg.get_connection()) {
                        let probs = crate::verify::probability::calculate_probabilities(&dataset);
                        plan.apply_probability_ordering(&probs);
                        info!("Probabilistic verification ordering applied ({} active models).", probs.len());
                    }
                }

                print_verify_plan(&plan);
                let steps = plan.steps.clone();
                (Some(plan), steps)
            }
        }
    };

    // 4. Execute
    let mut report = VerifyEngine::execute(&mut ctx, plan, &steps, manual_requested)?;

    // 5. Generate Suggestions
    let ledger_status = query_ledger_status(&layout);
    let suggestions = if health {
        generate_health_suggestions(&ledger_status)
    } else {
        generate_suggestions(&report, &ledger_status)
    };

    report = report.with_suggested_actions(suggestions);

    // 6. Final Reporting & IPC
    VerificationReporter::report(&ctx, &report);
    
    // Push results to AI-Brains
    let bridge_outcomes = report.results
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
