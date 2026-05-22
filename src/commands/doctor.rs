use crate::config::model::Config;
use crate::output::human::print_doctor_report;
use crate::platform::{check_tools, classify_path, current_platform, detect_shell};
use crate::state::layout::Layout;
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

    let current_dir_display = current_dir.display().to_string();
    let current_dir_utf8: &camino::Utf8Path = camino::Utf8Path::new(&current_dir_display);
    let layout = Layout::new(current_dir_utf8);

    let config = crate::config::load::load_config(&layout).unwrap_or_else(|_| Config::default());

    let embedding_model_status =
        if config.local_model.base_url.is_empty() && config.local_model.embedding_url.is_none() {
            "Not configured".to_string()
        } else {
            let mut probe_config = config.local_model.clone();
            probe_config.timeout_secs = 5; // Fail fast for doctor
            match crate::embed::client::check_local_model(&probe_config) {
                Ok(dims) => format!(
                    "{} ({} dims) @ {}",
                    dims.model_name,
                    dims.dimensions,
                    config
                        .local_model
                        .embedding_url
                        .as_deref()
                        .unwrap_or(&config.local_model.base_url)
                ),
                Err(e) => format!("unreachable ({})", e),
            }
        };

    let completion_model_status =
        if config.local_model.base_url.is_empty() && config.local_model.generation_url.is_none() {
            "Not configured".to_string()
        } else {
            let mut probe_config = config.local_model.clone();
            probe_config.timeout_secs = 5; // Fail fast for doctor
            match crate::local_model::client::ping_completions(&probe_config) {
                Ok(model) => format!(
                    "{} @ {}",
                    model,
                    config
                        .local_model
                        .generation_url
                        .as_deref()
                        .unwrap_or(&config.local_model.base_url)
                ),
                Err(e) => format!("unreachable ({})", e),
            }
        };

    let cozo_path = layout.state_subdir().join("ledger.cozo");
    let native_graph_status = if cozo_path.exists() {
        match crate::state::storage_cozo::CozoStorage::new(cozo_path.as_std_path()) {
            Ok(cozo) => match (cozo.node_count(), cozo.edge_count()) {
                (Ok(nodes), Ok(edges)) => {
                    format!("Ready (CozoDB active, {} nodes, {} edges)", nodes, edges)
                }
                _ => "Ready (CozoDB active)".to_string(),
            },
            Err(_) => "Unavailable".to_string(),
        }
    } else {
        "Not initialized (run `changeguard index --analyze-graph`)".to_string()
    };

    let report = crate::output::human::DoctorReport {
        platform: &platform_str,
        shell: &shell_str,
        tools: &tools,
        path_display: &current_dir.display().to_string(),
        path_kind: &kind_str,
        is_wsl_mounted: path_kind == crate::platform::PathKind::WslMounted,
        embedding_model_status: &embedding_model_status,
        completion_model_status: &completion_model_status,
        native_graph_status: &native_graph_status,
    };

    print_doctor_report(&report);

    // VRAM pressure (Windows-only via DXGI)
    print_vram_section();

    Ok(())
}

fn print_vram_section() {
    #[cfg(target_os = "windows")]
    {
        match crate::platform::gpu::query_vram_usage() {
            Ok(info) => {
                let used_gb = info.current_usage as f64 / 1_000_000_000.0;
                let budget_gb = info.budget_bytes as f64 / 1_000_000_000.0;
                let pressure = crate::platform::gpu::classify(&info);
                let icon = match pressure {
                    crate::platform::gpu::VramPressure::Critical => "X".red().to_string(),
                    crate::platform::gpu::VramPressure::High => "!".yellow().to_string(),
                    crate::platform::gpu::VramPressure::Ok => "V".green().to_string(),
                };
                println!(
                    "{:<20} {:.1} GB / {:.1} GB  {}",
                    "GPU VRAM:".bold(),
                    used_gb,
                    budget_gb,
                    icon,
                );
                if matches!(
                    pressure,
                    crate::platform::gpu::VramPressure::Critical
                        | crate::platform::gpu::VramPressure::High
                ) {
                    println!(
                        "{}",
                        "  High VRAM pressure detected — avoid running both models simultaneously"
                            .yellow()
                            .italic()
                    );
                }
            }
            Err(e) => println!("{:<20} unavailable ({})", "GPU VRAM:".bold(), e.yellow()),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!("{:<20} n/a (Windows-only monitoring)", "GPU VRAM:".bold());
    }
}
