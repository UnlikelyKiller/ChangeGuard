use rusqlite::{params, Connection, OptionalExtension};
use crate::ledger::error::LedgerError;
use crate::ledger::types::*;

pub struct LedgerDb<'a> {
    conn: &'a Connection,
}

impl<'a> LedgerDb<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn insert_transaction(&self, tx: &Transaction) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT INTO transactions (
                tx_id, operation_id, status, category, entity, entity_normalized,
                planned_action, session_id, source, started_at, issue_ref
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                tx.tx_id,
                tx.operation_id,
                tx.status,
                serde_json::to_string(&tx.category).unwrap().trim_matches('"'),
                tx.entity,
                tx.entity_normalized,
                tx.planned_action,
                tx.session_id,
                tx.source,
                tx.started_at,
                tx.issue_ref,
            ],
        )?;
        Ok(())
    }

    pub fn update_transaction_status(&self, tx_id: &str, status: &str, resolved_at: Option<&str>) -> Result<(), LedgerError> {
        self.conn.execute(
            "UPDATE transactions SET status = ?1, resolved_at = ?2 WHERE tx_id = ?3",
            params![status, resolved_at, tx_id],
        )?;
        Ok(())
    }

    pub fn get_transaction(&self, tx_id: &str) -> Result<Option<Transaction>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT tx_id, operation_id, status, category, entity, entity_normalized,
                    planned_action, session_id, source, started_at, resolved_at, issue_ref
             FROM transactions WHERE tx_id = ?1"
        )?;

        stmt.query_row([tx_id], |row| {
            let cat_str: String = row.get(3)?;
            let category: Category = serde_json::from_str(&format!("\"{}\"", cat_str)).map_err(|_| rusqlite::Error::InvalidQuery)?;
            Ok(Transaction {
                tx_id: row.get(0)?,
                operation_id: row.get(1)?,
                status: row.get(2)?,
                category,
                entity: row.get(4)?,
                entity_normalized: row.get(5)?,
                planned_action: row.get(6)?,
                session_id: row.get(7)?,
                source: row.get(8)?,
                started_at: row.get(9)?,
                resolved_at: row.get(10)?,
                issue_ref: row.get(11)?,
            })
        }).optional().map_err(LedgerError::from)
    }

    pub fn get_pending_by_entity(&self, entity_normalized: &str) -> Result<Option<Transaction>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT tx_id, operation_id, status, category, entity, entity_normalized,
                    planned_action, session_id, source, started_at, resolved_at, issue_ref
             FROM transactions WHERE entity_normalized = ?1 AND status = 'PENDING'
             ORDER BY started_at DESC LIMIT 1"
        )?;

        stmt.query_row([entity_normalized], |row| {
            let cat_str: String = row.get(3)?;
            let category: Category = serde_json::from_str(&format!("\"{}\"", cat_str)).map_err(|_| rusqlite::Error::InvalidQuery)?;
            Ok(Transaction {
                tx_id: row.get(0)?,
                operation_id: row.get(1)?,
                status: row.get(2)?,
                category,
                entity: row.get(4)?,
                entity_normalized: row.get(5)?,
                planned_action: row.get(6)?,
                session_id: row.get(7)?,
                source: row.get(8)?,
                started_at: row.get(9)?,
                resolved_at: row.get(10)?,
                issue_ref: row.get(11)?,
            })
        }).optional().map_err(LedgerError::from)
    }

    pub fn resolve_tx_id_fuzzy(&self, prefix: &str) -> Result<Vec<String>, LedgerError> {
        let mut stmt = self.conn.prepare("SELECT tx_id FROM transactions WHERE tx_id LIKE ?1")?;
        let rows = stmt.query_map([format!("{}%", prefix)], |row| row.get(0))?;
        let mut ids = Vec::new();
        for id in rows {
            ids.push(id?);
        }
        Ok(ids)
    }

    pub fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT INTO ledger_entries (
                tx_id, category, entry_type, entity, entity_normalized,
                change_type, summary, reason, is_breaking, committed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                entry.tx_id,
                serde_json::to_string(&entry.category).unwrap().trim_matches('"'),
                serde_json::to_string(&entry.entry_type).unwrap().trim_matches('"'),
                entry.entity,
                entry.entity_normalized,
                serde_json::to_string(&entry.change_type).unwrap().trim_matches('"'),
                entry.summary,
                entry.reason,
                entry.is_breaking as i32,
                entry.committed_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_ledger_entries_for_tx(&self, tx_id: &str) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tx_id, category, entry_type, entity, entity_normalized,
                    change_type, summary, reason, is_breaking, committed_at
             FROM ledger_entries WHERE tx_id = ?1"
        )?;

        let rows = stmt.query_map([tx_id], |row| {
            let cat_str: String = row.get(2)?;
            let category: Category = serde_json::from_str(&format!("\"{}\"", cat_str)).map_err(|_| rusqlite::Error::InvalidQuery)?;
            let et_str: String = row.get(3)?;
            let entry_type: EntryType = serde_json::from_str(&format!("\"{}\"", et_str)).map_err(|_| rusqlite::Error::InvalidQuery)?;
            let ct_str: String = row.get(6)?;
            let change_type: ChangeType = serde_json::from_str(&format!("\"{}\"", ct_str)).map_err(|_| rusqlite::Error::InvalidQuery)?;

            Ok(LedgerEntry {
                id: row.get(0)?,
                tx_id: row.get(1)?,
                category,
                entry_type,
                entity: row.get(4)?,
                entity_normalized: row.get(5)?,
                change_type,
                summary: row.get(7)?,
                reason: row.get(8)?,
                is_breaking: row.get::<_, i32>(9)? != 0,
                committed_at: row.get(10)?,
            })
        })?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn get_ledger_entries_by_entity(&self, entity_normalized: &str) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tx_id, category, entry_type, entity, entity_normalized,
                    change_type, summary, reason, is_breaking, committed_at
             FROM ledger_entries WHERE entity_normalized = ?1
             ORDER BY committed_at DESC"
        )?;

        let rows = stmt.query_map([entity_normalized], |row| {
             // ... duplicate parsing logic as above ...
             let cat_str: String = row.get(2)?;
             let category: Category = serde_json::from_str(&format!("\"{}\"", cat_str)).map_err(|_| rusqlite::Error::InvalidQuery)?;
             let et_str: String = row.get(3)?;
             let entry_type: EntryType = serde_json::from_str(&format!("\"{}\"", et_str)).map_err(|_| rusqlite::Error::InvalidQuery)?;
             let ct_str: String = row.get(6)?;
             let change_type: ChangeType = serde_json::from_str(&format!("\"{}\"", ct_str)).map_err(|_| rusqlite::Error::InvalidQuery)?;
 
             Ok(LedgerEntry {
                 id: row.get(0)?,
                 tx_id: row.get(1)?,
                 category,
                 entry_type,
                 entity: row.get(4)?,
                 entity_normalized: row.get(5)?,
                 change_type,
                 summary: row.get(7)?,
                 reason: row.get(8)?,
                 is_breaking: row.get::<_, i32>(9)? != 0,
                 committed_at: row.get(10)?,
             })
        })?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }
}
