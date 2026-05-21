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
use crate::ledger::ui::{get_category_icon, get_change_type_icon, get_status_icon, LedgerStatus};
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::verify::results::{VERIFY_HISTORY, VerifyHistoryRecord};
use crate::impact::hotspots::calculate_hotspots;
use crate::impact::temporal::GixHistoryProvider;

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
    let mut storage = StorageManager::open_read_only(&layout.root)?;
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

    let db = LedgerDb::new(manager.get_connection());
    let layout = get_layout()?;

    // 1. Commit Velocity (30d)
    let velocity = db
        .get_transaction_velocity(30)
        .map_err(|e| miette::miette!("{}", e))?;
    println!(
        "\n{} {} commits in the last 30 days",
        "Commit Velocity:".bold(),
        velocity.cyan()
    );

    // 2. Top Churned Files
    let churned = db
        .get_top_churned_entities(5)
        .map_err(|e| miette::miette!("{}", e))?;
    println!("\n{}", "TOP CHURNED FILES".yellow().bold());
    if churned.is_empty() {
        println!("  None.");
    } else {
        for (entity, count) in churned {
            println!("  {:<40} {:>5} commits", entity.cyan(), count.yellow());
        }
    }

    // 3. Oldest ADR
    let oldest_adr = db.get_oldest_adr().map_err(|e| miette::miette!("{}", e))?;
    println!(
        "\n{}",
        "OLDEST ARCHITECTURAL DECISION (ADR)".yellow().bold()
    );
    if let Some(entry) = oldest_adr {
        println!(
            "  {} committed on {}",
            entry.summary.bold(),
            entry.committed_at.dimmed()
        );
        println!("  Entity: {}", entry.entity.cyan());
    } else {
        println!("  None.");
    }

    // 4. Hotspot Delta (Current Top 3)
    let storage = StorageManager::open_read_only(&layout.root)?;
    let discovered = gix::discover(&layout.root).into_diagnostic()?;
    let history_provider = GixHistoryProvider::new(&discovered);
    let hotspots = calculate_hotspots(&storage, &history_provider, 500, 3, false, None, None)
        .unwrap_or_default();

    println!("\n{}", "TOP 3 HOTSPOTS".yellow().bold());
    if hotspots.is_empty() {
        println!("  None.");
    } else {
        for h in hotspots {
            println!(
                "  {:<40} score: {:.2}",
                h.path.display().to_string().cyan(),
                h.display_score.yellow()
            );
        }
    }

    // 5. CI Trend
    let history_path = layout.reports_dir().join(VERIFY_HISTORY);
    println!("\n{}", "CI TREND (Last 10)".yellow().bold());
    if history_path.exists() {
        let content = std::fs::read_to_string(&history_path).into_diagnostic()?;
        let history: Vec<VerifyHistoryRecord> = serde_json::from_str(&content).unwrap_or_default();
        if history.is_empty() {
            println!("  No history yet.");
        } else {
            let mut trend = String::new();
            for h in history.iter().rev().take(10).rev() {
                if h.passed {
                    trend.push_str(&"✔".green().to_string());
                } else {
                    trend.push_str(&"✘".red().to_string());
                }
                trend.push(' ');
            }
            println!("  {}", trend);
        }
    } else {
        println!("  No history yet.");
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
