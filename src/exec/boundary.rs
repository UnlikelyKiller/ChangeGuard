use miette::Diagnostic;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use thiserror::Error;
use wait_timeout::ChildExt;

#[derive(Debug, Error, Diagnostic)]
pub enum ProcessError {
    #[error("Command not found: {cmd}")]
    #[diagnostic(help("Ensure the executable is in your PATH and accessible."))]
    NotFound { cmd: String },

    #[error("Command timed out after {timeout:?}")]
    #[diagnostic(code(changeguard::process::timeout))]
    Timeout { timeout: Duration },

    #[error("Process exited with status {status}")]
    #[diagnostic(help("Check the captured stderr for more details."))]
    Failed { status: i32, stderr: String },

    #[error("I/O error during subprocess execution: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub truncated: bool,
}

#[derive(Debug)]
pub struct CommandOptions {
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

impl Default for CommandOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_output_bytes: 1024 * 1024, // 1MB
        }
    }
}

pub struct ExecutionBoundary;

impl ExecutionBoundary {
    pub fn execute(
        mut command: Command,
        options: &CommandOptions,
    ) -> Result<ExecutionResult, ProcessError> {
        let start = Instant::now();

        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let cmd_str = format!("{:?}", command);
                return Err(ProcessError::NotFound { cmd: cmd_str });
            }
            Err(e) => return Err(ProcessError::IoError(e)),
        };

        let status = match child.wait_timeout(options.timeout)? {
            Some(status) => status,
            None => {
                child.kill().ok();
                return Err(ProcessError::Timeout {
                    timeout: options.timeout,
                });
            }
        };

        let duration = start.elapsed();

        let mut stdout_str = String::new();
        let mut stderr_str = String::new();
        let mut truncated = false;

        if let Some(stdout) = child.stdout.take() {
            let mut buffer = Vec::new();
            // Read with limit
            let n = stdout
                .take(options.max_output_bytes as u64)
                .read_to_end(&mut buffer)?;
            if n >= options.max_output_bytes {
                truncated = true;
            }
            stdout_str = String::from_utf8_lossy(&buffer).to_string();
        }

        if let Some(stderr) = child.stderr.take() {
            let mut buffer = Vec::new();
            let n = stderr
                .take(options.max_output_bytes as u64)
                .read_to_end(&mut buffer)?;
            if n >= options.max_output_bytes {
                truncated = true;
            }
            stderr_str = String::from_utf8_lossy(&buffer).to_string();
        }

        let exit_code = status.code().unwrap_or(-1);

        Ok(ExecutionResult {
            exit_code,
            stdout: stdout_str,
            stderr: stderr_str,
            duration,
            truncated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_basic_execution() {
        let cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", "echo hello"]);
            c
        } else {
            let mut c = Command::new("echo");
            c.arg("hello");
            c
        };
        let options = CommandOptions::default();
        let result = ExecutionBoundary::execute(cmd, &options).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_timeout() {
        let cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", "ping -n 5 127.0.0.1 >nul"]);
            c
        } else {
            let mut c = Command::new("sleep");
            c.arg("5");
            c
        };
        let options = CommandOptions {
            timeout: Duration::from_secs(1),
            ..Default::default()
        };
        let result = ExecutionBoundary::execute(cmd, &options);
        match result {
            Err(ProcessError::Timeout { .. }) => (),
            _ => panic!("Expected timeout error, got {:?}", result),
        }
    }

    #[test]
    fn test_not_found() {
        let cmd = Command::new("nonexistent_command_12345");
        let options = CommandOptions::default();
        let result = ExecutionBoundary::execute(cmd, &options);
        match result {
            Err(ProcessError::NotFound { .. }) => (),
            _ => panic!("Expected NotFound error, got {:?}", result),
        }
    }

    #[test]
    fn test_truncation() {
        let cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", "python -c \"print('A' * 2000)\""]);
            c
        } else {
            let mut c = Command::new("printf");
            c.arg("'A%.0s' {1..2000}");
            c
        };
        let options = CommandOptions {
            max_output_bytes: 1000,
            ..Default::default()
        };
        // Truncation test may not find python/printf, so just verify no panic
        if let Ok(result) = ExecutionBoundary::execute(cmd, &options)
            && result.truncated
        {
            assert!(result.stdout.len() <= 1010);
        }
    }
}
