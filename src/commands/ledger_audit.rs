use camino::Utf8PathBuf;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;

use crate::config::load::load_config;
use crate::config::model::Config;
use crate::ledger::db::LedgerDb;
use crate::ledger::transaction::TransactionManager;
use crate::ledger::ui::{LedgerStatus, get_category_icon, get_change_type_icon, get_status_icon};
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

pub fn execute_ledger_audit(entity: Option<String>, include_unaudited: bool) -> Result<()> {
    let layout = get_layout()?;
    let db_path = layout.state_subdir().join("ledger.db");
    let mut storage = StorageManager::init(db_path.as_std_path())?;
    let config = load_ledger_config(&layout);
    let manager = TransactionManager::new(
        storage.get_connection_mut(),
        layout.root.clone().into(),
        config,
    );

    println!("{}", "ChangeGuard Project Audit".bold().underline());

    if let Some(path) = entity {
        audit_entity(&manager, &path)?;
    } else {
        audit_global(&manager, include_unaudited)?;
    }

    Ok(())
}

fn audit_global(manager: &TransactionManager, include_unaudited: bool) -> Result<()> {
    let pending = manager
        .get_all_pending()
        .map_err(|e| miette::miette!("{}", e))?;
    let unaudited = manager
        .get_all_unaudited()
        .map_err(|e| miette::miette!("{}", e))?;

    println!("\n{}", "PENDING TRANSACTIONS".yellow().bold());
    if pending.is_empty() {
        println!("  None.");
    } else {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                Cell::new("").fg(Color::Yellow),
                Cell::new("TX ID").fg(Color::Cyan),
                Cell::new("Entity").fg(Color::Cyan),
                Cell::new("Category").fg(Color::Cyan),
                Cell::new("Started").fg(Color::Cyan),
            ]);

        for tx in pending {
            table.add_row(vec![
                Cell::new(get_status_icon(LedgerStatus::Pending)),
                Cell::new(&tx.tx_id[..8]).fg(Color::Yellow),
                Cell::new(&tx.entity).fg(Color::Cyan),
                Cell::new(format!(
                    "{} {:?}",
                    get_category_icon(&tx.category),
                    tx.category
                )),
                Cell::new(&tx.started_at),
            ]);
        }
        println!("{table}");
    }

    if include_unaudited {
        println!("\n{}", "UNAUDITED DRIFT".red().bold());
        if unaudited.is_empty() {
            println!("  None.");
        } else {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec![
                    Cell::new("").fg(Color::Red),
                    Cell::new("Entity").fg(Color::Cyan),
                    Cell::new("Drift Count").fg(Color::Cyan),
                    Cell::new("Last Seen").fg(Color::Cyan),
                ]);

            for tx in unaudited {
                table.add_row(vec![
                    Cell::new(get_status_icon(LedgerStatus::Stale)),
                    Cell::new(&tx.entity).fg(Color::Cyan),
                    Cell::new(tx.drift_count.to_string()).fg(Color::Red),
                    Cell::new(tx.last_seen_at.unwrap_or_default()),
                ]);
            }
            println!("{table}");
        }
    }

    Ok(())
}

fn audit_entity(manager: &TransactionManager, entity: &str) -> Result<()> {
    println!("\nAudit History for {}:", entity.cyan());

    let entries = manager
        .get_ledger_entries(entity)
        .map_err(|e| miette::miette!("{}", e))?;
    if entries.is_empty() {
        println!("  No committed entries found.");
    } else {
        for entry in entries {
            let prefix = if entry.origin == "LOCAL" {
                format!(
                    "{} [{:04}]",
                    get_status_icon(LedgerStatus::Committed),
                    entry.id
                )
                .yellow()
                .to_string()
            } else {
                format!(
                    "{} [FEDERATED: {}]",
                    get_status_icon(LedgerStatus::Federated),
                    entry.trace_id.as_deref().unwrap_or("UNKNOWN")
                )
                .magenta()
                .bold()
                .to_string()
            };

            println!("\n{} committed on {}", prefix, entry.committed_at.dimmed());
            println!("  Summary: {}", entry.summary.bold());
            println!(
                "  Change:  {} {:?}",
                get_change_type_icon(&entry.change_type),
                entry.change_type
            );
            println!("  Reason:  {}", entry.reason);

            // Show token provenance
            let db = LedgerDb::new(manager.get_connection());
            let provenance = db
                .get_token_provenance_for_tx(&entry.tx_id)
                .map_err(|e| miette::miette!("{}", e))?;

            // Filter provenance by this entity (since a TX could span multiple, though usually one per start)
            let entity_prov: Vec<_> = provenance
                .into_iter()
                .filter(|p| p.entity == entity)
                .collect();

            if !entity_prov.is_empty() {
                println!("  Symbols:");
                for p in entity_prov {
                    let action_str = p.action.to_string();
                    let formatted = match p.action {
                        crate::ledger::provenance::ProvenanceAction::Added => {
                            action_str.green().to_string()
                        }
                        crate::ledger::provenance::ProvenanceAction::Modified => {
                            action_str.blue().to_string()
                        }
                        crate::ledger::provenance::ProvenanceAction::Deleted => {
                            action_str.red().to_string()
                        }
                    };
                    println!(
                        "    {} {:<10} {} ({})",
                        "•".dimmed(),
                        formatted,
                        p.symbol_name,
                        p.symbol_type.dimmed()
                    );
                }
            }
        }
    }

    Ok(())
}
