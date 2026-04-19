use std::env;
use std::time::Duration;
use camino::Utf8PathBuf;
use miette::Result;
use owo_colors::OwoColorize;

use crate::watch::debounce::Watcher;
use crate::watch::batch::WatchBatch;

pub fn execute_watch(interval_ms: u64) -> Result<()> {
    let current_dir = env::current_dir().map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let path = Utf8PathBuf::from_path_buf(current_dir).map_err(|e| miette::miette!("Invalid UTF-8 path: {:?}", e))?;

    println!("{}", "ChangeGuard Watch Mode Started".bold().green());
    println!("Watching: {}", path.cyan());
    println!("Press Ctrl+C to stop.\n");

    let callback = Box::new(|batch: WatchBatch| {
        println!("\n{} - Received batch of {} events", 
            batch.timestamp.format("%Y-%m-%d %H:%M:%S").to_string().dimmed(),
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
    });

    let _watcher = Watcher::new(
        vec![path],
        Duration::from_millis(interval_ms),
        callback,
    ).map_err(|e| miette::miette!("Failed to start watcher: {}", e))?;

    // Keep the main thread alive
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
