use crate::federated::scanner::FederatedScanner;
use crate::federated::schema::{FederatedSchema, PublicInterface};
use crate::federated::storage::{
    clear_federated_dependencies, get_federated_links, save_federated_dependencies,
    update_federated_link,
};
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
    let repo = open_repo(&current_dir).into_diagnostic()?;
    let repo_root = repo
        .workdir()
        .ok_or_else(|| miette::miette!("Could not determine repository root"))?
        .to_path_buf();

    let layout = Layout::new(repo_root.to_string_lossy().as_ref());
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    let repo_name = repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| miette::miette!("Could not determine repository name for export"))?
        .to_string();

    println!("Exporting public interfaces for {}...", repo_name.cyan());

    let symbols = get_public_symbols(storage.get_connection())?;
    let mut public_interfaces = symbols
        .into_iter()
        .map(|s| PublicInterface {
            symbol: s.name,
            file: s.file_path,
            kind: s.kind,
        })
        .collect::<Vec<_>>();

    public_interfaces.retain(|interface| {
        crate::impact::redact::sanitize_prompt(
            &interface.symbol,
            crate::impact::redact::DEFAULT_MAX_BYTES,
        )
        .redactions
        .is_empty()
    });

    let ledger_entries =
        crate::ledger::federation::export_ledger_entries(storage.get_connection(), 30)
            .into_diagnostic()?;

    let schema = FederatedSchema::new(repo_name, public_interfaces).with_ledger(ledger_entries);
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
    let repo = open_repo(&current_dir).into_diagnostic()?;
    let repo_root = repo
        .workdir()
        .ok_or_else(|| miette::miette!("Could not determine repository root"))?
        .to_path_buf();

    let utf8_repo_root = Utf8PathBuf::from_path_buf(repo_root.clone())
        .map_err(|_| miette::miette!("Invalid UTF-8 path"))?;
    let layout = Layout::new(repo_root.to_string_lossy().as_ref());
    let db_path = layout.state_subdir().join("ledger.db");
    let mut storage = StorageManager::init(db_path.as_std_path())?;

    let local_packet = storage
        .get_latest_packet()?
        .ok_or_else(|| miette::miette!("No local index found. Run 'changeguard scan' first."))?;

    println!("Scanning for sibling repositories...");

    let scanner = FederatedScanner::new(utf8_repo_root);
    let (siblings, warnings) = scanner.scan_siblings()?;

    for warning in &warnings {
        println!("{} {}", "WARN".yellow().bold(), warning);
    }

    if siblings.is_empty() {
        println!("No siblings with ChangeGuard schemas found.");
        return Ok(());
    }

    let timestamp = Utc::now().to_rfc3339();
    for (path, schema) in &siblings {
        println!(
            "  Processing {}: {}",
            schema.repo_name.cyan(),
            path.dimmed()
        );
        update_federated_link(
            storage.get_connection(),
            &schema.repo_name,
            path.as_str(),
            &timestamp,
        )?;

        // Task 2.2: Discover and save dependencies
        clear_federated_dependencies(storage.get_connection(), &schema.repo_name)?;
        let dependencies =
            scanner.discover_dependencies(&local_packet, &schema.repo_name, schema)?;

        for (local_symbol, sibling_symbol) in dependencies {
            save_federated_dependencies(
                storage.get_connection(),
                &schema.repo_name,
                &local_symbol,
                &sibling_symbol,
            )?;
        }

        // Import federated ledger entries if present
        if let Some(entries) = &schema.ledger {
            crate::ledger::federation::import_federated_entries(
                storage.get_connection_mut(),
                &repo_root,
                &schema.repo_name,
                entries,
            )
            .into_diagnostic()?;
        }
    }

    println!(
        "{} Processed {} sibling(s).",
        "SUCCESS".green().bold(),
        siblings.len()
    );
    Ok(())
}

pub fn execute_federate_status() -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let repo = open_repo(&current_dir).into_diagnostic()?;
    let repo_root = repo
        .workdir()
        .ok_or_else(|| miette::miette!("Could not determine repository root"))?;

    let layout = Layout::new(repo_root.to_string_lossy().as_ref());
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
