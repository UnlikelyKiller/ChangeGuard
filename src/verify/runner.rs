use crate::commands::CommandError;
use crate::exec::{CommandOptions, ExecutionBoundary, ExecutionResult, ProcessError};
use crate::platform::process_policy::{ProcessPolicy, check_policy};
use crate::verify::plan::VerificationStep;
use miette::{IntoDiagnostic, Result};
use std::env;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Direct,
    Shell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedStep {
    pub display_command: String,
    pub executable: String,
    pub args: Vec<String>,
    pub timeout_secs: u64,
    pub description: String,
    pub execution_mode: ExecutionMode,
}

pub fn prepare_manual_step(step: &VerificationStep) -> PreparedStep {
    shell_step(step)
}

pub fn prepare_rule_step(step: &VerificationStep) -> PreparedStep {
    if contains_shell_metacharacters(&step.command) {
        return shell_step(step);
    }

    match split_command_string(&step.command) {
        Some(tokens) if !tokens.is_empty() => PreparedStep {
            display_command: step.command.clone(),
            executable: tokens[0].clone(),
            args: tokens[1..].to_vec(),
            timeout_secs: step.timeout_secs,
            description: step.description.clone(),
            execution_mode: ExecutionMode::Direct,
        },
        _ => shell_step(step),
    }
}

pub fn execute_step(step: &PreparedStep, policy: &ProcessPolicy) -> Result<ExecutionResult> {
    check_policy(&step.executable, policy).into_diagnostic()?;

    let mut command = Command::new(&step.executable);
    command.args(&step.args);
    command.stdin(Stdio::null());
    command
        .current_dir(env::current_dir().into_diagnostic()?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let options = CommandOptions {
        timeout: Duration::from_secs(step.timeout_secs),
        ..Default::default()
    };

    match ExecutionBoundary::execute(command, &options) {
        Ok(result) => {
            if step.execution_mode == ExecutionMode::Shell && looks_like_command_not_found(&result)
            {
                return Err(CommandError::Verify(format!(
                    "Command not found via shell fallback: {}",
                    step.display_command
                ))
                .into());
            }
            Ok(result)
        }
        Err(ProcessError::Timeout { timeout }) => {
            Err(CommandError::Verify(format!("Timed out after {:?}", timeout)).into())
        }
        Err(ProcessError::NotFound { cmd }) => {
            Err(CommandError::Verify(format!("Command not found: {}", cmd)).into())
        }
        Err(ProcessError::Failed { status, stderr }) => Err(CommandError::Verify(format!(
            "Process exited with status {}: {}",
            status, stderr
        ))
        .into()),
        Err(e) => Err(e.into()),
    }
}

fn shell_step(step: &VerificationStep) -> PreparedStep {
    if cfg!(target_os = "windows") {
        PreparedStep {
            display_command: step.command.clone(),
            executable: "cmd".to_string(),
            args: vec!["/C".to_string(), step.command.clone()],
            timeout_secs: step.timeout_secs,
            description: step.description.clone(),
            execution_mode: ExecutionMode::Shell,
        }
    } else {
        PreparedStep {
            display_command: step.command.clone(),
            executable: "sh".to_string(),
            args: vec!["-c".to_string(), step.command.clone()],
            timeout_secs: step.timeout_secs,
            description: step.description.clone(),
            execution_mode: ExecutionMode::Shell,
        }
    }
}

fn contains_shell_metacharacters(command: &str) -> bool {
    command.chars().any(|ch| {
        matches!(
            ch,
            '|' | '&' | ';' | '>' | '<' | '(' | ')' | '$' | '*' | '?' | '{' | '}' | '\n'
        )
    })
}

fn split_command_string(command: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in command.chars() {
        match quote {
            Some(active_quote) if ch == active_quote => quote = None,
            Some(_) => current.push(ch),
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            None => current.push(ch),
        }
    }

    if quote.is_some() {
        return None;
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    if tokens.is_empty() {
        None
    } else {
        Some(tokens)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base_step(command: &str, timeout_secs: u64) -> VerificationStep {
        VerificationStep {
            description: "test".to_string(),
            command: command.to_string(),
            timeout_secs,
        }
    }

    #[test]
    fn prepare_rule_step_uses_direct_execution_for_simple_commands() {
        let step = base_step("cargo test -j 1 -- --test-threads=1", 5);
        let prepared = prepare_rule_step(&step);

        assert_eq!(prepared.execution_mode, ExecutionMode::Direct);
        assert_eq!(prepared.executable, "cargo");
        assert_eq!(
            prepared.args,
            vec!["test", "-j", "1", "--", "--test-threads=1"]
        );
    }

    #[test]
    fn prepare_rule_step_uses_shell_fallback_for_shell_syntax() {
        let step = base_step("echo hello | sort", 5);
        let prepared = prepare_rule_step(&step);

        assert_eq!(prepared.execution_mode, ExecutionMode::Shell);
    }

    #[test]
    fn execute_step_denies_blocked_commands() {
        let prepared = PreparedStep {
            display_command: "cargo test".to_string(),
            executable: "cargo".to_string(),
            args: vec!["test".to_string()],
            timeout_secs: 5,
            description: "test".to_string(),
            execution_mode: ExecutionMode::Direct,
        };
        let policy = ProcessPolicy {
            denied_commands: vec!["cargo".to_string()],
            ..ProcessPolicy::default()
        };

        let err = execute_step(&prepared, &policy).unwrap_err();
        assert!(format!("{err:?}").contains("denied"));
    }

    #[test]
    fn execute_step_direct_process_succeeds() {
        let prepared = PreparedStep {
            display_command: "rustc --version".to_string(),
            executable: "rustc".to_string(),
            args: vec!["--version".to_string()],
            timeout_secs: 5,
            description: "test".to_string(),
            execution_mode: ExecutionMode::Direct,
        };

        let result = execute_step(&prepared, &ProcessPolicy::default()).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.to_ascii_lowercase().contains("rustc"));
    }

    #[test]
    fn execute_step_manual_shell_succeeds() {
        let prepared = prepare_manual_step(&base_step("echo hello", 5));
        let result = execute_step(&prepared, &ProcessPolicy::default()).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.to_ascii_lowercase().contains("hello"));
    }

    #[test]
    fn execute_step_timeout_errors() {
        let prepared = if cfg!(target_os = "windows") {
            PreparedStep {
                display_command: "ping -n 10 127.0.0.1".to_string(),
                executable: "ping".to_string(),
                args: vec!["-n".to_string(), "10".to_string(), "127.0.0.1".to_string()],
                timeout_secs: 1,
                description: "timeout".to_string(),
                execution_mode: ExecutionMode::Direct,
            }
        } else {
            PreparedStep {
                display_command: "sleep 10".to_string(),
                executable: "sleep".to_string(),
                args: vec!["10".to_string()],
                timeout_secs: 1,
                description: "timeout".to_string(),
                execution_mode: ExecutionMode::Direct,
            }
        };

        let err = execute_step(&prepared, &ProcessPolicy::default()).unwrap_err();
        assert!(format!("{err:?}").contains("Timed out"));
    }
}
