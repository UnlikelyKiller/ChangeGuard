use crate::output::human::print_doctor_report;
use crate::platform::{check_tools, classify_path, current_platform, detect_shell};
use crate::state::layout::Layout;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;

use crate::state::storage::StorageManager;

pub fn execute_doctor() -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let platform = current_platform();
    let shell = detect_shell();
    let tools = check_tools();

    layout.ensure_state_dir()?;
    let storage_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(storage_path.as_std_path())?;

    let platform_str = format!("{:?}", platform);
    let shell_str = format!("{:?}", shell);
    let path_kind_str = format!("{:?}", classify_path(&current_dir));

    let mut report = crate::output::human::DoctorReport {
        platform: &platform_str,
        shell: &shell_str,
        tools: &tools,
        path_display: &current_dir.to_string_lossy(),
        path_kind: &path_kind_str,
        is_wsl_mounted: false,
        embedding_model_status: "checking...".to_string(),
        completion_model_status: "checking...".to_string(),
        native_graph_status: "checking...".to_string(),
        index_health: Vec::new(),
    };

    // --- Intelligence Probes ---
    let config = crate::config::load::load_config(&layout)?;
    let mut model_config = config.local_model.clone();
    model_config.timeout_secs = 2;

    report.embedding_model_status = match crate::embed::client::check_local_model(&model_config) {
        Ok(dims) => format!(
            "{} ({} dims) @ {}",
            config.local_model.embedding_model,
            dims.dimensions,
            config
                .local_model
                .embedding_url
                .as_deref()
                .unwrap_or(&config.local_model.base_url)
        ),
        Err(e) => format!("unreachable ({})", e.yellow()),
    };

    report.completion_model_status =
        match crate::local_model::client::ping_completions(&model_config) {
            Ok(model) => format!(
                "{} @ {}",
                model,
                config
                    .local_model
                    .generation_url
                    .as_deref()
                    .unwrap_or(&config.local_model.base_url)
            ),
            Err(e) => format!("unreachable ({})", e.yellow()),
        };

    // --- Graph Probe ---
    if let Some(cozo) = &storage.cozo {
        match cozo.run_script("?[count(n)] := *node{id: n}") {
            Ok(res) => {
                let node_count = res
                    .rows
                    .first()
                    .and_then(|r| r.first())
                    .and_then(|v| match v {
                        cozo::DataValue::Num(cozo::Num::Int(i)) => Some(*i),
                        _ => None,
                    })
                    .unwrap_or(0);

                let edge_res = cozo.run_script("?[count(s)] := *edge{source: s}");
                let edge_count = edge_res
                    .ok()
                    .and_then(|res| res.rows.first().cloned())
                    .and_then(|r| r.first().cloned())
                    .and_then(|v| match v {
                        cozo::DataValue::Num(cozo::Num::Int(i)) => Some(i),
                        _ => None,
                    })
                    .unwrap_or(0);

                report.native_graph_status = format!(
                    "Ready (CozoDB active, {} nodes, {} edges)",
                    node_count, edge_count
                );
            }
            Err(e) => report.native_graph_status = format!("Error ({})", e.red()),
        }
    } else {
        report.native_graph_status = "Not initialized".to_string();
    }

    // --- Index Health Probes ---
    // 1. Tantivy Search Index
    let index_path = layout.search_index_dir();
    if !index_path.exists() {
        report
            .index_health
            .push("Search index: Missing (run 'changeguard index')".to_string());
    } else {
        let engine = crate::search::tantivy_engine::TantivySearchEngine::open_or_create(
            index_path.as_std_path(),
        );
        match engine {
            Ok(e) => {
                if let Err(err) = e.verify_index_integrity(index_path.as_std_path()) {
                    report.index_health.push(format!(
                        "Search index: Corrupt ({}) - run 'changeguard index --full'",
                        err.red()
                    ));
                } else {
                    let docs = e.document_count();
                    report
                        .index_health
                        .push(format!("Search index: OK ({} documents)", docs));
                }
            }
            Err(e) => report
                .index_health
                .push(format!("Search index: Load failed ({})", e.red())),
        }
    }

    // 2. Knowledge Graph Staleness
    if let Some(stale_res) =
        crate::index::staleness::check_index_staleness(&storage, config.index.stale_threshold_days)
    {
        if stale_res.is_missing {
            report
                .index_health
                .push("Graph state: Empty (never indexed)".yellow().to_string());
        } else {
            report.index_health.push(
                format!(
                    "Graph state: STALE ({} files affected) - run 'changeguard index'",
                    stale_res.stale_files
                )
                .yellow()
                .to_string(),
            );
        }
    } else {
        report.index_health.push("Graph state: Current".to_string());
    }

    print_doctor_report(&report);
    print_vram_section();

    Ok(())
}

fn print_vram_section() {
    #[cfg(target_os = "windows")]
    {
        use crate::platform::gpu::{VramPressure, classify, query_vram_usage};
        match query_vram_usage() {
            Ok(info) => {
                let usage_gb = info.current_usage as f64 / 1_073_741_824.0;
                let budget_gb = info.budget_bytes as f64 / 1_073_741_824.0;
                let pressure = classify(&info);

                let is_arc = info.adapter_name.to_lowercase().contains("arc");
                let note = if is_arc && info.current_usage == 0 {
                    " (Driver limitation: zero-usage reporting on Intel Arc)"
                        .yellow()
                        .to_string()
                } else {
                    "".to_string()
                };

                let usage_str = format!("{:.1}", usage_gb);
                let color_usage = match pressure {
                    VramPressure::Ok => usage_str.white().to_string(),
                    VramPressure::High => usage_str.yellow().bold().to_string(),
                    VramPressure::Critical => usage_str.red().bold().to_string(),
                };
                println!(
                    "{:<20} {} GB / {:.1} GB{}",
                    "GPU VRAM:".bold(),
                    color_usage,
                    budget_gb,
                    note
                );
            }
            Err(e) => println!("{:<20} unavailable ({})", "GPU VRAM:".bold(), e.yellow()),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!("{:<20} n/a (Windows-only monitoring)\", \"GPU VRAM:\".bold()");
    }
}
