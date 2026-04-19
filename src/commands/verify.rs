use crate::commands::CommandError;
use crate::exec::{CommandOptions, ExecutionBoundary, ExecutionResult, ProcessError};
use crate::output::human::{print_verify_plan, print_verify_result};
use crate::policy::load::load_rules;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::verify::plan::{VerificationPlan, VerificationStep, build_plan};
use crate::verify::results::{
    VerificationReport, VerificationResult, write_verify_report,
};
use chrono::Utc;
use miette::Result;
use std::env;
use std::process::Command;
use std::time::Duration;
use tracing::{info, warn};

pub fn execute_verify(command_str: Option<String>, timeout_secs: u64) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let (plan, steps) = match command_str {
        Some(cmd) => (None, vec![manual_step(cmd, timeout_secs)]),
        None => {
            let db_path = layout.state_subdir().join("ledger.db");
            let packet = match StorageManager::init(db_path.as_std_path()) {
                Ok(storage) => storage.get_latest_packet()?,
                Err(_) => None,
            };

            let rules = load_rules(&layout).unwrap_or_default();
            let plan = match &packet {
                Some(packet) => build_plan(packet, &rules),
                None => VerificationPlan {
                    steps: vec![manual_step(
                        "cargo test -j 1 -- --test-threads=1".to_string(),
                        300,
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

    for step in &steps {
        info!("Running verification command: {}", step.command);
        let result = run_shell_command(&step.command, step.timeout_secs)?;
        print_verify_result(&step.command, step.timeout_secs, &result);

        persisted_results.push(to_report_result(&step.command, &result));

        if result.exit_code != 0 && final_error.is_none() {
            final_error = Some(
                CommandError::Verify(format!("Process exited with code {}", result.exit_code)).into(),
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

fn run_shell_command(command_str: &str, timeout_secs: u64) -> Result<ExecutionResult> {
    let cmd = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command_str]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", command_str]);
        cmd
    };

    let options = CommandOptions {
        timeout: Duration::from_secs(timeout_secs),
        ..Default::default()
    };

    match ExecutionBoundary::execute(cmd, &options) {
        Ok(result) => {
            if looks_like_command_not_found(&result) {
                return Err(
                    CommandError::Verify(format!("Command not found: {}", command_str)).into(),
                );
            }
            Ok(result)
        }
        Err(ProcessError::Timeout { timeout }) => {
            Err(CommandError::Verify(format!("Timed out after {:?}", timeout)).into())
        }
        Err(ProcessError::NotFound { cmd }) => {
            Err(CommandError::Verify(format!("Command not found: {}", cmd)).into())
        }
        Err(ProcessError::Failed { status, stderr }) => Err(
            CommandError::Verify(format!("Process exited with status {}: {}", status, stderr)).into(),
        ),
        Err(e) => Err(e.into()),
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

fn looks_like_command_not_found(result: &ExecutionResult) -> bool {
    let stderr = result.stderr.to_ascii_lowercase();
    let stdout = result.stdout.to_ascii_lowercase();
    result.exit_code != 0
        && (stderr.contains("not recognized as an internal or external command")
            || stderr.contains("command not found")
            || stdout.contains("not recognized as an internal or external command")
            || stdout.contains("command not found"))
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

    let Ok(run_id) = storage.save_verification_run(
        &report.timestamp,
        plan_json.as_deref(),
        report.overall_pass,
    ) else {
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
