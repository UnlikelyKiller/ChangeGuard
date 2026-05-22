use crate::cli::HotspotArgs;
use crate::commands::helpers::get_layout;
use crate::config::load_config;
use crate::git::repo::open_repo;
use crate::impact::hotspots::{HotspotQuery, calculate_hotspots};
use crate::impact::temporal::GixHistoryProvider;
use crate::index::warn_if_stale;
use crate::state::storage::StorageManager;
use miette::Result;
use std::env;

pub fn execute_hotspots(args: HotspotArgs) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let repo = open_repo(&current_dir)?;
    let layout = get_layout()?;

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
            println!("{}", serde_json::to_string_pretty(&matches).unwrap());
        } else {
            crate::output::human::print_semantic_hotspots(&matches);
        }
        return Ok(());
    }

    let history_provider = GixHistoryProvider::new(&repo);
    let query = HotspotQuery {
        limit: args.limit.unwrap_or(config.hotspots.limit),
        commits: args.commits.unwrap_or(config.hotspots.max_commits),
        days: args.days.map(|d| d as u64),
        decay_half_life: config.hotspots.decay_half_life,
        dir_filter: args.entity,
        ..Default::default()
    };

    let mut hotspots = calculate_hotspots(&storage, &history_provider, &query)?;

    if args.centrality {
        let cozo = storage
            .cozo
            .as_ref()
            .ok_or_else(|| miette::miette!("CozoDB storage not initialized"))?;
        crate::index::centrality::enrich_hotspots_with_centrality(&mut hotspots, cozo)?;
    }

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&hotspots).map_err(|e| miette::miette!("{}", e))?
        );
    } else if args.centrality {
        crate::output::human::print_hotspots_table_with_centrality(&hotspots);
    } else {
        crate::output::human::print_hotspots_table(&hotspots);
    }

    Ok(())
}
