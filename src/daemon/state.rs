use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::thread;
use std::time::Duration;
use tracing::warn;
use miette::{IntoDiagnostic, Result};
use crate::impact::packet::ImpactPacket;

pub struct QueryResult<T> {
    pub data: T,
    pub data_stale: bool,
}

pub struct ReadOnlyStorage {
    db_path: std::path::PathBuf,
}

impl ReadOnlyStorage {
    pub fn new(db_path: &Path) -> Self {
        Self {
            db_path: db_path.to_path_buf(),
        }
    }

    fn get_connection(&self) -> Result<Connection> {
        // Open in read-only mode and enable WAL compatibility
        let conn = Connection::open_with_flags(
            &self.db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ).into_diagnostic()?;
        
        // Ensure WAL mode is handled correctly by SQLite
        conn.execute_batch("PRAGMA journal_mode=WAL;").into_diagnostic()?;
        
        Ok(conn)
    }

    pub fn query<F, T>(&self, f: F) -> Result<QueryResult<Option<T>>>
    where
        F: Fn(&Connection) -> Result<Option<T>>,
    {
        let mut backoff = Duration::from_millis(100);
        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            match self.get_connection() {
                Ok(conn) => {
                    match f(&conn) {
                        Ok(data) => return Ok(QueryResult { data, data_stale: false }),
                        Err(e) => {
                            // Check if it's a rusqlite error and specifically SQLITE_BUSY
                            if let Some(rusqlite_err) = e.downcast_ref::<rusqlite::Error>() {
                                if matches!(rusqlite_err, rusqlite::Error::SqliteFailure(err, _) if err.code == rusqlite::ErrorCode::DatabaseBusy) {
                                    if attempts < max_attempts {
                                        warn!("Database busy, retrying in {:?} (attempt {})", backoff, attempts + 1);
                                        thread::sleep(backoff);
                                        attempts += 1;
                                        backoff *= 2;
                                        continue;
                                    } else {
                                        warn!("Database busy, max attempts reached. Returning stale/empty data.");
                                        return Ok(QueryResult { data: None, data_stale: true });
                                    }
                                }
                            }
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    if attempts < max_attempts {
                         warn!("Failed to open connection (busy?), retrying in {:?} (attempt {})", backoff, attempts + 1);
                         thread::sleep(backoff);
                         attempts += 1;
                         backoff *= 2;
                         continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    pub fn get_latest_packet(&self) -> Result<QueryResult<Option<ImpactPacket>>> {
        self.query(|conn| {
            let mut stmt = conn.prepare("SELECT packet_json FROM snapshots ORDER BY id DESC LIMIT 1")
                .into_diagnostic()?;
            let mut rows = stmt.query([]).into_diagnostic()?;

            if let Some(row) = rows.next().into_diagnostic()? {
                let json: String = row.get(0).into_diagnostic()?;
                let packet: ImpactPacket = serde_json::from_str(&json).into_diagnostic()?;
                Ok(Some(packet))
            } else {
                Ok(None)
            }
        })
    }
}
