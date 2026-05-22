use crate::config::model::Config;
use crate::output::human::print_doctor_report;
use crate::platform::{check_tools, classify_path, current_platform, detect_shell};
use crate::state::layout::Layout;
use miette::{IntoDiagnostic, Result};
use std::env;

pub fn execute_doctor() -> Result<()> {
    let platform = current_platform();
    let shell = detect_shell();
    let tools = check_tools();

    let current_dir = env::current_dir().into_diagnostic()?;
    let current_dir_display = current_dir.display().to_string();
    let path_kind = classify_path(&current_dir);

    let platform_str = format!("{platform:?}");
    let shell_str = format!("{shell:?}");
    let kind_str = format!("{path_kind:?}");

    let current_dir_utf8: &camino::Utf8Path = camino::Utf8Path::new(&current_dir_display);
    let layout = Layout::new(current_dir_utf8.as_str());

    let config = crate::config::load_config(&layout).unwrap_or_else(|_| Config::default());

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
        match crate::state::storage_cozo::CozoStorage::new_read_only(cozo_path.as_std_path()) {
            Ok(cozo) => match (cozo.node_count(), cozo.edge_count()) {
                (Ok(nodes), Ok(edges)) => {
                    format!("Ready (CozoDB active, {} nodes, {} edges)", nodes, edges)
                }
                _ => "Ready (CozoDB active)".to_string(),
            },
            Err(e) => format!("Unavailable ({})", e),
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
    use owo_colors::OwoColorize;

    #[cfg(target_os = "windows")]
    {
        match crate::platform::gpu::query_vram_usage() {
            Ok(info) => {
                let usage_gb = info.current_usage as f64 / 1_073_741_824.0;
                let budget_gb = info.budget_bytes as f64 / 1_073_741_824.0;
                let pressure = crate::platform::gpu::classify(&info);

                let status = match pressure {
                    crate::platform::gpu::VramPressure::Ok => {
                        format!("{:.1} GB / {:.1} GB", usage_gb, budget_gb)
                            .green()
                            .to_string()
                    }
                    crate::platform::gpu::VramPressure::High => {
                        format!("{:.1} GB / {:.1} GB (HIGH)", usage_gb, budget_gb)
                            .yellow()
                            .to_string()
                    }
                    crate::platform::gpu::VramPressure::Critical => format!(
                        "{:.1} GB / {:.1} GB (CRITICAL - may block model load)",
                        usage_gb, budget_gb
                    )
                    .red()
                    .bold()
                    .to_string(),
                };

                println!("{:<20} {}", "GPU VRAM:".bold(), status);

                if matches!(
                    pressure,
                    crate::platform::gpu::VramPressure::Critical
                        | crate::platform::gpu::VramPressure::High
                ) {
                    println!(
                        "{}",
                        "  Warning: High VRAM pressure may cause model load failures. \n  Only one model (Embedding or Completion) may fit."
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
