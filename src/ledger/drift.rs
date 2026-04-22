use chrono::Utc;
use rusqlite::Connection;
use std::path::PathBuf;
use uuid::Uuid;

use crate::ledger::db::LedgerDb;
use crate::ledger::error::LedgerError;
use crate::ledger::session::get_session_id;
use crate::ledger::types::*;

pub struct DriftManager<'a> {
    conn: &'a mut Connection,
    repo_root: PathBuf,
    is_case_insensitive: bool,
}

impl<'a> DriftManager<'a> {
    pub fn new(conn: &'a mut Connection, repo_root: PathBuf) -> Self {
        let is_case_insensitive = repo_root.join(".GIT").exists();
        Self {
            conn,
            repo_root,
            is_case_insensitive,
        }
    }

    pub fn process_event(&mut self, path: &str) -> Result<(), LedgerError> {
        let normalized = self.entity_normalized(path)?;
        let db = LedgerDb::new(self.conn);

        // Check for PENDING transaction
        if db.get_pending_by_entity(&normalized)?.is_some() {
            // Already tracked
            return Ok(());
        }

        // If none, call upsert_unaudited_transaction
        let now = Utc::now().to_rfc3339();
        let tx = Transaction {
            tx_id: Uuid::new_v4().to_string(),
            operation_id: None,
            status: "UNAUDITED".to_string(),
            category: Category::Feature, // Default for untracked changes
            entity: path.to_string(),
            entity_normalized: normalized,
            planned_action: None,
            session_id: get_session_id().to_string(),
            source: "WATCHER".to_string(),
            started_at: now.clone(),
            resolved_at: None,
            detected_at: Some(now.clone()),
            drift_count: 1,
            first_seen_at: Some(now.clone()),
            last_seen_at: Some(now.clone()),
            issue_ref: None,
        };

        db.upsert_unaudited_transaction(&tx)?;
        Ok(())
    }

    fn entity_normalized(&self, entity: &str) -> Result<String, LedgerError> {
        let mut normalized = crate::util::path::normalize_relative_path(&self.repo_root, entity)
            .map_err(LedgerError::Config)?;

        if self.is_case_insensitive {
            normalized = normalized.to_lowercase();
        }

        Ok(normalized)
    }
}
