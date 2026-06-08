use crate::cli::{HotspotArgs, HotspotSubcommands};
use crate::commands::helpers::get_layout;
use crate::config::load_config;
use crate::git::repo::open_repo;
use crate::impact::hotspots::{HotspotQuery, calculate_hotspots};
use crate::impact::temporal::{GixHistoryProvider, TemporalEngine};
use crate::index::warn_if_stale;
use crate::state::storage::StorageManager;
use chrono::Utc;
use miette::{IntoDiagnostic, Result};
use std::env;
use owo_colors::OwoColorize;

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

    if let Some(command) = args.command {
        match command {
            HotspotSubcommands::Trend {
                entity,
                days,
                json,
            } => {
                return execute_hotspots_trend(&storage, entity, days, json);
            }
            HotspotSubcommands::Explain { entity } => {
                return execute_hotspots_explain(&storage, entity, &repo);
            }
            HotspotSubcommands::Budget { json } => {
                return execute_hotspots_budget(&storage, &config, json);
            }
        }
    }

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
        dir_filter: args.entity.clone(),
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

    if args.snapshot {
        persist_hotspots_and_couplings(&storage, &repo, &hotspots, &config)?;
        if !args.json {
            println!("Hotspot and temporal coupling snapshot persisted to SQLite.");
        }
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

fn persist_hotspots_and_couplings(
    storage: &StorageManager,
    repo: &gix::Repository,
    hotspots: &[crate::impact::packet::Hotspot],
    config: &crate::config::model::Config,
) -> Result<()> {
    let conn = storage.get_connection();
    let timestamp = Utc::now().to_rfc3339();

    let snapshot_id: Option<i64> = conn
        .query_row("SELECT id FROM snapshots ORDER BY id DESC LIMIT 1", [], |row| {
            row.get(0)
        })
        .ok();

    // Insert Hotspots
    for hotspot in hotspots {
        conn.execute(
            "INSERT INTO hotspot_history (snapshot_id, file_path, score, display_score, complexity, frequency, centrality, timestamp) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                snapshot_id,
                hotspot.path.to_string_lossy().to_string(),
                hotspot.score,
                hotspot.display_score,
                hotspot.complexity,
                hotspot.frequency,
                hotspot.centrality.map(|c| c as i64),
                timestamp
            ],
        ).into_diagnostic()?;
    }

    // Calculate and Insert Temporal Couplings
    let history_provider = GixHistoryProvider::new(repo);
    let engine = TemporalEngine::new(history_provider, config.temporal.clone());
    let couplings = engine
        .calculate_couplings()
        .map_err(|e| miette::miette!("Failed to calculate temporal couplings: {}", e))?;

    for coupling in couplings {
        conn.execute(
            "INSERT INTO temporal_coupling_history (snapshot_id, file_a, file_b, score, timestamp) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                snapshot_id,
                coupling.file_a.to_string_lossy().to_string(),
                coupling.file_b.to_string_lossy().to_string(),
                coupling.score,
                timestamp
            ],
        ).into_diagnostic()?;
    }

    Ok(())
}

fn execute_hotspots_trend(
    storage: &StorageManager,
    entity: Option<String>,
    days: u32,
    json: bool,
) -> Result<()> {
    let conn = storage.get_connection();
    let cutoff = Utc::now() - chrono::Duration::days(days as i64);
    let cutoff_str = cutoff.to_rfc3339();

    let rows: Vec<(String, String, f64)> = if let Some(ref path) = entity {
        let mut stmt = conn.prepare(
            "SELECT file_path, timestamp, score FROM hotspot_history \
             WHERE timestamp >= ?1 AND file_path = ?2 \
             ORDER BY timestamp ASC",
        ).into_diagnostic()?;
        stmt.query_map(rusqlite::params![&cutoff_str, path], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).into_diagnostic()?.collect::<rusqlite::Result<Vec<_>>>().into_diagnostic()?
    } else {
        let mut stmt = conn.prepare(
            "SELECT file_path, timestamp, score FROM hotspot_history \
             WHERE timestamp >= ?1 \
             ORDER BY timestamp ASC",
        ).into_diagnostic()?;
        stmt.query_map([&cutoff_str], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).into_diagnostic()?.collect::<rusqlite::Result<Vec<_>>>().into_diagnostic()?
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&rows).unwrap());
    } else {
        println!("Hotspot Trends (Last {} days):", days);
        if rows.is_empty() {
            println!("  No trend data available. Run 'hotspots --snapshot' to start tracking.");
        } else {
            for (path, ts, score) in rows {
                println!("  {} | {} | Score: {:.4}", ts, path, score);
            }
        }
    }

    Ok(())
}

fn execute_hotspots_explain(
    storage: &StorageManager,
    entity: String,
    repo: &gix::Repository,
) -> Result<()> {
    println!("Explaining hotspots for: {}", entity);

    // 1. Complexity factor
    let conn = storage.get_connection();
    let complexity: i32 = conn.query_row(
        "SELECT MAX(IFNULL(cognitive_complexity, 0), IFNULL(cyclomatic_complexity, 0)) \
         FROM project_symbols ps JOIN project_files pf ON ps.file_id = pf.id WHERE pf.file_path = ?1",
        [&entity],
        |row| row.get(0)
    ).unwrap_or(0);

    // 2. Frequency factor
    let history_provider = GixHistoryProvider::new(repo);
    let query = HotspotQuery {
        dir_filter: Some(entity.clone()),
        ..Default::default()
    };
    let hotspots = calculate_hotspots(storage, &history_provider, &query)?;
    let frequency = hotspots.first().map(|h| h.frequency).unwrap_or(0.0);

    // 3. Temporal couplings
    let config = load_config(&get_layout()?).unwrap_or_default();
    let engine = TemporalEngine::new(history_provider, config.temporal.clone());
    let couplings = engine.calculate_couplings().unwrap_or_default();
    let entity_couplings: Vec<_> = couplings
        .into_iter()
        .filter(|c| {
            c.file_a.to_string_lossy() == entity || c.file_b.to_string_lossy() == entity
        })
        .collect();

    println!("\nMetrics:");
    println!("  Complexity: {}", complexity);
    println!("  Change Frequency (weighted): {:.2}", frequency);
    println!("  Temporal Couplings: {}", entity_couplings.len());

    if !entity_couplings.is_empty() {
        println!("\nTop Couplings:");
        for c in entity_couplings.iter().take(5) {
            let other = if c.file_a.to_string_lossy() == entity {
                &c.file_b
            } else {
                &c.file_a
            };
            println!("  {:<40} | Score: {:.2}", other.to_string_lossy(), c.score);
        }
    }

    Ok(())
}

fn execute_hotspots_budget(
    storage: &StorageManager,
    _config: &crate::config::model::Config,
    json: bool,
) -> Result<()> {
    let conn = storage.get_connection();
    
    let mut stmt = conn.prepare(
        "SELECT file_path, score FROM hotspot_history \
         WHERE timestamp = (SELECT MAX(timestamp) FROM hotspot_history) \
         ORDER BY score DESC"
    ).into_diagnostic()?;

    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    }).into_diagnostic()?;

    let mut violations = Vec::new();
    let threshold = 5.0; 

    for row in rows {
        let (path, score) = row.into_diagnostic()?;
        if score > threshold {
            violations.push(serde_json::json!({
                "path": path,
                "score": score,
                "threshold": threshold,
            }));
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "status": if violations.is_empty() { "OK" } else { "VIOLATION" },
            "violations": violations,
        })).unwrap());
    } else {
        println!("{}", "Hotspot Budget Check".bold().cyan());
        if violations.is_empty() {
            println!("  Status: {}", "OK".green());
            println!("  All hotspots within risk budget.");
        } else {
            println!("  Status: {}", "VIOLATION".red().bold());
            for v in &violations {
                println!("  ! {} exceeds budget: {:.2} > {:.2}", 
                    v["path"].as_str().unwrap().yellow(),
                    v["score"].as_f64().unwrap(),
                    v["threshold"].as_f64().unwrap()
                );
            }
        }
    }

    Ok(())
}
