use crate::bridge::model::{BridgeDirection, BridgePayload, BridgeRecord, Privacy};
use crate::config::load_config;
use crate::index::{ProjectIndexer, warn_if_stale};
use crate::search::{RegexFilter, StreamIndexer, TantivySearchEngine};
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;
use tracing::info;

#[allow(clippy::too_many_arguments)]
pub fn execute_search(
    query: String,
    regex: bool,
    semantic: bool,
    limit: usize,
    index: bool,
    json: bool,
    auto_index: bool,
) -> Result<()> {
    let root = get_repo_root()?;
    let layout = Layout::new(&root);
    layout.ensure_state_dir()?;
    let project_id = layout.get_project_id();

    // --- Staleness check (applies to both semantic and BM25 paths) ---
    // When --index is used, skip staleness check (full re-index supersedes it).
    // Otherwise use read-only fast-path, gracefully skipping if DB not initialized.
    if !index {
        let config = load_config(&layout)?;
        if let Ok(storage) = StorageManager::open_read_only(&layout.root) {
            let threshold = config.index.stale_threshold_days;
            if auto_index {
                if let Err(e) = run_incremental_index(&layout, &storage) {
                    tracing::warn!("incremental index failed, proceeding with current index: {e}");
                }
                // Re-open storage after index mutation
                let _ = StorageManager::open_read_only(&layout.root)?;
            } else {
                let _ = warn_if_stale(&storage, threshold);
            }
        }
    }

    if semantic {
        let config = load_config(&layout)?;
        let storage = StorageManager::open_read_only(&layout.root)?;
        let cozo = storage
            .cozo
            .as_ref()
            .ok_or_else(|| miette::miette!("CozoDB storage not initialized"))?;

        let semantic_engine =
            crate::semantic::SemanticDiscovery::new(config.local_model.clone(), cozo)?;

        info!("Performing semantic search for: {}", query);
        let results = semantic_engine.query(&query, limit)?;

        if results.is_empty() {
            if !json {
                println!("No relevant code snippets found.");
            }
        } else if json {
            for (path, name, offset, dist) in results {
                let record = BridgeRecord {
                    bridge_version: BridgeRecord::VERSION.to_string(),
                    direction: BridgeDirection::Outbound,
                    timestamp: chrono::Utc::now(),
                    parent_hash: None,
                    project_id: project_id.clone(),
                    session_id: None,
                    tx_id: None,
                    record_kind: "insight".to_string(),
                    payload: BridgePayload::Insight {
                        memory_id: format!("{}::{}", path, name),
                        relevance: 1.0 - dist as f64,
                        content: format!("{} (offset {}, dist {:.4})", name, offset, dist),
                    },
                    privacy: Privacy::ProjectLocal,
                };
                println!("{}", serde_json::to_string(&record).unwrap_or_default());
            }
        } else {
            println!("\n{}", "Semantic Search Results:".bold().cyan());
            for (path, name, offset, dist) in results {
                println!(
                    "- {} ({} at offset {}) [dist: {:.4}]",
                    name.bold(),
                    path,
                    offset,
                    dist
                );
            }
            println!();
        }
        return Ok(());
    }

    let index_path = layout.search_index_dir();
    let engine = TantivySearchEngine::open_or_create(index_path.as_std_path())?;

    if index || is_index_empty(&index_path) {
        info!("Indexing repository for search...");
        engine.clear()?;
        let indexer = StreamIndexer::new(engine);
        indexer.index_repository(&root)?;
        // Re-open engine to pick up new index
        let engine = TantivySearchEngine::open_or_create(index_path.as_std_path())?;
        perform_search(engine, &root, query, regex, limit, json, &project_id)?;
    } else {
        perform_search(engine, &root, query, regex, limit, json, &project_id)?;
    }

    Ok(())
}

/// Run an incremental index against the SQLite/CozoDB project index.
fn run_incremental_index(layout: &Layout, _storage: &StorageManager) -> Result<()> {
    let repo_path = layout.root.clone();
    // We need an owned StorageManager — clone the connection by re-initializing.
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;
    let mut indexer = ProjectIndexer::new(storage, repo_path);
    let _stats = indexer.incremental_index()?;
    Ok(())
}

fn perform_search(
    engine: TantivySearchEngine,
    root: &camino::Utf8Path,
    query: String,
    regex: bool,
    limit: usize,
    json: bool,
    project_id: &str,
) -> Result<()> {
    if regex {
        let filter = RegexFilter::new(&engine);
        let matches = filter.search(root, &query, limit)?;
        if matches.is_empty() {
            if !json {
                println!("No matches found.");
            }
        } else if json {
            for m in matches {
                let record = BridgeRecord {
                    bridge_version: BridgeRecord::VERSION.to_string(),
                    direction: BridgeDirection::Outbound,
                    timestamp: chrono::Utc::now(),
                    parent_hash: None,
                    project_id: project_id.to_string(),
                    session_id: None,
                    tx_id: None,
                    record_kind: "insight".to_string(),
                    payload: BridgePayload::Insight {
                        memory_id: format!("{}:{}", m.path, m.line_number),
                        relevance: 1.0,
                        content: m.content,
                    },
                    privacy: Privacy::ProjectLocal,
                };
                println!("{}", serde_json::to_string(&record).unwrap_or_default());
            }
        } else {
            for m in matches {
                println!("{}:{}: {}", m.path, m.line_number, m.content);
            }
        }
    } else {
        let results = engine.search(&query, limit)?;
        if results.is_empty() {
            if !json {
                println!("No matches found.");
            }
        } else if json {
            for r in results {
                let record = BridgeRecord {
                    bridge_version: BridgeRecord::VERSION.to_string(),
                    direction: BridgeDirection::Outbound,
                    timestamp: chrono::Utc::now(),
                    parent_hash: None,
                    project_id: project_id.to_string(),
                    session_id: None,
                    tx_id: None,
                    record_kind: "insight".to_string(),
                    payload: BridgePayload::Insight {
                        memory_id: r.path.clone(),
                        relevance: r.score as f64,
                        content: r.path,
                    },
                    privacy: Privacy::ProjectLocal,
                };
                println!("{}", serde_json::to_string(&record).unwrap_or_default());
            }
        } else {
            for r in results {
                println!("{} (score: {:.2})", r.path, r.score);
            }
        }
    }
    Ok(())
}

fn get_repo_root() -> Result<Utf8PathBuf> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let discovered = gix::discover(&current_dir).into_diagnostic()?;
    let root = discovered
        .workdir()
        .ok_or_else(|| miette::miette!("Failed to find work directory for repository"))?;

    Utf8PathBuf::from_path_buf(root.to_path_buf())
        .map_err(|_| miette::miette!("Repository root is not valid UTF-8"))
}

fn is_index_empty(path: &camino::Utf8Path) -> bool {
    if !path.exists() {
        return true;
    }
    std::fs::read_dir(path)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(true)
}
