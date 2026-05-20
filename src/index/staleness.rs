use crate::state::storage::StorageManager;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Warning emitted when the index has not been refreshed recently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalenessWarning {
    /// Approximate number of days since the last index operation completed.
    pub days_since_indexed: u64,
    /// Number of files whose content has changed since they were last indexed.
    pub stale_files: usize,
}

/// Check whether the Tantivy/CozoDB index is stale relative to the configured
/// threshold.
///
/// Returns `Some(StalenessWarning)` when `days_since_indexed > threshold_days`,
/// or when no index has ever been run. Returns `None` when the index is fresh
/// enough.
///
/// # Parameters
///
/// * `storage`  – opened `StorageManager` whose SQLite connection holds the
///   `project_files` table.
/// * `threshold_days` – number of days that may elapse before the index is
///   considered stale (e.g. 3).
pub fn check_index_staleness(
    _storage: &StorageManager,
    _threshold_days: u64,
) -> Option<StalenessWarning> {
    // RED phase stub: always returns None.
    // GREEN phase will implement the actual staleness query.
    None
}

/// Emit the staleness warning to stderr so it does not interfere with --json
/// output on stdout.
pub fn print_staleness_warning(warning: &StalenessWarning) {
    eprintln!(
        "⚠  Index is {} day{} old with {} stale file{} — results may be degraded. \
         Run `changeguard index` to refresh.",
        warning.days_since_indexed,
        if warning.days_since_indexed == 1 { "" } else { "s" },
        warning.stale_files,
        if warning.stale_files == 1 { "" } else { "s" },
    );
}

/// Run `check_index_staleness` and print the warning banner when stale.
/// Returns `true` if a warning was printed.
pub fn warn_if_stale(storage: &StorageManager, threshold_days: u64) -> bool {
    if let Some(warning) = check_index_staleness(storage, threshold_days) {
        print_staleness_warning(&warning);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::migrations::get_migrations;
    use rusqlite::Connection;

    fn in_memory_storage() -> StorageManager {
        let conn = Connection::open_in_memory().unwrap();
        let mut conn = conn;
        get_migrations().to_latest(&mut conn).unwrap();
        StorageManager::init_from_conn(conn)
    }

    #[test]
    fn staleness_check_fresh() {
        let storage = in_memory_storage();
        let now = Utc::now().to_rfc3339();
        let conn = storage.get_connection();

        conn.execute(
            "INSERT INTO project_files (file_path, parse_status, last_indexed_at) \
             VALUES (?1, ?2, ?3)",
            rusqlite::params!["src/main.rs", "OK", &now],
        )
        .unwrap();

        let result = check_index_staleness(&storage, 3);
        assert!(result.is_none(), "fresh index should not be stale");
    }

    #[test]
    fn staleness_check_stale() {
        let storage = in_memory_storage();
        let old_date = (Utc::now() - chrono::Duration::days(10)).to_rfc3339();
        let conn = storage.get_connection();

        conn.execute(
            "INSERT INTO project_files (file_path, parse_status, last_indexed_at) \
             VALUES (?1, ?2, ?3)",
            rusqlite::params!["src/main.rs", "OK", &old_date],
        )
        .unwrap();

        let result = check_index_staleness(&storage, 3);
        assert!(result.is_some(), "stale index should return warning");
        if let Some(warning) = result {
            assert!(
                warning.days_since_indexed >= 10,
                "days_since_indexed should be >= 10, got {}",
                warning.days_since_indexed
            );
            assert!(
                warning.stale_files >= 1,
                "should have at least 1 stale file"
            );
        }
    }

    #[test]
    fn staleness_check_threshold_respected() {
        let storage = in_memory_storage();
        let old_date = (Utc::now() - chrono::Duration::days(2)).to_rfc3339();
        let conn = storage.get_connection();

        conn.execute(
            "INSERT INTO project_files (file_path, parse_status, last_indexed_at) \
             VALUES (?1, ?2, ?3)",
            rusqlite::params!["src/main.rs", "OK", &old_date],
        )
        .unwrap();

        let result = check_index_staleness(&storage, 1);
        assert!(
            result.is_some(),
            "should be stale with threshold=1 day and age=2 days"
        );
    }

    #[test]
    fn staleness_check_empty_db() {
        let storage = in_memory_storage();
        // No project_files rows at all.
        let result = check_index_staleness(&storage, 3);
        assert!(result.is_none(), "empty DB should not warn (no data)");
    }

    #[test]
    fn staleness_check_clock_skew() {
        let storage = in_memory_storage();
        // future timestamp => clock skew, should not warn
        let future = (Utc::now() + chrono::Duration::days(1)).to_rfc3339();
        let conn = storage.get_connection();
        conn.execute(
            "INSERT INTO project_files (file_path, parse_status, last_indexed_at) \
             VALUES (?1, ?2, ?3)",
            rusqlite::params!["src/main.rs", "OK", &future],
        )
        .unwrap();

        let result = check_index_staleness(&storage, 3);
        assert!(result.is_none(), "clock skew should not trigger staleness");
    }

    #[test]
    fn warn_if_stale_prints_when_stale() {
        let storage = in_memory_storage();
        let old_date = (Utc::now() - chrono::Duration::days(10)).to_rfc3339();
        let conn = storage.get_connection();
        conn.execute(
            "INSERT INTO project_files (file_path, parse_status, last_indexed_at) \
             VALUES (?1, ?2, ?3)",
            rusqlite::params!["src/main.rs", "OK", &old_date],
        )
        .unwrap();

        // Capture stderr
        let result = warn_if_stale(&storage, 3);
        assert!(result, "warn_if_stale should return true when stale");
    }
}
