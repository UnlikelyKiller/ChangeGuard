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
}
