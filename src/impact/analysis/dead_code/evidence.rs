use super::*;

impl<'a> ConfidenceScorer<'a> {
    // ------------------------------------------------------------------
    // Reachability
    // ------------------------------------------------------------------

    pub(super) fn reachability_score(&self, symbol: &Symbol, file_path: &Path) -> Result<f64> {
        let reachable = match self.cozo {
            Some(cozo) => self.reachability_via_cozo(symbol, cozo),
            None => self.reachability_via_sqlite(symbol, file_path),
        };

        match reachable {
            Ok(true) => Ok(0.0),
            Ok(false) => Ok(1.0),
            Err(e) => {
                warn!("Reachability query failed for {}: {}", symbol.name, e);
                Ok(0.0)
            }
        }
    }

    fn reachability_via_cozo(&self, symbol: &Symbol, cozo: &CozoStorage) -> Result<bool> {
        use crate::platform::urn::build_urn;
        use crate::state::graph_kinds::NodeKind;

        let entrypoints = self.get_entrypoint_qualified_names()?;
        if entrypoints.is_empty() {
            return Ok(false);
        }

        let qualified = match &symbol.qualified_name {
            Some(q) => q.clone(),
            None => symbol.name.clone(),
        };

        let target_urn = build_urn(NodeKind::Symbol, &qualified);
        let entry_values: Vec<serde_json::Value> = entrypoints
            .iter()
            .map(|e| serde_json::json!([build_urn(NodeKind::Symbol, e)]))
            .collect();
        let entry_list_json = serde_json::Value::Array(entry_values);

        let mut params = std::collections::BTreeMap::new();
        params.insert(
            "entry_list".to_string(),
            cozo::DataValue::from(entry_list_json),
        );
        params.insert(
            "target_node".to_string(),
            cozo::DataValue::Str(target_urn.into()),
        );

        let script = "
            entry[id] <- $entry_list
            reachable[node] := entry[e], *edge{source: e, target: node}
            reachable[node] := reachable[mid], *edge{source: mid, target: node}
            ?[count(node)] := reachable[node], node = $target_node
        ";

        let res = cozo.run_script_with_params(script, params, cozo::ScriptMutability::Immutable)?;
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

    fn reachability_via_sqlite(&self, symbol: &Symbol, _file_path: &Path) -> Result<bool> {
        let conn = self.storage.get_connection();

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

        let symbol_id = self.find_symbol_id(symbol, _file_path)?;
        match symbol_id {
            Some(id) => Ok(visited.contains(&id)),
            None => Ok(false),
        }
    }

    pub(super) fn get_entrypoint_qualified_names(&self) -> Result<Vec<String>> {
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

    pub(super) fn find_symbol_id(&self, symbol: &Symbol, file_path: &Path) -> Result<Option<i64>> {
        let conn = self.storage.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT ps.id FROM project_symbols ps\n                 JOIN project_files pf ON ps.file_id = pf.id\n                 WHERE pf.file_path = ?1 AND ps.symbol_name = ?2 AND ps.symbol_kind = ?3",
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

    pub(super) fn git_activity_score(&self, file_path: &Path) -> Result<f64> {
        let days = match self.days_since_last_commit(file_path)? {
            Some(d) => d,
            None => {
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

    pub(super) fn days_since_last_commit(&self, file_path: &Path) -> Result<Option<u32>> {
        if let Some(cached) = self.git_activity_cache.borrow().get(file_path) {
            return Ok(*cached);
        }

        let calculate = || -> Result<Option<u32>> {
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
            let target_path = file_str.replace('\\', "/");

            let walk = match head.id().ancestors().all() {
                Ok(w) => w,
                Err(_) => return Ok(None),
            };

            let max_commits = 1000usize;
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
                        | gix::object::tree::diff::ChangeDetached::Modification {
                            location, ..
                        } => String::from_utf8_lossy(&location).into_owned(),
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

            Ok(Some(self.config.git_inactivity_days))
        };

        let result = calculate()?;
        self.git_activity_cache
            .borrow_mut()
            .insert(file_path.to_path_buf(), result);
        Ok(result)
    }

    // ------------------------------------------------------------------
    // Test Coverage
    // ------------------------------------------------------------------

    pub(super) fn test_coverage_score(&self, symbol: &Symbol, file_path: &Path) -> Result<f64> {
        let symbol_id = match self.find_symbol_id(symbol, file_path)? {
            Some(id) => id,
            None => return Ok(1.0),
        };

        let conn = self.storage.get_connection();

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

        let fallback_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM test_outcome_history toh\n                 JOIN embeddings e ON toh.diff_embedding_id = e.id\n                 WHERE e.entity_id = ?1",
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

    pub(super) fn get_symbols_for_file(&self, file_path: &Path) -> Result<Vec<Symbol>> {
        let conn = self.storage.get_connection();
        let path_str = file_path.to_string_lossy().to_string();
        let mut stmt = conn
            .prepare(
                "SELECT ps.symbol_name, ps.symbol_kind, ps.is_public, ps.cognitive_complexity,\n                        ps.cyclomatic_complexity, ps.line_start, ps.line_end, ps.qualified_name,\n                        ps.byte_start, ps.byte_end, ps.entrypoint_kind, ps.metadata\n                 FROM project_symbols ps\n                 JOIN project_files pf ON ps.file_id = pf.id\n                 WHERE pf.file_path = ?1",
            )
            .into_diagnostic()?;
        let rows = stmt
            .query_map([&path_str], |row| {
                let kind_str: String = row.get(1)?;
                let kind = crate::index::symbols::SymbolKind::parse(&kind_str)
                    .unwrap_or(crate::index::symbols::SymbolKind::Function);
                let is_public: i32 = row.get(2)?;
                let entrypoint: Option<String> = row.get(10)?;
                let metadata_str: Option<String> = row.get(11)?;
                let metadata = metadata_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default();

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
                    metadata,
                })
            })
            .into_diagnostic()?;
        let mut symbols = Vec::new();
        for row in rows {
            symbols.push(row.into_diagnostic()?);
        }
        Ok(symbols)
    }

    pub(super) fn get_all_symbols(&self) -> Result<Vec<(Symbol, PathBuf)>> {
        let conn = self.storage.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT ps.symbol_name, ps.symbol_kind, ps.is_public, ps.cognitive_complexity,\n                        ps.cyclomatic_complexity, ps.line_start, ps.line_end, ps.qualified_name,\n                        ps.byte_start, ps.byte_end, ps.entrypoint_kind, pf.file_path, ps.metadata\n                 FROM project_symbols ps\n                 JOIN project_files pf ON ps.file_id = pf.id\n                 WHERE pf.parse_status != 'DELETED'",
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
                let metadata_str: Option<String> = row.get(12)?;
                let metadata = metadata_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default();

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
                        metadata,
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
