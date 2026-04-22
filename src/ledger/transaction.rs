use chrono::Utc;
use globset::{Glob, GlobSetBuilder};
use rusqlite::Connection;
use std::path::PathBuf;
use uuid::Uuid;

use crate::config::model::Config;
use crate::ledger::db::LedgerDb;
use crate::ledger::enforcement::ValidationLevel;
use crate::ledger::error::LedgerError;
use crate::ledger::provenance::{ProvenanceAction, TokenProvenance};
use crate::ledger::session::get_session_id;
use crate::ledger::types::*;
use crate::ledger::validators::ValidatorRunner;
use crate::platform::process_policy::ProcessPolicy;

pub struct TransactionManager<'a> {
    conn: &'a mut Connection,
    repo_root: PathBuf,
    is_case_insensitive: bool,
    config: Config,
}

impl<'a> TransactionManager<'a> {
    pub fn new(conn: &'a mut Connection, repo_root: PathBuf, config: Config) -> Self {
        let is_case_insensitive = repo_root.join(".GIT").exists();
        Self {
            conn,
            repo_root,
            is_case_insensitive,
            config,
        }
    }

    pub fn get_connection(&self) -> &Connection {
        self.conn
    }

    pub fn start_change(&mut self, req: TransactionRequest) -> Result<String, LedgerError> {
        let normalized = self.entity_normalized(&req.entity)?;

        let db = LedgerDb::new(self.conn);
        if let Some(pending) = db.get_pending_by_entity(&normalized)? {
            return Err(LedgerError::Conflict(pending.entity));
        }

        // Tech Stack Enforcement
        if self.config.ledger.enforcement_enabled {
            let cat_str = serde_json::to_string(&req.category)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            let mappings = db.get_category_mappings(Some(&cat_str))?;

            for m in mappings {
                // Check CategoryStackMapping.glob against entity if present
                if let Some(ref mapping_glob) = m.glob
                    && let Ok(glob) = Glob::new(mapping_glob)
                    && let Ok(set) = GlobSetBuilder::new().add(glob).build()
                    && !set.is_match(&normalized)
                {
                    continue;
                }

                if let Some(rule) = db.get_tech_stack_rule(&m.stack_category)? {
                    for rule_text in &rule.rules {
                        if rule_text.to_uppercase().starts_with("NO ") {
                            let term = &rule_text[3..];
                            let term_lower = term.to_lowercase();

                            // Check planned_action text (if provided)
                            if let Some(ref action) = req.planned_action
                                && action.to_lowercase().contains(&term_lower)
                            {
                                return Err(LedgerError::RuleViolation(format!(
                                    "Planned action violates tech stack rule for {} (forbidden term: {})",
                                    rule.name,
                                    term_lower
                                )));
                            }

                            // Also check entity path
                            if normalized.to_lowercase().contains(&term_lower) {
                                return Err(LedgerError::RuleViolation(format!(
                                    "Entity path violates tech stack rule for {} (forbidden term: {})",
                                    rule.name,
                                    term_lower
                                )));
                            }
                        }
                    }
                }
            }
        }

        let tx_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
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
            started_at: now.clone(),
            resolved_at: None,
            issue_ref: req.issue_ref,
            detected_at: None,
            drift_count: 1,
            first_seen_at: Some(now.clone()),
            last_seen_at: Some(now),
        };

        db.insert_transaction(&tx).map_err(|e| {
            if let LedgerError::Database(ref sqlite_err) = e
                && sqlite_err.to_string().contains("UNIQUE constraint failed")
            {
                return LedgerError::Conflict(tx.entity);
            }
            e
        })?;
        Ok(tx_id)
    }

    pub fn commit_change(&mut self, tx_id: String, req: CommitRequest) -> Result<(), LedgerError> {
        let tx_id = self.resolve_tx_id(&tx_id)?;

        let tx = {
            let db = LedgerDb::new(self.conn);
            db.get_transaction(&tx_id)?
                .ok_or_else(|| LedgerError::NotFound(tx_id.clone()))?
        };

        if tx.status != "PENDING" {
            return Err(LedgerError::InvalidState(tx_id, tx.status));
        }

        // Commit Validation
        let db = LedgerDb::new(self.conn);
        let cat_str = serde_json::to_string(&tx.category)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        let validators = db.get_commit_validators(Some(&cat_str))?;
        if !validators.is_empty() {
            // Build process policy from config
            let policy = ProcessPolicy {
                default_timeout_secs: self.config.verify.default_timeout_secs,
                ..Default::default()
            };

            for v in validators {
                if !v.enabled {
                    continue;
                }
                // Check glob if present using proper globset matching
                if let Some(ref pattern) = v.glob {
                    let glob = Glob::new(pattern)
                        .map_err(|e| LedgerError::Validation(format!("Invalid glob pattern '{}': {}", pattern, e)))?;
                    let mut builder = GlobSetBuilder::new();
                    builder.add(glob);
                    let globset = builder.build()
                        .map_err(|e| LedgerError::Validation(format!("Failed to build globset for '{}': {}", pattern, e)))?;
                    if !globset.is_match(&tx.entity_normalized) {
                        continue;
                    }
                }

                let absolute_path = self.repo_root.join(&tx.entity_normalized);
                let entity_path_str = absolute_path.to_string_lossy();

                let result = ValidatorRunner::run(
                    v.name.clone(),
                    &v.executable,
                    &v.args,
                    &entity_path_str,
                    v.timeout_ms as u64,
                    v.validation_level,
                    &policy,
                )?;

                if !result.success {
                    match v.validation_level {
                        ValidationLevel::Error => {
                            return Err(LedgerError::ValidatorFailed(
                                v.name,
                                format!("STDOUT: {}\nSTDERR: {}", result.stdout, result.stderr),
                            ));
                        }
                        ValidationLevel::Warning => {
                            eprintln!(
                                "WARNING: Validator '{}' failed:\nSTDOUT: {}\nSTDERR: {}",
                                v.name, result.stdout, result.stderr
                            );
                        }
                    }
                }
            }
        }

        let now = Utc::now().to_rfc3339();

        // Use a database transaction to ensure atomicity
        let sqlite_tx = self.conn.transaction().map_err(LedgerError::from)?;
        {
            let db = LedgerDb::new(&sqlite_tx);

            // 1. Update transaction status to COMMITTED
            let count = db.update_transaction_status(&tx_id, "COMMITTED", Some(&now))?;
            if count == 0 {
                return Err(LedgerError::InvalidState(
                    tx_id,
                    "already resolved".to_string(),
                ));
            }

            // 2. Create ledger entry
            let entry_type = if tx.category == Category::Architecture {
                EntryType::Architecture
            } else {
                EntryType::Implementation
            };

            let entry = LedgerEntry {
                id: 0, // DB will assign
                tx_id,
                category: tx.category,
                entry_type,
                entity: tx.entity,
                entity_normalized: tx.entity_normalized,
                change_type: req.change_type,
                summary: req.summary,
                reason: req.reason,
                is_breaking: req.is_breaking,
                committed_at: now,
                verification_status: req.verification_status,
                verification_basis: req.verification_basis,
                outcome_notes: req.outcome_notes,
                origin: "LOCAL".to_string(),
                trace_id: None,
            };

            db.insert_ledger_entry(&entry)?;
        }
        sqlite_tx.commit().map_err(LedgerError::from)?;

        Ok(())
    }

    pub fn rollback_change(&mut self, tx_id: String) -> Result<(), LedgerError> {
        let tx_id = self.resolve_tx_id(&tx_id)?;
        let db = LedgerDb::new(self.conn);
        let tx = db
            .get_transaction(&tx_id)?
            .ok_or_else(|| LedgerError::NotFound(tx_id.clone()))?;

        if tx.status != "PENDING" {
            return Err(LedgerError::InvalidState(tx_id, tx.status));
        }

        let count =
            db.update_transaction_status(&tx_id, "ROLLED_BACK", Some(&Utc::now().to_rfc3339()))?;
        if count == 0 {
            return Err(LedgerError::InvalidState(
                tx_id,
                "already resolved".to_string(),
            ));
        }
        Ok(())
    }

    pub fn atomic_change(
        &mut self,
        tx_req: TransactionRequest,
        commit_req: CommitRequest,
    ) -> Result<(), LedgerError> {
        let tx_id = self.start_change(tx_req)?;
        if let Err(commit_err) = self.commit_change(tx_id.clone(), commit_req) {
            // Attempt cleanup rollback; prefer returning the original error
            if let Err(rollback_err) = self.rollback_change(tx_id) {
                tracing::warn!("atomic_change: rollback after commit failure also failed: {rollback_err}");
            }
            return Err(commit_err);
        }
        Ok(())
    }

    pub fn reconcile_drift(
        &mut self,
        tx_id: Option<String>,
        pattern: Option<String>,
        all: bool,
        reason: String,
    ) -> Result<(), LedgerError> {
        let db = LedgerDb::new(self.conn);
        let to_reconcile = if all {
            db.get_all_unaudited()?
        } else if let Some(p) = pattern {
            db.get_unaudited_by_pattern(&p)?
        } else if let Some(id) = tx_id {
            let full_id = self.resolve_tx_id(&id)?;
            let tx = db
                .get_transaction(&full_id)?
                .ok_or_else(|| LedgerError::NotFound(full_id.clone()))?;
            if tx.status != "UNAUDITED" {
                return Err(LedgerError::InvalidState(full_id, tx.status));
            }
            vec![tx]
        } else {
            return Err(LedgerError::Config(
                "Must specify --tx-id, --entity-pattern, or --all for reconciliation".to_string(),
            ));
        };

        if to_reconcile.is_empty() {
            return Ok(());
        }

        let now = Utc::now().to_rfc3339();
        let sqlite_tx = self.conn.transaction().map_err(LedgerError::from)?;
        {
            let db = LedgerDb::new(&sqlite_tx);
            let tx_ids: Vec<String> = to_reconcile.iter().map(|tx| tx.tx_id.clone()).collect();
            db.update_transaction_status_bulk(&tx_ids, "RECONCILED", Some(&now))?;

            for tx in to_reconcile {
                let entry = LedgerEntry {
                    id: 0,
                    tx_id: tx.tx_id,
                    category: tx.category,
                    entry_type: EntryType::Reconciliation,
                    entity: tx.entity,
                    entity_normalized: tx.entity_normalized,
                    change_type: ChangeType::Modify,
                    summary: format!("Reconciled drift ({} changes)", tx.drift_count),
                    reason: reason.clone(),
                    is_breaking: false,
                    committed_at: now.clone(),
                    verification_status: None,
                    verification_basis: None,
                    outcome_notes: None,
                    origin: "LOCAL".to_string(),
                    trace_id: None,
                };
                db.insert_ledger_entry(&entry)?;
            }
        }
        sqlite_tx.commit().map_err(LedgerError::from)?;

        Ok(())
    }

    pub fn adopt_drift(
        &mut self,
        tx_id: Option<String>,
        pattern: Option<String>,
        all: bool,
        reason: Option<String>,
    ) -> Result<(), LedgerError> {
        let db = LedgerDb::new(self.conn);
        let to_adopt = if all {
            db.get_all_unaudited()?
        } else if let Some(p) = pattern {
            db.get_unaudited_by_pattern(&p)?
        } else if let Some(id) = tx_id {
            let full_id = self.resolve_tx_id(&id)?;
            let tx = db
                .get_transaction(&full_id)?
                .ok_or_else(|| LedgerError::NotFound(full_id.clone()))?;
            if tx.status != "UNAUDITED" {
                return Err(LedgerError::InvalidState(full_id, tx.status));
            }
            vec![tx]
        } else {
            return Err(LedgerError::Config(
                "Must specify --tx-id, --entity-pattern, or --all for adoption".to_string(),
            ));
        };

        if to_adopt.is_empty() {
            return Ok(());
        }

        let tx_ids: Vec<String> = to_adopt.iter().map(|tx| tx.tx_id.clone()).collect();
        db.update_transaction_status_bulk(&tx_ids, "PENDING", None)?;

        if let Some(reason_text) = reason {
            tracing::info!("Adopted drift with reason: {reason_text}");
        }

        Ok(())
    }

    pub fn auto_reconcile_entity(
        &mut self,
        entity_normalized: &str,
        reason: String,
    ) -> Result<(), LedgerError> {
        let db = LedgerDb::new(self.conn);
        if let Some(tx) = db.get_unaudited_by_entity(entity_normalized)? {
            self.reconcile_drift(Some(tx.tx_id), None, false, reason)?;
        }
        Ok(())
    }

    pub fn resolve_tx_id(&self, tx_id_or_prefix: &str) -> Result<String, LedgerError> {
        let db = LedgerDb::new(self.conn);
        if tx_id_or_prefix.len() == 36 && db.get_transaction(tx_id_or_prefix)?.is_some() {
            return Ok(tx_id_or_prefix.to_string());
        }

        let matches = db.resolve_tx_id_fuzzy(tx_id_or_prefix)?;
        if matches.is_empty() {
            return Err(LedgerError::NotFound(tx_id_or_prefix.to_string()));
        }
        if matches.len() > 1 {
            return Err(LedgerError::Config(format!(
                "Ambiguous transaction ID prefix '{}': matched {}",
                tx_id_or_prefix,
                matches.join(", ")
            )));
        }
        Ok(matches[0].clone())
    }

    pub fn get_pending(&self, entity: &str) -> Result<Option<Transaction>, LedgerError> {
        let normalized = self.entity_normalized(entity)?;
        let db = LedgerDb::new(self.conn);
        db.get_pending_by_entity(&normalized)
    }

    pub fn get_transaction(&self, tx_id: &str) -> Result<Option<Transaction>, LedgerError> {
        let db = LedgerDb::new(self.conn);
        db.get_transaction(tx_id)
    }

    pub fn get_ledger_entries_for_tx(&self, tx_id: &str) -> Result<Vec<LedgerEntry>, LedgerError> {
        let db = LedgerDb::new(self.conn);
        db.get_ledger_entries_for_tx(tx_id)
    }

    pub fn get_adr_entries(&self, days: Option<u64>) -> Result<Vec<LedgerEntry>, LedgerError> {
        let db = LedgerDb::new(self.conn);
        db.get_adr_entries(days)
    }

    pub fn search_ledger(
        &self,
        query: &str,
        category: Option<&str>,
        days: Option<u64>,
        breaking_only: bool,
        limit: Option<usize>,
    ) -> Result<Vec<LedgerEntry>, LedgerError> {
        let db = LedgerDb::new(self.conn);
        db.search_ledger(query, category, days, breaking_only, limit)
    }

    pub fn get_ledger_entries(&self, entity: &str) -> Result<Vec<LedgerEntry>, LedgerError> {
        let normalized = self.entity_normalized(entity)?;
        let db = LedgerDb::new(self.conn);
        db.get_ledger_entries_by_entity(&normalized)
    }

    pub fn get_all_pending(&self) -> Result<Vec<Transaction>, LedgerError> {
        let db = LedgerDb::new(self.conn);
        db.get_all_pending()
    }

    pub fn get_all_unaudited(&self) -> Result<Vec<Transaction>, LedgerError> {
        let db = LedgerDb::new(self.conn);
        db.get_all_unaudited()
    }

    pub fn record_token_provenance(
        &mut self,
        tx_id: &str,
        symbol_diff: Vec<(crate::index::symbols::Symbol, ProvenanceAction)>,
    ) -> Result<(), LedgerError> {
        let tx_id = self.resolve_tx_id(tx_id)?;
        let tx = {
            let db = LedgerDb::new(self.conn);
            db.get_transaction(&tx_id)?
                .ok_or_else(|| LedgerError::NotFound(tx_id.clone()))?
        };

        let db = LedgerDb::new(self.conn);
        for (symbol, action) in symbol_diff {
            let prov = TokenProvenance {
                id: None,
                tx_id: tx_id.clone(),
                entity: tx.entity.clone(),
                entity_normalized: tx.entity_normalized.clone(),
                symbol_name: symbol.name,
                symbol_type: format!("{:?}", symbol.kind),
                action,
            };
            db.insert_token_provenance(&prov)?;
        }
        Ok(())
    }

    pub fn entity_normalized(&self, entity: &str) -> Result<String, LedgerError> {
        let mut normalized = crate::util::path::normalize_relative_path(&self.repo_root, entity)
            .map_err(LedgerError::Config)?;

        if self.is_case_insensitive {
            normalized = normalized.to_lowercase();
        }

        Ok(normalized)
    }
}
