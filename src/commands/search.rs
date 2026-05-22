use crate::bridge::model::{BridgeDirection, BridgePayload, BridgeRecord, Privacy};
use crate::commands::helpers::{get_layout, get_repo_root};
use crate::config::load_config;
use crate::index::warn_if_stale;
use crate::search::{RegexFilter, StreamIndexer, TantivySearchEngine};
use crate::state::storage::StorageManager;
use camino::Utf8Path;
use miette::Result;
use owo_colors::OwoColorize;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct SearchArgs {
    pub query: String,
    pub regex: bool,
    pub semantic: bool,
    pub limit: usize,
    pub index: bool,
    pub json: bool,
    pub auto_index: bool,
    pub project_id: String,
}

pub fn execute_search(args: SearchArgs) -> Result<()> {
    let layout = get_layout()?;
    let _root = get_repo_root()?;

    // --- Staleness check (applies to both semantic and BM25 paths) ---
    if !args.index {
        let config = load_config(&layout)?;
        let storage_opt = StorageManager::open_read_only(&layout.root).ok();

        if let Some(storage) = storage_opt {
            let threshold = config.index.stale_threshold_days;
            if args.auto_index {
                let _ = crate::index::staleness::try_auto_index(storage, threshold);
            } else {
                let _ = warn_if_stale(&storage, threshold);
            }
        }
    }

    if args.semantic {
        let config = load_config(&layout)?;
        let storage = StorageManager::open_read_only(&layout.root)?;
        let cozo = storage
            .cozo
            .as_ref()
            .ok_or_else(|| miette::miette!("CozoDB storage not initialized"))?;

        let semantic_engine =
            crate::semantic::SemanticDiscovery::new(config.local_model.clone(), cozo)?;

        // --- Phase 1: Readiness Check ---
        let readiness = semantic_engine.check_readiness()?;
        if args.json {
            let record = BridgeRecord {
                bridge_version: BridgeRecord::VERSION.to_string(),
                direction: BridgeDirection::Outbound,
                timestamp: chrono::Utc::now(),
                parent_hash: None,
                project_id: args.project_id.clone(),
                session_id: None,
                tx_id: None,
                record_kind: "semantic_readiness".to_string(),
                payload: BridgePayload::Insight {
                    memory_id: "readiness".to_string(),
                    relevance: 1.0,
                    content: serde_json::to_string(&readiness).unwrap_or_default(),
                },
                privacy: Privacy::ProjectLocal,
            };
            println!("{}", serde_json::to_string(&record).unwrap_or_default());
        } else {
            if !readiness.endpoint_available {
                println!(
                    "{} Local embedding endpoint unreachable. Check your model server.",
                    "WARN".yellow().bold()
                );
            }
            if readiness.vector_count == 0 {
                println!(
                    "{} Semantic index is empty. Run {} to populate.",
                    "WARN".yellow().bold(),
                    "changeguard index --semantic".cyan().bold()
                );
            }
            if readiness.dimension_mismatch {
                println!(
                    "{} Model/Index dimension mismatch ({} vs {}). Run {} to fix.",
                    "ERROR".red().bold(),
                    readiness.model_name,
                    readiness.dimensions,
                    "changeguard update --migrate".cyan().bold()
                );
            }
        }

        debug!("Performing semantic search for: {}", args.query);
        let results = semantic_engine.query(&args.query, args.limit)?;

        if !results.is_empty() {
            if args.json {
                for (path, name, offset, dist) in results {
                    let record = BridgeRecord {
                        bridge_version: BridgeRecord::VERSION.to_string(),
                        direction: BridgeDirection::Outbound,
                        timestamp: chrono::Utc::now(),
                        parent_hash: None,
                        project_id: args.project_id.clone(),
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

        if !args.json {
            println!(
                "{} No relevant code snippets found semantically. Falling back to keyword search...",
                "INFO".blue().bold()
            );
        }
    }

    let index_path = layout.search_index_dir();
    let engine = TantivySearchEngine::open_or_create(index_path.as_std_path())?;

    if args.index || engine.document_count() == 0 {
        debug!("Indexing repository for search...");
        {
            engine.clear()?;
            let indexer = StreamIndexer::new(engine);
            indexer.index_repository(&layout.root)?;
        }

        let engine = TantivySearchEngine::open_or_create(index_path.as_std_path())?;
        engine.verify_index_integrity(index_path.as_std_path())?;
        debug!("Tantivy index integrity verified.");

        perform_search(engine, &layout.root, &args)?;
    } else {
        perform_search(engine, &layout.root, &args)?;
    }

    Ok(())
}

fn perform_search(engine: TantivySearchEngine, root: &Utf8Path, args: &SearchArgs) -> Result<()> {
    if args.regex {
        let filter = RegexFilter::new(&engine);
        let matches = filter.search(root, &args.query, args.limit)?;

        if args.json {
            for m in matches {
                let record = BridgeRecord {
                    bridge_version: BridgeRecord::VERSION.to_string(),
                    direction: BridgeDirection::Outbound,
                    timestamp: chrono::Utc::now(),
                    parent_hash: None,
                    project_id: args.project_id.clone(),
                    session_id: None,
                    tx_id: None,
                    record_kind: "regex_match".to_string(),
                    payload: BridgePayload::Insight {
                        memory_id: format!("{}::{}", m.path, m.line_number),
                        relevance: 1.0,
                        content: format!("{}:{}: {}", m.path, m.line_number, m.content),
                    },
                    privacy: Privacy::ProjectLocal,
                };
                println!("{}", serde_json::to_string(&record).unwrap_or_default());
            }
        } else {
            println!("\n{}", "Regex Search Results:".bold().cyan());
            if matches.is_empty() {
                println!("No matches found.");
            } else {
                for m in matches {
                    println!(
                        "{}:{}: {}",
                        m.path.cyan(),
                        m.line_number.to_string().yellow(),
                        m.content.trim()
                    );
                }
            }
            println!();
        }
    } else {
        let results = engine.search(&args.query, args.limit)?;

        if args.json {
            for r in results {
                let record = BridgeRecord {
                    bridge_version: BridgeRecord::VERSION.to_string(),
                    direction: BridgeDirection::Outbound,
                    timestamp: chrono::Utc::now(),
                    parent_hash: None,
                    project_id: args.project_id.clone(),
                    session_id: None,
                    tx_id: None,
                    record_kind: "bm25_match".to_string(),
                    payload: BridgePayload::Insight {
                        memory_id: r.path.clone(),
                        relevance: r.score as f64,
                        content: format!("{} ({})", r.path, r.snippet.unwrap_or_default()),
                    },
                    privacy: Privacy::ProjectLocal,
                };
                println!("{}", serde_json::to_string(&record).unwrap_or_default());
            }
        } else {
            println!("\n{}", "Ranked Search Results (BM25):".bold().cyan());
            if results.is_empty() {
                println!("No matches found.");
            } else {
                for r in results {
                    let line_info = if let Some(line) = r.line_number {
                        format!(":{}", line.to_string().yellow())
                    } else {
                        String::new()
                    };
                    println!(
                        "{} [score: {:.2}]",
                        format!("{}{}", r.path.cyan(), line_info).bold(),
                        r.score.to_string().yellow()
                    );
                    if let Some(snippet) = r.highlighted {
                        println!("  {}", snippet.trim());
                    }
                }
            }
            println!();
        }
    }

    Ok(())
}

pub fn execute_search_trigrams(trigrams: Vec<String>, limit: usize) -> Result<()> {
    let layout = get_layout()?;
    let index_path = layout.search_index_dir();
    let engine = TantivySearchEngine::open_or_create(index_path.as_std_path())?;
    let results = engine.search_trigrams(&trigrams, limit)?;
    for path in results {
        println!("{path}");
    }
    Ok(())
}
