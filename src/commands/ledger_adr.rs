use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::fs;

use crate::commands::helpers::{get_layout, load_ledger_config};
use crate::ledger::adr::{generate_madr_content, slugify_summary};
use crate::ledger::transaction::TransactionManager;
use crate::state::storage::StorageManager;

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
