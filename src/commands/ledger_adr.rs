use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::env;
use std::fs;

use crate::config::load::load_config;
use crate::config::model::Config;
use crate::ledger::adr::{generate_madr_content, slugify_summary};
use crate::ledger::transaction::TransactionManager;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;

fn get_repo_root() -> Result<Utf8PathBuf> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let discovered = gix::discover(&current_dir).into_diagnostic()?;
    let root = discovered
        .workdir()
        .ok_or_else(|| miette::miette!("Failed to find work directory for repository"))?;

    Utf8PathBuf::from_path_buf(root.to_path_buf())
        .map_err(|_| miette::miette!("Repository root is not valid UTF-8"))
}

fn get_layout() -> Result<Layout> {
    let root = get_repo_root()?;
    Ok(Layout::new(root))
}

fn load_ledger_config(layout: &Layout) -> Config {
    load_config(layout).unwrap_or_else(|e| {
        tracing::warn!("Failed to load config: {e}. Using defaults.");
        Config::default()
    })
}

pub fn execute_ledger_adr(output_dir: Option<Utf8PathBuf>, days: Option<u64>) -> Result<()> {
    let layout = get_layout()?;
    let db_path = layout.state_subdir().join("ledger.db");
    let mut storage = StorageManager::init(db_path.as_std_path())?;
    let config = load_ledger_config(&layout);
    let manager = TransactionManager::new(
        storage.get_connection_mut(),
        layout.root.clone().into(),
        config,
    );

    let entries = manager
        .get_adr_entries(days)
        .map_err(|e| miette::miette!("{}", e))?;

    if entries.is_empty() {
        println!("No architectural decisions found to export.");
        return Ok(());
    }

    let out_dir = output_dir.unwrap_or_else(|| layout.root.join("docs/adr"));

    if !out_dir.exists() {
        fs::create_dir_all(&out_dir).into_diagnostic()?;
    }

    let mut count = 0;
    for entry in entries {
        let slug = slugify_summary(&entry.summary);
        let filename = format!("{:04}-{}.md", entry.id, slug);
        let file_path = out_dir.join(filename);

        let content = generate_madr_content(&entry);
        fs::write(&file_path, content).into_diagnostic()?;
        count += 1;
    }

    println!("Successfully exported {} ADR(s) to {}", count, out_dir);
    Ok(())
}
