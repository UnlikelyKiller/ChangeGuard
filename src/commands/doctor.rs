use crate::platform::{
    ExecutableStatus, check_tools, classify_path, current_platform, detect_shell,
};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;

pub fn execute_doctor() -> Result<()> {
    println!(
        "\n{}",
        "ChangeGuard Doctor - Environment Health Check"
            .bold()
            .bright_cyan()
    );
    println!("{}", "=".repeat(50).cyan());

    // 1. Environment (OS + WSL status)
    let platform = current_platform();
    let platform_str = match platform {
        crate::platform::PlatformType::Windows => "Windows".green().to_string(),
        crate::platform::PlatformType::Linux => "Linux".green().to_string(),
        crate::platform::PlatformType::Wsl => {
            "Windows Subsystem for Linux (WSL)".green().to_string()
        }
        crate::platform::PlatformType::Unknown => "Unknown".yellow().to_string(),
    };
    println!("{:<20} {}", "Environment:".bold(), platform_str);

    // 2. Active Shell
    let shell = detect_shell();
    let shell_str = match shell {
        crate::platform::ShellType::Powershell => "PowerShell".green().to_string(),
        crate::platform::ShellType::Bash => "Bash".green().to_string(),
        crate::platform::ShellType::Zsh => "Zsh".green().to_string(),
        crate::platform::ShellType::Cmd => "Command Prompt".green().to_string(),
        crate::platform::ShellType::Unknown => "Unknown".yellow().to_string(),
    };
    println!("{:<20} {}", "Active Shell:".bold(), shell_str);

    // 3. Tool status (git, gemini-cli)
    println!("\n{}", "Tools:".bold().bright_cyan());
    let tools = check_tools();
    for (name, status) in tools {
        match status {
            ExecutableStatus::Found(path) => {
                println!(
                    "  {:<18} {} ({})",
                    name.bold(),
                    "Found".green(),
                    path.display().to_string().dimmed()
                );
            }
            ExecutableStatus::NotFound => {
                println!("  {:<18} {}", name.bold(), "Not Found".red());
            }
        }
    }

    // 4. Path classification of the current repository
    let current_dir = env::current_dir().into_diagnostic()?;
    let path_kind = classify_path(&current_dir);
    let kind_str = match path_kind {
        crate::platform::PathKind::Native => "Native".green().to_string(),
        crate::platform::PathKind::WslMounted => "WSL Mounted (Cross-FS)".yellow().to_string(),
        crate::platform::PathKind::Network => "Network Drive".yellow().to_string(),
        crate::platform::PathKind::Unknown => "Unknown".red().to_string(),
    };
    println!("\n{:<20} {}", "Current Path:".bold(), current_dir.display());
    println!("{:<20} {}", "Path Type:".bold(), kind_str);

    if path_kind == crate::platform::PathKind::WslMounted {
        println!("\n{}", "Warning: Running on a WSL mounted drive may be slower due to cross-filesystem overhead.".yellow().italic());
    }

    println!("\n{}", "Doctor check complete.".bright_cyan());
    Ok(())
}
