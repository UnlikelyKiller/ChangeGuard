use crate::ledger::enforcement::*;
use crate::ledger::error::LedgerError;
use crate::ledger::provenance::{ProvenanceAction, TokenProvenance};
use crate::ledger::types::*;
use rusqlite::{Connection, OptionalExtension, params};

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
                planned_action, session_id, source, started_at, resolved_at, issue_ref,
                detected_at, drift_count, first_seen_at, last_seen_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                tx.tx_id,
                tx.operation_id,
                tx.status,
                serde_json::to_string(&tx.category)
                    .map_err(|e| LedgerError::Config(e.to_string()))?
                    .trim_matches('"'),
                tx.entity,
                tx.entity_normalized,
                tx.planned_action,
                tx.session_id,
                tx.source,
                tx.started_at,
                tx.resolved_at,
                tx.issue_ref,
                tx.detected_at,
                tx.drift_count,
                tx.first_seen_at,
                tx.last_seen_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_transaction(&self, tx_id: &str) -> Result<Option<Transaction>, LedgerError> {
        self.conn
            .query_row(
                "SELECT tx_id, operation_id, status, category, entity, entity_normalized,
                    planned_action, session_id, source, started_at, resolved_at, issue_ref,
                    detected_at, drift_count, first_seen_at, last_seen_at
             FROM transactions WHERE tx_id = ?1",
                [tx_id],
                |row| self.map_transaction(row),
            )
            .optional()
            .map_err(LedgerError::from)
    }

    pub fn get_pending_by_entity(
        &self,
        entity_normalized: &str,
    ) -> Result<Option<Transaction>, LedgerError> {
        self.conn
            .query_row(
                "SELECT tx_id, operation_id, status, category, entity, entity_normalized,
                    planned_action, session_id, source, started_at, resolved_at, issue_ref,
                    detected_at, drift_count, first_seen_at, last_seen_at
             FROM transactions WHERE entity_normalized = ?1 AND status = 'PENDING'",
                [entity_normalized],
                |row| self.map_transaction(row),
            )
            .optional()
            .map_err(LedgerError::from)
    }

    pub fn get_unaudited_by_entity(
        &self,
        entity_normalized: &str,
    ) -> Result<Option<Transaction>, LedgerError> {
        self.conn
            .query_row(
                "SELECT tx_id, operation_id, status, category, entity, entity_normalized,
                    planned_action, session_id, source, started_at, resolved_at, issue_ref,
                    detected_at, drift_count, first_seen_at, last_seen_at
             FROM transactions WHERE entity_normalized = ?1 AND status = 'UNAUDITED'",
                [entity_normalized],
                |row| self.map_transaction(row),
            )
            .optional()
            .map_err(LedgerError::from)
    }

    pub fn upsert_unaudited_transaction(&self, tx: &Transaction) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT INTO transactions (
                tx_id, status, category, entity, entity_normalized, session_id, source, 
                started_at, detected_at, drift_count, first_seen_at, last_seen_at
            ) VALUES (?1, 'UNAUDITED', ?2, ?3, ?4, ?5, 'WATCHER', ?6, ?7, 1, ?8, ?9)
            ON CONFLICT(entity_normalized) WHERE status = 'UNAUDITED' DO UPDATE SET
                drift_count = drift_count + 1,
                last_seen_at = EXCLUDED.last_seen_at",
            params![
                tx.tx_id,
                serde_json::to_string(&tx.category)
                    .map_err(|e| LedgerError::Config(e.to_string()))?
                    .trim_matches('"'),
                tx.entity,
                tx.entity_normalized,
                tx.session_id,
                tx.started_at,
                tx.detected_at,
                tx.first_seen_at,
                tx.last_seen_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_transaction_status(
        &self,
        tx_id: &str,
        status: &str,
        resolved_at: Option<&str>,
    ) -> Result<usize, LedgerError> {
        let count = self.conn.execute(
            "UPDATE transactions SET status = ?1, resolved_at = ?2 WHERE tx_id = ?3 AND status = 'PENDING'",
            params![status, resolved_at, tx_id],
        )?;
        Ok(count)
    }

    pub fn update_transaction_status_bulk(
        &self,
        tx_ids: &[String],
        status: &str,
        expected_status: &str,
        resolved_at: Option<&str>,
    ) -> Result<usize, LedgerError> {
        if tx_ids.is_empty() {
            return Ok(0);
        }
        let placeholders: Vec<String> = tx_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "UPDATE transactions SET status = ?1, resolved_at = ?2 WHERE status = ?3 AND tx_id IN ({})",
            placeholders.join(",")
        );
        let mut params: Vec<&dyn rusqlite::ToSql> = vec![&status, &resolved_at, &expected_status];
        for id in tx_ids {
            params.push(id);
        }
        let count = self
            .conn
            .execute(&sql, rusqlite::params_from_iter(params))?;
        Ok(count)
    }

    fn map_transaction(&self, row: &rusqlite::Row) -> rusqlite::Result<Transaction> {
        let cat_str: String = row.get(3)?;
        let category: Category = serde_json::from_str(&format!("\"{}\"", cat_str))
            .map_err(|_| rusqlite::Error::InvalidQuery)?;

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
            detected_at: row.get(12)?,
            drift_count: row.get(13)?,
            first_seen_at: row.get(14)?,
            last_seen_at: row.get(15)?,
        })
    }

    pub fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT INTO ledger_entries (
                tx_id, category, entry_type, entity, entity_normalized,
                change_type, summary, reason, is_breaking, committed_at,
                verification_status, verification_basis, outcome_notes,
                origin, trace_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                entry.tx_id,
                serde_json::to_string(&entry.category)
                    .map_err(|e| LedgerError::Config(e.to_string()))?
                    .trim_matches('"'),
                serde_json::to_string(&entry.entry_type)
                    .map_err(|e| LedgerError::Config(e.to_string()))?
                    .trim_matches('"'),
                entry.entity,
                entry.entity_normalized,
                serde_json::to_string(&entry.change_type)
                    .map_err(|e| LedgerError::Config(e.to_string()))?
                    .trim_matches('"'),
                entry.summary,
                entry.reason,
                entry.is_breaking as i32,
                entry.committed_at,
                entry
                    .verification_status
                    .map(|s| {
                        serde_json::to_string(&s)
                            .map(|json| json.trim_matches('"').to_string())
                            .map_err(|e| LedgerError::Config(e.to_string()))
                    })
                    .transpose()?,
                entry
                    .verification_basis
                    .map(|b| {
                        serde_json::to_string(&b)
                            .map(|json| json.trim_matches('"').to_string())
                            .map_err(|e| LedgerError::Config(e.to_string()))
                    })
                    .transpose()?,
                entry.outcome_notes,
                entry.origin,
                entry.trace_id,
            ],
        )?;
        Ok(())
    }

    pub fn get_ledger_entries_for_tx(&self, tx_id: &str) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tx_id, category, entry_type, entity, entity_normalized,
                    change_type, summary, reason, is_breaking, committed_at,
                    verification_status, verification_basis, outcome_notes,
                    origin, trace_id
             FROM ledger_entries WHERE tx_id = ?1",
        )?;

        let rows = stmt.query_map([tx_id], |row| self.map_ledger_entry(row))?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn get_ledger_entries_by_entity(
        &self,
        entity_normalized: &str,
    ) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tx_id, category, entry_type, entity, entity_normalized,
                    change_type, summary, reason, is_breaking, committed_at,
                    verification_status, verification_basis, outcome_notes,
                    origin, trace_id
             FROM ledger_entries WHERE entity_normalized = ?1
             ORDER BY committed_at DESC",
        )?;

        let rows = stmt.query_map([entity_normalized], |row| self.map_ledger_entry(row))?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn get_federated_entries_by_entity(
        &self,
        entity_normalized: &str,
        sibling_name: &str,
        days: u64,
    ) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tx_id, category, entry_type, entity, entity_normalized,
                    change_type, summary, reason, is_breaking, committed_at,
                    verification_status, verification_basis, outcome_notes,
                    origin, trace_id
             FROM ledger_entries 
             WHERE entity_normalized = ?1 
               AND origin = 'SIBLING' 
               AND trace_id = ?2
               AND committed_at >= strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?3)
             ORDER BY committed_at DESC",
        )?;

        let delta = format!("-{} days", days);
        let rows = stmt.query_map([entity_normalized, sibling_name, &delta], |row| {
            self.map_ledger_entry(row)
        })?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn get_adr_entries(&self, days: Option<u64>) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut sql = "SELECT id, tx_id, category, entry_type, entity, entity_normalized,
                    change_type, summary, reason, is_breaking, committed_at,
                    verification_status, verification_basis, outcome_notes,
                    origin, trace_id
             FROM ledger_entries WHERE (entry_type = 'ARCHITECTURE' OR is_breaking = 1)"
            .to_string();

        if let Some(d) = days {
            sql.push_str(&format!(
                " AND committed_at >= strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-{} days')",
                d
            ));
        }
        sql.push_str(" ORDER BY committed_at DESC");

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| self.map_ledger_entry(row))?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn search_ledger(
        &self,
        query: &str,
        category: Option<&str>,
        days: Option<u64>,
        breaking_only: bool,
        limit: Option<usize>,
    ) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut sql =
            "SELECT l.id, l.tx_id, l.category, l.entry_type, l.entity, l.entity_normalized,
                    l.change_type, l.summary, l.reason, l.is_breaking, l.committed_at,
                    l.verification_status, l.verification_basis, l.outcome_notes,
                    l.origin, l.trace_id
             FROM ledger_entries l
             JOIN ledger_fts f ON f.rowid = l.id
             WHERE ledger_fts MATCH ?1"
                .to_string();

        let mut param_idx = 2u32;
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(query.to_string())];

        if let Some(cat) = category {
            sql.push_str(&format!(" AND l.category = ?{param_idx}"));
            params.push(Box::new(cat.to_string()));
            param_idx += 1;
        }

        if let Some(d) = days {
            sql.push_str(&format!(
                " AND l.committed_at >= strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?{param_idx})"
            ));
            params.push(Box::new(format!("-{d} days")));
            param_idx += 1;
        }

        if breaking_only {
            sql.push_str(" AND l.is_breaking = 1");
        }

        sql.push_str(" ORDER BY f.rank, l.committed_at DESC");

        if let Some(lim) = limit {
            sql.push_str(&format!(" LIMIT ?{param_idx}"));
            params.push(Box::new(lim as i64));
        }

        let mut stmt = self.conn.prepare(&sql).map_err(|e| {
            if let rusqlite::Error::SqliteFailure(_err, Some(msg)) = &e
                && msg.contains("syntax error")
            {
                return LedgerError::Validation(format!("Invalid search query: {}", msg));
            }
            LedgerError::from(e)
        })?;

        let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            self.map_ledger_entry(row)
        })?;

        let mut entries = Vec::new();
        for entry in rows {
            match entry {
                Ok(e) => entries.push(e),
                Err(e) => {
                    if let rusqlite::Error::SqliteFailure(_err, Some(msg)) = &e
                        && msg.contains("syntax error")
                    {
                        return Err(LedgerError::Validation(format!(
                            "Invalid search query syntax: {}",
                            msg
                        )));
                    }
                    return Err(LedgerError::from(e));
                }
            }
        }
        Ok(entries)
    }

    fn map_ledger_entry(&self, row: &rusqlite::Row) -> rusqlite::Result<LedgerEntry> {
        let cat_str: String = row.get(2)?;
        let category: Category = serde_json::from_str(&format!("\"{}\"", cat_str))
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let et_str: String = row.get(3)?;
        let entry_type: EntryType = serde_json::from_str(&format!("\"{}\"", et_str))
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let ct_str: String = row.get(6)?;
        let change_type: ChangeType = serde_json::from_str(&format!("\"{}\"", ct_str))
            .map_err(|_| rusqlite::Error::InvalidQuery)?;

        let vs_str: Option<String> = row.get(11)?;
        let verification_status = match vs_str {
            Some(s) => Some(
                serde_json::from_str(&format!("\"{}\"", s))
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
            ),
            None => None,
        };
        let vb_str: Option<String> = row.get(12)?;
        let verification_basis = match vb_str {
            Some(s) => Some(
                serde_json::from_str(&format!("\"{}\"", s))
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
            ),
            None => None,
        };

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
            verification_status,
            verification_basis,
            outcome_notes: row.get(13)?,
            origin: row.get(14)?,
            trace_id: row.get(15)?,
        })
    }

    pub fn get_all_pending(&self) -> Result<Vec<Transaction>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT tx_id, operation_id, status, category, entity, entity_normalized,
                    planned_action, session_id, source, started_at, resolved_at, issue_ref,
                    detected_at, drift_count, first_seen_at, last_seen_at
             FROM transactions WHERE status = 'PENDING' ORDER BY started_at DESC",
        )?;

        let rows = stmt.query_map([], |row| self.map_transaction(row))?;
        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn get_all_unaudited(&self) -> Result<Vec<Transaction>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT tx_id, operation_id, status, category, entity, entity_normalized,
                    planned_action, session_id, source, started_at, resolved_at, issue_ref,
                    detected_at, drift_count, first_seen_at, last_seen_at
             FROM transactions WHERE status = 'UNAUDITED' ORDER BY last_seen_at DESC",
        )?;

        let rows = stmt.query_map([], |row| self.map_transaction(row))?;
        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn get_unaudited_by_pattern(&self, pattern: &str) -> Result<Vec<Transaction>, LedgerError> {
        let sql_pattern = pattern.replace('*', "%");
        let mut stmt = self.conn.prepare(
            "SELECT tx_id, operation_id, status, category, entity, entity_normalized,
                    planned_action, session_id, source, started_at, resolved_at, issue_ref,
                    detected_at, drift_count, first_seen_at, last_seen_at
             FROM transactions WHERE status = 'UNAUDITED' AND entity_normalized LIKE ?1",
        )?;

        let rows = stmt.query_map([sql_pattern], |row| self.map_transaction(row))?;
        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn resolve_tx_id_fuzzy(&self, prefix: &str) -> Result<Vec<String>, LedgerError> {
        let sql_prefix = format!("{}%", prefix.replace('_', "\\_").replace('%', "\\%"));
        let mut stmt = self
            .conn
            .prepare("SELECT tx_id FROM transactions WHERE tx_id LIKE ?1 ESCAPE '\\'")?;

        let rows = stmt.query_map([sql_prefix], |row| row.get(0))?;
        let mut matches = Vec::new();
        for m in rows {
            matches.push(m?);
        }
        Ok(matches)
    }

    pub fn insert_tech_stack_rule(&self, rule: &TechStackRule) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT INTO tech_stack (
                category, name, version_constraint, rules, locked, status, entity_type, registered_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(category) DO UPDATE SET
                name = EXCLUDED.name,
                version_constraint = EXCLUDED.version_constraint,
                rules = EXCLUDED.rules,
                locked = EXCLUDED.locked,
                status = EXCLUDED.status,
                entity_type = EXCLUDED.entity_type,
                registered_at = EXCLUDED.registered_at",
            params![
                rule.category,
                rule.name,
                rule.version_constraint,
                serde_json::to_string(&rule.rules)
                    .map_err(|e| LedgerError::Config(e.to_string()))?,
                rule.locked as i32,
                rule.status,
                rule.entity_type,
                rule.registered_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_tech_stack_rules(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<TechStackRule>, LedgerError> {
        let mut sql = "SELECT category, name, version_constraint, rules, locked, status, entity_type, registered_at
             FROM tech_stack".to_string();

        let rules = if let Some(cat) = category {
            sql.push_str(" WHERE category = ?1");
            sql.push_str(" ORDER BY category ASC");
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map([cat], |row| self.map_tech_stack_rule(row))?;
            rows.collect::<Result<Vec<_>, _>>()?
        } else {
            sql.push_str(" ORDER BY category ASC");
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map([], |row| self.map_tech_stack_rule(row))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(rules)
    }

    fn map_tech_stack_rule(&self, row: &rusqlite::Row) -> rusqlite::Result<TechStackRule> {
        let rules_json: String = row.get(3)?;
        let rules: Vec<String> =
            serde_json::from_str(&rules_json).map_err(|_| rusqlite::Error::InvalidQuery)?;
        Ok(TechStackRule {
            category: row.get(0)?,
            name: row.get(1)?,
            version_constraint: row.get(2)?,
            rules,
            locked: row.get::<_, i32>(4)? != 0,
            status: row.get(5)?,
            entity_type: row.get(6)?,
            registered_at: row.get(7)?,
        })
    }

    pub fn get_tech_stack_rule(
        &self,
        category: &str,
    ) -> Result<Option<TechStackRule>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT category, name, version_constraint, rules, locked, status, entity_type, registered_at
             FROM tech_stack WHERE category = ?1",
        )?;

        stmt.query_row([category], |row| self.map_tech_stack_rule(row))
            .optional()
            .map_err(LedgerError::from)
    }

    pub fn insert_commit_validator(&self, validator: &CommitValidator) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT INTO commit_validators (
                category, name, description, executable, args, timeout_ms, glob, validation_level, enabled
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                validator.category,
                validator.name,
                validator.description,
                validator.executable,
                serde_json::to_string(&validator.args)
                    .map_err(|e| LedgerError::Config(e.to_string()))?,
                validator.timeout_ms,
                validator.glob,
                serde_json::to_string(&validator.validation_level)
                    .map_err(|e| LedgerError::Config(e.to_string()))?
                    .trim_matches('"'),
                validator.enabled as i32,
            ],
        )?;
        Ok(())
    }

    pub fn get_commit_validators(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<CommitValidator>, LedgerError> {
        let mut sql = "SELECT id, category, name, description, executable, args, timeout_ms, glob, validation_level, enabled
             FROM commit_validators".to_string();

        let validators = if let Some(cat) = category {
            sql.push_str(" WHERE (category = ?1 OR category = 'ALL')");
            sql.push_str(" ORDER BY category ASC");
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map([cat], |row| self.map_commit_validator(row))?;
            rows.collect::<Result<Vec<_>, _>>()?
        } else {
            sql.push_str(" ORDER BY category ASC");
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map([], |row| self.map_commit_validator(row))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(validators)
    }

    fn map_commit_validator(&self, row: &rusqlite::Row) -> rusqlite::Result<CommitValidator> {
        let args_json: String = row.get(5)?;
        let args: Vec<String> =
            serde_json::from_str(&args_json).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let vl_str: String = row.get(8)?;
        let validation_level: ValidationLevel = serde_json::from_str(&format!("\"{}\"", vl_str))
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        Ok(CommitValidator {
            id: Some(row.get(0)?),
            category: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            executable: row.get(4)?,
            args,
            timeout_ms: row.get(6)?,
            glob: row.get(7)?,
            validation_level,
            enabled: row.get::<_, i32>(9)? != 0,
        })
    }

    pub fn insert_category_mapping(
        &self,
        mapping: &CategoryStackMapping,
    ) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT INTO category_stack_mappings (
                ledger_category, stack_category, glob, description
            ) VALUES (?1, ?2, ?3, ?4)",
            params![
                mapping.ledger_category,
                mapping.stack_category,
                mapping.glob,
                mapping.description,
            ],
        )?;
        Ok(())
    }

    pub fn get_category_mappings(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<CategoryStackMapping>, LedgerError> {
        let mut sql = "SELECT id, ledger_category, stack_category, glob, description
             FROM category_stack_mappings"
            .to_string();

        let mappings = if let Some(cat) = category {
            sql.push_str(" WHERE ledger_category = ?1 OR stack_category = ?1");
            sql.push_str(" ORDER BY ledger_category ASC");
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map([cat], |row| {
                Ok(CategoryStackMapping {
                    id: Some(row.get(0)?),
                    ledger_category: row.get(1)?,
                    stack_category: row.get(2)?,
                    glob: row.get(3)?,
                    description: row.get(4)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        } else {
            sql.push_str(" ORDER BY ledger_category ASC");
            let mut stmt = self.conn.prepare(&sql)?;
            let rows = stmt.query_map([], |row| {
                Ok(CategoryStackMapping {
                    id: Some(row.get(0)?),
                    ledger_category: row.get(1)?,
                    stack_category: row.get(2)?,
                    glob: row.get(3)?,
                    description: row.get(4)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(mappings)
    }

    pub fn insert_watcher_pattern(&self, pattern: &WatcherPattern) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT INTO watcher_patterns (
                glob, category, source, description
            ) VALUES (?1, ?2, ?3, ?4)",
            params![
                pattern.glob,
                pattern.category,
                pattern.source,
                pattern.description,
            ],
        )?;
        Ok(())
    }

    pub fn get_watcher_patterns(&self) -> Result<Vec<WatcherPattern>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, glob, category, source, description
             FROM watcher_patterns ORDER BY glob ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(WatcherPattern {
                id: Some(row.get(0)?),
                glob: row.get(1)?,
                category: row.get(2)?,
                source: row.get(3)?,
                description: row.get(4)?,
            })
        })?;

        let mut patterns = Vec::new();
        for p in rows {
            patterns.push(p?);
        }
        Ok(patterns)
    }

    pub fn insert_token_provenance(&self, prov: &TokenProvenance) -> Result<(), LedgerError> {
        self.conn.execute(
            "INSERT INTO token_provenance (
                tx_id, entity, entity_normalized, symbol_name, symbol_type, action
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                prov.tx_id,
                prov.entity,
                prov.entity_normalized,
                prov.symbol_name,
                prov.symbol_type,
                prov.action.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn get_token_provenance_for_tx(
        &self,
        tx_id: &str,
    ) -> Result<Vec<TokenProvenance>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tx_id, entity, entity_normalized, symbol_name, symbol_type, action
             FROM token_provenance WHERE tx_id = ?1",
        )?;

        let rows = stmt.query_map([tx_id], |row| self.map_token_provenance(row))?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn get_token_provenance_by_entity(
        &self,
        entity_normalized: &str,
    ) -> Result<Vec<TokenProvenance>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tx_id, entity, entity_normalized, symbol_name, symbol_type, action
             FROM token_provenance WHERE entity_normalized = ?1
             ORDER BY id DESC",
        )?;

        let rows = stmt.query_map([entity_normalized], |row| self.map_token_provenance(row))?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    fn map_token_provenance(&self, row: &rusqlite::Row) -> rusqlite::Result<TokenProvenance> {
        use std::str::FromStr;
        let action_str: String = row.get(6)?;
        let action =
            ProvenanceAction::from_str(&action_str).map_err(|_| rusqlite::Error::InvalidQuery)?;

        Ok(TokenProvenance {
            id: Some(row.get(0)?),
            tx_id: row.get(1)?,
            entity: row.get(2)?,
            entity_normalized: row.get(3)?,
            symbol_name: row.get(4)?,
            symbol_type: row.get(5)?,
            action,
        })
    }

    pub fn get_all_committed_ledger_entries(&self) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tx_id, category, entry_type, entity, entity_normalized,
                    change_type, summary, reason, is_breaking, committed_at,
                    verification_status, verification_basis, outcome_notes,
                    origin, trace_id
             FROM ledger_entries ORDER BY committed_at ASC",
        )?;

        let rows = stmt.query_map([], |row| self.map_ledger_entry(row))?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }
}
