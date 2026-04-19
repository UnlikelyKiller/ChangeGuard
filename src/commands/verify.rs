use miette::Result;
use crate::exec::{ExecutionBoundary, CommandOptions, ProcessError};
use crate::commands::CommandError;
use std::process::Command;
use std::time::Duration;
use owo_colors::OwoColorize;
use tracing::info;

pub fn execute_verify(command_str: Option<String>, timeout_secs: u64) -> Result<()> {
    let cmd_to_run = command_str.unwrap_or_else(|| "cargo test -j 1 -- --test-threads=1".to_string());
    
    info!("Running verification command: {}", cmd_to_run);
    println!("\n{}", "ChangeGuard Verification".bold().bright_cyan());
    println!("{}", "=".repeat(50).cyan());
    println!("{:<15} {}", "Command:".bold(), cmd_to_run.yellow());
    println!("{:<15} {}s", "Timeout:".bold(), timeout_secs);
    println!();

    let mut parts = cmd_to_run.split_whitespace();
    let program = parts.next().ok_or_else(|| CommandError::Verify("Empty command".into()))?;
    let args: Vec<&str> = parts.collect();

    let cmd = if cfg!(target_os = "windows") && program == "cargo" {
        // Use powershell to run cargo on windows to handle path issues better in some envs
        let mut c = Command::new("powershell");
        c.args(["-Command", &cmd_to_run]);
        c
    } else {
        let mut c = Command::new(program);
        c.args(args);
        c
    };

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
            println!("{:<15} {}", "Exit Code:".bold(), if result.exit_code == 0 { result.exit_code.green().to_string() } else { result.exit_code.red().to_string() });
            println!("{:<15} {:?}", "Duration:".bold(), result.duration);
            
            if result.truncated {
                println!("{}", "Warning: Output was truncated due to size limits.".yellow().italic());
            }

            if result.exit_code == 0 {
                println!("\n{}", "Verification PASSED".green().bold());
                Ok(())
            } else {
                println!("\n{}", "Verification FAILED".red().bold());
                Err(CommandError::Verify(format!("Process exited with code {}", result.exit_code)).into())
            }
        }
        Err(ProcessError::Timeout { timeout }) => {
            println!("\n{}", format!("Verification TIMED OUT after {:?}", timeout).red().bold());
            Err(CommandError::Verify(format!("Timed out after {:?}", timeout)).into())
        }
        Err(ProcessError::NotFound { cmd }) => {
            Err(CommandError::Verify(format!("Command not found: {}", cmd)).into())
        }
        Err(e) => {
            Err(e.into())
        }
    }
}
