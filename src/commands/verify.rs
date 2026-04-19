use crate::commands::CommandError;
use crate::exec::{CommandOptions, ExecutionBoundary, ProcessError};
use crate::output::human::{print_verify_plan, print_verify_result};
use crate::policy::load::load_rules;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::verify::plan::build_plan;
use miette::Result;
use std::env;
use std::process::Command;
use std::time::Duration;
use tracing::info;

pub fn execute_verify(command_str: Option<String>, timeout_secs: u64) -> Result<()> {
    let cmd_to_run = match command_str {
        Some(cmd) => cmd,
        None => {
            // Build a plan from rules + latest packet
            let current_dir = env::current_dir()
                .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
            let layout = Layout::new(current_dir.to_string_lossy().as_ref());

            let db_path = layout.state_subdir().join("ledger.db");
            let packet = match StorageManager::init(db_path.as_std_path()) {
                Ok(storage) => storage.get_latest_packet()?,
                Err(_) => None,
            };

            let rules = load_rules(&layout).unwrap_or_default();
            let plan = match &packet {
                Some(p) => build_plan(p, &rules),
                None => crate::verify::plan::VerificationPlan {
                    steps: vec![crate::verify::plan::VerificationStep {
                        command: "cargo test -j 1 -- --test-threads=1".to_string(),
                        timeout_secs: 300,
                        description: "Default: run project tests".to_string(),
                    }],
                },
            };

            print_verify_plan(&plan);

            // Use the first step from the plan
            if let Some(step) = plan.steps.first() {
                step.command.clone()
            } else {
                "cargo test -j 1 -- --test-threads=1".to_string()
            }
        }
    };

    info!("Running verification command: {}", cmd_to_run);

    let mut parts = cmd_to_run.split_whitespace();
    let program = parts
        .next()
        .ok_or_else(|| CommandError::Verify("Empty command".into()))?;
    let args: Vec<&str> = parts.collect();

    let mut cmd = Command::new(program);
    cmd.args(&args);

    let options = CommandOptions {
        timeout: Duration::from_secs(timeout_secs),
        ..Default::default()
    };

    match ExecutionBoundary::execute(cmd, &options) {
        Ok(result) => {
            print_verify_result(&cmd_to_run, timeout_secs, &result);

            if result.exit_code == 0 {
                Ok(())
            } else {
                Err(
                    CommandError::Verify(format!("Process exited with code {}", result.exit_code))
                        .into(),
                )
            }
        }
        Err(ProcessError::Timeout { timeout }) => {
            Err(CommandError::Verify(format!("Timed out after {:?}", timeout)).into())
        }
        Err(ProcessError::NotFound { cmd }) => {
            Err(CommandError::Verify(format!("Command not found: {}", cmd)).into())
        }
        Err(e) => Err(e.into()),
    }
}
