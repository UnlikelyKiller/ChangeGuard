use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::env;

use crate::ledger::*;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;

fn get_layout() -> Result<Layout> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let utf8_dir = Utf8PathBuf::from_path_buf(current_dir)
        .map_err(|_| miette::miette!("Current directory is not valid UTF-8"))?;
    Ok(Layout::new(utf8_dir))
}

fn get_tx_manager(layout: &Layout) -> Result<StorageManager> {
    let db_path = layout.state_subdir().join("ledger.db");
    StorageManager::init(db_path.as_std_path())
}

pub fn execute_ledger_start(
    entity: String,
    category: Category,
    message: Option<String>,
    issue: Option<String>,
) -> Result<()> {
    let layout = get_layout()?;
    let storage = get_tx_manager(&layout)?;
    let tx_mgr = TransactionManager::new(storage.get_connection());

    let tx_id = tx_mgr
        .start_change(TransactionRequest {
            category,
            entity,
            planned_action: message,
            issue_ref: issue,
            ..Default::default()
        })
        .map_err(|e| miette::miette!("{}", e))?;

    println!("Transaction started: {}", tx_id);
    Ok(())
}

pub fn execute_ledger_commit(
    tx_id: String,
    summary: String,
    reason: String,
    change_type: ChangeType,
    breaking: bool,
) -> Result<()> {
    let layout = get_layout()?;
    let storage = get_tx_manager(&layout)?;
    let tx_mgr = TransactionManager::new(storage.get_connection());

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
        )
        .map_err(|e| miette::miette!("{}", e))?;

    println!("Transaction committed.");
    Ok(())
}

pub fn execute_ledger_rollback(tx_id: String) -> Result<()> {
    let layout = get_layout()?;
    let storage = get_tx_manager(&layout)?;
    let tx_mgr = TransactionManager::new(storage.get_connection());

    tx_mgr
        .rollback_change(tx_id)
        .map_err(|e| miette::miette!("{}", e))?;

    println!("Transaction rolled back.");
    Ok(())
}

pub fn execute_ledger_atomic(
    entity: String,
    summary: String,
    reason: String,
    category: Category,
) -> Result<()> {
    let layout = get_layout()?;
    let storage = get_tx_manager(&layout)?;
    let tx_mgr = TransactionManager::new(storage.get_connection());

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
        )
        .map_err(|e| miette::miette!("{}", e))?;

    println!("Atomic change committed.");
    Ok(())
}

pub fn execute_ledger_note(entity: String, note: String) -> Result<()> {
    let layout = get_layout()?;
    let storage = get_tx_manager(&layout)?;
    let tx_mgr = TransactionManager::new(storage.get_connection());

    // note is a lightweight mode as per plan L1-1
    // For now we'll implement it as an atomic chore
    tx_mgr
        .atomic_change(
            TransactionRequest {
                category: Category::Chore,
                entity,
                ..Default::default()
            },
            CommitRequest {
                change_type: ChangeType::Modify,
                summary: note,
                reason: "Lightweight note".to_string(),
                ..Default::default()
            },
        )
        .map_err(|e| miette::miette!("{}", e))?;

    println!("Note added to ledger.");
    Ok(())
}

pub fn execute_ledger_status(entity_filter: Option<String>) -> Result<()> {
    let layout = get_layout()?;
    let storage = get_tx_manager(&layout)?;
    let tx_mgr = TransactionManager::new(storage.get_connection());

    // Basic implementation of status
    // In Phase L2/L4 this will be more comprehensive
    println!("Ledger Status:");
    if let Some(entity) = entity_filter {
        if let Some(pending) = tx_mgr.get_pending(&entity).map_err(|e| miette::miette!("{}", e))? {
            println!(
                "PENDING: {} [{:?}] ({})",
                pending.tx_id, pending.category, pending.entity
            );
        } else {
            println!("No pending transaction for {}", entity);
        }

        println!("\nRecent History:");
        let entries = tx_mgr
            .get_ledger_entries(&entity)
            .map_err(|e| miette::miette!("{}", e))?;
        for entry in entries {
            println!(
                "- {}: {} ({:?})",
                entry.committed_at, entry.summary, entry.change_type
            );
        }
    } else {
        // Show all pending
        // Need a method in TransactionManager or LedgerDb to list all pending
        println!("(Global status view not yet fully implemented in L1-2)");
    }

    Ok(())
}

pub fn execute_ledger_resume(tx_id: String) -> Result<()> {
    let layout = get_layout()?;
    let storage = get_tx_manager(&layout)?;
    let tx_mgr = TransactionManager::new(storage.get_connection());

    let full_id = tx_mgr
        .resolve_tx_id(&tx_id)
        .map_err(|e| miette::miette!("{}", e))?;
    println!("Resumed transaction: {}", full_id);
    // In a real implementation this might set a "current" TX ID in some local state file.
    Ok(())
}
