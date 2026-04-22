use chrono::Utc;
use rusqlite::Connection;
use uuid::Uuid;

use crate::ledger::db::LedgerDb;
use crate::ledger::error::LedgerError;
use crate::ledger::session::get_session_id;
use crate::ledger::types::*;

pub struct TransactionManager<'a> {
    db: LedgerDb<'a>,
}

impl<'a> TransactionManager<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            db: LedgerDb::new(conn),
        }
    }

    pub fn start_change(&self, req: TransactionRequest) -> Result<String, LedgerError> {
        let normalized = entity_normalized(&req.entity);
        
        if let Some(pending) = self.db.get_pending_by_entity(&normalized)? {
            return Err(LedgerError::Conflict(pending.entity));
        }

        let tx_id = Uuid::new_v4().to_string();
        let tx = Transaction {
            tx_id: tx_id.clone(),
            operation_id: req.operation_id,
            status: "PENDING".to_string(),
            category: req.category,
            entity: req.entity,
            entity_normalized: normalized,
            planned_action: req.planned_action,
            session_id: get_session_id().to_string(),
            source: req.source.unwrap_or_else(|| "CLI".to_string()),
            started_at: Utc::now().to_rfc3339(),
            resolved_at: None,
            issue_ref: req.issue_ref,
        };

        self.db.insert_transaction(&tx)?;
        Ok(tx_id)
    }

    pub fn commit_change(&self, tx_id: String, req: CommitRequest) -> Result<(), LedgerError> {
        let tx_id = self.resolve_tx_id(&tx_id)?;
        let tx = self.db.get_transaction(&tx_id)?
            .ok_or_else(|| LedgerError::NotFound(tx_id.clone()))?;

        if tx.status != "PENDING" {
            return Err(LedgerError::InvalidState(tx_id, tx.status));
        }

        let now = Utc::now().to_rfc3339();
        
        // 1. Update transaction status
        self.db.update_transaction_status(&tx_id, "RESOLVED", Some(&now))?;

        // 2. Create ledger entry
        let entry = LedgerEntry {
            id: 0, // DB will assign
            tx_id,
            category: tx.category,
            entry_type: EntryType::Implementation,
            entity: tx.entity,
            entity_normalized: tx.entity_normalized,
            change_type: req.change_type,
            summary: req.summary,
            reason: req.reason,
            is_breaking: req.is_breaking,
            committed_at: now,
        };

        self.db.insert_ledger_entry(&entry)?;

        Ok(())
    }

    pub fn rollback_change(&self, tx_id: String) -> Result<(), LedgerError> {
        let tx_id = self.resolve_tx_id(&tx_id)?;
        let tx = self.db.get_transaction(&tx_id)?
            .ok_or_else(|| LedgerError::NotFound(tx_id.clone()))?;

        if tx.status != "PENDING" {
            return Err(LedgerError::InvalidState(tx_id, tx.status));
        }

        self.db.update_transaction_status(&tx_id, "ROLLED_BACK", Some(&Utc::now().to_rfc3339()))?;
        Ok(())
    }

    pub fn atomic_change(&self, tx_req: TransactionRequest, commit_req: CommitRequest) -> Result<(), LedgerError> {
        let tx_id = self.start_change(tx_req)?;
        self.commit_change(tx_id, commit_req)
    }

    pub fn resolve_tx_id(&self, tx_id_or_prefix: &str) -> Result<String, LedgerError> {
        if tx_id_or_prefix.len() == 36 {
             // Likely a full UUID, but let's check it exists
             if self.db.get_transaction(tx_id_or_prefix)?.is_some() {
                 return Ok(tx_id_or_prefix.to_string());
             }
        }

        let matches = self.db.resolve_tx_id_fuzzy(tx_id_or_prefix)?;
        if matches.is_empty() {
            return Err(LedgerError::NotFound(tx_id_or_prefix.to_string()));
        }
        if matches.len() > 1 {
            return Err(LedgerError::Config(format!("Ambiguous transaction ID prefix '{}': matched {}", tx_id_or_prefix, matches.join(", "))));
        }
        Ok(matches[0].clone())
    }

    pub fn get_pending(&self, entity: &str) -> Result<Option<Transaction>, LedgerError> {
        self.db.get_pending_by_entity(&entity_normalized(entity))
    }

    pub fn get_transaction(&self, tx_id: &str) -> Result<Option<Transaction>, LedgerError> {
        self.db.get_transaction(tx_id)
    }

    pub fn get_ledger_entries_for_tx(&self, tx_id: &str) -> Result<Vec<LedgerEntry>, LedgerError> {
        self.db.get_ledger_entries_for_tx(tx_id)
    }

    pub fn get_ledger_entries(&self, entity: &str) -> Result<Vec<LedgerEntry>, LedgerError> {
        self.db.get_ledger_entries_by_entity(&entity_normalized(entity))
    }
}

pub fn entity_normalized(entity: &str) -> String {
    let mut normalized = entity.replace('\\', "/");
    if normalized.starts_with("./") {
        normalized = normalized[2..].to_string();
    }
    // ChangeGuard standard is forward slashes, relative to repo root.
    // In Windows, we might want to case-fold, but for now we'll stick to basic normalization.
    normalized
}
