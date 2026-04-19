use crate::impact::packet::ChangedFile;
use crate::index::normalize::normalize_repo_path;
use miette::{IntoDiagnostic, Result};
use rusqlite::Connection;

pub fn persist_symbols(conn: &Connection, snapshot_id: i64, files: &[ChangedFile]) -> Result<()> {
    for file in files {
        let Some(symbols) = &file.symbols else {
            continue;
        };

        let file_path = normalize_repo_path(&file.path);
        for symbol in symbols {
            conn.execute(
                "INSERT INTO symbols (snapshot_id, file_path, symbol_name, symbol_kind, is_public)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    snapshot_id,
                    &file_path,
                    &symbol.name,
                    format!("{:?}", symbol.kind),
                    symbol.is_public as i32,
                ),
            )
            .into_diagnostic()?;
        }
    }

    Ok(())
}
