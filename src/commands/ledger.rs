use camino::Utf8PathBuf;
use chrono::{DateTime, Utc};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;

use crate::config::load::load_config;
use crate::config::model::Config;
use crate::ledger::*;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::util::clock::{Clock, SystemClock};

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

pub fn execute_ledger_start(
    entity: String,
    category: Category,
    message: Option<String>,
    issue: Option<String>,
) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout);
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    let tx_id = tx_mgr
        .start_change(TransactionRequest {
            category,
            entity,
            planned_action: message,
            issue_ref: issue,
            ..Default::default()
        })
        .map_err(|e| miette::miette!("{}", e))?;

    println!("Transaction started: {}", tx_id.cyan());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn execute_ledger_commit(
    tx_id: String,
    summary: String,
    reason: String,
    change_type: ChangeType,
    breaking: bool,
    auto_reconcile: bool,
    no_auto_reconcile: bool,
    force: bool,
    with_git: bool,
    git_message: Option<String>,
    no_signoff: bool,
    dry_run: bool,
) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout);
    let should_auto_reconcile = if no_auto_reconcile {
        false
    } else if auto_reconcile {
        true
    } else {
        config.ledger.auto_reconcile
    };
    let mut tx_mgr = TransactionManager::new(
        storage.get_connection_mut(),
        layout.root.into(),
        config.clone(),
    );

    if should_auto_reconcile {
        let full_id = tx_mgr
            .resolve_tx_id(&tx_id)
            .map_err(|e| miette::miette!("{}", e))?;
        if let Some(tx) = tx_mgr
            .get_transaction(&full_id)
            .map_err(|e| miette::miette!("{}", e))?
        {
            tx_mgr
                .auto_reconcile_entity(
                    &tx.entity_normalized,
                    format!("Auto-reconciled by commit {}", full_id),
                )
                .map_err(|e| miette::miette!("{}", e))?;
        }
    }

    // Clone values needed for git commit message before they are moved
    let tx_id_for_git = tx_id.clone();
    let summary_for_git = summary.clone();
    let full_id_resolved = tx_mgr
        .resolve_tx_id(&tx_id_for_git)
        .map_err(|e| miette::miette!("{}", e))?;
    let tx_info = tx_mgr
        .get_transaction(&full_id_resolved)
        .map_err(|e| miette::miette!("{}", e))?;

    tx_mgr
        .commit_change(
            tx_id,
            CommitRequest {
                change_type,
                summary,
                reason,
                is_breaking: breaking,
                ..Default::default()
            },
            force,
        )
        .map_err(|e| miette::miette!("{}", e))?;

    println!("{}", "Transaction committed.".green().bold());

    if with_git {
        let category_str = tx_info
            .as_ref()
            .map(|t| format!("{:?}", t.category))
            .unwrap_or_else(|| "Unknown".to_string());

        let msg = if let Some(ref gm) = git_message {
            gm.clone()
        } else {
            let template = config
                .ledger
                .git_commit_template
                .clone()
                .unwrap_or_else(|| crate::git::commit::DEFAULT_COMMIT_MESSAGE_TEMPLATE.to_string());
            crate::git::commit::format_commit_message(
                &template,
                &category_str,
                &summary_for_git,
                &full_id_resolved,
            )
        };

        if dry_run {
            let signoff_flag = if no_signoff { "" } else { " --signoff" };
            println!(
                "[DRY RUN] Would execute: git commit -m \"{}\"{}",
                msg, signoff_flag
            );
            return Ok(());
        }

        // Pre-flight check
        match crate::git::commit::can_commit() {
            Ok(false) => {
                eprintln!(
                    "Warning: No staged changes detected. Stage files with `git add` first, then use `git commit --amend` if needed."
                );
            }
            Err(e) => {
                eprintln!("Warning: Cannot create git commit: {}", e);
            }
            Ok(true) => {
                // Only attempt git commit if can_commit returns true
                match crate::git::commit::git_commit(&msg, !no_signoff) {
                    Ok(()) => {
                        println!("{}", "Git commit created successfully.".green());
                    }
                    Err(ref e) => {
                        let retry_msg = msg.replace('"', "\\\"");
                        match e {
                            crate::git::commit::GitCommitError::NothingToCommit => {
                                eprintln!(
                                    "Git commit skipped: nothing to commit. Ledger commit is complete."
                                );
                            }
                            _ => {
                                eprintln!(
                                    "Git commit failed: {}. Ledger commit is complete. Retry with: git commit -m \"{}\"",
                                    e, retry_msg
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn execute_ledger_rollback(tx_id: String, reason: String) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout);
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    tx_mgr
        .rollback_change(tx_id)
        .map_err(|e| miette::miette!("{}", e))?;

    println!("Transaction rolled back. Reason: {}", reason.dimmed());
    Ok(())
}

pub fn execute_ledger_reconcile(
    tx_id: Option<String>,
    pattern: Option<String>,
    all: bool,
    reason: String,
) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout);
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    tx_mgr
        .reconcile_drift(tx_id, pattern, all, reason)
        .map_err(|e| miette::miette!("{}", e))?;

    println!("{}", "Drift reconciled.".green());
    Ok(())
}

pub fn execute_ledger_adopt(
    tx_id: Option<String>,
    pattern: Option<String>,
    all: bool,
    reason: String,
) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout);
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    tx_mgr
        .adopt_drift(tx_id, pattern, all, Some(reason))
        .map_err(|e| miette::miette!("{}", e))?;

    println!("{}", "Drift adopted into pending transactions.".green());
    Ok(())
}

pub fn execute_ledger_atomic(
    entity: String,
    summary: String,
    reason: String,
    category: Category,
) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout);
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    tx_mgr
        .atomic_change(
            TransactionRequest {
                category,
                entity,
                ..Default::default()
            },
            CommitRequest {
                change_type: ChangeType::Modify,
                summary,
                reason,
                ..Default::default()
            },
            false, // atomic commits don't support --force bypass
        )
        .map_err(|e| miette::miette!("{}", e))?;

    println!("{}", "Atomic change committed.".green().bold());
    Ok(())
}

pub fn execute_ledger_note(
    entity: String,
    message: Option<String>,
    note: Option<String>,
) -> Result<()> {
    let note_text = match (message, note) {
        (Some(msg), None) => msg,
        (None, Some(pos)) => pos,
        (Some(msg), Some(_)) => msg,
        (None, None) => {
            return Err(miette::miette!(
                "Missing required note text. Use --message <note> or pass it as a positional argument."
            ));
        }
    };

    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout);
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    tx_mgr
        .atomic_change(
            TransactionRequest {
                category: Category::Chore,
                entity,
                ..Default::default()
            },
            CommitRequest {
                change_type: ChangeType::Modify,
                summary: note_text,
                reason: "Lightweight note".to_string(),
                ..Default::default()
            },
            false, // note commits use Chore category, never requires verification
        )
        .map_err(|e| miette::miette!("{}", e))?;

    println!("{}", "Note added to ledger.".green());
    Ok(())
}

pub fn execute_ledger_status(
    entity_filter: Option<String>,
    compact: bool,
    exit_code: bool,
) -> Result<()> {
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout);
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
                pending_count.yellow(),
                unaudited_count.red()
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
    let config = load_ledger_config(&layout);
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
