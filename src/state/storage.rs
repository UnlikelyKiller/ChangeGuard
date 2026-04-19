use crate::impact::packet::ImpactPacket;
use crate::state::migrations::get_migrations;
use miette::{IntoDiagnostic, Result};
use rusqlite::Connection;
use std::path::Path;
use tracing::info;

pub struct StorageManager {
    conn: Connection,
}

impl StorageManager {
    pub fn init(db_path: &Path) -> Result<Self> {
        let mut conn = Connection::open(db_path).into_diagnostic()?;

        let migrations = get_migrations();
        migrations.to_latest(&mut conn).into_diagnostic()?;

        info!("Initialized storage at {:?}", db_path);
        Ok(Self { conn })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impact::packet::ImpactPacket;

    #[test]
    fn test_storage_basic_ops() {
        let conn = Connection::open_in_memory().unwrap();
        let mut conn = conn;
        get_migrations().to_latest(&mut conn).unwrap();
        let storage = StorageManager { conn };

        let mut packet = ImpactPacket::default();
        packet.head_hash = Some("test_hash".to_string());

        storage.save_packet(&packet).unwrap();

        let latest = storage.get_latest_packet().unwrap().unwrap();
        assert_eq!(latest.head_hash, Some("test_hash".to_string()));
    }
}
