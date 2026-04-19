use crate::output::human::print_doctor_report;
use crate::platform::{check_tools, classify_path, current_platform, detect_shell};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;

pub fn execute_doctor() -> Result<()> {
    let platform = current_platform();
    let platform_str = match platform {
        crate::platform::PlatformType::Windows => "Windows".green().to_string(),
        crate::platform::PlatformType::Linux => "Linux".green().to_string(),
        crate::platform::PlatformType::Wsl => {
            "Windows Subsystem for Linux (WSL)".green().to_string()
        }
        crate::platform::PlatformType::Unknown => "Unknown".yellow().to_string(),
    };

    let shell = detect_shell();
    let shell_str = match shell {
        crate::platform::ShellType::Powershell => "PowerShell".green().to_string(),
        crate::platform::ShellType::Bash => "Bash".green().to_string(),
        crate::platform::ShellType::Zsh => "Zsh".green().to_string(),
        crate::platform::ShellType::Cmd => "Command Prompt".green().to_string(),
        crate::platform::ShellType::Unknown => "Unknown".yellow().to_string(),
    };

    let tools = check_tools();
    let current_dir = env::current_dir().into_diagnostic()?;
    let path_kind = classify_path(&current_dir);
    let kind_str = match path_kind {
        crate::platform::PathKind::Native => "Native".green().to_string(),
        crate::platform::PathKind::WslMounted => "WSL Mounted (Cross-FS)".yellow().to_string(),
        crate::platform::PathKind::Network => "Network Drive".yellow().to_string(),
        crate::platform::PathKind::Unknown => "Unknown".red().to_string(),
    };

    print_doctor_report(
        &platform_str,
        &shell_str,
        &tools,
        &current_dir.display().to_string(),
        &kind_str,
        path_kind == crate::platform::PathKind::WslMounted,
    );

    Ok(())
}
