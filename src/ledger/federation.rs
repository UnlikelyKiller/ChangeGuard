use crate::federated::schema::FederatedLedgerEntry;
use crate::ledger::db::LedgerDb;
use crate::ledger::error::LedgerError;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::Path;

pub fn export_ledger_entries(
    conn: &Connection,
    days: i64,
) -> Result<Vec<FederatedLedgerEntry>, LedgerError> {
    let db = LedgerDb::new(conn);
    let all_entries = db.get_all_committed_ledger_entries()?;

    let limit = Utc::now() - chrono::Duration::days(days);

    let federated_entries: Vec<FederatedLedgerEntry> = all_entries
        .into_iter()
        .filter(|e| e.origin == "LOCAL")
        .filter(|e| {
            DateTime::parse_from_rfc3339(&e.committed_at)
                .map(|dt| dt.with_timezone(&Utc) >= limit)
                .unwrap_or(false)
        })
        .map(|e| FederatedLedgerEntry {
            tx_id: e.tx_id,
            category: e.category,
            entry_type: e.entry_type,
            entity: e.entity_normalized,
            change_type: e.change_type,
            summary: e.summary,
            reason: e.reason,
            is_breaking: e.is_breaking,
            committed_at: e.committed_at,
        })
        .collect();

    Ok(federated_entries)
}

pub fn import_federated_entries(
    conn: &mut Connection,
    repo_root: &Path,
    sibling_name: &str,
    entries: &[FederatedLedgerEntry],
) -> Result<(), LedgerError> {
    let is_case_insensitive = repo_root.join(".git").exists() || repo_root.join(".GIT").exists();
    let sqlite_tx = conn.transaction().map_err(LedgerError::from)?;
    {
        for entry in entries {
            // Security: Path Confinement and Normalization
            let mut normalized =
                crate::util::path::normalize_relative_path(repo_root, &entry.entity)
                    .map_err(LedgerError::Validation)?;

            if is_case_insensitive {
                normalized = normalized.to_lowercase();
            }

            // Check if it already exists (by tx_id and the sibling name as trace_id)
            let mut stmt = sqlite_tx.prepare(
                "SELECT COUNT(*) FROM ledger_entries WHERE tx_id = ?1 AND trace_id = ?2 AND origin = 'SIBLING'",
            )?;
            let count: i64 = stmt.query_row([&entry.tx_id, sibling_name], |row| row.get(0))?;
            if count > 0 {
                continue;
            }

            // FK Requirement: Must have a matching transaction record.
            // We'll insert a stub transaction record for the federated entry.
            sqlite_tx.execute(
                "INSERT OR IGNORE INTO transactions (
                    tx_id, status, category, entity, entity_normalized, session_id, source, started_at, resolved_at
                ) VALUES (?1, 'COMMITTED', ?2, ?3, ?4, 'FEDERATED', 'FEDERATED', ?5, ?5)",
                rusqlite::params![
                    entry.tx_id,
                    serde_json::to_string(&entry.category).map_err(|e| LedgerError::Config(e.to_string()))?.trim_matches('"'),
                    entry.entity,
                    normalized,
                    entry.committed_at,
                ],
            )?;

            // Insert into ledger_entries
            // origin = 'SIBLING', trace_id = sibling_name
            sqlite_tx.execute(
                "INSERT INTO ledger_entries (
                    tx_id, category, entry_type, entity, entity_normalized,
                    change_type, summary, reason, is_breaking, committed_at,
                    origin, trace_id
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'SIBLING', ?11)",
                rusqlite::params![
                    entry.tx_id,
                    serde_json::to_string(&entry.category)
                        .map_err(|e| LedgerError::Config(e.to_string()))?
                        .trim_matches('"'),
                    serde_json::to_string(&entry.entry_type)
                        .map_err(|e| LedgerError::Config(e.to_string()))?
                        .trim_matches('"'),
                    entry.entity,
                    normalized,
                    serde_json::to_string(&entry.change_type)
                        .map_err(|e| LedgerError::Config(e.to_string()))?
                        .trim_matches('"'),
                    entry.summary,
                    entry.reason,
                    entry.is_breaking as i32,
                    entry.committed_at,
                    sibling_name,
                ],
            )?;
        }
    }
    sqlite_tx.commit().map_err(LedgerError::from)?;
    Ok(())
}
