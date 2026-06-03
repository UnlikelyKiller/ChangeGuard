use crate::commands::helpers::{get_layout, load_ledger_config};
use crate::ledger::*;
use crate::state::storage::StorageManager;
use crate::util::clock::{Clock, SystemClock};
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

pub fn execute_ledger_start(entity: String, category: &str, message: &str) -> Result<()> {
    let category = resolve_start_category(category)?;
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    let tx_id = tx_mgr
        .start_change(TransactionRequest {
            category,
            entity,
            planned_action: Some(message.to_string()),
            ..Default::default()
        })
        .map_err(|e| miette::miette!("{}", e))?;

    println!("Transaction started: {}", tx_id.cyan());
    Ok(())
}

fn resolve_start_category(input: &str) -> Result<Category> {
    if let Ok(category) = Category::from_str(input, true) {
        return Ok(category);
    }

    let suggestions = Category::suggestions_for(input);
    if crate::util::term::is_interactive() && !suggestions.is_empty() {
        let choice = inquire::Select::new(
            &format!("Unknown ledger category '{input}'. Select a category:"),
            suggestions,
        )
        .prompt()
        .map_err(|e| miette::miette!("Category selection failed: {e}"))?;
        return Ok(choice);
    }

    if let Some(category) = suggestions.first().copied() {
        eprintln!(
            "{}",
            format!("Unknown ledger category '{input}', using closest match: {category}").yellow()
        );
        return Ok(category);
    }

    Err(miette::miette!(
        "Unknown ledger category '{input}'. Valid categories: ARCHITECTURE, FEATURE, BUGFIX, REFACTOR, INFRA, TOOLING, DOCS, CHORE"
    ))
}

pub fn execute_ledger_commit(
    tx_id: Option<String>,
    summary: &str,
    reason: &str,
    breaking: bool,
) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;

    let mut tx_mgr = TransactionManager::new(
        storage.get_connection_mut(),
        layout.root.into(),
        config.clone(),
    );

    let resolved_id = if let Some(id) = tx_id {
        tx_mgr
            .resolve_tx_id(&id)
            .map_err(|e| miette::miette!("{}", e))?
    } else {
        tx_mgr
            .get_all_pending()
            .map_err(|e| miette::miette!("{}", e))?
            .first()
            .map(|t| t.tx_id.clone())
            .ok_or_else(|| miette::miette!("No active transaction found to commit"))?
    };

    tx_mgr
        .commit_change(
            resolved_id,
            CommitRequest {
                change_type: ChangeType::Modify,
                summary: summary.to_string(),
                reason: reason.to_string(),
                is_breaking: breaking,
                ..Default::default()
            },
            false,
        )
        .map_err(|e| miette::miette!("{}", e))?;

    println!("{}", "Transaction committed.".green().bold());
    Ok(())
}

pub fn execute_ledger_rollback(tx_id: Option<String>, reason: String) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    let resolved_id = if let Some(id) = tx_id {
        tx_mgr
            .resolve_tx_id(&id)
            .map_err(|e| miette::miette!("{}", e))?
    } else {
        tx_mgr
            .get_all_pending()
            .map_err(|e| miette::miette!("{}", e))?
            .first()
            .map(|t| t.tx_id.clone())
            .ok_or_else(|| miette::miette!("No active transaction found to rollback"))?
    };

    tx_mgr
        .rollback_change(resolved_id, reason)
        .map_err(|e| miette::miette!("{}", e))?;

    println!("Transaction rolled back.");
    Ok(())
}

pub fn execute_ledger_reconcile(
    tx_id: Option<String>,
    pattern: Option<String>,
    all: bool,
    reason: Option<String>,
) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    tx_mgr
        .reconcile_drift(tx_id, pattern, all, reason.unwrap_or_default())
        .map_err(|e| miette::miette!("{}", e))?;

    println!("{}", "Drift reconciled.".green());
    Ok(())
}

pub fn execute_ledger_adopt(
    pattern: Option<String>,
    all: bool,
    category: &str,
    summary: &str,
    reason: &str,
) -> Result<()> {
    let category = Category::from_str(category, true).map_err(|e| miette::miette!("{}", e))?;
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    let tx_id = tx_mgr
        .start_change(TransactionRequest {
            category,
            entity: "drift_adoption".to_string(),
            planned_action: Some(summary.to_string()),
            ..Default::default()
        })
        .map_err(|e| miette::miette!("{}", e))?;

    tx_mgr
        .adopt_drift(Some(tx_id.clone()), pattern, all, Some(reason.to_string()))
        .map_err(|e| miette::miette!("{}", e))?;

    tx_mgr
        .commit_change(
            tx_id,
            CommitRequest {
                change_type: ChangeType::Modify,
                summary: summary.to_string(),
                reason: reason.to_string(),
                ..Default::default()
            },
            false,
        )
        .map_err(|e| miette::miette!("{}", e))?;

    println!("{}", "Drift adopted and committed.".green());
    Ok(())
}

pub fn execute_ledger_atomic(
    entity: &str,
    category: &str,
    summary: &str,
    reason: &str,
) -> Result<()> {
    let category = Category::from_str(category, true).map_err(|e| miette::miette!("{}", e))?;
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    tx_mgr
        .atomic_change(
            TransactionRequest {
                category,
                entity: entity.to_string(),
                ..Default::default()
            },
            CommitRequest {
                change_type: ChangeType::Modify,
                summary: summary.to_string(),
                reason: reason.to_string(),
                ..Default::default()
            },
            false,
        )
        .map_err(|e| miette::miette!("{}", e))?;

    println!("{}", "Atomic change committed.".green().bold());
    Ok(())
}

pub fn execute_ledger_status(
    entity_filter: Option<String>,
    compact: bool,
    exit_code: bool,
    verify_signatures: bool,
) -> Result<()> {
    let layout = get_layout()?;

    if verify_signatures {
        crate::commands::verify::verify_ledger_signatures(&layout)?;
    }

    let mut storage = StorageManager::open_read_only_sqlite_only(&layout.root)?;
    let config = load_ledger_config(&layout)?;
    let stale_threshold = config.ledger.stale_threshold_hours as i64;
    let tx_mgr = TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);
    let clock = SystemClock;

    if let Some(entity) = entity_filter {
        println!("Ledger Status for {}:", entity.cyan());
        if let Some(pending) = tx_mgr
            .get_pending(&entity)
            .map_err(|e| miette::miette!("{}", e))?
        {
            let started_at = DateTime::parse_from_rfc3339(&pending.started_at).into_diagnostic()?;
            let age_str = clock.relative_time(started_at.with_timezone(&Utc));

            let status_icon = if Utc::now()
                .signed_duration_since(started_at.with_timezone(&Utc))
                .num_hours()
                >= stale_threshold
            {
                get_status_icon(LedgerStatus::Stale)
            } else {
                get_status_icon(LedgerStatus::Pending)
            };

            println!(
                "  {} PENDING: {} [{}] {}",
                status_icon,
                pending.tx_id.yellow(),
                get_category_icon(&pending.category),
                age_str.dimmed()
            );
        } else {
            println!("  No pending transaction.");
        }

        println!("\nRecent History:");
        let entries = tx_mgr
            .get_ledger_entries(&entity)
            .map_err(|e| miette::miette!("{}", e))?;

        if entries.is_empty() {
            println!("  No history found.");
        } else {
            let mut table =
                crate::output::table::build_table(vec!["Time", "Icon", "Type", "Summary"]);
            for entry in entries.iter().take(10) {
                let committed_at =
                    DateTime::parse_from_rfc3339(&entry.committed_at).into_diagnostic()?;
                table.add_row(vec![
                    clock
                        .relative_time(committed_at.with_timezone(&Utc))
                        .dimmed()
                        .to_string(),
                    get_change_type_icon(&entry.change_type),
                    format!("{:?}", entry.change_type).blue().to_string(),
                    entry.summary.clone(),
                ]);
            }
            println!("{}", table);
        }
    } else {
        let pending = tx_mgr
            .get_all_pending()
            .map_err(|e| miette::miette!("{}", e))?;
        let unaudited = tx_mgr
            .get_all_unaudited()
            .map_err(|e| miette::miette!("{}", e))?;

        let pending_count = pending.len();
        let unaudited_count = unaudited.len();

        if compact {
            println!(
                "Ledger: {} pending, {} unaudited drift.",
                pending_count.to_string().yellow(),
                unaudited_count.to_string().red()
            );
            if exit_code && (pending_count > 0 || unaudited_count > 0) {
                std::process::exit(1);
            }
            return Ok(());
        }

        println!("{}", "ChangeGuard Ledger Status".bold().underline());

        println!(
            "\n{} {}",
            get_status_icon(LedgerStatus::Pending),
            "PENDING TRANSACTIONS".yellow().bold()
        );
        if pending.is_empty() {
            println!("  None.");
        } else {
            let mut table =
                crate::output::table::build_table(vec!["ID", "Category", "Entity", "Age"]);
            for tx in pending {
                let started_at = DateTime::parse_from_rfc3339(&tx.started_at).into_diagnostic()?;
                let age_str = clock.relative_time(started_at.with_timezone(&Utc));
                let is_stale = Utc::now()
                    .signed_duration_since(started_at.with_timezone(&Utc))
                    .num_hours()
                    >= stale_threshold;
                let stale_indicator = if is_stale {
                    format!("{} STALE", "󰀦".red())
                } else {
                    "".to_string()
                };

                table.add_row(vec![
                    tx.tx_id.yellow().to_string(),
                    format!("{} {:?}", get_category_icon(&tx.category), tx.category),
                    tx.entity.cyan().to_string(),
                    format!("{} {}", age_str.dimmed(), stale_indicator),
                ]);
            }
            println!("{}", table);
        }

        println!("\n{} {}", "󰀦".red(), "UNAUDITED DRIFT".red().bold());
        if unaudited.is_empty() {
            println!("  None.");
        } else {
            let mut table =
                crate::output::table::build_table(vec!["Entity", "Changes", "Last Seen"]);
            for tx in unaudited {
                let last_seen = if let Some(ts) = tx.last_seen_at {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(&ts) {
                        clock.relative_time(dt.with_timezone(&Utc))
                    } else {
                        ts
                    }
                } else {
                    "unknown".to_string()
                };

                table.add_row(vec![
                    tx.entity.cyan().to_string(),
                    tx.drift_count.to_string().bold().to_string(),
                    last_seen.dimmed().to_string(),
                ]);
            }
            println!("{}", table);
        }

        if exit_code && (pending_count > 0 || unaudited_count > 0) {
            std::process::exit(1);
        }
    }

    Ok(())
}

pub fn execute_ledger_resume(tx_id: Option<String>) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;
    let tx_mgr = TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    if let Some(id) = tx_id {
        let full_id = tx_mgr
            .resolve_tx_id(&id)
            .map_err(|e| miette::miette!("{}", e))?;
        println!("Resumed transaction: {}", full_id.yellow());
    } else {
        println!("Searching for most recent pending transaction in current context...");
        let pending = tx_mgr
            .get_all_pending()
            .map_err(|e| miette::miette!("{}", e))?;
        if let Some(latest) = pending.first() {
            println!(
                "Resumed most recent: {} ({})",
                latest.tx_id.yellow(),
                latest.entity.cyan()
            );
        } else {
            println!("No pending transactions found to resume.");
        }
    }
    Ok(())
}

pub fn execute_ledger_register_rule(term: &str, category: &str, reason: &str) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;
    let tx_mgr = TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    let db = LedgerDb::new(tx_mgr.get_connection());
    db.register_forbidden_term(term, category, reason)
        .map_err(|e| miette::miette!("{}", e))?;

    println!(
        "Rule registered: NO {} in {}",
        term.red().bold(),
        category.yellow()
    );
    Ok(())
}

pub fn execute_ledger_register_validator(
    name: &str,
    command: &str,
    category: &str,
    timeout: u64,
) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;
    let tx_mgr = TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    let db = LedgerDb::new(tx_mgr.get_connection());
    db.register_validator(name, command, category, timeout)
        .map_err(|e| miette::miette!("{}", e))?;

    println!(
        "Validator registered: {} for {}",
        name.cyan().bold(),
        category.yellow()
    );
    Ok(())
}

pub fn execute_ledger_gc(orphans: bool, ttl_days: u64, force: bool) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;

    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    if orphans {
        let stale_ids = {
            let db = LedgerDb::new(tx_mgr.get_connection());
            db.get_stale_pending_transactions(ttl_days)
                .map_err(|e| miette::miette!("Failed to scan for orphans: {}", e))?
        };

        if stale_ids.is_empty() {
            println!("No orphaned transactions found.");
            return Ok(());
        }

        println!(
            "Found {} orphaned PENDING transaction(s) (older than {} days).",
            stale_ids.len(),
            ttl_days
        );

        if !force {
            println!(
                "{} This will mark them as ROLLED_BACK in the ledger history.",
                "WARNING".yellow().bold()
            );
            // In autonomous mode/agent environments we might want a simple check or default to no
            // but the SOP says "respect --force flag".
            // For now, let's assume we need to prompt or just fail if not forced in non-interactive.
            if !crate::util::term::is_interactive() {
                return Err(miette::miette!(
                    "Use --force to run GC in non-interactive shells."
                ));
            }

            print!("Proceed with cleanup? (y/N): ");
            use std::io::Write;
            std::io::stdout().flush().into_diagnostic()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).into_diagnostic()?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        let mut count = 0;
        let mut failures = 0;
        for id in stale_ids {
            if let Err(e) = tx_mgr.rollback_change(
                id.clone(),
                "Garbage collection of orphaned PENDING transaction".to_string(),
            ) {
                tracing::warn!("Failed to rollback tx {}: {}", id, e);
                failures += 1;
            } else {
                count += 1;
            }
        }

        if count > 0 {
            println!(
                "{} Successfully cleaned up {} orphaned transaction(s).",
                "DONE".green().bold(),
                count
            );
        }

        if failures > 0 {
            if count == 0 {
                return Err(miette::miette!(
                    "GC failed to clean up any of the {} orphaned transaction(s). Check logs.",
                    failures
                ));
            } else {
                println!(
                    "{} Failed to clean up {} transaction(s). Check logs for details.",
                    "WARN:".yellow().bold(),
                    failures
                );
            }
        }
    } else {
        if force {
            return Err(miette::miette!("--force requires --orphans (or other GC mode)."));
        }
        println!("Please specify a GC mode (e.g. --orphans)");
    }

    Ok(())
}
