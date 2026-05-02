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
                id              INTEGER PRIMARY KEY,
                dir_path        TEXT NOT NULL,
                role            TEXT NOT NULL,
                confidence      REAL NOT NULL DEFAULT 1.0,
                evidence        TEXT,
                last_indexed_at TEXT NOT NULL,
                UNIQUE(dir_path)
            );",
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
}
