pub mod prompt;
pub mod sanitize;

use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

const DEFAULT_GEMINI_TIMEOUT_SECS: u64 = 120;

pub fn run_query(system_prompt: &str, user_prompt: &str, timeout_secs: Option<u64>) -> Result<()> {
    let timeout = Duration::from_secs(timeout_secs.unwrap_or(DEFAULT_GEMINI_TIMEOUT_SECS));

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message("Consulting Gemini...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut cmd = Command::new("gemini");

    let full_input = format!("{}\n\n{}", system_prompt, user_prompt);

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| miette::miette!("Failed to spawn gemini: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(full_input.as_bytes()).into_diagnostic()?;
    }

    // Wait for result with timeout
    let exit_status = match child.wait_timeout(timeout).into_diagnostic()? {
        Some(status) => status,
        None => {
            // Timeout: kill the child process
            let _ = child.kill();
            let _ = child.wait();
            pb.finish_and_clear();
            return Err(miette::miette!(
                "Gemini command timed out after {}s",
                timeout.as_secs()
            ));
        }
    };

    let output = child.wait_with_output().into_diagnostic()?;

    pb.finish_and_clear();

    if exit_status.success() {
        println!("\n{}", "Gemini Response:".bold().green());
        println!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        println!("\n{}", "Gemini Error:".bold().red());
        println!("{}", err);
        Err(miette::miette!(
            "Gemini failed with exit code {}",
            exit_status.code().unwrap_or(-1)
        ))
    }
}
