use crate::config::model::DeadCodeConfig;
use crate::impact::packet::{ConfidenceFactor, DeadCodeFinding};
use crate::index::symbols::Symbol;
use crate::state::storage::StorageManager;
use crate::state::storage_cozo::CozoStorage;
use miette::{IntoDiagnostic, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

pub struct ConfidenceScorer<'a> {
    cozo: Option<&'a CozoStorage>,
    storage: &'a StorageManager,
    config: &'a DeadCodeConfig,
    repo_path: &'a Path,
}

impl<'a> ConfidenceScorer<'a> {
    pub fn new(
        cozo: Option<&'a CozoStorage>,
        storage: &'a StorageManager,
        config: &'a DeadCodeConfig,
        repo_path: &'a Path,
    ) -> Self {
        Self {
            cozo,
            storage,
            config,
            repo_path,
        }
    }

    /// Score a single symbol. Returns `None` if the symbol is an entrypoint itself
    /// or if the confidence is below the threshold.
    pub fn score_symbol(
        &self,
        symbol: &Symbol,
        file_path: &Path,
    ) -> Result<Option<DeadCodeFinding>> {
        if Self::is_entrypoint(symbol) {
            return Ok(None);
        }

        let reachability = self.reachability_score(symbol, file_path)?;
        let git_activity = self.git_activity_score(file_path)?;
        let test_coverage = self.test_coverage_score(symbol, file_path)?;

        let confidence = self.blend(reachability, git_activity, test_coverage);

        if confidence < self.config.confidence_threshold {
            return Ok(None);
        }

        let mut factors = Vec::new();
        if reachability >= 1.0 {
            factors.push(ConfidenceFactor::UnreachableFromEntrypoints);
        }
        if git_activity > 0.0 {
            let days = self
                .days_since_last_commit(file_path)?
                .unwrap_or(self.config.git_inactivity_days);
            factors.push(ConfidenceFactor::GitInactive {
                days_since_last_commit: days,
            });
        }
        if test_coverage >= 1.0 {
            factors.push(ConfidenceFactor::NoTestCoverage);
        }

        let recommendation = format!(
            "Symbol '{}' in {} has {:.0}% confidence of being dead code. Consider reviewing for removal or adding tests.",
            symbol.name,
            file_path.display(),
            confidence * 100.0
        );

        Ok(Some(DeadCodeFinding {
            symbol_name: symbol.name.clone(),
            file_path: file_path.to_path_buf(),
            confidence,
            factors,
            recommendation,
        }))
    }

    /// Score all symbols in a file.
    pub fn score_file(&self, file_path: &Path) -> Result<Vec<DeadCodeFinding>> {
        let symbols = self.get_symbols_for_file(file_path)?;
        let mut findings = Vec::new();
        for symbol in symbols {
            if let Some(finding) = self.score_symbol(&symbol, file_path)? {
                findings.push(finding);
            }
        }
        findings.sort_unstable();
        Ok(findings)
    }

    /// Full-repo scan (used by the standalone `dead-code` command).
    pub fn scan_repo(&self, limit: usize) -> Result<Vec<DeadCodeFinding>> {
        let symbols = self.get_all_symbols()?;
        let mut findings = Vec::new();
        for (symbol, file_path) in symbols {
            if let Some(finding) = self.score_symbol(&symbol, &file_path)? {
                findings.push(finding);
                if findings.len() >= limit {
                    break;
                }
            }
        }
        findings.sort_unstable();
        Ok(findings)
    }

    fn is_entrypoint(symbol: &Symbol) -> bool {
        matches!(
            symbol.entrypoint_kind.as_deref(),
            Some("ENTRYPOINT") | Some("HANDLER") | Some("PUBLIC_API")
        )
    }

    fn blend(&self, reachability: f64, git_activity: f64, test_coverage: f64) -> f64 {
        let sum = self.config.reachability_weight
            + self.config.git_activity_weight
            + self.config.test_coverage_weight;
        if sum <= 0.0 {
            return 0.0;
        }
        (self.config.reachability_weight * reachability
            + self.config.git_activity_weight * git_activity
            + self.config.test_coverage_weight * test_coverage)
            / sum
    }

    // ------------------------------------------------------------------
    // Reachability
    // ------------------------------------------------------------------

    fn reachability_score(&self, symbol: &Symbol, file_path: &Path) -> Result<f64> {
        // Prefer CozoDB if available, otherwise fall back to SQLite structural_edges
        let reachable = match self.cozo {
            Some(cozo) => self.reachability_via_cozo(symbol, cozo),
            None => self.reachability_via_sqlite(symbol, file_path),
        };

        match reachable {
            Ok(true) => Ok(0.0),
            Ok(false) => Ok(1.0),
            Err(e) => {
                warn!("Reachability query failed for {}: {}", symbol.name, e);
                // Graceful degradation: assume reachable (no dead-code risk) on error
                Ok(0.0)
            }
        }
    }

    /// Query CozoDB to determine if the symbol is reachable from any entrypoint.
    fn reachability_via_cozo(&self, symbol: &Symbol, cozo: &CozoStorage) -> Result<bool> {
        let entrypoints = self.get_entrypoint_qualified_names()?;
        if entrypoints.is_empty() {
            return Ok(false);
        }

        let qualified = match &symbol.qualified_name {
            Some(q) => q.clone(),
            None => symbol.name.clone(),
        };

        // Build a fixed-point reachability query from entrypoints.
        // CozoDB recursion: reachable[node] appears in both head and body.
        let entry_list = serde_json::to_string(
            &entrypoints
                .iter()
                .map(|e| vec![e.as_str()])
                .collect::<Vec<_>>(),
        )
        .into_diagnostic()?;

        let script = format!(
            "entry[id] <- {}
             reachable[node] := entry[e], *edge{{source: e, target: node}}
             reachable[node] := reachable[mid], *edge{{source: mid, target: node}}
             ?[count(node)] := reachable[node], node = '{}'",
            entry_list,
            qualified.replace('\'', "\\'")
        );

        let res = cozo.run_script(&script)?;
        let count = res
            .rows
            .first()
            .and_then(|r| r.first())
            .and_then(|v| match v {
                cozo::DataValue::Num(cozo::Num::Int(n)) => Some(*n),
                _ => None,
            })
            .unwrap_or(0);

        Ok(count > 0)
    }

    /// Fallback: use SQLite structural_edges to compute reachability via BFS.
    fn reachability_via_sqlite(&self, symbol: &Symbol, file_path: &Path) -> Result<bool> {
        let conn = self.storage.get_connection();

        // Get entrypoint symbol IDs
        let mut stmt = conn
            .prepare(
                "SELECT id FROM project_symbols WHERE entrypoint_kind IN ('ENTRYPOINT', 'HANDLER', 'PUBLIC_API')"
            )
            .into_diagnostic()?;
        let entrypoint_ids: HashSet<i64> = stmt
            .query_map([], |row| row.get::<_, i64>(0))
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?
            .into_iter()
            .collect();
        drop(stmt);

        if entrypoint_ids.is_empty() {
            return Ok(false);
        }

        // Build forward adjacency list from structural_edges
        let mut stmt = conn
            .prepare("SELECT caller_symbol_id, callee_symbol_id FROM structural_edges WHERE callee_symbol_id IS NOT NULL")
            .into_diagnostic()?;
        let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .into_diagnostic()?;
        for row in rows {
            let (caller, callee) = row.into_diagnostic()?;
            adj.entry(caller).or_default().push(callee);
        }
        drop(stmt);

        // BFS from all entrypoints
        let mut visited: HashSet<i64> = HashSet::new();
        let mut queue: Vec<i64> = entrypoint_ids.iter().copied().collect();
        for &id in &queue {
            visited.insert(id);
        }

        let mut idx = 0;
        while idx < queue.len() {
            let current = queue[idx];
            idx += 1;
            if let Some(neighbors) = adj.get(&current) {
                for &neighbor in neighbors {
                    if visited.insert(neighbor) {
                        queue.push(neighbor);
                    }
                }
            }
        }

        // Find the symbol's ID
        let symbol_id = self.find_symbol_id(symbol, file_path)?;
        match symbol_id {
            Some(id) => Ok(visited.contains(&id)),
            None => Ok(false),
        }
    }

    fn get_entrypoint_qualified_names(&self) -> Result<Vec<String>> {
        let conn = self.storage.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT qualified_name FROM project_symbols WHERE entrypoint_kind IN ('ENTRYPOINT', 'HANDLER', 'PUBLIC_API')"
            )
            .into_diagnostic()?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .into_diagnostic()?;
        let mut names = Vec::new();
        for row in rows {
            names.push(row.into_diagnostic()?);
        }
        Ok(names)
    }

    fn find_symbol_id(&self, symbol: &Symbol, file_path: &Path) -> Result<Option<i64>> {
        let conn = self.storage.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT ps.id FROM project_symbols ps
                 JOIN project_files pf ON ps.file_id = pf.id
                 WHERE pf.file_path = ?1 AND ps.symbol_name = ?2 AND ps.symbol_kind = ?3",
            )
            .into_diagnostic()?;
        let mut rows = stmt
            .query([
                file_path.to_string_lossy().as_ref(),
                symbol.name.as_str(),
                symbol.kind.as_str(),
            ])
            .into_diagnostic()?;
        if let Some(row) = rows.next().into_diagnostic()? {
            let id: i64 = row.get(0).into_diagnostic()?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    // ------------------------------------------------------------------
    // Git Activity
    // ------------------------------------------------------------------

    fn git_activity_score(&self, file_path: &Path) -> Result<f64> {
        let days = match self.days_since_last_commit(file_path)? {
            Some(d) => d,
            None => {
                // No history found for file — treat as fully inactive
                return Ok(1.0);
            }
        };

        let threshold = self.config.git_inactivity_days as f64;
        if threshold <= 0.0 {
            return Ok(0.0);
        }

        let score = (days as f64 / threshold).min(1.0);
        Ok(score)
    }

    fn days_since_last_commit(&self, file_path: &Path) -> Result<Option<u32>> {
        let repo = match gix::discover(self.repo_path) {
            Ok(discovered) => gix::open(discovered.path()),
            Err(_) => return Ok(None),
        };
        let repo = match repo {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };

        let head = match repo.head_commit() {
            Ok(h) => h,
            Err(_) => return Ok(None),
        };

        let file_str = file_path.to_string_lossy();
        // Normalize path separators for comparison with git paths
        let target_path = file_str.replace('\\', "/");

        let walk = match head.id().ancestors().all() {
            Ok(w) => w,
            Err(_) => return Ok(None),
        };

        let max_commits = 1000usize; // Cap traversal for performance
        let mut commit_count = 0;

        for res in walk {
            if commit_count >= max_commits {
                break;
            }
            let info = match res {
                Ok(info) => info,
                Err(e) => {
                    debug!("Skip commit during git walk: {}", e);
                    continue;
                }
            };

            let commit = match info.id().object().map(|obj| obj.into_commit()) {
                Ok(commit) => commit,
                Err(e) => {
                    debug!("Skip commit object: {}", e);
                    continue;
                }
            };

            let current_tree = match commit.tree() {
                Ok(tree) => tree,
                Err(e) => {
                    debug!("Skip tree: {}", e);
                    continue;
                }
            };

            let parent_id = commit.parent_ids().next();
            let parent_tree = if let Some(p_id) = parent_id {
                match p_id.object().map(|obj| obj.into_commit().tree()) {
                    Ok(Ok(tree)) => tree,
                    _ => repo.empty_tree(),
                }
            } else {
                repo.empty_tree()
            };

            let changes =
                match repo.diff_tree_to_tree(Some(&parent_tree), Some(&current_tree), None) {
                    Ok(changes) => changes,
                    Err(e) => {
                        debug!("Skip diff: {}", e);
                        continue;
                    }
                };

            let mut touches_file = false;
            for change in changes {
                let location = match change {
                    gix::object::tree::diff::ChangeDetached::Addition { location, .. }
                    | gix::object::tree::diff::ChangeDetached::Deletion { location, .. }
                    | gix::object::tree::diff::ChangeDetached::Modification { location, .. } => {
                        String::from_utf8_lossy(&location).into_owned()
                    }
                    gix::object::tree::diff::ChangeDetached::Rewrite {
                        location,
                        source_location,
                        ..
                    } => {
                        let loc = String::from_utf8_lossy(&location).into_owned();
                        let src = String::from_utf8_lossy(&source_location).into_owned();
                        if loc.replace('\\', "/") == target_path
                            || src.replace('\\', "/") == target_path
                        {
                            touches_file = true;
                        }
                        continue;
                    }
                };
                if location.replace('\\', "/") == target_path {
                    touches_file = true;
                }
            }

            if touches_file {
                let commit_time = match commit.time() {
                    Ok(t) => t,
                    Err(e) => {
                        debug!("Skip commit with unreadable time: {}", e);
                        continue;
                    }
                };
                let commit_secs = commit_time.seconds;
                let now_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(commit_secs);
                let days = ((now_secs - commit_secs).max(0) as f64 / 86400.0).ceil() as u32;
                return Ok(Some(days));
            }

            commit_count += 1;
        }

        // File never touched in available history
        Ok(Some(self.config.git_inactivity_days))
    }

    // ------------------------------------------------------------------
    // Test Coverage
    // ------------------------------------------------------------------

    fn test_coverage_score(&self, symbol: &Symbol, file_path: &Path) -> Result<f64> {
        let symbol_id = match self.find_symbol_id(symbol, file_path)? {
            Some(id) => id,
            None => return Ok(1.0), // Unknown symbol = no coverage info = risk
        };

        let conn = self.storage.get_connection();

        // Primary: test_mapping table
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM test_mapping WHERE tested_symbol_id = ?1",
                [symbol_id],
                |row| row.get(0),
            )
            .into_diagnostic()?;

        if count > 0 {
            return Ok(0.0);
        }

        // Fallback: test_outcome_history joined via embeddings
        let fallback_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM test_outcome_history toh
                 JOIN embeddings e ON toh.diff_embedding_id = e.id
                 WHERE e.entity_id = ?1",
                [symbol_id.to_string()],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if fallback_count > 0 {
            return Ok(0.0);
        }

        Ok(1.0)
    }

    // ------------------------------------------------------------------
    // Symbol resolution helpers
    // ------------------------------------------------------------------

    fn get_symbols_for_file(&self, file_path: &Path) -> Result<Vec<Symbol>> {
        let conn = self.storage.get_connection();
        let path_str = file_path.to_string_lossy().to_string();
        let mut stmt = conn
            .prepare(
                "SELECT ps.symbol_name, ps.symbol_kind, ps.is_public, ps.cognitive_complexity,
                        ps.cyclomatic_complexity, ps.line_start, ps.line_end, ps.qualified_name,
                        ps.byte_start, ps.byte_end, ps.entrypoint_kind
                 FROM project_symbols ps
                 JOIN project_files pf ON ps.file_id = pf.id
                 WHERE pf.file_path = ?1",
            )
            .into_diagnostic()?;
        let rows = stmt
            .query_map([&path_str], |row| {
                let kind_str: String = row.get(1)?;
                let kind = crate::index::symbols::SymbolKind::parse(&kind_str)
                    .unwrap_or(crate::index::symbols::SymbolKind::Function);
                let is_public: i32 = row.get(2)?;
                let entrypoint: Option<String> = row.get(10)?;
                Ok(Symbol {
                    name: row.get(0)?,
                    kind,
                    is_public: is_public != 0,
                    cognitive_complexity: row.get(3)?,
                    cyclomatic_complexity: row.get(4)?,
                    line_start: row.get(5)?,
                    line_end: row.get(6)?,
                    qualified_name: row.get(7)?,
                    byte_start: row.get(8)?,
                    byte_end: row.get(9)?,
                    entrypoint_kind: entrypoint,
                })
            })
            .into_diagnostic()?;
        let mut symbols = Vec::new();
        for row in rows {
            symbols.push(row.into_diagnostic()?);
        }
        Ok(symbols)
    }

    fn get_all_symbols(&self) -> Result<Vec<(Symbol, PathBuf)>> {
        let conn = self.storage.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT ps.symbol_name, ps.symbol_kind, ps.is_public, ps.cognitive_complexity,
                        ps.cyclomatic_complexity, ps.line_start, ps.line_end, ps.qualified_name,
                        ps.byte_start, ps.byte_end, ps.entrypoint_kind, pf.file_path
                 FROM project_symbols ps
                 JOIN project_files pf ON ps.file_id = pf.id
                 WHERE pf.parse_status != 'DELETED'",
            )
            .into_diagnostic()?;
        let rows = stmt
            .query_map([], |row| {
                let kind_str: String = row.get(1)?;
                let kind = crate::index::symbols::SymbolKind::parse(&kind_str)
                    .unwrap_or(crate::index::symbols::SymbolKind::Function);
                let is_public: i32 = row.get(2)?;
                let entrypoint: Option<String> = row.get(10)?;
                let file_path: String = row.get(11)?;
                Ok((
                    Symbol {
                        name: row.get(0)?,
                        kind,
                        is_public: is_public != 0,
                        cognitive_complexity: row.get(3)?,
                        cyclomatic_complexity: row.get(4)?,
                        line_start: row.get(5)?,
                        line_end: row.get(6)?,
                        qualified_name: row.get(7)?,
                        byte_start: row.get(8)?,
                        byte_end: row.get(9)?,
                        entrypoint_kind: entrypoint,
                    },
                    PathBuf::from(file_path),
                ))
            })
            .into_diagnostic()?;
        let mut symbols = Vec::new();
        for row in rows {
            symbols.push(row.into_diagnostic()?);
        }
        Ok(symbols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::symbols::{Symbol, SymbolKind};
    use crate::state::storage::StorageManager;
    use crate::state::storage_cozo::CozoStorage;
    use std::path::PathBuf;

    fn in_memory_storage_with_cozo() -> (StorageManager, CozoStorage) {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let mut conn = conn;
        crate::state::migrations::get_migrations()
            .to_latest(&mut conn)
            .unwrap();
        let storage = StorageManager::init_from_conn(conn);
        let cozo = CozoStorage::new(&PathBuf::from("")).unwrap();
        (storage, cozo)
    }

    fn default_config() -> DeadCodeConfig {
        DeadCodeConfig {
            enabled: true,
            confidence_threshold: 0.75,
            git_inactivity_days: 90,
            reachability_weight: 1.0,
            git_activity_weight: 1.0,
            test_coverage_weight: 1.0,
        }
    }

    fn make_symbol(name: &str, qualified: Option<&str>, entrypoint: Option<&str>) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind: SymbolKind::Function,
            is_public: false,
            cognitive_complexity: None,
            cyclomatic_complexity: None,
            line_start: None,
            line_end: None,
            qualified_name: qualified.map(|s| s.to_string()),
            byte_start: None,
            byte_end: None,
            entrypoint_kind: entrypoint.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_entrypoint_skipped() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."));

        let symbol = make_symbol("main", Some("crate::main"), Some("ENTRYPOINT"));
        let result = scorer
            .score_symbol(&symbol, Path::new("src/main.rs"))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_reachability_unreachable_sqlite() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let conn = storage.get_connection();

        // Insert files and symbols
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) VALUES ('src/main.rs', 'Rust', 'h1', 100, 'OK', '2026-01-01')",
            [],
        ).unwrap();
        let main_file = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) VALUES ('src/lib.rs', 'Rust', 'h2', 100, 'OK', '2026-01-01')",
            [],
        ).unwrap();
        let lib_file = conn.last_insert_rowid();

        // Entrypoint
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::main', 'main', 'Function', 'ENTRYPOINT', '2026-01-01')",
            [main_file],
        ).unwrap();
        let main_sym = conn.last_insert_rowid();

        // Helper that IS called by main
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::helper', 'helper', 'Function', 'INTERNAL', '2026-01-01')",
            [lib_file],
        ).unwrap();
        let helper_sym = conn.last_insert_rowid();

        // Unused helper (not in edges)
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::unused', 'unused', 'Function', 'INTERNAL', '2026-01-01')",
            [lib_file],
        ).unwrap();

        // Edge: main -> helper
        conn.execute(
            "INSERT INTO structural_edges (caller_symbol_id, caller_file_id, callee_symbol_id, callee_file_id, call_kind, resolution_status) VALUES (?1, ?2, ?3, ?4, 'DIRECT', 'RESOLVED')",
            [main_sym, main_file, helper_sym, lib_file],
        ).unwrap();

        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."));

        // Helper is reachable from entrypoint
        let helper = make_symbol("helper", Some("crate::helper"), None);
        let score = scorer
            .reachability_score(&helper, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 0.0);

        // Unused is unreachable
        let unused = make_symbol("unused", Some("crate::unused"), None);
        let score = scorer
            .reachability_score(&unused, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_reachability_via_cozo() {
        let (storage, cozo) = in_memory_storage_with_cozo();

        // Setup CozoDB graph
        cozo.run_script(
            "?[id, label, category, risk_score, metadata] <- [
                ['crate::main', 'main', 'code', 0.0, {}],
                ['crate::helper', 'helper', 'code', 0.0, {}],
                ['crate::unused', 'unused', 'code', 0.0, {}]
            ] :put node",
        )
        .unwrap();

        cozo.run_script(
            "?[source, target, relation, confidence, provenance_id] <- [
                ['crate::main', 'crate::helper', 'calls', 1.0, 'tx1']
            ] :put edge",
        )
        .unwrap();

        // Insert entrypoint into SQLite so the scorer finds it
        let conn = storage.get_connection();
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) VALUES ('src/main.rs', 'Rust', 'h1', 100, 'OK', '2026-01-01')",
            [],
        ).unwrap();
        let main_file = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::main', 'main', 'Function', 'ENTRYPOINT', '2026-01-01')",
            [main_file],
        ).unwrap();

        let config = default_config();
        let scorer = ConfidenceScorer::new(Some(&cozo), &storage, &config, Path::new("."));

        let helper = make_symbol("helper", Some("crate::helper"), None);
        let score = scorer
            .reachability_score(&helper, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 0.0);

        let unused = make_symbol("unused", Some("crate::unused"), None);
        let score = scorer
            .reachability_score(&unused, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_test_coverage_no_mapping() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."));

        let symbol = make_symbol("foo", Some("crate::foo"), None);
        let score = scorer
            .test_coverage_score(&symbol, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_test_coverage_with_mapping() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let conn = storage.get_connection();

        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) VALUES ('src/lib.rs', 'Rust', 'h1', 100, 'OK', '2026-01-01')",
            [],
        ).unwrap();
        let file_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::foo', 'foo', 'Function', 'INTERNAL', '2026-01-01')",
            [file_id],
        ).unwrap();
        let sym_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, entrypoint_kind, last_indexed_at) VALUES (?1, 'crate::test_foo', 'test_foo', 'Function', 'TEST', '2026-01-01')",
            [file_id],
        ).unwrap();
        let test_sym_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO test_mapping (test_symbol_id, test_file_id, tested_symbol_id, tested_file_id, mapping_kind, last_indexed_at) VALUES (?1, ?2, ?3, ?4, 'IMPORT', '2026-01-01')",
            [test_sym_id, file_id, sym_id, file_id],
        ).unwrap();

        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."));

        let symbol = make_symbol("foo", Some("crate::foo"), None);
        let score = scorer
            .test_coverage_score(&symbol, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_blend_expected_value() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let config = default_config();
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."));

        // All weights are 1.0, so average of (1.0, 0.5, 0.0) = 0.5
        let confidence = scorer.blend(1.0, 0.5, 0.0);
        assert!((confidence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_with_zero_weights() {
        let (storage, _cozo) = in_memory_storage_with_cozo();
        let config = DeadCodeConfig {
            enabled: true,
            confidence_threshold: 0.75,
            git_inactivity_days: 90,
            reachability_weight: 0.0,
            git_activity_weight: 0.0,
            test_coverage_weight: 0.0,
        };
        let scorer = ConfidenceScorer::new(None, &storage, &config, Path::new("."));
        let confidence = scorer.blend(1.0, 1.0, 1.0);
        assert_eq!(confidence, 0.0);
    }
}
