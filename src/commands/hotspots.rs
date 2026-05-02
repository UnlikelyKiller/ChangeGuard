use crate::git::repo::open_repo;
use crate::impact::temporal::GixHistoryProvider;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use std::env;

pub fn execute_hotspots(
    limit: usize,
    commits: usize,
    json: bool,
    dir: Option<String>,
    lang: Option<String>,
    all_parents: bool,
    centrality: bool,
) -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let repo = open_repo(&current_dir)?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    if !json {
        println!("Analyzing {} commits for temporal hotspots...", commits);
    }

    let history_provider = GixHistoryProvider::new(&repo);
    let mut hotspots = crate::impact::hotspots::calculate_hotspots(
        &storage,
        &history_provider,
        commits,
        limit,
        all_parents,
        dir.as_deref(),
        lang.as_deref(),
    )?;

    // Enrich with centrality data if requested
    if centrality {
        let conn = storage.get_connection();
        let has_centrality: bool = match conn.query_row(
            "SELECT count(*) FROM symbol_centrality LIMIT 1",
            [],
            |row| row.get::<_, i64>(0),
        ) {
            Ok(count) => count > 0,
            Err(_) => false,
        };

        if has_centrality {
            for hotspot in &mut hotspots {
                let path_str = hotspot.path.to_string_lossy();
                let max_reachable: Option<i64> = conn
                    .query_row(
                        "SELECT MAX(sc.entrypoints_reachable)
                         FROM symbol_centrality sc
                         JOIN project_symbols ps ON sc.symbol_id = ps.id
                         JOIN project_files pf ON ps.file_id = pf.id
                         WHERE pf.file_path = ?1",
                        [&*path_str],
                        |row| row.get(0),
                    )
                    .ok()
                    .flatten();
                hotspot.centrality = max_reachable.map(|v| v as usize);
            }
        }
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&hotspots).into_diagnostic()?
        );
    } else if centrality {
        crate::output::human::print_hotspots_table_with_centrality(&hotspots);
    } else {
        crate::output::human::print_hotspots_table(&hotspots);
    }

    Ok(())
}
