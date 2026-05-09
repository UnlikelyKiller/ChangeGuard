use crate::state::storage::StorageManager;
use crate::state::storage_cozo::CozoStorage;
use miette::{IntoDiagnostic, Result};
use serde_json::json;
use tracing::info;

#[derive(Debug, Clone)]
pub struct GraphStats {
    pub nodes_added: usize,
    pub edges_added: usize,
    pub files_indexed: usize,
    pub symbols_indexed: usize,
}

#[derive(Debug, Clone)]
pub struct Community {
    pub id: usize,
    pub node_ids: Vec<String>,
    pub size: usize,
}

/// Build a native graph in CozoDB by reading from SQLite tables.
pub fn build_native_graph(
    storage: &StorageManager,
    cozo: &CozoStorage,
    provenance_id: &str,
) -> Result<GraphStats> {
    let conn = storage.get_connection();

    // --- 1. Read project_files → file nodes ---
    let mut file_stmt = conn
        .prepare("SELECT file_path, language FROM project_files WHERE parse_status != 'DELETED'")
        .into_diagnostic()?;

    let file_rows: Vec<(String, Option<String>)> = file_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(file_stmt);

    let mut node_batch = Vec::new();
    let mut files_indexed = 0usize;
    for (file_path, language) in &file_rows {
        let metadata = json!({ "language": language });
        node_batch.push(json!([
            file_path.as_str(),
            file_path.as_str(),
            "file",
            0.0,
            metadata
        ]));
        files_indexed += 1;
    }

    // --- 2. Read project_symbols → symbol nodes ---
    let mut sym_stmt = conn
        .prepare("SELECT qualified_name, symbol_name, symbol_kind FROM project_symbols")
        .into_diagnostic()?;

    let sym_rows: Vec<(String, String, String)> = sym_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(sym_stmt);

    let mut symbols_indexed = 0usize;
    for (qualified_name, symbol_name, symbol_kind) in &sym_rows {
        let metadata = json!({ "kind": symbol_kind });
        node_batch.push(json!([
            qualified_name.as_str(),
            symbol_name.as_str(),
            "symbol",
            0.0,
            metadata
        ]));
        symbols_indexed += 1;
    }

    if !node_batch.is_empty() {
        let script = format!(
            "?[id, label, category, risk_score, metadata] <- {} :put node",
            serde_json::to_string(&node_batch).into_diagnostic()?
        );
        cozo.run_script(&script)?;
    }

    // --- 3. Read structural_edges → edge relations ---
    let mut edge_stmt = conn
        .prepare(
            "SELECT \
             ps_caller.qualified_name, \
             COALESCE(ps_callee.qualified_name, se.unresolved_callee), \
             se.call_kind \
             FROM structural_edges se \
             JOIN project_symbols ps_caller ON se.caller_symbol_id = ps_caller.id \
             LEFT JOIN project_symbols ps_callee ON se.callee_symbol_id = ps_callee.id",
        )
        .into_diagnostic()?;

    let edge_rows: Vec<(String, Option<String>, String)> = edge_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .into_diagnostic()?
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    drop(edge_stmt);

    let mut edge_batch = Vec::new();
    let mut edges_added = 0usize;
    for (source, target_opt, _call_kind) in &edge_rows {
        let target = match target_opt {
            Some(t) => t.as_str(),
            None => continue,
        };
        edge_batch.push(json!([
            source.as_str(),
            target,
            "calls",
            1.0,
            provenance_id
        ]));
        edges_added += 1;
    }

    if !edge_batch.is_empty() {
        let script = format!(
            "?[source, target, relation, confidence, provenance_id] <- {} :put edge",
            serde_json::to_string(&edge_batch).into_diagnostic()?
        );
        cozo.run_script(&script)?;
    }

    info!(
        "Native graph built: {} files, {} symbols, {} edges",
        files_indexed, symbols_indexed, edges_added
    );

    Ok(GraphStats {
        nodes_added: files_indexed + symbols_indexed,
        edges_added,
        files_indexed,
        symbols_indexed,
    })
}

/// Run Louvain community detection on the CozoDB graph and group results.
/// Note: CozoDB 0.7 does not ship with Leiden; Louvain is the closest available algorithm.
pub fn run_community_louvain(cozo: &CozoStorage) -> Result<Vec<Community>> {
    let raw = cozo.run_community_louvain()?;
    let mut groups: std::collections::HashMap<i64, Vec<String>> = std::collections::HashMap::new();
    for (node, comm) in raw {
        groups.entry(comm).or_default().push(node);
    }

    let mut communities: Vec<Community> = groups
        .into_iter()
        .enumerate()
        .map(|(id, (_, node_ids))| {
            let size = node_ids.len();
            Community { id, node_ids, size }
        })
        .collect();

    communities.sort_by_key(|a| a.id);
    Ok(communities)
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_build_native_graph_populates_nodes_and_edges() {
        let (storage, cozo) = in_memory_storage_with_cozo();
        let conn = storage.get_connection();

        // Insert project_files
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) \
             VALUES ('src/main.rs', 'Rust', 'hash1', 100, 'OK', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let file_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, parse_status, last_indexed_at) \
             VALUES ('src/lib.rs', 'Rust', 'hash2', 200, 'OK', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let file_id2 = conn.last_insert_rowid();

        // Insert project_symbols
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at) \
             VALUES (?1, 'crate::main', 'main', 'Function', '2026-01-01T00:00:00Z')",
            [file_id],
        )
        .unwrap();
        let sym_main = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at) \
             VALUES (?1, 'crate::helper', 'helper', 'Function', '2026-01-01T00:00:00Z')",
            [file_id2],
        )
        .unwrap();
        let sym_helper = conn.last_insert_rowid();

        // Insert structural_edges
        conn.execute(
            "INSERT INTO structural_edges (caller_symbol_id, caller_file_id, callee_symbol_id, callee_file_id, call_kind, resolution_status) \
             VALUES (?1, ?2, ?3, ?4, 'DIRECT', 'RESOLVED')",
            [sym_main, file_id, sym_helper, file_id2],
        )
        .unwrap();

        let stats = build_native_graph(&storage, &cozo, "test_provenance").unwrap();

        // Verify stats
        assert_eq!(stats.files_indexed, 2);
        assert_eq!(stats.symbols_indexed, 2);
        assert_eq!(stats.edges_added, 1);
        assert_eq!(stats.nodes_added, 4);

        // Verify CozoDB nodes
        let res = cozo.run_script("?[id] := *node{id: id}").unwrap();
        let ids: Vec<String> = res
            .rows
            .iter()
            .filter_map(|row| match row.first() {
                Some(cozo::DataValue::Str(s)) => Some(s.to_string()),
                _ => None,
            })
            .collect();
        assert!(ids.contains(&"src/main.rs".to_string()));
        assert!(ids.contains(&"crate::main".to_string()));
        assert!(ids.contains(&"crate::helper".to_string()));

        // Verify CozoDB edges
        let res = cozo
            .run_script("?[source, target] := *edge{source: source, target: target}")
            .unwrap();
        assert_eq!(res.rows.len(), 1);
        if let (Some(cozo::DataValue::Str(src)), Some(cozo::DataValue::Str(tgt))) =
            (res.rows[0].first(), res.rows[0].get(1))
        {
            assert_eq!(src.as_str(), "crate::main");
            assert_eq!(tgt.as_str(), "crate::helper");
        } else {
            panic!("Expected string edge endpoints");
        }
    }

    #[test]
    fn test_run_community_louvain_finds_communities() {
        let cozo = CozoStorage::new(&PathBuf::from("")).unwrap();

        // Two disconnected clusters
        cozo.run_script(
            "?[id, label, category, risk_score, metadata] <- [
                ['a1', 'A1', 'code', 0.0, {}],
                ['a2', 'A2', 'code', 0.0, {}],
                ['b1', 'B1', 'code', 0.0, {}],
                ['b2', 'B2', 'code', 0.0, {}]
            ] :put node",
        )
        .unwrap();

        cozo.run_script(
            "?[source, target, relation, confidence, provenance_id] <- [
                ['a1', 'a2', 'calls', 1.0, 'tx1'],
                ['b1', 'b2', 'calls', 1.0, 'tx1']
            ] :put edge",
        )
        .unwrap();

        let communities = run_community_louvain(&cozo).unwrap();
        assert!(!communities.is_empty());

        let distinct_ids: std::collections::HashSet<usize> =
            communities.iter().map(|c| c.id).collect();
        assert!(
            distinct_ids.len() >= 2,
            "Expected at least 2 communities, got {:?}",
            distinct_ids.len()
        );

        let total_nodes: usize = communities.iter().map(|c| c.size).sum();
        assert_eq!(total_nodes, 4);
    }

    #[test]
    fn test_graph_stats_counts_correct() {
        let stats = GraphStats {
            nodes_added: 10,
            edges_added: 5,
            files_indexed: 3,
            symbols_indexed: 7,
        };
        assert_eq!(stats.nodes_added, 10);
        assert_eq!(stats.edges_added, 5);
        assert_eq!(stats.files_indexed, 3);
        assert_eq!(stats.symbols_indexed, 7);
        assert_eq!(
            stats.nodes_added,
            stats.files_indexed + stats.symbols_indexed
        );
    }
}
