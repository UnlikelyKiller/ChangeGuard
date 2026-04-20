use camino::Utf8PathBuf;
use miette::Result;
use owo_colors::OwoColorize;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::config::load::load_config;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::watch::batch::WatchBatch;
use crate::watch::debounce::Watcher;

pub fn execute_watch(interval_ms: u64, json_output: bool) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let path = Utf8PathBuf::from_path_buf(current_dir)
        .map_err(|e| miette::miette!("Invalid UTF-8 path: {:?}", e))?;
    let layout = Layout::new(path.as_str());
    let config = load_config(&layout)?;
    let running = Arc::new(AtomicBool::new(true));
    let signal = running.clone();

    ctrlc::set_handler(move || {
        signal.store(false, Ordering::SeqCst);
    })
    .map_err(|e| miette::miette!("Failed to install Ctrl+C handler: {}", e))?;

    if !json_output {
        println!("{}", "ChangeGuard Watch Mode Started".bold().green());
        println!("Watching: {}", path.cyan());
        println!("Press Ctrl+C to stop.\n");
    }

    let batch_path = layout.state_subdir().join("current-batch.json");
    let db_path = layout.state_subdir().join("ledger.db");
    let callback = Box::new(move |batch: WatchBatch| {
        if !json_output {
            println!(
                "\n{} - Received batch of {} events",
                batch
                    .timestamp
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
                    .dimmed(),
                batch.events.len().bold()
            );

            for event in &batch.events {
                let kind_str = format!("{:?}", event.kind);
                let kind_colored = match event.kind {
                    crate::watch::batch::WatchEventKind::Create => kind_str.green().to_string(),
                    crate::watch::batch::WatchEventKind::Modify => kind_str.yellow().to_string(),
                    crate::watch::batch::WatchEventKind::Delete => kind_str.red().to_string(),
                    _ => kind_str,
                };
                println!("  [{}] {}", kind_colored, event.path);
            }
        } else {
            // In JSON mode, we just emit the batch as a single line
            if let Ok(json) = serde_json::to_string(&batch) {
                println!("{}", json);
            }
        }

        if let Err(err) = batch.save(&batch_path) {
            tracing::warn!("Failed to save watch batch JSON: {err}");
        }

        if let Ok(storage) = StorageManager::init(db_path.as_std_path())
            && let Ok(batch_json) = serde_json::to_string(&batch)
        {
            let _ = storage.save_batch(
                &batch.timestamp.to_rfc3339(),
                batch.events.len() as u32,
                &batch_json,
            );
        }
    });

    let _watcher = Watcher::new(
        vec![path],
        Duration::from_millis(interval_ms),
        config.watch.ignore_patterns,
        callback,
    )
    .map_err(|e| miette::miette!("Failed to start watcher: {}", e))?;

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_secs(1));
    }

    if !json_output {
        println!("{}", "Watch mode stopped.".yellow());
    }
    Ok(())
}
