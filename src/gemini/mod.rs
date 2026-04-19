pub mod prompt;

use miette::{IntoDiagnostic, Result};
use std::process::{Command, Stdio};
use std::io::Write;
use owo_colors::OwoColorize;

pub fn run_query(system_prompt: &str, user_prompt: &str) -> Result<()> {
    println!("\n{}", "Consulting Gemini...".bold().bright_cyan());

    // In a real environment, we'd use gemini-cli directly.
    // For this implementation, we'll try to find 'gemini' in PATH.
    
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("powershell");
        c.args(["-Command", "gemini"]);
        c
    } else {
        Command::new("gemini")
    };

    // Construct full input
    let full_input = format!("{}\n\n{}", system_prompt, user_prompt);
    
    // Configure command for piping input
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // We can't use ExecutionBoundary::execute directly because it doesn't handle stdin yet.
    // I'll implement a simplified version here that handles stdin.
    
    let mut child = cmd.spawn().map_err(|e| miette::miette!("Failed to spawn gemini: {}", e))?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(full_input.as_bytes()).into_diagnostic()?;
    }

    // Wait for result
    let output = child.wait_with_output().into_diagnostic()?;

    if output.status.success() {
        println!("\n{}", "Gemini Response:".bold().green());
        println!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        println!("\n{}", "Gemini Error:".bold().red());
        println!("{}", err);
        Err(miette::miette!("Gemini failed with exit code {}", output.status.code().unwrap_or(-1)))
    }
}
