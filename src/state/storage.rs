use crate::impact::packet::{ChangedFile, ImpactPacket};
use crate::index::storage::persist_symbols;
use crate::state::migrations::get_migrations;
use miette::{IntoDiagnostic, Result};
use rusqlite::Connection;
use std::path::Path;
use tracing::info;

use crate::index::symbols::SymbolKind;

pub struct StoredSymbol {
    pub file_path: String,
    pub name: String,
    pub kind: SymbolKind,
    pub is_public: bool,
}

pub struct StorageManager {
    conn: Connection,
}

impl StorageManager {
    pub fn init(db_path: &Path) -> Result<Self> {
        let mut conn = Connection::open(db_path).into_diagnostic()?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000; PRAGMA foreign_keys = ON;",
        )
        .into_diagnostic()?;

        let migrations = get_migrations();
        migrations.to_latest(&mut conn).into_diagnostic()?;

        info!("Initialized storage at {:?}", db_path);
        Ok(Self { conn })
    }

    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }

    pub fn get_connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    pub fn init_from_conn(conn: Connection) -> Self {
        Self { conn }
    }

    pub fn save_packet(&self, packet: &ImpactPacket) -> Result<()> {
        let packet_json = serde_json::to_string(packet).into_diagnostic()?;
        let is_clean = if packet.changes.is_empty() { 1 } else { 0 };

        self.conn
            .execute(
                "INSERT INTO snapshots (timestamp, head_hash, branch_name, is_clean, packet_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    &packet.timestamp_utc,
                    &packet.head_hash,
                    &packet.branch_name,
                    is_clean,
                    &packet_json,
                ),
            )
            .into_diagnostic()?;

        let snapshot_id = self.conn.last_insert_rowid();
        self.save_changed_files(snapshot_id, &packet.changes)?;
        persist_symbols(&self.conn, snapshot_id, &packet.changes)?;

        Ok(())
    }

    pub fn get_latest_packet(&self) -> Result<Option<ImpactPacket>> {
        let mut stmt = self
            .conn
            .prepare("SELECT packet_json FROM snapshots ORDER BY id DESC LIMIT 1")
            .into_diagnostic()?;

        let mut rows = stmt.query([]).into_diagnostic()?;

        if let Some(row) = rows.next().into_diagnostic()? {
            let json: String = row.get(0).into_diagnostic()?;
            let packet: ImpactPacket = serde_json::from_str(&json).into_diagnostic()?;
            Ok(Some(packet))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_packets(&self) -> Result<Vec<ImpactPacket>> {
        let mut stmt = self
            .conn
            .prepare("SELECT packet_json FROM snapshots ORDER BY id ASC")
            .into_diagnostic()?;

        let rows = stmt
            .query_map([], |row| {
                let json: String = row.get(0)?;
                serde_json::from_str(&json).map_err(|_e| rusqlite::Error::InvalidQuery)
            })
            .into_diagnostic()?;

        let mut packets = Vec::new();
        for packet in rows {
            packets.push(packet.into_diagnostic()?);
        }
        Ok(packets)
    }

    pub fn save_batch(&self, timestamp: &str, event_count: u32, batch_json: &str) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO batches (timestamp, event_count, batch_json) VALUES (?1, ?2, ?3)",
                (timestamp, event_count, batch_json),
            )
            .into_diagnostic()?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn save_verification_run(
        &self,
        timestamp: &str,
        plan_json: Option<&str>,
        overall_pass: bool,
    ) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO verification_runs (timestamp, plan_json, overall_pass) VALUES (?1, ?2, ?3)",
                (timestamp, plan_json, overall_pass as i32),
            )
            .into_diagnostic()?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn save_verification_result(
        &self,
        run_id: i64,
        command: &str,
        exit_code: i32,
        duration_ms: u64,
        truncated: bool,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO verification_results (run_id, command, exit_code, duration_ms, truncated) VALUES (?1, ?2, ?3, ?4, ?5)",
                (run_id, command, exit_code, duration_ms as i64, truncated as i32),
            )
            .into_diagnostic()?;
        Ok(())
    }

    pub fn save_changed_files(&self, snapshot_id: i64, files: &[ChangedFile]) -> Result<()> {
        for file in files {
            self.conn
                .execute(
                    "INSERT INTO changed_files (snapshot_id, path, status, is_staged) VALUES (?1, ?2, ?3, ?4)",
                    (snapshot_id, file.path.to_string_lossy().as_ref(), &file.status, file.is_staged as i32),
                )
                .into_diagnostic()?;
        }
        Ok(())
    }

    pub fn get_latest_verification_run(&self) -> Result<Option<(i64, String, bool)>> {
        let result = self.conn.query_row(
            "SELECT id, timestamp, overall_pass FROM verification_runs ORDER BY id DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, i64>(2)? != 0)),
        );

        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).into_diagnostic(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::{FileAnalysisStatus, ImpactPacket};
    use std::path::PathBuf;

    fn in_memory_storage() -> StorageManager {
        let conn = Connection::open_in_memory().unwrap();
        let mut conn = conn;
        get_migrations().to_latest(&mut conn).unwrap();
        StorageManager { conn }
    }

    #[test]
    fn test_storage_basic_ops() {
        let storage = in_memory_storage();

        let packet = ImpactPacket {
            head_hash: Some("test_hash".to_string()),
            ..Default::default()
        };

        storage.save_packet(&packet).unwrap();

        let latest = storage.get_latest_packet().unwrap().unwrap();
        assert_eq!(latest.head_hash, Some("test_hash".to_string()));
    }

    #[test]
    fn test_save_batch() {
        let storage = in_memory_storage();
        let id = storage
            .save_batch("2026-01-01T00:00:00Z", 3, r#"{"events":[]}"#)
            .unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_save_verification_run() {
        let storage = in_memory_storage();
        let id = storage
            .save_verification_run("2026-01-01T00:00:00Z", Some(r#"{"steps":[]}"#), true)
            .unwrap();
        assert!(id > 0);

        let latest = storage.get_latest_verification_run().unwrap().unwrap();
        assert_eq!(latest.0, id);
        assert!(latest.2);
    }

    #[test]
    fn test_save_verification_result() {
        let storage = in_memory_storage();
        let run_id = storage
            .save_verification_run("2026-01-01T00:00:00Z", None, false)
            .unwrap();
        storage
            .save_verification_result(run_id, "cargo test", 1, 3000, false)
            .unwrap();
    }

    #[test]
    fn test_save_changed_files() {
        let storage = in_memory_storage();
        let packet = ImpactPacket {
            head_hash: Some("abc".to_string()),
            changes: vec![ChangedFile {
                path: PathBuf::from("src/main.rs"),
                status: "Modified".to_string(),
                is_staged: true,
                symbols: None,
                imports: None,
                runtime_usage: None,
                analysis_status: FileAnalysisStatus::default(),
                analysis_warnings: Vec::new(),
                api_routes: Vec::new(),
            }],
            ..Default::default()
        };
        storage.save_packet(&packet).unwrap();

        let snapshot_id = storage.conn.last_insert_rowid();
        storage
            .save_changed_files(snapshot_id, &packet.changes)
            .unwrap();
    }

    #[test]
    fn test_get_latest_verification_run_empty() {
        let storage = in_memory_storage();
        let result = storage.get_latest_verification_run().unwrap();
        assert!(result.is_none());
    }
}
