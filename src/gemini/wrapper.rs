use crate::platform::env::{ExecutableStatus, find_executable};
use indicatif::{ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

const DEFAULT_GEMINI_TIMEOUT_SECS: u64 = 120;

pub fn run_query(
    system_prompt: &str,
    user_prompt: &str,
    timeout_secs: Option<u64>,
    model: &str,
    api_key: Option<&str>,
) -> Result<()> {
    let timeout = Duration::from_secs(timeout_secs.unwrap_or(DEFAULT_GEMINI_TIMEOUT_SECS));

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message(format!("Consulting Gemini ({model})..."));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut cmd = gemini_command()?;
    cmd.args(["--model", model, "--prompt", ""]);
    let full_input = format!("{}\n\n{}", system_prompt, user_prompt);

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    configure_api_key(&mut cmd, api_key);

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            pb.finish_and_clear();
            return Err(miette::miette!(
                "Gemini CLI not found. Install Gemini CLI to enable narrative summaries."
            ));
        }
        Err(e) => {
            pb.finish_and_clear();
            return Err(miette::miette!("Failed to spawn gemini: {}", e));
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(full_input.as_bytes()).into_diagnostic()?;
    }

    let exit_status = match child.wait_timeout(timeout).into_diagnostic()? {
        Some(status) => status,
        None => {
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
        let code = exit_status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "signal".to_string());
        Err(miette::miette!("Gemini failed with exit code {}", code))
    }
}

fn gemini_command() -> Result<Command> {
    match find_executable("gemini") {
        ExecutableStatus::Found(path) => Ok(command_for_executable(&path)),
        ExecutableStatus::NotFound => Err(miette::miette!(
            "Gemini CLI not found. Install Gemini CLI to enable narrative summaries."
        )),
    }
}

fn command_for_executable(path: &Path) -> Command {
    if cfg!(target_os = "windows")
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ps1"))
    {
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]);
        cmd.arg(path);
        return cmd;
    }

    Command::new(path)
}

fn configure_api_key(cmd: &mut Command, configured_key: Option<&str>) {
    if let Some(key) = configured_key.and_then(non_empty) {
        cmd.env("GEMINI_API_KEY", key);
        return;
    }

    if std::env::var_os("GEMINI_API_KEY").is_some() {
        return;
    }

    if let Some(key) = read_env_key(Path::new(".env")) {
        cmd.env("GEMINI_API_KEY", key);
    }
}

fn read_env_key(path: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(path).ok()?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim().strip_prefix("export ").unwrap_or(key.trim());
        if key != "GEMINI_API_KEY" {
            continue;
        }

        let value = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if !value.is_empty() {
            return Some(value);
        }
    }

    None
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::{command_for_executable, read_env_key};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn reads_gemini_key_from_env_file() {
        let tmp = tempdir().unwrap();
        let env_path = tmp.path().join(".env");
        fs::write(
            &env_path,
            "\n# local secret\nIGNORED\nexport GEMINI_API_KEY=\"test-key-value\"\nOTHER=value\n",
        )
        .unwrap();

        assert_eq!(read_env_key(&env_path), Some("test-key-value".to_string()));
    }

    #[test]
    fn ignores_missing_or_empty_env_key() {
        let tmp = tempdir().unwrap();
        let env_path = tmp.path().join(".env");
        fs::write(&env_path, "GEMINI_API_KEY=\n").unwrap();

        assert_eq!(read_env_key(&env_path), None);
    }

    #[test]
    fn wraps_windows_powershell_shims() {
        let cmd = command_for_executable(Path::new("gemini.ps1"));
        if cfg!(target_os = "windows") {
            assert_eq!(cmd.get_program().to_string_lossy(), "powershell");
            let args: Vec<_> = cmd
                .get_args()
                .map(|arg| arg.to_string_lossy().to_string())
                .collect();
            assert!(args.iter().any(|arg| arg == "-File"));
            assert!(args.iter().any(|arg| arg == "gemini.ps1"));
        } else {
            assert_eq!(cmd.get_program().to_string_lossy(), "gemini.ps1");
        }
    }
}
