use crate::commands::helpers::{get_layout, load_ledger_config};
use crate::git::commit::{DEFAULT_COMMIT_MESSAGE_TEMPLATE, format_commit_message, git_commit};
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

#[derive(Debug, Clone, Default)]
pub struct LedgerCommitGitOptions {
    pub with_git: bool,
    pub git_message: Option<String>,
    pub signoff: bool,
    pub dry_run: bool,
}

pub fn execute_ledger_commit(
    tx_id: Option<String>,
    summary: &str,
    reason: &str,
    breaking: bool,
    git_options: LedgerCommitGitOptions,
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

    let tx_category = tx_mgr
        .get_transaction(&resolved_id)
        .map_err(|e| miette::miette!("{}", e))?
        .ok_or_else(|| miette::miette!("Transaction not found: {resolved_id}"))?
        .category
        .to_string();

    tx_mgr
        .commit_change(
            resolved_id.clone(),
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

    let changed_files = match tx_mgr.get_transaction_files(&resolved_id) {
        Ok(files) => files,
        Err(e) => {
            tracing::warn!(
                "ledger commit: could not discover changed files for KG edges (tx={resolved_id}): {e}"
            );
            vec![]
        }
    };
    drop(tx_mgr);

    write_ledger_graph_edges(
        &storage.cozo,
        &resolved_id,
        summary,
        reason,
        &tx_category,
        changed_files,
    );

    println!("{}", "Transaction committed.".green().bold());

    if git_options.with_git {
        execute_git_commit(
            &config.ledger.git_commit_template,
            &tx_category,
            summary,
            &resolved_id,
            git_options,
        );
    }

    Ok(())
}

fn execute_git_commit(
    configured_template: &Option<String>,
    category: &str,
    summary: &str,
    tx_id: &str,
    options: LedgerCommitGitOptions,
) {
    let message = options.git_message.unwrap_or_else(|| {
        let template = configured_template
            .as_deref()
            .unwrap_or(DEFAULT_COMMIT_MESSAGE_TEMPLATE);
        format_commit_message(template, category, summary, tx_id)
    });

    if options.dry_run {
        println!(
            "Dry run: {}",
            display_git_commit_command(&message, options.signoff)
        );
        return;
    }

    match crate::git::commit::can_commit() {
        Ok(true) => {}
        Ok(false) => {
            eprintln!(
                "{}",
                "Warning: Git commit skipped because no files are staged. Ledger commit is complete. Stage files and retry git manually.".yellow()
            );
            return;
        }
        Err(err) => {
            eprintln!(
                "{}",
                format!(
                    "Warning: Git commit skipped: {err}. Ledger commit is complete. Resolve git state and retry manually."
                )
                .yellow()
            );
            return;
        }
    }

    match git_commit(&message, options.signoff) {
        Ok(()) => println!("{}", "Git commit created.".green().bold()),
        Err(err) => {
            eprintln!(
                "{}",
                format!(
                    "Warning: Git commit failed: {err}. Ledger commit is complete. Retry with: {}",
                    display_git_commit_command(&message, options.signoff)
                )
                .yellow()
            );
        }
    }
}

fn display_git_commit_command(message: &str, signoff: bool) -> String {
    let escaped_message = message.replace('"', "\\\"");
    let mut command = format!("git commit -m \"{escaped_message}\"");
    if signoff {
        command.push_str(" --signoff");
    }
    command
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

    // Collect unaudited drift entities *before* adopt to derive real file set for KG edges.
    // adopt_drift returns the tx_ids it promoted; we then gather files from each.
    let adopted_tx_ids = tx_mgr
        .adopt_drift(None, pattern, all, Some(reason.to_string()))
        .map_err(|e| miette::miette!("{}", e))?;

    if adopted_tx_ids.is_empty() {
        println!("No unaudited drift found to adopt.");
        return Ok(());
    }

    // Collect all changed files from the adopted drift transactions for KG provenance.
    let mut changed_files: Vec<String> = Vec::new();
    for adopted_id in &adopted_tx_ids {
        match tx_mgr.get_transaction_files(adopted_id) {
            Ok(mut files) => changed_files.append(&mut files),
            Err(e) => tracing::warn!(
                "ledger adopt: could not discover changed files for KG edges (adopted_tx={adopted_id}): {e}"
            ),
        }
    }
    changed_files.sort_unstable();
    changed_files.dedup();

    // Use the first adopted entity as the ledger transaction entity for human readability.
    // For multi-entity adoption, we use the count as a synthetic label.
    let adopt_entity = if adopted_tx_ids.len() == 1 {
        adopted_tx_ids[0].clone()
    } else {
        format!("drift_adoption:{}_items", adopted_tx_ids.len())
    };

    let tx_id = tx_mgr
        .start_change(TransactionRequest {
            category,
            entity: adopt_entity,
            planned_action: Some(summary.to_string()),
            ..Default::default()
        })
        .map_err(|e| miette::miette!("{}", e))?;

    let category_str = category.to_string();
    tx_mgr
        .commit_change(
            tx_id.clone(),
            CommitRequest {
                change_type: ChangeType::Modify,
                summary: summary.to_string(),
                reason: reason.to_string(),
                ..Default::default()
            },
            false,
        )
        .map_err(|e| miette::miette!("{}", e))?;

    drop(tx_mgr);

    write_ledger_graph_edges(
        &storage.cozo,
        &tx_id,
        summary,
        reason,
        &category_str,
        changed_files,
    );

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
    let category_str = category.to_string();
    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;
    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    let tx_id = tx_mgr
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

    let changed_files = match tx_mgr.get_transaction_files(&tx_id) {
        Ok(files) => files,
        Err(e) => {
            tracing::warn!(
                "ledger atomic: could not discover changed files for KG edges (tx={tx_id}): {e}"
            );
            vec![]
        }
    };
    drop(tx_mgr);

    write_ledger_graph_edges(
        &storage.cozo,
        &tx_id,
        summary,
        reason,
        &category_str,
        changed_files,
    );

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

pub fn execute_ledger_gc(stale: bool, orphans: bool, ttl_hours: u64, force: bool) -> Result<()> {
    // No-args UX: show usage if neither mode was selected
    if !stale && !orphans {
        println!(
            "{}",
            "Usage: changeguard ledger gc --stale [--ttl-hours <N>] | --orphans [--force]".cyan()
        );
        println!();
        println!(
            "  {}  Remove PENDING transactions older than TTL (default: 72h)",
            "--stale".bold()
        );
        println!(
            "  {}  Remove transactions with no corresponding git commit",
            "--orphans".bold()
        );
        return Ok(());
    }

    let layout = get_layout()?;
    let mut storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let config = load_ledger_config(&layout)?;

    let mut tx_mgr =
        TransactionManager::new(storage.get_connection_mut(), layout.root.into(), config);

    if stale {
        let stale_ids = {
            let db = LedgerDb::new(tx_mgr.get_connection());
            let ttl_days = ttl_hours.div_ceil(24);
            db.get_stale_pending_transactions(ttl_days)
                .map_err(|e| miette::miette!("Failed to scan for stale transactions: {}", e))?
        };

        if stale_ids.is_empty() {
            println!("No stale PENDING transactions found.");
            return Ok(());
        }

        println!(
            "Found {} stale PENDING transaction(s) (older than {} hours).",
            stale_ids.len(),
            ttl_hours
        );

        if !force {
            println!(
                "{} This will mark them as ROLLED_BACK in the ledger history.",
                "WARNING".yellow().bold()
            );
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
                "Garbage collection of stale PENDING transaction".to_string(),
            ) {
                tracing::warn!("Failed to rollback tx {}: {}", id, e);
                failures += 1;
            } else {
                count += 1;
            }
        }

        if count > 0 {
            println!(
                "{} Successfully cleaned up {} stale transaction(s).",
                "DONE".green().bold(),
                count
            );
        }

        if failures > 0 {
            if count == 0 {
                return Err(miette::miette!(
                    "GC failed to clean up any of the {} stale transaction(s). Check logs.",
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
    }

    if orphans {
        let stale_ids = {
            let db = LedgerDb::new(tx_mgr.get_connection());
            let ttl_days = ttl_hours.div_ceil(24);
            db.get_stale_pending_transactions(ttl_days)
                .map_err(|e| miette::miette!("Failed to scan for orphans: {}", e))?
        };

        if stale_ids.is_empty() {
            println!("No orphaned transactions found.");
            return Ok(());
        }

        println!(
            "Found {} orphaned PENDING transaction(s) (older than {} hours).",
            stale_ids.len(),
            ttl_hours
        );

        if !force {
            println!(
                "{} This will mark them as ROLLED_BACK in the ledger history.",
                "WARNING".yellow().bold()
            );
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
    }

    Ok(())
}

pub fn execute_ledger_hook_repair(force: bool) -> Result<()> {
    let layout = get_layout()?;
    let sidecar_path = layout.state_subdir().join("pending_hook_tx");

    println!("{}", "ChangeGuard Hook Repair".bold().cyan());

    if !sidecar_path.exists() {
        println!("No pending hook sidecar found. Lifecycle is consistent.");
        return Ok(());
    }

    let sidecar_content = std::fs::read_to_string(&sidecar_path).into_diagnostic()?;
    let pending: crate::commands::hook_post_commit::PendingHookTx =
        serde_json::from_str(&sidecar_content).into_diagnostic()?;

    // Fetch HEAD commit msg
    let repo_root = layout.root.clone();
    let git_out = std::process::Command::new("git")
        .args(["log", "-1", "--format=%B"])
        .current_dir(repo_root)
        .output()
        .into_diagnostic()?;

    let current_commit_msg = String::from_utf8_lossy(&git_out.stdout).to_string();
    let cleaned_msg = crate::util::text::clean_commit_msg(&current_commit_msg);

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(cleaned_msg.as_bytes());
    let current_hash = hex::encode(hasher.finalize());

    if pending.commit_msg_hash == current_hash {
        println!("Hook sidecar is matching HEAD commit hash. No repair needed.");
        return Ok(());
    }

    println!(
        "{} Hook sidecar mismatch detected!",
        "WARNING:".yellow().bold()
    );
    println!("  Pending Tx ID: {}", pending.tx_id);
    println!(
        "  Expected commit message hash: {}",
        pending.commit_msg_hash
    );
    println!("  Current HEAD commit message hash: {}", current_hash);

    if force {
        println!("Repairing hook state (force)...");
        let storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
        let db = LedgerDb::new(storage.get_connection());
        let now = chrono::Utc::now().to_rfc3339();
        let _ = db.update_transaction_status(&pending.tx_id, "ROLLBACK", Some(&now));
        let _ = std::fs::remove_file(&sidecar_path);
        println!(
            "{} Stale sidecar removed and transaction rolled back in DB.",
            "SUCCESS:".green().bold()
        );
    } else {
        println!(
            "\nRun with --force to repair the hook state by rolling back the stale transaction and removing the sidecar."
        );
    }

    Ok(())
}

pub fn execute_ledger_export_provenance(output: Option<String>) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::open_read_only(&layout.root)?;
    let db = LedgerDb::new(storage.get_connection());
    let entries = db
        .get_all_committed_ledger_entries()
        .map_err(|e| miette::miette!("{}", e))?;

    let output_path = output.unwrap_or_else(|| "provenance-export.json".to_string());
    let file = std::fs::File::create(&output_path).into_diagnostic()?;
    serde_json::to_writer_pretty(file, &entries).into_diagnostic()?;

    println!(
        "{} Stable provenance exported to {}",
        "SUCCESS:".green().bold(),
        output_path
    );
    Ok(())
}

/// Write transaction-affects edges to the KG (CozoDB) so that
/// `ledger graph <tx-id>` returns the entity neighborhood.
fn write_ledger_graph_edges(
    cozo_opt: &Option<crate::state::storage_cozo::CozoStorage>,
    tx_id: &str,
    summary: &str,
    reason: &str,
    category: &str,
    changed_files: Vec<String>,
) {
    if let Some(cozo) = cozo_opt {
        use crate::platform::urn::build_urn;
        use crate::state::graph_kinds::{EdgeKind, NodeKind};
        use crate::state::storage_cozo::{GraphEdge, GraphNode};

        let tx_urn = build_urn(NodeKind::LedgerTransaction, tx_id);

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        nodes.push(GraphNode {
            id: tx_urn.clone(),
            label: tx_id.to_string(),
            category: NodeKind::LedgerTransaction,
            risk_score: 0.0,
            metadata: Some(serde_json::json!({
                "summary": summary,
                "reason": reason,
                "category": category,
            })),
        });

        for file in changed_files {
            let file_urn = build_urn(NodeKind::File, &file);
            edges.push(GraphEdge {
                source: tx_urn.clone(),
                target: file_urn,
                relation: EdgeKind::Affects,
                confidence: 1.0,
                provenance_id: tx_id.to_string(),
            });
        }

        if let Err(e) = cozo.insert_nodes(&nodes) {
            tracing::warn!("ledger graph: failed to write transaction node: {}", e);
        }
        if let Err(e) = cozo.insert_edges(&edges) {
            tracing::warn!("ledger graph: failed to write KG edges: {}", e);
        }
    }
}
