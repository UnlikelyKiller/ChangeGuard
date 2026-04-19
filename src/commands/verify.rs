use crate::commands::CommandError;
use crate::exec::ExecutionResult;
use crate::output::human::{print_verify_plan, print_verify_result};
use crate::platform::process_policy::ProcessPolicy;
use crate::policy::load::load_rules;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::verify::plan::{VerificationPlan, VerificationStep, build_plan};
use crate::verify::results::{VerificationReport, VerificationResult, write_verify_report};
use crate::verify::runner::{execute_step, prepare_manual_step, prepare_rule_step};
use crate::verify::timeouts::{DEFAULT_AUTO_TIMEOUT_SECS, manual_timeout};
use chrono::Utc;
use miette::Result;
use std::env;
use tracing::{info, warn};

pub fn execute_verify(command_str: Option<String>, timeout_secs: u64) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let manual_requested = command_str.is_some();

    let (plan, steps) = match command_str {
        Some(cmd) => (None, vec![manual_step(cmd, manual_timeout(timeout_secs))]),
        None => {
            let db_path = layout.state_subdir().join("ledger.db");
            let packet = match StorageManager::init(db_path.as_std_path()) {
                Ok(storage) => storage.get_latest_packet()?,
                Err(_) => None,
            };

            let rules = load_rules(&layout)?;
            let plan = match &packet {
                Some(packet) => build_plan(packet, &rules),
                None => VerificationPlan {
                    steps: vec![manual_step(
                        "cargo test -j 1 -- --test-threads=1".to_string(),
                        DEFAULT_AUTO_TIMEOUT_SECS,
                    )],
                },
            };
            print_verify_plan(&plan);
            let steps = plan.steps.clone();
            (Some(plan), steps)
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

    let report = VerificationReport::new(plan.clone(), persisted_results);
    write_verify_report(&layout, &report)?;
    persist_verify_report(&layout, &report);

    if let Some(error) = final_error {
        Err(error)
    } else {
        Ok(())
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
