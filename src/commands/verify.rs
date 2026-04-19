use crate::commands::CommandError;
use crate::exec::{CommandOptions, ExecutionBoundary, ProcessError};
use crate::policy::load::load_rules;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::verify::plan::build_plan;
use miette::Result;
use owo_colors::OwoColorize;
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
                Ok(storage) => storage.get_latest_packet()?.or_else(|| {
                    // No packet available, use default command
                    None
                }),
                Err(_) => None,
            };

            let rules = load_rules(&layout).unwrap_or_default();
            let plan = match &packet {
                Some(p) => build_plan(p, &rules),
                None => {
                    // No packet, use default
                    let default_plan = crate::verify::plan::VerificationPlan {
                        steps: vec![crate::verify::plan::VerificationStep {
                            command: "cargo test -j 1 -- --test-threads=1".to_string(),
                            timeout_secs: 300,
                            description: "Default: run project tests".to_string(),
                        }],
                    };
                    default_plan
                }
            };

            // Use the first step from the plan
            if let Some(step) = plan.steps.first() {
                println!("\n{}", "Verification Plan".bold().bright_cyan());
                println!("{}", "=".repeat(50).cyan());
                for (i, step) in plan.steps.iter().enumerate() {
                    println!(
                        "  {}. {} ({})",
                        i + 1,
                        step.command.yellow(),
                        step.description.dimmed()
                    );
                }
                println!("{}", "=".repeat(50).cyan());
                step.command.clone()
            } else {
                "cargo test -j 1 -- --test-threads=1".to_string()
            }
        }
    };

    info!("Running verification command: {}", cmd_to_run);
    println!("\n{}", "ChangeGuard Verification".bold().bright_cyan());
    println!("{}", "=".repeat(50).cyan());
    println!("{:<15} {}", "Command:".bold(), cmd_to_run.yellow());
    println!("{:<15} {}s", "Timeout:".bold(), timeout_secs);
    println!();

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
            println!("{}", "Output:".bold());
            println!("{}", result.stdout);

            if !result.stderr.is_empty() {
                println!("\n{}", "Errors:".bold().red());
                println!("{}", result.stderr.red());
            }

            println!("\n{}", "=".repeat(50).cyan());
            println!(
                "{:<15} {}",
                "Exit Code:".bold(),
                if result.exit_code == 0 {
                    result.exit_code.green().to_string()
                } else {
                    result.exit_code.red().to_string()
                }
            );
            println!("{:<15} {:?}", "Duration:".bold(), result.duration);

            if result.truncated {
                println!(
                    "{}",
                    "Warning: Output was truncated due to size limits."
                        .yellow()
                        .italic()
                );
            }

            if result.exit_code == 0 {
                println!("\n{}", "Verification PASSED".green().bold());
                Ok(())
            } else {
                println!("\n{}", "Verification FAILED".red().bold());
                Err(
                    CommandError::Verify(format!("Process exited with code {}", result.exit_code))
                        .into(),
                )
            }
        }
        Err(ProcessError::Timeout { timeout }) => {
            println!(
                "\n{}",
                format!("Verification TIMED OUT after {:?}", timeout)
                    .red()
                    .bold()
            );
            Err(CommandError::Verify(format!("Timed out after {:?}", timeout)).into())
        }
        Err(ProcessError::NotFound { cmd }) => {
            Err(CommandError::Verify(format!("Command not found: {}", cmd)).into())
        }
        Err(e) => Err(e.into()),
    }
}
