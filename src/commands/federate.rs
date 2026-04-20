use crate::federated::scanner::FederatedScanner;
use crate::federated::schema::{FederatedSchema, PublicInterface};
use crate::federated::storage::{get_federated_links, update_federated_link};
use crate::git::repo::open_repo;
use crate::index::storage::get_public_symbols;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use camino::Utf8PathBuf;
use chrono::Utc;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;
use std::fs;

pub fn execute_federate_export() -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    let repo = open_repo(&current_dir)?;
    let repo_name = repo
        .workdir()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    println!("Exporting public interfaces for {}...", repo_name.cyan());

    let symbols = get_public_symbols(storage.get_connection())?;
    let public_interfaces = symbols
        .into_iter()
        .map(|s| PublicInterface {
            symbol: s.name,
            file: s.file_path,
            kind: s.kind,
        })
        .collect();

    let schema = FederatedSchema::new(repo_name, public_interfaces);
    let schema_json = serde_json::to_string_pretty(&schema).into_diagnostic()?;

    let schema_path = layout.state_subdir().join("schema.json");
    fs::write(&schema_path, schema_json).into_diagnostic()?;

    println!(
        "{} Schema exported to {}",
        "SUCCESS".green().bold(),
        schema_path.cyan()
    );
    Ok(())
}

pub fn execute_federate_scan() -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let utf8_current_dir = Utf8PathBuf::from_path_buf(current_dir.clone())
        .map_err(|_| miette::miette!("Invalid UTF-8 path"))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    println!("Scanning for sibling repositories...");

    let scanner = FederatedScanner::new(utf8_current_dir);
    let siblings = scanner.scan_siblings()?;

    if siblings.is_empty() {
        println!("No siblings with ChangeGuard schemas found.");
        return Ok(());
    }

    let timestamp = Utc::now().to_rfc3339();
    for (path, schema) in &siblings {
        println!("  Found {}: {}", schema.repo_name.cyan(), path.dimmed());
        update_federated_link(
            storage.get_connection(),
            &schema.repo_name,
            path.as_str(),
            &timestamp,
        )?;
    }

    println!(
        "{} Discovered {} sibling(s).",
        "SUCCESS".green().bold(),
        siblings.len()
    );
    Ok(())
}

pub fn execute_federate_status() -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    let links = get_federated_links(storage.get_connection())?;

    if links.is_empty() {
        println!("No federated links found. Run 'changeguard federate scan' to discover siblings.");
        return Ok(());
    }

    println!("{} known federated repositories:", links.len().bold());
    for (name, path, last_scan) in links {
        println!("- {} (at {})", name.cyan(), path.dimmed());
        println!("  Last scanned: {}", last_scan.dimmed());
    }

    Ok(())
}
