use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use chrono::Utc;
use miette::Result;
use serde::{Deserialize, Serialize};

/// Warning emitted when the index has not been refreshed recently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalenessWarning {
    /// Approximate number of days since the last index operation completed.
    pub days_since_indexed: u64,
    /// Number of files whose content has changed since they were last indexed.
    pub stale_files: usize,
    /// Number of tracked files that have not been indexed yet.
    #[serde(default)]
    pub unindexed_files: usize,
    /// Sample paths that are stale.
    pub sample_paths: Vec<String>,
    /// Last successful index completion timestamp.
    pub last_indexed_at: Option<String>,
    /// Whether the index is completely missing (no storage found).
    #[serde(default)]
    pub is_missing: bool,
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
    storage: &StorageManager,
    threshold_days: u64,
) -> Option<StalenessWarning> {
    let conn = storage.get_connection();

    // Find the most recent last_indexed_at across all non-deleted files.
    let max_indexed: Option<String> = conn
        .query_row(
            "SELECT MAX(last_indexed_at) FROM project_files WHERE parse_status != 'DELETED'",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten();

    let last_indexed = match max_indexed {
        Some(ts) => match chrono::DateTime::parse_from_rfc3339(&ts) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return None,
        },
        // No files indexed yet => stale.
        None => {
            return Some(StalenessWarning {
                days_since_indexed: 999, // Very stale
                stale_files: 0,
                unindexed_files: 0,
                sample_paths: Vec::new(),
                last_indexed_at: None,
                is_missing: true,
            });
        }
    };

    let now = Utc::now();
    let days = (now - last_indexed).num_days();

    if days < 0 {
        // Clock skew; treat as fresh.
        return None;
    }

    let days_since_indexed = days as u64;

    if days_since_indexed <= threshold_days {
        return None;
    }

    // Count total indexed (non-deleted) files — these are the files affected
    // by the stale index.
    let stale_files: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM project_files WHERE parse_status != 'DELETED'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .ok()
        .unwrap_or(0) as usize;

    // Get some sample stale paths for context
    let mut stmt = conn
        .prepare("SELECT file_path FROM project_files WHERE parse_status != 'DELETED' LIMIT 3")
        .ok();
    let sample_paths = if let Some(ref mut stmt) = stmt {
        stmt.query_map([], |row| row.get::<_, String>(0))
            .ok()
            .map(|iter| iter.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    Some(StalenessWarning {
        days_since_indexed,
        stale_files,
        sample_paths,
        last_indexed_at: Some(last_indexed.to_rfc3339()),
        is_missing: false,
        unindexed_files: 0,
    })
}

/// Emit the staleness warning to stderr so it does not interfere with --json
/// output on stdout.
pub fn print_staleness_warning(warning: &StalenessWarning) {
    use owo_colors::OwoColorize;

    eprintln!(
        "\n{} [STALE] Index is {} day{} old with {} indexed file{} and {} unindexed file{}.",
        "WARN".yellow().bold(),
        warning.days_since_indexed,
        if warning.days_since_indexed == 1 {
            ""
        } else {
            "s"
        },
        warning.stale_files,
        if warning.stale_files == 1 { "" } else { "s" },
        warning.unindexed_files,
        if warning.unindexed_files == 1 {
            ""
        } else {
            "s"
        },
    );

    if !warning.sample_paths.is_empty() {
        eprintln!(
            "  Sample paths: {}",
            warning.sample_paths.join(", ").dimmed()
        );
    }

    eprintln!(
        "  {} Results may be degraded. Run {} to refresh.",
        "➜".blue(),
        "changeguard index --incremental".cyan().bold()
    );
}

/// Check whether the CHANGEGUARD_NON_INTERACTIVE env var is set.
/// When non-empty, interactive prompts (e.g. inquire confirmations) should be skipped.
pub fn is_non_interactive() -> bool {
    std::env::var("CHANGEGUARD_NON_INTERACTIVE")
        .ok()
        .is_some_and(|v| !v.is_empty())
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

/// Run an incremental index if the current index is stale.
/// Returns the (possibly re-opened) StorageManager.
pub fn try_auto_index(storage: StorageManager, threshold_days: u64) -> Result<StorageManager> {
    if let Some(warning) = check_index_staleness(&storage, threshold_days) {
        use crate::index::ProjectIndexer;
        use owo_colors::OwoColorize;

        eprintln!(
            "{} Index is stale ({} days old). Running auto-index...",
            "INFO".blue().bold(),
            warning.days_since_indexed
        );

        let root = storage.root().to_path_buf();

        // StorageManager::init handles write-mode and migrations
        let write_storage = StorageManager::init(
            Layout::new(&root)
                .state_subdir()
                .join("ledger.db")
                .as_std_path(),
        )?;

        use crate::config::model::Config;
        let mut indexer = ProjectIndexer::new(write_storage, root.clone(), Config::default());
        indexer.incremental_index()?;

        // Re-open in read-only mode for the caller
        StorageManager::open_read_only(&root)
    } else {
        Ok(storage)
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
        assert!(
            result.is_some(),
            "empty DB should warn as stale to trigger initial index"
        );
        assert_eq!(result.unwrap().days_since_indexed, 999);
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
