use serde::Serialize;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Powershell,
    Bash,
    Zsh,
    Cmd,
    Unknown,
}

pub fn detect_shell() -> ShellType {
    if cfg!(target_os = "windows") {
        if env::var("PSModulePath").is_ok() {
            ShellType::Powershell
        } else {
            ShellType::Cmd
        }
    } else {
        if let Ok(shell_var) = env::var("SHELL") {
            if shell_var.contains("bash") {
                ShellType::Bash
            } else if shell_var.contains("zsh") {
                ShellType::Zsh
            } else {
                ShellType::Unknown
            }
        } else {
            ShellType::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_detection() {
        let shell = detect_shell();
        #[cfg(target_os = "windows")]
        {
            // Usually we are in powershell in this env
            if env::var("PSModulePath").is_ok() {
                assert_eq!(shell, ShellType::Powershell);
            } else {
                assert_eq!(shell, ShellType::Cmd);
            }
        }
    }
}
