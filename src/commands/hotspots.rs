use crate::config::load_config;
use crate::git::repo::open_repo;
use crate::impact::temporal::GixHistoryProvider;
use crate::index::warn_if_stale;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use std::env;

#[allow(clippy::too_many_arguments)]
pub fn execute_hotspots(
    limit: usize,
    commits: usize,
    days: Option<u64>,
    since: Option<String>,
    json: bool,
    dir: Option<String>,
    lang: Option<String>,
    all_parents: bool,
    centrality: bool,
    semantic: bool,
) -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let repo = open_repo(&current_dir)?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let storage = StorageManager::open_read_only(&layout.root)?;

    // --- Staleness check ---
    let config = load_config(&layout).unwrap_or_default();
    let threshold_days = config.index.stale_threshold_days;
    let _ = warn_if_stale(&storage, threshold_days);

    if semantic {
        let cozo = storage
            .cozo
            .as_ref()
            .ok_or_else(|| miette::miette!("CozoDB storage not initialized"))?;

        if !json {
            println!("Analyzing semantic similarity hotspots (duplication)...");
        }

        let matches = crate::semantic::hotspots::find_semantic_hotspots(cozo, 0.85)?;

        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&matches).into_diagnostic()?
            );
        } else {
            use owo_colors::OwoColorize;
            if matches.is_empty() {
                println!("No significant semantic duplication found.");
            } else {
                println!(
                    "\n{}",
                    "Semantic Duplication Hotspots (Similarity > 0.85):"
                        .bold()
                        .cyan()
                );
                for m in matches {
                    println!(
                        "- {} ({}:{}) <-> {} ({}:{}) [{:.2}% match]",
                        m.name1.bold(),
                        m.file1,
                        m.offset1,
                        m.name2.bold(),
                        m.file2,
                        m.offset2,
                        m.similarity * 100.0
                    );
                }
                println!();
            }
        }
        return Ok(());
    }

    if !json {
        let mut filters = Vec::new();
        filters.push(format!("limit: {}", limit));
        filters.push(format!("commits: {}", commits));
        if let Some(d) = days {
            filters.push(format!("days: {}", d));
        }
        if let Some(s) = &since {
            filters.push(format!("since: {}", s));
        }
        println!("Analyzing hotspots [{}]...", filters.join(", "));
    }

    // Resolve 'since' to a commit ID if provided
    let since_commit = if let Some(ref s) = since {
        Some(
            repo.find_reference(s)
                .map_err(|e| miette::miette!("Failed to find 'since' reference '{}': {}", s, e))?
                .id()
                .to_string(),
        )
    } else {
        None
    };

    let history_provider = GixHistoryProvider::new(&repo);
    let mut hotspots = crate::impact::hotspots::calculate_hotspots(
        &storage,
        &history_provider,
        commits,
        days,
        since_commit,
        limit,
        all_parents,
        config.hotspots.decay_half_life,
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
