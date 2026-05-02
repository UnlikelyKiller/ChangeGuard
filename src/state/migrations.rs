use rusqlite_migration::{M, Migrations};

pub fn get_migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            "CREATE TABLE snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                head_hash TEXT,
                branch_name TEXT,
                is_clean INTEGER NOT NULL,
                packet_json TEXT NOT NULL
            );",
        ),
        M::up(
            "CREATE TABLE batches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                event_count INTEGER NOT NULL,
                batch_json TEXT NOT NULL
            );",
        ),
        M::up(
            "CREATE TABLE changed_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                snapshot_id INTEGER REFERENCES snapshots(id),
                path TEXT NOT NULL,
                status TEXT NOT NULL,
                is_staged INTEGER NOT NULL
            );",
        ),
        M::up(
            "CREATE TABLE verification_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                plan_json TEXT,
                overall_pass INTEGER NOT NULL
            );",
        ),
        M::up(
            "CREATE TABLE verification_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id INTEGER REFERENCES verification_runs(id),
                command TEXT NOT NULL,
                exit_code INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                truncated INTEGER NOT NULL
            );",
        ),
        M::up(
            "CREATE TABLE symbols (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                snapshot_id INTEGER REFERENCES snapshots(id),
                file_path TEXT NOT NULL,
                symbol_name TEXT NOT NULL,
                symbol_kind TEXT NOT NULL,
                is_public INTEGER NOT NULL
            );",
        ),
        M::up("ALTER TABLE symbols ADD COLUMN cognitive_complexity INTEGER;"),
        M::up("ALTER TABLE symbols ADD COLUMN cyclomatic_complexity INTEGER;"),
        M::up(
            "CREATE TABLE federated_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sibling_name TEXT NOT NULL UNIQUE,
                sibling_path TEXT NOT NULL,
                last_scanned_at TEXT NOT NULL
            );",
        ),
        M::up(
            "CREATE TABLE federated_dependencies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                local_symbol TEXT NOT NULL,
                sibling_name TEXT NOT NULL,
                sibling_symbol TEXT NOT NULL,
                FOREIGN KEY (sibling_name) REFERENCES federated_links(sibling_name)
            );",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS transactions (
                tx_id              TEXT PRIMARY KEY,
                operation_id       TEXT,
                status             TEXT NOT NULL,
                category           TEXT NOT NULL,
                entity             TEXT NOT NULL,
                entity_normalized  TEXT NOT NULL,
                planned_action     TEXT,
                session_id         TEXT NOT NULL,
                source             TEXT NOT NULL DEFAULT 'CLI',
                started_at         TEXT NOT NULL,
                resolved_at        TEXT,
                detected_at        TEXT,
                drift_count        INTEGER DEFAULT 1,
                first_seen_at      TEXT,
                last_seen_at       TEXT,
                issue_ref          TEXT,
                change_type        TEXT,
                summary            TEXT,
                reason             TEXT,
                is_breaking        INTEGER DEFAULT 0,
                verification_status TEXT,
                verification_basis TEXT,
                outcome_notes      TEXT,
                snapshot_id        INTEGER REFERENCES snapshots(id),
                tree_hash          TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_transactions_entity_status ON transactions(entity_normalized, status);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_transactions_unaudited_entity ON transactions(entity_normalized) WHERE status = 'UNAUDITED';
            CREATE UNIQUE INDEX IF NOT EXISTS idx_transactions_pending_entity ON transactions(entity_normalized) WHERE status = 'PENDING';
            CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);
            CREATE INDEX IF NOT EXISTS idx_transactions_session_id ON transactions(session_id);
            CREATE INDEX IF NOT EXISTS idx_transactions_operation_id ON transactions(operation_id);",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS ledger_entries (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                tx_id              TEXT NOT NULL REFERENCES transactions(tx_id),
                operation_id       TEXT,
                category           TEXT NOT NULL,
                entry_type         TEXT NOT NULL DEFAULT 'IMPLEMENTATION',
                entity             TEXT NOT NULL,
                entity_normalized  TEXT NOT NULL,
                change_type        TEXT NOT NULL,
                summary            TEXT NOT NULL,
                reason             TEXT NOT NULL,
                is_breaking        INTEGER DEFAULT 0,
                committed_at       TEXT NOT NULL,
                verification_status TEXT,
                verification_basis TEXT,
                outcome_notes      TEXT,
                issue_ref          TEXT,
                trace_id           TEXT,
                origin             TEXT NOT NULL DEFAULT 'LOCAL',
                snapshot_id        INTEGER REFERENCES snapshots(id),
                tree_hash          TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_ledger_entries_entity ON ledger_entries(entity_normalized);
            CREATE INDEX IF NOT EXISTS idx_ledger_entries_category ON ledger_entries(category);
            CREATE INDEX IF NOT EXISTS idx_ledger_entries_committed_at ON ledger_entries(committed_at);
            CREATE INDEX IF NOT EXISTS idx_ledger_entries_operation_id ON ledger_entries(operation_id);
            
            CREATE VIRTUAL TABLE IF NOT EXISTS ledger_fts
                USING fts5(entity, summary, reason, content=ledger_entries, content_rowid=id);

            CREATE TRIGGER IF NOT EXISTS ledger_fts_ai AFTER INSERT ON ledger_entries BEGIN
                INSERT INTO ledger_fts(rowid, entity, summary, reason) VALUES (new.id, new.entity, new.summary, new.reason);
            END;
            CREATE TRIGGER IF NOT EXISTS ledger_fts_ad AFTER DELETE ON ledger_entries BEGIN
                INSERT INTO ledger_fts(ledger_fts, rowid, entity, summary, reason) VALUES ('delete', old.id, old.entity, old.summary, old.reason);
            END;
            CREATE TRIGGER IF NOT EXISTS ledger_fts_au AFTER UPDATE ON ledger_entries BEGIN
                INSERT INTO ledger_fts(ledger_fts, rowid, entity, summary, reason) VALUES ('delete', old.id, old.entity, old.summary, old.reason);
                INSERT INTO ledger_fts(rowid, entity, summary, reason) VALUES (new.id, new.entity, new.summary, new.reason);
            END;",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS tech_stack (
                category           TEXT PRIMARY KEY,
                name               TEXT NOT NULL,
                version_constraint TEXT,
                rules              TEXT NOT NULL DEFAULT '[]',
                locked             INTEGER DEFAULT 0,
                status             TEXT DEFAULT 'ACTIVE',
                entity_type        TEXT DEFAULT 'FILE',
                registered_at      TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS commit_validators (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                category           TEXT NOT NULL,
                name               TEXT NOT NULL,
                description        TEXT,
                executable         TEXT NOT NULL,
                args               TEXT NOT NULL,
                timeout_ms         INTEGER DEFAULT 30000,
                glob               TEXT,
                validation_level   TEXT DEFAULT 'ERROR',
                enabled            INTEGER DEFAULT 1
            );
            CREATE TABLE IF NOT EXISTS category_stack_mappings (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                ledger_category    TEXT NOT NULL,
                stack_category     TEXT NOT NULL REFERENCES tech_stack(category),
                glob               TEXT,
                description        TEXT
            );
            CREATE TABLE IF NOT EXISTS watcher_patterns (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                glob               TEXT NOT NULL,
                category           TEXT NOT NULL,
                source             TEXT NOT NULL DEFAULT 'CONFIG',
                description        TEXT
            );",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS token_provenance (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                tx_id              TEXT NOT NULL REFERENCES transactions(tx_id),
                entity             TEXT NOT NULL,
                entity_normalized  TEXT NOT NULL,
                symbol_name        TEXT NOT NULL,
                symbol_type        TEXT NOT NULL,
                action             TEXT NOT NULL -- 'ADDED', 'MODIFIED', 'DELETED'
            );
            CREATE INDEX IF NOT EXISTS idx_token_provenance_tx_id ON token_provenance(tx_id);
            CREATE INDEX IF NOT EXISTS idx_token_provenance_entity_symbol ON token_provenance(entity_normalized, symbol_name);",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS project_files (
                id              INTEGER PRIMARY KEY,
                file_path       TEXT NOT NULL,
                language        TEXT,
                content_hash    TEXT,
                git_blob_oid    TEXT,
                file_size       INTEGER,
                mtime_ns        INTEGER,
                parser_version  TEXT NOT NULL DEFAULT '1',
                parse_status    TEXT NOT NULL DEFAULT 'OK',
                last_indexed_at TEXT NOT NULL,
                UNIQUE(file_path)
            );
            CREATE TABLE IF NOT EXISTS index_metadata (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS project_symbols (
                id                    INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id               INTEGER NOT NULL REFERENCES project_files(id),
                qualified_name        TEXT NOT NULL,
                symbol_name           TEXT NOT NULL,
                symbol_kind           TEXT NOT NULL,
                visibility            TEXT,
                entrypoint_kind       TEXT NOT NULL DEFAULT 'INTERNAL',
                is_public             INTEGER DEFAULT 0,
                cognitive_complexity  INTEGER,
                cyclomatic_complexity  INTEGER,
                line_start           INTEGER,
                line_end             INTEGER,
                byte_start           INTEGER,
                byte_end             INTEGER,
                signature_hash       TEXT,
                confidence           REAL NOT NULL DEFAULT 1.0,
                evidence             TEXT,
                last_indexed_at      TEXT NOT NULL,
                UNIQUE(file_id, qualified_name, symbol_kind)
            );
            CREATE INDEX IF NOT EXISTS idx_project_symbols_file ON project_symbols(file_id);
            CREATE INDEX IF NOT EXISTS idx_project_symbols_qualified ON project_symbols(qualified_name);
            CREATE INDEX IF NOT EXISTS idx_project_symbols_name ON project_symbols(symbol_name);
            CREATE INDEX IF NOT EXISTS idx_project_symbols_kind ON project_symbols(symbol_kind);
            CREATE INDEX IF NOT EXISTS idx_project_symbols_entrypoint ON project_symbols(entrypoint_kind);
            CREATE TABLE IF NOT EXISTS project_docs (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id         INTEGER NOT NULL REFERENCES project_files(id),
                title           TEXT,
                summary         TEXT,
                sections        JSON,
                code_blocks     JSON,
                internal_links  JSON,
                confidence      REAL NOT NULL DEFAULT 1.0,
                last_indexed_at TEXT NOT NULL,
                UNIQUE(file_id)
            );
            CREATE INDEX IF NOT EXISTS idx_project_docs_file_id ON project_docs(file_id);
            CREATE TABLE IF NOT EXISTS project_topology (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                dir_path        TEXT NOT NULL,
                role            TEXT NOT NULL,
                confidence      REAL NOT NULL DEFAULT 1.0,
                evidence        TEXT,
                last_indexed_at TEXT NOT NULL,
                UNIQUE(dir_path)
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_project_topology_dir_path
                ON project_topology(dir_path);
            CREATE INDEX IF NOT EXISTS idx_project_topology_role
                ON project_topology(role);",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS structural_edges (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                caller_symbol_id    INTEGER NOT NULL REFERENCES project_symbols(id),
                caller_file_id      INTEGER NOT NULL REFERENCES project_files(id),
                callee_symbol_id    INTEGER REFERENCES project_symbols(id),
                callee_file_id      INTEGER REFERENCES project_files(id),
                unresolved_callee   TEXT,
                call_kind           TEXT NOT NULL DEFAULT 'DIRECT',
                resolution_status   TEXT NOT NULL DEFAULT 'RESOLVED',
                confidence          REAL NOT NULL DEFAULT 1.0,
                evidence            TEXT,
                FOREIGN KEY (caller_symbol_id) REFERENCES project_symbols(id),
                FOREIGN KEY (caller_file_id) REFERENCES project_files(id),
                FOREIGN KEY (callee_symbol_id) REFERENCES project_symbols(id),
                FOREIGN KEY (callee_file_id) REFERENCES project_files(id)
            );
            CREATE INDEX IF NOT EXISTS idx_structural_edges_caller
                ON structural_edges(caller_symbol_id, caller_file_id);
            CREATE INDEX IF NOT EXISTS idx_structural_edges_callee
                ON structural_edges(callee_symbol_id, callee_file_id);

            CREATE TABLE IF NOT EXISTS api_routes (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                method              TEXT NOT NULL,
                path_pattern        TEXT NOT NULL,
                handler_symbol_id   INTEGER REFERENCES project_symbols(id),
                handler_symbol_name TEXT,
                handler_file_id     INTEGER NOT NULL REFERENCES project_files(id),
                framework           TEXT NOT NULL,
                route_source        TEXT NOT NULL DEFAULT 'DECORATOR',
                mount_prefix        TEXT,
                is_dynamic          INTEGER DEFAULT 0,
                route_confidence    REAL NOT NULL DEFAULT 1.0,
                evidence            TEXT,
                last_indexed_at     TEXT NOT NULL,
                FOREIGN KEY (handler_symbol_id) REFERENCES project_symbols(id),
                FOREIGN KEY (handler_file_id) REFERENCES project_files(id)
            );
            CREATE INDEX IF NOT EXISTS idx_api_routes_handler
                ON api_routes(handler_symbol_id, handler_file_id);
            CREATE INDEX IF NOT EXISTS idx_api_routes_path
                ON api_routes(path_pattern);

            CREATE TABLE IF NOT EXISTS data_models (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                model_name      TEXT NOT NULL,
                model_file_id   INTEGER NOT NULL REFERENCES project_files(id),
                language        TEXT NOT NULL,
                model_kind      TEXT NOT NULL DEFAULT 'STRUCT',
                confidence      REAL NOT NULL DEFAULT 1.0,
                evidence        TEXT,
                fields          TEXT,
                last_indexed_at TEXT NOT NULL,
                FOREIGN KEY (model_file_id) REFERENCES project_files(id)
            );
            CREATE INDEX IF NOT EXISTS idx_data_models_name
                ON data_models(model_name);
            CREATE INDEX IF NOT EXISTS idx_data_models_file
                ON data_models(model_file_id);

            CREATE TABLE IF NOT EXISTS symbol_centrality (
                id                      INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol_id               INTEGER NOT NULL REFERENCES project_symbols(id),
                file_id                 INTEGER NOT NULL REFERENCES project_files(id),
                entrypoints_reachable   INTEGER NOT NULL DEFAULT 0,
                betweenness             REAL DEFAULT 0.0,
                last_computed_at        TEXT NOT NULL,
                FOREIGN KEY (symbol_id) REFERENCES project_symbols(id),
                FOREIGN KEY (file_id) REFERENCES project_files(id)
            );
            CREATE INDEX IF NOT EXISTS idx_symbol_centrality_symbol
                ON symbol_centrality(symbol_id);
            CREATE INDEX IF NOT EXISTS idx_symbol_centrality_file
                ON symbol_centrality(file_id);
            CREATE INDEX IF NOT EXISTS idx_symbol_centrality_reachable
                ON symbol_centrality(entrypoints_reachable);",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS observability_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL REFERENCES project_files(id),
                line_start INTEGER,
                pattern_kind TEXT NOT NULL DEFAULT 'LOG',
                level TEXT,
                framework TEXT,
                confidence REAL NOT NULL DEFAULT 1.0,
                evidence TEXT,
                in_test INTEGER DEFAULT 0,
                last_indexed_at TEXT NOT NULL,
                FOREIGN KEY (file_id) REFERENCES project_files(id)
            );
            CREATE INDEX IF NOT EXISTS idx_obs_patterns_file ON observability_patterns(file_id);
            CREATE INDEX IF NOT EXISTS idx_obs_patterns_kind ON observability_patterns(pattern_kind);",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS test_mapping (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                test_symbol_id INTEGER NOT NULL REFERENCES project_symbols(id),
                test_file_id INTEGER NOT NULL REFERENCES project_files(id),
                tested_symbol_id INTEGER REFERENCES project_symbols(id),
                tested_file_id INTEGER REFERENCES project_files(id),
                confidence REAL NOT NULL DEFAULT 1.0,
                mapping_kind TEXT NOT NULL DEFAULT 'IMPORT',
                evidence TEXT,
                last_indexed_at TEXT NOT NULL,
                FOREIGN KEY (test_symbol_id) REFERENCES project_symbols(id),
                FOREIGN KEY (test_file_id) REFERENCES project_files(id),
                FOREIGN KEY (tested_symbol_id) REFERENCES project_symbols(id),
                FOREIGN KEY (tested_file_id) REFERENCES project_files(id),
                UNIQUE(test_symbol_id, tested_symbol_id)
            );
            CREATE INDEX IF NOT EXISTS idx_test_mapping_tested ON test_mapping(tested_symbol_id);
            CREATE INDEX IF NOT EXISTS idx_test_mapping_test ON test_mapping(test_symbol_id);",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS ci_gates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ci_file_id INTEGER NOT NULL REFERENCES project_files(id),
                platform TEXT NOT NULL,
                job_name TEXT NOT NULL,
                trigger TEXT,
                steps TEXT,
                last_indexed_at TEXT NOT NULL,
                FOREIGN KEY (ci_file_id) REFERENCES project_files(id)
            );
            CREATE INDEX IF NOT EXISTS idx_ci_gates_file ON ci_gates(ci_file_id);
            CREATE INDEX IF NOT EXISTS idx_ci_gates_platform ON ci_gates(platform);",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS env_declarations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                var_name TEXT NOT NULL,
                source_file_id INTEGER NOT NULL REFERENCES project_files(id),
                source_kind TEXT NOT NULL,
                required INTEGER DEFAULT 0,
                default_value_redacted TEXT,
                description TEXT,
                confidence REAL NOT NULL DEFAULT 1.0,
                last_indexed_at TEXT NOT NULL,
                UNIQUE(var_name, source_file_id, source_kind)
            );
            CREATE INDEX IF NOT EXISTS idx_env_declarations_var ON env_declarations(var_name);
            CREATE INDEX IF NOT EXISTS idx_env_declarations_file ON env_declarations(source_file_id);",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS env_references (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL REFERENCES project_files(id),
                symbol_id INTEGER REFERENCES project_symbols(id),
                var_name TEXT NOT NULL,
                reference_kind TEXT NOT NULL,
                confidence REAL NOT NULL DEFAULT 1.0,
                line_start INTEGER,
                last_indexed_at TEXT NOT NULL,
                UNIQUE(file_id, symbol_id, var_name, reference_kind)
            );
            CREATE INDEX IF NOT EXISTS idx_env_references_file ON env_references(file_id);
            CREATE INDEX IF NOT EXISTS idx_env_references_var ON env_references(var_name);",
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_validate() {
        let migrations = get_migrations();
        // validate() runs all migrations on an internal in-memory DB
        migrations.validate().unwrap();
    }

    #[test]
    fn test_all_tables_exist() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        let expected_tables = [
            "snapshots",
            "batches",
            "changed_files",
            "verification_runs",
            "verification_results",
            "symbols",
            "federated_links",
            "federated_dependencies",
            "transactions",
            "ledger_entries",
            "ledger_fts",
            "tech_stack",
            "commit_validators",
            "category_stack_mappings",
            "watcher_patterns",
            "token_provenance",
            "project_files",
            "index_metadata",
            "project_symbols",
            "project_docs",
            "project_topology",
            "structural_edges",
            "api_routes",
            "data_models",
            "symbol_centrality",
            "observability_patterns",
            "test_mapping",
            "ci_gates",
            "env_declarations",
            "env_references",
        ];

        for table in &expected_tables {
            let count: i64 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "Table {} should exist", table);
        }
    }

    #[test]
    fn test_insert_and_query_token_provenance() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        let tx_id = "550e8400-e29b-41d4-a716-446655440000";
        conn.execute(
            "INSERT INTO transactions (tx_id, status, category, entity, entity_normalized, session_id, started_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (tx_id, "PENDING", "FEATURE", "src/main.rs", "src/main.rs", "session-1", "2026-01-01T00:00:00Z"),
        ).unwrap();

        conn.execute(
            "INSERT INTO token_provenance (tx_id, entity, entity_normalized, symbol_name, symbol_type, action)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (tx_id, "src/main.rs", "src/main.rs", "main", "Function", "ADDED"),
        ).unwrap();

        let (name, action): (String, String) = conn
            .query_row(
                "SELECT symbol_name, action FROM token_provenance WHERE tx_id = ?1",
                [tx_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(name, "main");
        assert_eq!(action, "ADDED");
    }

    #[test]
    fn test_insert_and_query_batches() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        conn.execute(
            "INSERT INTO batches (timestamp, event_count, batch_json) VALUES (?1, ?2, ?3)",
            ("2026-01-01T00:00:00Z", 5, "{}"),
        )
        .unwrap();

        let (ts, count): (String, i64) = conn
            .query_row(
                "SELECT timestamp, event_count FROM batches WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(ts, "2026-01-01T00:00:00Z");
        assert_eq!(count, 5);
    }

    #[test]
    fn test_insert_and_query_verification_run() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        conn.execute(
            "INSERT INTO verification_runs (timestamp, plan_json, overall_pass) VALUES (?1, ?2, ?3)",
            ("2026-01-01T00:00:00Z", r#"{"steps":[]}"#, 1),
        )
        .unwrap();

        let (pass,): (bool,) = conn
            .query_row(
                "SELECT overall_pass FROM verification_runs WHERE id = 1",
                [],
                |row| Ok((row.get::<_, i64>(0)? != 0,)),
            )
            .unwrap();
        assert!(pass);
    }

    #[test]
    fn test_insert_and_query_verification_result() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        conn.execute(
            "INSERT INTO verification_runs (timestamp, plan_json, overall_pass) VALUES (?1, ?2, ?3)",
            ("2026-01-01T00:00:00Z", None::<String>, 0),
        )
        .unwrap();

        conn.execute(
            "INSERT INTO verification_results (run_id, command, exit_code, duration_ms, truncated) VALUES (?1, ?2, ?3, ?4, ?5)",
            (1, "cargo test", 0, 5000, 0),
        )
        .unwrap();

        let (cmd, exit): (String, i64) = conn
            .query_row(
                "SELECT command, exit_code FROM verification_results WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(cmd, "cargo test");
        assert_eq!(exit, 0);
    }

    #[test]
    fn test_insert_and_query_changed_files() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        conn.execute(
            "INSERT INTO snapshots (timestamp, head_hash, branch_name, is_clean, packet_json) VALUES (?1, ?2, ?3, ?4, ?5)",
            ("2026-01-01T00:00:00Z", "abc123", "main", 0, "{}"),
        )
        .unwrap();

        conn.execute(
            "INSERT INTO changed_files (snapshot_id, path, status, is_staged) VALUES (?1, ?2, ?3, ?4)",
            (1, "src/main.rs", "Modified", 1),
        )
        .unwrap();

        let (path, status): (String, String) = conn
            .query_row(
                "SELECT path, status FROM changed_files WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(path, "src/main.rs");
        assert_eq!(status, "Modified");
    }

    #[test]
    fn test_insert_and_query_ledger_transaction() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        let tx_id = "550e8400-e29b-41d4-a716-446655440000";
        conn.execute(
            "INSERT INTO transactions (tx_id, status, category, entity, entity_normalized, session_id, started_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (tx_id, "PENDING", "FEATURE", "src/main.rs", "src/main.rs", "session-1", "2026-01-01T00:00:00Z"),
        ).unwrap();

        conn.execute(
            "INSERT INTO ledger_entries (tx_id, category, entry_type, entity, entity_normalized, change_type, summary, reason, committed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (tx_id, "FEATURE", "IMPLEMENTATION", "src/main.rs", "src/main.rs", "MODIFY", "Add feature X", "Required for Y", "2026-01-01T01:00:00Z"),
        ).unwrap();

        let (summary, reason): (String, String) = conn
            .query_row(
                "SELECT summary, reason FROM ledger_entries WHERE tx_id = ?1",
                [tx_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(summary, "Add feature X");
        assert_eq!(reason, "Required for Y");

        // Verify FTS5
        let fts_summary: String = conn
            .query_row(
                "SELECT summary FROM ledger_fts WHERE summary MATCH 'feature'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(fts_summary, "Add feature X");
    }

    #[test]
    fn test_insert_and_query_project_files_symbols() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert a project_files row
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, git_blob_oid, file_size, mtime_ns, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            ("src/lib.rs", "Rust", "abc123hash", "def456oid", 2048, 1700000000000000000i64, "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Query back the file and verify defaults
        let (file_path, parse_status, parser_version): (String, String, String) = conn
            .query_row(
                "SELECT file_path, parse_status, parser_version FROM project_files WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(file_path, "src/lib.rs");
        assert_eq!(parse_status, "OK", "parse_status should default to 'OK'");
        assert_eq!(parser_version, "1", "parser_version should default to '1'");

        // Insert a project_symbols row referencing that file
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (1i64, "crate::my_func", "my_func", "Function", "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Query back the symbol and verify FK relationship + defaults
        let (sym_name, entrypoint_kind, confidence): (String, String, f64) = conn
            .query_row(
                "SELECT symbol_name, entrypoint_kind, confidence FROM project_symbols WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(sym_name, "my_func");
        assert_eq!(
            entrypoint_kind, "INTERNAL",
            "entrypoint_kind should default to 'INTERNAL'"
        );
        assert!(
            (confidence - 1.0).abs() < f64::EPSILON,
            "confidence should default to 1.0"
        );

        // Verify the FK join works
        let (file_path_from_sym,): (String,) = conn
            .query_row(
                "SELECT pf.file_path FROM project_symbols ps JOIN project_files pf ON ps.file_id = pf.id WHERE ps.qualified_name = ?1",
                ["crate::my_func"],
                |row| Ok((row.get(0)?,)),
            )
            .unwrap();
        assert_eq!(file_path_from_sym, "src/lib.rs");
    }

    #[test]
    fn test_insert_and_query_project_docs() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert a project_files row first (FK dependency)
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("README.md", "Markdown", "md5hash", 1024, "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Insert a project_docs row
        conn.execute(
            "INSERT INTO project_docs (file_id, title, summary, sections, code_blocks, internal_links, confidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (1i64, "My Project", "A test project summary", "[]", "[]", "[]", 1.0_f64, "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Query back
        let (title, summary, confidence): (String, String, f64) = conn
            .query_row(
                "SELECT title, summary, confidence FROM project_docs WHERE file_id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(title, "My Project");
        assert_eq!(summary, "A test project summary");
        assert!((confidence - 1.0).abs() < f64::EPSILON);

        // Verify UNIQUE constraint on file_id
        let result = conn.execute(
            "INSERT INTO project_docs (file_id, title, confidence, last_indexed_at) VALUES (?1, ?2, ?3, ?4)",
            (1i64, "Duplicate", 0.5_f64, "2026-05-01T00:00:00Z"),
        );
        assert!(result.is_err(), "Should not allow duplicate file_id");
    }

    #[test]
    fn test_insert_and_query_project_topology() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert topology rows
        conn.execute(
            "INSERT INTO project_topology (dir_path, role, confidence, evidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                "src",
                "SOURCE",
                1.0_f64,
                "Path pattern match: src",
                "2026-05-01T00:00:00Z",
            ),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO project_topology (dir_path, role, confidence, evidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                "tests",
                "TEST",
                1.0_f64,
                "Path pattern match: tests",
                "2026-05-01T00:00:00Z",
            ),
        )
        .unwrap();

        // Query back
        let (role, confidence): (String, f64) = conn
            .query_row(
                "SELECT role, confidence FROM project_topology WHERE dir_path = 'src'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(role, "SOURCE");
        assert!((confidence - 1.0).abs() < f64::EPSILON);

        // Verify unique constraint on dir_path
        let result = conn.execute(
            "INSERT INTO project_topology (dir_path, role, confidence, evidence, last_indexed_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src", "SOURCE", 0.9_f64, "duplicate", "2026-05-01T00:00:00Z"),
        );
        assert!(result.is_err(), "Should not allow duplicate dir_path");

        // Verify role index works
        let test_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM project_topology WHERE role = 'TEST'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(test_count, 1);
    }

    #[test]
    fn test_project_symbols_entrypoint_kinds() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert a project_files row first (FK dependency)
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/main.rs", "Rust", "hash1", 100, "2026-05-01T00:00:00Z"),
        ).unwrap();

        let file_id = conn.last_insert_rowid();

        // Insert symbols with various entrypoint_kind values
        let kinds = ["ENTRYPOINT", "HANDLER", "PUBLIC_API", "TEST", "INTERNAL"];
        for (i, kind) in kinds.iter().enumerate() {
            conn.execute(
                "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, visibility, \
                 entrypoint_kind, is_public, cognitive_complexity, cyclomatic_complexity, \
                 confidence, last_indexed_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    file_id,
                    format!("fn{}", i),
                    format!("symbol_{}", i),
                    "Function",
                    "public",
                    kind,
                    1,
                    1,
                    1,
                    1.0,
                    "2026-05-01T00:00:00Z",
                ],
            ).unwrap();
        }

        // Verify each kind is stored and retrievable
        for kind in &kinds {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM project_symbols WHERE entrypoint_kind = ?1",
                    [kind],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(
                count, 1,
                "Expected 1 symbol with entrypoint_kind = {}",
                kind
            );
        }

        // Verify default is INTERNAL
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, visibility, \
             is_public, cognitive_complexity, cyclomatic_complexity, confidence, last_indexed_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                file_id,
                "fn_default",
                "default_sym",
                "Function",
                "private",
                0,
                0,
                0,
                1.0,
                "2026-05-01T00:00:00Z",
            ],
        ).unwrap();

        let default_kind: String = conn
            .query_row(
                "SELECT entrypoint_kind FROM project_symbols WHERE symbol_name = 'default_sym'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            default_kind, "INTERNAL",
            "entrypoint_kind should default to INTERNAL"
        );
    }

    #[test]
    fn test_insert_and_query_structural_edges() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert two project_files rows (caller and callee files)
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/caller.rs", "Rust", "hash_caller", 512, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let caller_file_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/callee.rs", "Rust", "hash_callee", 256, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let callee_file_id = conn.last_insert_rowid();

        // Insert two project_symbols rows (caller and callee)
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (caller_file_id, "crate::caller_fn", "caller_fn", "Function", "2026-05-01T00:00:00Z"),
        ).unwrap();
        let caller_symbol_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (callee_file_id, "crate::callee_fn", "callee_fn", "Function", "2026-05-01T00:00:00Z"),
        ).unwrap();
        let callee_symbol_id = conn.last_insert_rowid();

        // Insert DIRECT edge (resolution_status='RESOLVED', confidence=1.0)
        conn.execute(
            "INSERT INTO structural_edges
                (caller_symbol_id, caller_file_id, callee_symbol_id, callee_file_id, call_kind, resolution_status, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (caller_symbol_id, caller_file_id, callee_symbol_id, callee_file_id, "DIRECT", "RESOLVED", 1.0_f64),
        ).unwrap();

        // Insert METHOD_CALL edge (resolution_status='RESOLVED', confidence=1.0)
        conn.execute(
            "INSERT INTO structural_edges
                (caller_symbol_id, caller_file_id, callee_symbol_id, callee_file_id, call_kind, resolution_status, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (caller_symbol_id, caller_file_id, callee_symbol_id, callee_file_id, "METHOD_CALL", "RESOLVED", 1.0_f64),
        ).unwrap();

        // Insert DYNAMIC edge (resolution_status='UNRESOLVED', callee_symbol_id=NULL, unresolved_callee='some_func', confidence=0.5)
        conn.execute(
            "INSERT INTO structural_edges
                (caller_symbol_id, caller_file_id, callee_symbol_id, callee_file_id, unresolved_callee, call_kind, resolution_status, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (caller_symbol_id, caller_file_id, None::<i64>, None::<i64>, "some_func", "DYNAMIC", "UNRESOLVED", 0.5_f64),
        ).unwrap();

        // Verify all three rows can be queried back
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM structural_edges", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 3, "Should have 3 structural edges");

        // Verify DIRECT edge
        let (call_kind, resolution_status, confidence): (String, String, f64) = conn
            .query_row(
                "SELECT call_kind, resolution_status, confidence FROM structural_edges WHERE call_kind = 'DIRECT'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(call_kind, "DIRECT");
        assert_eq!(resolution_status, "RESOLVED");
        assert!((confidence - 1.0).abs() < f64::EPSILON);

        // Verify METHOD_CALL edge
        let (call_kind, resolution_status, confidence): (String, String, f64) = conn
            .query_row(
                "SELECT call_kind, resolution_status, confidence FROM structural_edges WHERE call_kind = 'METHOD_CALL'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(call_kind, "METHOD_CALL");
        assert_eq!(resolution_status, "RESOLVED");
        assert!((confidence - 1.0).abs() < f64::EPSILON);

        // Verify DYNAMIC edge with unresolved callee
        let (call_kind, resolution_status, unresolved_callee, callee_sym_id, confidence): (String, String, String, Option<i64>, f64) = conn
            .query_row(
                "SELECT call_kind, resolution_status, unresolved_callee, callee_symbol_id, confidence FROM structural_edges WHERE call_kind = 'DYNAMIC'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap();
        assert_eq!(call_kind, "DYNAMIC");
        assert_eq!(resolution_status, "UNRESOLVED");
        assert_eq!(unresolved_callee, "some_func");
        assert!(
            callee_sym_id.is_none(),
            "callee_symbol_id should be NULL for unresolved edge"
        );
        assert!((confidence - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_insert_and_query_api_routes() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert project_files row (FK prerequisite)
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/routes.rs", "Rust", "hash_routes", 1024, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let handler_file_id = conn.last_insert_rowid();

        // Insert project_symbols row (FK prerequisite)
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (handler_file_id, "crate::get_users", "get_users", "Function", "2026-05-01T00:00:00Z"),
        ).unwrap();
        let handler_symbol_id = conn.last_insert_rowid();

        // Insert GET route with DECORATOR source, confidence 1.0
        conn.execute(
            "INSERT INTO api_routes
                (method, path_pattern, handler_symbol_id, handler_symbol_name, handler_file_id,
                 framework, route_source, mount_prefix, is_dynamic, route_confidence, evidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                "GET",
                "/api/users",
                handler_symbol_id,
                "get_users",
                handler_file_id,
                "Axum",
                "DECORATOR",
                None::<String>,
                0,
                1.0_f64,
                Some("decorator attribute on function"),
                "2026-05-01T00:00:00Z",
            ],
        ).unwrap();

        // Insert POST route with APP_METHOD source
        conn.execute(
            "INSERT INTO api_routes
                (method, path_pattern, handler_symbol_id, handler_symbol_name, handler_file_id,
                 framework, route_source, mount_prefix, is_dynamic, route_confidence, evidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                "POST",
                "/api/users",
                handler_symbol_id,
                "create_user",
                handler_file_id,
                "Axum",
                "APP_METHOD",
                Some("/api"),
                0,
                1.0_f64,
                None::<String>,
                "2026-05-01T00:00:00Z",
            ],
        ).unwrap();

        // Insert dynamic route with is_dynamic=1, path_pattern="DYNAMIC", confidence 0.5
        conn.execute(
            "INSERT INTO api_routes
                (method, path_pattern, handler_symbol_id, handler_symbol_name, handler_file_id,
                 framework, route_source, mount_prefix, is_dynamic, route_confidence, evidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                "GET",
                "DYNAMIC",
                handler_symbol_id,
                "dynamic_handler",
                handler_file_id,
                "Express",
                "DECORATOR",
                None::<String>,
                1,
                0.5_f64,
                Some("inferred from framework convention"),
                "2026-05-01T00:00:00Z",
            ],
        ).unwrap();

        // Verify all three rows exist
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM api_routes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3, "Should have 3 api_routes rows");

        // Verify GET DECORATOR route
        let (method, path_pattern, route_source, confidence): (String, String, String, f64) = conn
            .query_row(
                "SELECT method, path_pattern, route_source, route_confidence FROM api_routes WHERE path_pattern = '/api/users' AND method = 'GET'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(method, "GET");
        assert_eq!(path_pattern, "/api/users");
        assert_eq!(route_source, "DECORATOR");
        assert!((confidence - 1.0).abs() < f64::EPSILON);

        // Verify POST APP_METHOD route
        let (method, route_source, mount_prefix): (String, String, Option<String>) = conn
            .query_row(
                "SELECT method, route_source, mount_prefix FROM api_routes WHERE method = 'POST'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(method, "POST");
        assert_eq!(route_source, "APP_METHOD");
        assert_eq!(mount_prefix, Some("/api".to_string()));

        // Verify dynamic route
        let (path_pattern, is_dynamic, confidence, evidence): (String, i64, f64, Option<String>) = conn
            .query_row(
                "SELECT path_pattern, is_dynamic, route_confidence, evidence FROM api_routes WHERE is_dynamic = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(path_pattern, "DYNAMIC");
        assert_eq!(is_dynamic, 1);
        assert!((confidence - 0.5).abs() < f64::EPSILON);
        assert_eq!(
            evidence,
            Some("inferred from framework convention".to_string())
        );

        // Verify FK join works - get handler file path through the relationship
        let (file_path,): (String,) = conn
            .query_row(
                "SELECT pf.file_path FROM api_routes ar JOIN project_files pf ON ar.handler_file_id = pf.id WHERE ar.method = 'GET' AND ar.path_pattern = '/api/users'",
                [],
                |row| Ok((row.get(0)?,)),
            )
            .unwrap();
        assert_eq!(file_path, "src/routes.rs");
    }

    #[test]
    fn test_insert_and_query_data_models() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert project_files row (FK prerequisite)
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/models/user.rs", "Rust", "hash_models", 2048, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let model_file_id = conn.last_insert_rowid();

        // Insert a STRUCT model with confidence 1.0
        conn.execute(
            "INSERT INTO data_models
                (model_name, model_file_id, language, model_kind, confidence, evidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            ("User", model_file_id, "Rust", "STRUCT", 1.0_f64, "derive: Serialize, Deserialize", "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Insert an INTERFACE model with confidence 0.7
        conn.execute(
            "INSERT INTO data_models
                (model_name, model_file_id, language, model_kind, confidence, evidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            ("UserRepository", model_file_id, "Rust", "INTERFACE", 0.7_f64, "dir: models/", "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Insert a GENERATED model with confidence 0.6 (no evidence)
        conn.execute(
            "INSERT INTO data_models
                (model_name, model_file_id, language, model_kind, confidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                "UserProto",
                model_file_id,
                "Rust",
                "GENERATED",
                0.6_f64,
                "2026-05-01T00:00:00Z",
            ),
        )
        .unwrap();

        // Verify all three rows exist
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM data_models", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3, "Should have 3 data_models rows");

        // Verify STRUCT model
        let (model_name, model_kind, confidence, evidence): (String, String, f64, Option<String>) = conn
            .query_row(
                "SELECT model_name, model_kind, confidence, evidence FROM data_models WHERE model_kind = 'STRUCT'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(model_name, "User");
        assert_eq!(model_kind, "STRUCT");
        assert!((confidence - 1.0).abs() < f64::EPSILON);
        assert_eq!(evidence, Some("derive: Serialize, Deserialize".to_string()));

        // Verify INTERFACE model
        let (model_name, model_kind, confidence, evidence): (String, String, f64, Option<String>) = conn
            .query_row(
                "SELECT model_name, model_kind, confidence, evidence FROM data_models WHERE model_kind = 'INTERFACE'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(model_name, "UserRepository");
        assert_eq!(model_kind, "INTERFACE");
        assert!((confidence - 0.7).abs() < f64::EPSILON);
        assert_eq!(evidence, Some("dir: models/".to_string()));

        // Verify GENERATED model (no evidence)
        let (model_name, model_kind, confidence, evidence): (String, String, f64, Option<String>) = conn
            .query_row(
                "SELECT model_name, model_kind, confidence, evidence FROM data_models WHERE model_kind = 'GENERATED'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(model_name, "UserProto");
        assert_eq!(model_kind, "GENERATED");
        assert!((confidence - 0.6).abs() < f64::EPSILON);
        assert!(
            evidence.is_none(),
            "GENERATED model should have no evidence"
        );

        // Verify FK join works
        let (file_path,): (String,) = conn
            .query_row(
                "SELECT pf.file_path FROM data_models dm JOIN project_files pf ON dm.model_file_id = pf.id WHERE dm.model_name = 'User'",
                [],
                |row| Ok((row.get(0)?,)),
            )
            .unwrap();
        assert_eq!(file_path, "src/models/user.rs");
    }

    #[test]
    fn test_insert_and_query_symbol_centrality() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert a project_files row (FK prerequisite)
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/lib.rs", "Rust", "hash_sc", 512, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let file_id = conn.last_insert_rowid();

        // Insert a project_symbols row (FK prerequisite)
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (file_id, "crate::my_func", "my_func", "Function", "2026-05-01T00:00:00Z"),
        ).unwrap();
        let symbol_id = conn.last_insert_rowid();

        // Insert a symbol_centrality row
        conn.execute(
            "INSERT INTO symbol_centrality (symbol_id, file_id, entrypoints_reachable, betweenness, last_computed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (symbol_id, file_id, 3i64, 0.42_f64, "2026-05-01T12:00:00Z"),
        ).unwrap();

        // Query back and verify
        let (entrypoints_reachable, betweenness, last_computed_at): (i64, f64, String) = conn
            .query_row(
                "SELECT entrypoints_reachable, betweenness, last_computed_at FROM symbol_centrality WHERE symbol_id = ?1",
                [symbol_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(entrypoints_reachable, 3);
        assert!((betweenness - 0.42).abs() < f64::EPSILON);
        assert_eq!(last_computed_at, "2026-05-01T12:00:00Z");

        // Verify FK join works — get file_path through the relationship
        let (file_path,): (String,) = conn
            .query_row(
                "SELECT pf.file_path FROM symbol_centrality sc JOIN project_files pf ON sc.file_id = pf.id WHERE sc.symbol_id = ?1",
                [symbol_id],
                |row| Ok((row.get(0)?,)),
            )
            .unwrap();
        assert_eq!(file_path, "src/lib.rs");

        // Verify default value for betweenness
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (file_id, "crate::other_func", "other_func", "Function", "2026-05-01T00:00:00Z"),
        ).unwrap();
        let other_symbol_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO symbol_centrality (symbol_id, file_id, entrypoints_reachable, last_computed_at)
             VALUES (?1, ?2, ?3, ?4)",
            (other_symbol_id, file_id, 1i64, "2026-05-01T12:00:00Z"),
        ).unwrap();

        let betweenness_default: f64 = conn
            .query_row(
                "SELECT betweenness FROM symbol_centrality WHERE symbol_id = ?1",
                [other_symbol_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            (betweenness_default - 0.0).abs() < f64::EPSILON,
            "betweenness should default to 0.0"
        );
    }

    #[test]
    fn test_insert_and_query_observability_patterns() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert two project_files rows (FK prerequisites)
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/lib.rs", "Rust", "hash_obs1", 2048, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let file_id_1 = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/errors.rs", "Rust", "hash_obs2", 1024, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let file_id_2 = conn.last_insert_rowid();

        // Insert a LOG pattern with level='info', framework='tracing', in_test=0
        conn.execute(
            "INSERT INTO observability_patterns (file_id, line_start, pattern_kind, level, framework, confidence, evidence, in_test, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (file_id_1, 42i64, "LOG", "info", "tracing", 1.0_f64, Some("tracing::info! macro call"), 0i64, "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Insert a LOG pattern with level='error', framework='logging', in_test=1
        conn.execute(
            "INSERT INTO observability_patterns (file_id, line_start, pattern_kind, level, framework, confidence, evidence, in_test, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (file_id_2, 100i64, "LOG", "error", "logging", 0.9_f64, Some("log::error! macro call"), 1i64, "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Insert an ERROR_HANDLE pattern with level=None
        conn.execute(
            "INSERT INTO observability_patterns (file_id, line_start, pattern_kind, confidence, in_test, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (file_id_2, 150i64, "ERROR_HANDLE", 0.8_f64, 0i64, "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Verify all three rows exist
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM observability_patterns", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 3, "Should have 3 observability_patterns rows");

        // Verify LOG pattern with level='info', framework='tracing', in_test=0
        let (pattern_kind, level, framework, in_test, confidence): (String, String, String, i64, f64) = conn
            .query_row(
                "SELECT pattern_kind, level, framework, in_test, confidence FROM observability_patterns WHERE level = 'info'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap();
        assert_eq!(pattern_kind, "LOG");
        assert_eq!(level, "info");
        assert_eq!(framework, "tracing");
        assert_eq!(in_test, 0);
        assert!((confidence - 1.0).abs() < f64::EPSILON);

        // Verify LOG pattern with level='error', framework='logging', in_test=1
        let (pattern_kind, level, framework, in_test): (String, String, String, i64) = conn
            .query_row(
                "SELECT pattern_kind, level, framework, in_test FROM observability_patterns WHERE level = 'error'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(pattern_kind, "LOG");
        assert_eq!(level, "error");
        assert_eq!(framework, "logging");
        assert_eq!(in_test, 1);

        // Verify ERROR_HANDLE pattern with level=None
        let (pattern_kind, level, confidence): (String, Option<String>, f64) = conn
            .query_row(
                "SELECT pattern_kind, level, confidence FROM observability_patterns WHERE pattern_kind = 'ERROR_HANDLE'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(pattern_kind, "ERROR_HANDLE");
        assert!(
            level.is_none(),
            "ERROR_HANDLE pattern should have level=NULL"
        );
        assert!((confidence - 0.8).abs() < f64::EPSILON);

        // Verify FK join works — get file_path through the relationship
        let (file_path,): (String,) = conn
            .query_row(
                "SELECT pf.file_path FROM observability_patterns op JOIN project_files pf ON op.file_id = pf.id WHERE op.level = 'info'",
                [],
                |row| Ok((row.get(0)?,)),
            )
            .unwrap();
        assert_eq!(file_path, "src/lib.rs");
    }

    #[test]
    fn test_insert_and_query_test_mapping() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert test file
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("tests/test_foo.rs", "Rust", "hash_test", 512, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let test_file_id = conn.last_insert_rowid();

        // Insert source file for foo
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/foo.rs", "Rust", "hash_foo", 1024, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let foo_file_id = conn.last_insert_rowid();

        // Insert source file for bar
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/bar.rs", "Rust", "hash_bar", 768, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let bar_file_id = conn.last_insert_rowid();

        // Insert test symbol
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (test_file_id, "crate::test_foo", "test_foo", "Function", "2026-05-01T00:00:00Z"),
        ).unwrap();
        let test_symbol_id = conn.last_insert_rowid();

        // Insert tested symbol foo
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (foo_file_id, "crate::foo", "foo", "Function", "2026-05-01T00:00:00Z"),
        ).unwrap();
        let foo_symbol_id = conn.last_insert_rowid();

        // Insert tested symbol bar
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (bar_file_id, "crate::bar", "bar", "Function", "2026-05-01T00:00:00Z"),
        ).unwrap();
        let bar_symbol_id = conn.last_insert_rowid();

        // Insert IMPORT mapping (confidence 1.0)
        conn.execute(
            "INSERT INTO test_mapping (test_symbol_id, test_file_id, tested_symbol_id, tested_file_id, confidence, mapping_kind, evidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (test_symbol_id, test_file_id, foo_symbol_id, foo_file_id, 1.0_f64, "IMPORT", Some("use crate::foo;"), "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Insert NAMING_CONVENTION mapping (confidence 0.5) — different tested symbol
        conn.execute(
            "INSERT INTO test_mapping (test_symbol_id, test_file_id, tested_symbol_id, tested_file_id, confidence, mapping_kind, evidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (test_symbol_id, test_file_id, bar_symbol_id, bar_file_id, 0.5_f64, "NAMING_CONVENTION", Some("test_foo -> foo"), "2026-05-01T00:00:00Z"),
        ).unwrap();

        // Verify both rows exist
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM test_mapping", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2, "Should have 2 test_mapping rows");

        // Verify IMPORT mapping
        let (mapping_kind, confidence, evidence): (String, f64, Option<String>) = conn
            .query_row(
                "SELECT mapping_kind, confidence, evidence FROM test_mapping WHERE mapping_kind = 'IMPORT'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(mapping_kind, "IMPORT");
        assert!((confidence - 1.0).abs() < f64::EPSILON);
        assert_eq!(evidence, Some("use crate::foo;".to_string()));

        // Verify NAMING_CONVENTION mapping
        let (mapping_kind, confidence): (String, f64) = conn
            .query_row(
                "SELECT mapping_kind, confidence FROM test_mapping WHERE mapping_kind = 'NAMING_CONVENTION'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(mapping_kind, "NAMING_CONVENTION");
        assert!((confidence - 0.5).abs() < f64::EPSILON);

        // Verify UNIQUE constraint on (test_symbol_id, tested_symbol_id)
        let result = conn.execute(
            "INSERT INTO test_mapping (test_symbol_id, test_file_id, tested_symbol_id, tested_file_id, confidence, mapping_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (test_symbol_id, test_file_id, foo_symbol_id, foo_file_id, 0.7_f64, "IMPORT", "2026-05-01T00:00:00Z"),
        );
        assert!(
            result.is_err(),
            "Should not allow duplicate (test_symbol_id, tested_symbol_id)"
        );

        // Verify index on tested_symbol_id works
        let tested_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM test_mapping WHERE tested_symbol_id = ?1",
                [foo_symbol_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tested_count, 1);

        // Verify FK join works — get test file path and tested symbol name through the relationship
        let (test_file, tested_name): (String, String) = conn
            .query_row(
                "SELECT pf.file_path, ps.symbol_name FROM test_mapping tm
                 JOIN project_files pf ON tm.test_file_id = pf.id
                 JOIN project_symbols ps ON tm.tested_symbol_id = ps.id
                 WHERE tm.mapping_kind = 'IMPORT'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(test_file, "tests/test_foo.rs");
        assert_eq!(tested_name, "foo");
    }

    #[test]
    fn test_insert_and_query_ci_gates() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert a project_files row first (FK prerequisite)
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (".github/workflows/ci.yml", "YAML", "hash_ci", 512, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let ci_file_id = conn.last_insert_rowid();

        // Insert a GitHub Actions CI gate
        conn.execute(
            "INSERT INTO ci_gates (ci_file_id, platform, job_name, trigger, steps, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                ci_file_id,
                "github_actions",
                "build",
                "push",
                "checkout, build, test",
                "2026-05-01T00:00:00Z",
            ),
        )
        .unwrap();

        // Insert a GitLab CI gate
        conn.execute(
            "INSERT INTO ci_gates (ci_file_id, platform, job_name, trigger, steps, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                ci_file_id,
                "gitlab_ci",
                "deploy",
                "merge_request",
                "build, deploy",
                "2026-05-01T00:00:00Z",
            ),
        )
        .unwrap();

        // Verify both rows exist
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM ci_gates", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2, "Should have 2 ci_gates rows");

        // Verify GitHub Actions gate
        let (platform, job_name, trigger): (String, String, Option<String>) = conn
            .query_row(
                "SELECT platform, job_name, trigger FROM ci_gates WHERE platform = 'github_actions'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(platform, "github_actions");
        assert_eq!(job_name, "build");
        assert_eq!(trigger, Some("push".to_string()));

        // Verify GitLab CI gate
        let (platform, job_name, trigger): (String, String, Option<String>) = conn
            .query_row(
                "SELECT platform, job_name, trigger FROM ci_gates WHERE platform = 'gitlab_ci'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(platform, "gitlab_ci");
        assert_eq!(job_name, "deploy");
        assert_eq!(trigger, Some("merge_request".to_string()));

        // Verify FK join works — get file_path through the relationship
        let (file_path,): (String,) = conn
            .query_row(
                "SELECT pf.file_path FROM ci_gates cg JOIN project_files pf ON cg.ci_file_id = pf.id WHERE cg.platform = 'github_actions'",
                [],
                |row| Ok((row.get(0)?,)),
            )
            .unwrap();
        assert_eq!(file_path, ".github/workflows/ci.yml");

        // Verify index on platform works
        let platform_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ci_gates WHERE platform = 'github_actions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(platform_count, 1);
    }

    #[test]
    fn test_insert_and_query_env_declarations() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert a project_files row first (FK prerequisite)
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (".env.example", "Dotenv", "hash_env", 256, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let env_file_id = conn.last_insert_rowid();

        // Insert a DOTENV_EXAMPLE declaration with HAS_DEFAULT
        conn.execute(
            "INSERT INTO env_declarations (var_name, source_file_id, source_kind, required, default_value_redacted, description, confidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                "DATABASE_URL",
                env_file_id,
                "DOTENV_EXAMPLE",
                0,
                "HAS_DEFAULT",
                Some("Database connection string"),
                1.0_f64,
                "2026-05-01T00:00:00Z",
            ],
        ).unwrap();

        // Insert a declaration with EMPTY_DEFAULT and required=1
        conn.execute(
            "INSERT INTO env_declarations (var_name, source_file_id, source_kind, required, default_value_redacted, confidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                "API_KEY",
                env_file_id,
                "DOTENV_EXAMPLE",
                1,
                "EMPTY_DEFAULT",
                1.0_f64,
                "2026-05-01T00:00:00Z",
            ],
        ).unwrap();

        // Verify both rows exist
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM env_declarations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 2, "Should have 2 env_declarations rows");

        // Verify DOTENV_EXAMPLE declaration
        let (var_name, source_kind, required, default_value_redacted, confidence): (String, String, i64, String, f64) = conn
            .query_row(
                "SELECT var_name, source_kind, required, default_value_redacted, confidence FROM env_declarations WHERE var_name = 'DATABASE_URL'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap();
        assert_eq!(var_name, "DATABASE_URL");
        assert_eq!(source_kind, "DOTENV_EXAMPLE");
        assert_eq!(required, 0);
        assert_eq!(default_value_redacted, "HAS_DEFAULT");
        assert!((confidence - 1.0).abs() < f64::EPSILON);

        // Verify required declaration
        let (var_name, required, default_value_redacted): (String, i64, String) = conn
            .query_row(
                "SELECT var_name, required, default_value_redacted FROM env_declarations WHERE var_name = 'API_KEY'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(var_name, "API_KEY");
        assert_eq!(required, 1);
        assert_eq!(default_value_redacted, "EMPTY_DEFAULT");

        // Verify FK join works — get file_path through the relationship
        let (file_path,): (String,) = conn
            .query_row(
                "SELECT pf.file_path FROM env_declarations ed JOIN project_files pf ON ed.source_file_id = pf.id WHERE ed.var_name = 'DATABASE_URL'",
                [],
                |row| Ok((row.get(0)?,)),
            )
            .unwrap();
        assert_eq!(file_path, ".env.example");

        // Verify index on var_name works
        let var_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM env_declarations WHERE var_name = 'DATABASE_URL'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(var_count, 1);

        // Verify UNIQUE constraint on (var_name, source_file_id, source_kind)
        let result = conn.execute(
            "INSERT INTO env_declarations (var_name, source_file_id, source_kind, required, confidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["DATABASE_URL", env_file_id, "DOTENV_EXAMPLE", 0, 1.0_f64, "2026-05-01T00:00:00Z"],
        );
        assert!(
            result.is_err(),
            "Should not allow duplicate (var_name, source_file_id, source_kind)"
        );
    }

    #[test]
    fn test_insert_and_query_env_references() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();

        // Insert project_files row (FK prerequisite)
        conn.execute(
            "INSERT INTO project_files (file_path, language, content_hash, file_size, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            ("src/main.rs", "Rust", "hash_main", 1024, "2026-05-01T00:00:00Z"),
        ).unwrap();
        let file_id = conn.last_insert_rowid();

        // Insert project_symbols row (FK prerequisite)
        conn.execute(
            "INSERT INTO project_symbols (file_id, qualified_name, symbol_name, symbol_kind, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (file_id, "crate::main", "main", "Function", "2026-05-01T00:00:00Z"),
        ).unwrap();
        let symbol_id = conn.last_insert_rowid();

        // Insert a READ reference
        conn.execute(
            "INSERT INTO env_references (file_id, symbol_id, var_name, reference_kind, confidence, line_start, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![file_id, symbol_id, "DATABASE_URL", "READ", 1.0_f64, 42i64, "2026-05-01T00:00:00Z"],
        ).unwrap();

        // Insert a DEFAULTED reference
        conn.execute(
            "INSERT INTO env_references (file_id, symbol_id, var_name, reference_kind, confidence, line_start, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![file_id, symbol_id, "DEBUG_MODE", "DEFAULTED", 0.9_f64, 55i64, "2026-05-01T00:00:00Z"],
        ).unwrap();

        // Insert a reference with NULL symbol_id
        conn.execute(
            "INSERT INTO env_references (file_id, symbol_id, var_name, reference_kind, confidence, line_start, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![file_id, None::<i64>, "API_TOKEN", "READ", 1.0_f64, 60i64, "2026-05-01T00:00:00Z"],
        ).unwrap();

        // Verify all three rows exist
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM env_references", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3, "Should have 3 env_references rows");

        // Verify READ reference
        let (var_name, reference_kind, confidence, line_start): (String, String, f64, Option<i64>) = conn
            .query_row(
                "SELECT var_name, reference_kind, confidence, line_start FROM env_references WHERE var_name = 'DATABASE_URL'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(var_name, "DATABASE_URL");
        assert_eq!(reference_kind, "READ");
        assert!((confidence - 1.0).abs() < f64::EPSILON);
        assert_eq!(line_start, Some(42));

        // Verify DEFAULTED reference
        let (var_name, reference_kind, confidence): (String, String, f64) = conn
            .query_row(
                "SELECT var_name, reference_kind, confidence FROM env_references WHERE var_name = 'DEBUG_MODE'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(var_name, "DEBUG_MODE");
        assert_eq!(reference_kind, "DEFAULTED");
        assert!((confidence - 0.9).abs() < f64::EPSILON);

        // Verify reference with NULL symbol_id
        let (symbol_id_val,): (Option<i64>,) = conn
            .query_row(
                "SELECT symbol_id FROM env_references WHERE var_name = 'API_TOKEN'",
                [],
                |row| Ok((row.get(0)?,)),
            )
            .unwrap();
        assert!(
            symbol_id_val.is_none(),
            "symbol_id should be NULL for reference without a symbol"
        );

        // Verify FK join works — get file_path through the relationship
        let (file_path,): (String,) = conn
            .query_row(
                "SELECT pf.file_path FROM env_references er JOIN project_files pf ON er.file_id = pf.id WHERE er.var_name = 'DATABASE_URL'",
                [],
                |row| Ok((row.get(0)?,)),
            )
            .unwrap();
        assert_eq!(file_path, "src/main.rs");

        // Verify index on var_name works
        let var_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM env_references WHERE var_name = 'DATABASE_URL'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(var_count, 1);

        // Verify UNIQUE constraint on (file_id, symbol_id, var_name, reference_kind)
        let result = conn.execute(
            "INSERT INTO env_references (file_id, symbol_id, var_name, reference_kind, confidence, last_indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![file_id, symbol_id, "DATABASE_URL", "READ", 1.0_f64, "2026-05-01T00:00:00Z"],
        );
        assert!(
            result.is_err(),
            "Should not allow duplicate (file_id, symbol_id, var_name, reference_kind)"
        );
    }
}
