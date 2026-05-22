use crate::cli::HotspotArgs;
use crate::config::load_config;
use crate::git::repo::open_repo;
use crate::impact::hotspots::{HotspotQuery, calculate_hotspots};
use crate::impact::temporal::GixHistoryProvider;
use crate::index::warn_if_stale;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use std::env;

impl From<HotspotArgs> for HotspotQuery {
    fn from(args: HotspotArgs) -> Self {
        Self {
            commits: args.commits,
            days: args.days,
            since_commit: None, // Resolved later
            limit: args.limit,
            all_parents: args.all_parents,
            decay_half_life: 0, // Resolved from config
            dir_filter: args.dir,
            lang_filter: args.lang,
        }
    }
}

pub fn execute_hotspots(args: HotspotArgs) -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let repo = open_repo(&current_dir)?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let storage = if args.semantic {
        StorageManager::open_read_only(&layout.root)?
    } else {
        StorageManager::open_read_only_sqlite_only(&layout.root)?
    };

    // --- Staleness check ---
    let config = load_config(&layout).unwrap_or_default();
    let threshold_days = config.index.stale_threshold_days;
    let storage = if args.auto_index {
        crate::index::staleness::try_auto_index(storage, threshold_days)?
    } else {
        let _ = warn_if_stale(&storage, threshold_days);
        storage
    };

    if args.semantic {
        let cozo = storage
            .cozo
            .as_ref()
            .ok_or_else(|| miette::miette!("CozoDB storage not initialized"))?;

        if !args.json {
            println!("Analyzing semantic similarity hotspots (duplication)...");
        }

        let matches = crate::semantic::hotspots::find_semantic_hotspots(cozo, 0.85)?;

        if args.json {
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

    if !args.json {
        let mut filters = Vec::new();
        filters.push(format!("limit: {}", args.limit));
        filters.push(format!("commits: {}", args.commits));
        if let Some(d) = args.days {
            filters.push(format!("days: {}", d));
        }
        if let Some(s) = &args.since {
            filters.push(format!("since: {}", s));
        }
        println!("Analyzing hotspots [{}]...", filters.join(", "));
    }

    // Resolve 'since' to a commit ID if provided
    let since_commit = if let Some(ref s) = args.since {
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
    let mut query = HotspotQuery::from(args.clone());
    query.since_commit = since_commit;
    query.decay_half_life = config.hotspots.decay_half_life;

    let mut hotspots = calculate_hotspots(&storage, &history_provider, &query)?;

    // Enrich with centrality data if requested
    if args.centrality {
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

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&hotspots).into_diagnostic()?
        );
    } else if args.centrality {
        crate::output::human::print_hotspots_table_with_centrality(&hotspots);
    } else {
        crate::output::human::print_hotspots_table(&hotspots);
    }

    Ok(())
}
