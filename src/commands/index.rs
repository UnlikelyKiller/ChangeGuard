use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::env;

use crate::config::load::load_config;
use crate::docs::index::run_docs_index;
use crate::index::{ProjectIndexer, ServiceIndexStats};
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use tracing::{info, warn};

fn get_repo_root() -> Result<Utf8PathBuf> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let discovered = gix::discover(&current_dir).into_diagnostic()?;
    let root = discovered
        .workdir()
        .ok_or_else(|| miette::miette!("Failed to find work directory for repository"))?;

    Utf8PathBuf::from_path_buf(root.to_path_buf())
        .map_err(|_| miette::miette!("Repository root is not valid UTF-8"))
}

fn get_layout() -> Result<Layout> {
    let root = get_repo_root()?;
    Ok(Layout::new(root))
}

#[derive(Default)]
pub struct IndexArgs {
    pub incremental: bool,
    pub check: bool,
    pub strict: bool,
    pub json: bool,
    pub analyze_graph: bool,
    pub docs: bool,
    pub contracts: bool,
    pub semantic: bool,
    pub scip: Option<std::path::PathBuf>,
    pub export_docs: bool,
    pub doc_type: Option<String>,
    /// CLI override for rayon thread count (HP2). `None` = use config or rayon default.
    pub concurrency: Option<usize>,
}

pub fn execute_index(args: IndexArgs) -> Result<()> {
    let layout = get_layout()?;
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;
    let repo_path = layout.root.clone();
    let config = load_config(&layout).unwrap_or_else(|err| {
        warn!("Failed to load config: {err}. Using defaults.");
        crate::config::model::Config::default()
    });

    if let Some(scip_path) = args.scip {
        return execute_scip_index(&layout, storage, scip_path);
    }

    if args.semantic {
        return execute_semantic_index(
            &layout,
            storage,
            &config,
            args.incremental,
            args.concurrency,
        );
    }

    if args.docs {
        if !args.analyze_graph {
            return execute_docs_index(&layout, &storage);
        }
        execute_docs_index(&layout, &storage)?;
    }

    let contracts_db_path = if args.contracts {
        Some(db_path.clone())
    } else {
        None
    };

    let mut indexer = ProjectIndexer::new(storage, repo_path);

    if args.check {
        let status = indexer.check_status()?;
        let discovered = indexer.discover_files()?;
        let is_missing = status.total_files == 0 && !discovered.is_empty();

        if args.json {
            let output = serde_json::to_string_pretty(&status).into_diagnostic()?;
            println!("{}", output);
        } else {
            if is_missing {
                eprintln!("Error: Index is missing or empty. Run 'changeguard index' to build it.");
            } else if status.stale_files > 0 {
                if args.strict {
                    eprintln!(
                        "Error: Index is stale ({} files) and --strict is enabled.",
                        status.stale_files
                    );
                } else {
                    println!(
                        "Warning: Index is stale ({} files). Run 'changeguard index --incremental' to update.",
                        status.stale_files
                    );
                }
            } else {
                println!("Index is up to date.");
            }

            println!("Index Status:");
            println!("  Files indexed:   {}", status.total_files);
            println!("  Symbols indexed: {}", status.total_symbols);
            println!("  Stale files:     {}", status.stale_files);
            if let Some(last) = &status.last_indexed_at {
                println!("  Last indexed:    {}", last);
            } else {
                println!("  Last indexed:     never");
            }
        }

        if is_missing {
            std::process::exit(1);
        }
        if status.stale_files > 0 && args.strict {
            std::process::exit(1);
        }
        return Ok(());
    }

    let stats = if args.incremental {
        indexer.incremental_index()?
    } else {
        indexer.full_index()?
    };

    // Index documentation files
    let doc_stats = indexer.index_docs()?;

    // Index directory topology
    let topo_stats = indexer.index_topology()?;

    // Classify entry points
    let ep_stats = indexer.classify_entrypoints()?;

    // Build call graph
    let cg_stats = indexer.build_call_graph()?;

    // Extract API routes
    let route_stats = indexer.extract_routes()?;

    // Extract data models
    let dm_stats = indexer.extract_data_models()?;

    // Extract observability patterns
    let obs_stats = indexer.extract_observability()?;

    // Extract test-to-symbol mappings
    let tm_stats = indexer.extract_test_mappings()?;

    // Extract CI/CD workflow gates
    let ci_stats = indexer.extract_ci_gates()?;

    // Extract env schema (declarations and references)
    let env_stats = indexer.extract_env_schema()?;

    // Infer service boundaries
    let service_stats = if config.coverage.enabled && config.coverage.services.enabled {
        indexer.infer_services()?
    } else {
        info!("Service inference disabled by coverage.services config.");
        ServiceIndexStats {
            services_inferred: 0,
            files_assigned: 0,
        }
    };

    // Compute centrality if requested
    let cent_stats = if args.analyze_graph {
        indexer.build_kg_native(&config.local_model)?;
        indexer.compute_centrality()?
    } else {
        info!("Centrality computation skipped (use --analyze-graph to enable).");
        crate::index::centrality::CentralityStats {
            entry_points_count: 0,
            symbols_computed: 0,
            max_reachable: 0,
        }
    };

    let contracts_summary: Option<crate::contracts::index::ContractsIndexSummary> =
        if let Some(ref db_path) = contracts_db_path {
            Some(execute_contracts_index(&layout, db_path.as_std_path())?)
        } else {
            None
        };

    // Update Tantivy search index (full-text search)
    // This ensures 'changeguard index' builds everything.
    let index_path = layout.search_index_dir();
    {
        let engine = crate::search::TantivySearchEngine::open_or_create(index_path.as_std_path())?;
        engine.clear()?;
        let stream_indexer = crate::search::StreamIndexer::new(engine);
        stream_indexer.index_repository(&layout.root)?;
    }

    // Verify search index integrity on disk
    let engine = crate::search::TantivySearchEngine::open_or_create(index_path.as_std_path())?;
    engine.verify_index_integrity(index_path.as_std_path())?;

    if args.json {
        let mut output = serde_json::to_value(&stats).into_diagnostic()?;
        let doc_obj = serde_json::to_value(&doc_stats).into_diagnostic()?;
        let topo_obj = serde_json::to_value(&topo_stats).into_diagnostic()?;
        let ep_obj = serde_json::to_value(&ep_stats).into_diagnostic()?;
        let service_obj = serde_json::to_value(&service_stats).into_diagnostic()?;
        if let (Some(map), Some(doc)) = (output.as_object_mut(), doc_obj.as_object()) {
            for (k, v) in doc {
                map.insert(format!("doc_{}", k), v.clone());
            }
        }
        if let (Some(map), Some(topo)) = (output.as_object_mut(), topo_obj.as_object()) {
            for (k, v) in topo {
                map.insert(format!("topo_{}", k), v.clone());
            }
        }
        if let (Some(map), Some(ep)) = (output.as_object_mut(), ep_obj.as_object()) {
            for (k, v) in ep {
                map.insert(format!("ep_{}", k), v.clone());
            }
        }
        if let (Some(map), Some(svc)) = (output.as_object_mut(), service_obj.as_object()) {
            for (k, v) in svc {
                map.insert(format!("service_{}", k), v.clone());
            }
        }
        let cg_obj = serde_json::to_value(&cg_stats).into_diagnostic()?;
        if let (Some(map), Some(cg)) = (output.as_object_mut(), cg_obj.as_object()) {
            for (k, v) in cg {
                map.insert(format!("cg_{}", k), v.clone());
            }
        }
        let route_obj = serde_json::to_value(&route_stats).into_diagnostic()?;
        if let (Some(map), Some(route)) = (output.as_object_mut(), route_obj.as_object()) {
            for (k, v) in route {
                map.insert(format!("route_{}", k), v.clone());
            }
        }
        let dm_obj = serde_json::to_value(&dm_stats).into_diagnostic()?;
        if let (Some(map), Some(dm)) = (output.as_object_mut(), dm_obj.as_object()) {
            for (k, v) in dm {
                map.insert(format!("dm_{}", k), v.clone());
            }
        }
        let obs_obj = serde_json::to_value(&obs_stats).into_diagnostic()?;
        if let (Some(map), Some(obs)) = (output.as_object_mut(), obs_obj.as_object()) {
            for (k, v) in obs {
                map.insert(format!("obs_{}", k), v.clone());
            }
        }
        let tm_obj = serde_json::to_value(&tm_stats).into_diagnostic()?;
        if let (Some(map), Some(tm)) = (output.as_object_mut(), tm_obj.as_object()) {
            for (k, v) in tm {
                map.insert(format!("tm_{}", k), v.clone());
            }
        }
        let ci_obj = serde_json::to_value(&ci_stats).into_diagnostic()?;
        if let (Some(map), Some(ci)) = (output.as_object_mut(), ci_obj.as_object()) {
            for (k, v) in ci {
                map.insert(format!("ci_{}", k), v.clone());
            }
        }
        let env_obj = serde_json::to_value(&env_stats).into_diagnostic()?;
        if let (Some(map), Some(env)) = (output.as_object_mut(), env_obj.as_object()) {
            for (k, v) in env {
                map.insert(format!("env_{}", k), v.clone());
            }
        }
        if args.analyze_graph {
            let cent_obj = serde_json::to_value(&cent_stats).into_diagnostic()?;
            if let (Some(map), Some(cent)) = (output.as_object_mut(), cent_obj.as_object()) {
                for (k, v) in cent {
                    map.insert(format!("cent_{}", k), v.clone());
                }
            }
        }
        if let Some(ref cs) = contracts_summary {
            let cs_obj = serde_json::to_value(cs).into_diagnostic()?;
            if let (Some(map), Some(cs)) = (output.as_object_mut(), cs_obj.as_object()) {
                for (k, v) in cs {
                    map.insert(format!("contracts_{}", k), v.clone());
                }
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&output).into_diagnostic()?
        );
    } else {
        println!("Indexing complete:");
        println!("  Files indexed:   {}", stats.files_indexed);
        println!("  Symbols indexed: {}", stats.symbols_indexed);
        if stats.parse_failures > 0 {
            println!("  Parse failures:  {}", stats.parse_failures);
        }
        if stats.skipped_binary > 0 {
            println!("  Skipped binary:  {}", stats.skipped_binary);
        }
        if stats.skipped_unsupported > 0 {
            println!("  Skipped unsupported: {}", stats.skipped_unsupported);
        }
        println!("  Duration:        {}ms", stats.duration_ms);
        println!();
        println!("Documentation:");
        println!("  Docs indexed:    {}", doc_stats.docs_indexed);
        if doc_stats.parse_failures > 0 {
            println!("  Doc parse failures: {}", doc_stats.parse_failures);
        }
        if doc_stats.missing_readme {
            println!("  README:          not found");
        } else {
            println!("  README:          found");
        }
        println!();
        println!("Topology:");
        println!(
            "  Directories classified: {}",
            topo_stats.directories_classified
        );
        if topo_stats.unclassified > 0 {
            println!("  Unclassified:    {}", topo_stats.unclassified);
        }
        let role_order = [
            crate::index::topology::DirectoryRole::Source,
            crate::index::topology::DirectoryRole::Test,
            crate::index::topology::DirectoryRole::Config,
            crate::index::topology::DirectoryRole::Infrastructure,
            crate::index::topology::DirectoryRole::Documentation,
            crate::index::topology::DirectoryRole::Generated,
            crate::index::topology::DirectoryRole::Vendor,
            crate::index::topology::DirectoryRole::BuildArtifact,
        ];
        for role in &role_order {
            if let Some(count) = topo_stats.role_counts.get(role) {
                println!("  {}: {}", role.as_str(), count);
            }
        }
        println!();
        println!("Entrypoints:");
        println!("  Entrypoints:   {}", ep_stats.entrypoints);
        println!("  Handlers:      {}", ep_stats.handlers);
        println!("  Public APIs:   {}", ep_stats.public_apis);
        println!("  Tests:         {}", ep_stats.tests);
        println!("  Internal:     {}", ep_stats.internal);
        println!();
        println!("Call Graph:");
        println!("  Edges:          {}", cg_stats.total_edges);
        println!("  Resolved:       {}", cg_stats.resolved_edges);
        println!("  Unresolved:     {}", cg_stats.unresolved_edges);
        println!("  Ambiguous:      {}", cg_stats.ambiguous_edges);
        println!("  Files processed: {}", cg_stats.files_processed);
        println!();
        println!("API Routes:");
        println!("  Total routes:   {}", route_stats.total_routes);
        if !route_stats.frameworks_detected.is_empty() {
            println!(
                "  Frameworks:    {}",
                route_stats.frameworks_detected.join(", ")
            );
        }
        println!("  Files processed: {}", route_stats.files_processed);
        println!();
        println!("Data Models:");
        println!("  Total models:   {}", dm_stats.total_models);
        println!("  Files processed: {}", dm_stats.files_processed);
        println!();
        println!("Observability:");
        println!("  Total patterns: {}", obs_stats.total_patterns);
        println!(
            "  Error handling patterns: {}",
            obs_stats.error_handling_patterns
        );
        println!("  Telemetry patterns: {}", obs_stats.telemetry_patterns);
        println!("  Files processed: {}", obs_stats.files_processed);
        println!();
        println!("Test Mapping:");
        println!("  Total mappings: {}", tm_stats.total_mappings);
        println!("  Import mappings: {}", tm_stats.import_mappings);
        println!(
            "  Naming convention mappings: {}",
            tm_stats.naming_convention_mappings
        );
        println!("  Files processed: {}", tm_stats.files_processed);
        println!();
        println!("CI/CD Gates:");
        println!("  Total gates: {}", ci_stats.total_gates);
        println!("  GitHub Actions: {}", ci_stats.github_actions_gates);
        println!("  GitLab CI: {}", ci_stats.gitlab_ci_gates);
        println!("  CircleCI: {}", ci_stats.circleci_gates);
        println!("  Makefile: {}", ci_stats.makefile_gates);
        println!("  Files processed: {}", ci_stats.files_processed);
        println!();
        println!("Env Schema:");
        println!("  Total declarations: {}", env_stats.total_declarations);
        println!("  Total references: {}", env_stats.total_references);
        println!("  Dotenv declarations: {}", env_stats.dotenv_declarations);
        println!("  Config declarations: {}", env_stats.config_declarations);
        println!("  Files processed: {}", env_stats.files_processed);
        if args.analyze_graph {
            println!();
            println!("Centrality:");
            println!("  Entry points:   {}", cent_stats.entry_points_count);
            println!("  Symbols computed: {}", cent_stats.symbols_computed);
            println!("  Max reachable:  {}", cent_stats.max_reachable);
        }

        if let Some(ref cs) = contracts_summary {
            println!();
            println!("Contracts:");
            println!("  Specs parsed:     {}", cs.specs_parsed);
            println!("  New endpoints:    {}", cs.endpoints_new);
            println!("  Skipped:          {}", cs.endpoints_skipped);
            println!("  Deleted:          {}", cs.endpoints_deleted);
        }

        println!();
        println!("Services:");
        println!("  Services inferred: {}", service_stats.services_inferred);
        println!("  Files assigned:    {}", service_stats.files_assigned);
    }

    if args.export_docs && !args.check {
        if let Some(cozo) = indexer.cozo() {
            match cozo.node_count() {
                Ok(0) => {
                    println!("Warning: Knowledge Graph is empty, skipping doc export.");
                }
                Ok(_) => {
                    let docs_dir = layout.docs_dir();
                    layout.ensure_dir(&docs_dir)?;
                    let registry = crate::docs::generator::DocRegistry::default_registry();
                    let doc_result = if let Some(ref dt) = args.doc_type {
                        let types: Vec<String> =
                            dt.split(',').map(|s| s.trim().to_string()).collect();
                        registry.run_filtered(&types, cozo, &docs_dir)
                    } else {
                        registry.run_all(cozo, &docs_dir)
                    };
                    match doc_result {
                        Ok(paths) => {
                            for path in &paths {
                                println!("Doc: {}", path);
                            }
                        }
                        Err(e) => {
                            warn!("Doc generation failed: {:#}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to query node count: {:#}", e);
                    println!("Warning: Knowledge Graph unavailable, skipping doc export.");
                }
            }
        } else {
            println!("Warning: Knowledge Graph unavailable, skipping doc export.");
        }
    }

    Ok(())
}

fn execute_docs_index(layout: &Layout, storage: &StorageManager) -> Result<()> {
    let config = match load_config(layout) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to load config: {:#}", e);
            println!("No doc paths configured — skipping doc index.");
            return Ok(());
        }
    };

    if config.docs.include.is_empty() {
        println!("No doc paths configured in [docs].include — skipping doc index.");
        return Ok(());
    }

    let conn = storage.get_connection();
    let summary = run_docs_index(&config, &layout.root, conn)
        .map_err(|e| miette::miette!("Docs index failed: {}", e))?;

    println!(
        "Docs indexed: {} files, {} new chunks, {} updated, {} deleted.",
        summary.files_crawled, summary.chunks_new, summary.chunks_updated, summary.chunks_deleted
    );

    Ok(())
}

fn execute_semantic_index(
    layout: &Layout,
    storage: StorageManager,
    config: &crate::config::model::Config,
    incremental: bool,
    concurrency_override: Option<usize>,
) -> Result<()> {
    use crate::semantic::SemanticDiscovery;
    use indicatif::{ProgressBar, ProgressStyle};
    use rayon::prelude::*;
    use std::sync::{Arc, Mutex};

    let cozo = storage
        .cozo
        .as_ref()
        .ok_or_else(|| miette::miette!("CozoDB storage not initialized"))?;

    let semantic = SemanticDiscovery::new(config.local_model.clone(), cozo)?;

    // HP3: ensure the semantic file-hash tracking schema exists
    semantic.ensure_file_hash_schema()?;

    info!("Indexing repository for semantic search...");

    // ── Phase 1: Collect candidate files ───────────────────────────────────
    let repo_root = layout.root.as_std_path();
    let mut candidate_paths: Vec<std::path::PathBuf> = Vec::new();

    fn walk_dir(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if matches!(name, ".git" | ".changeguard" | "target" | "node_modules") {
                    continue;
                }
                walk_dir(&path, out);
            } else {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if matches!(ext, "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go") {
                    out.push(path);
                }
            }
        }
    }

    walk_dir(repo_root, &mut candidate_paths);

    // HP3: On incremental runs filter to only files whose hash has changed.
    let files_to_process: Vec<std::path::PathBuf> = if incremental {
        let tracked_files = semantic.get_tracked_files()?;
        for tracked in tracked_files {
            let path = std::path::Path::new(&tracked);
            if !path.exists() {
                info!("Pruning deleted file from semantic index: {}", tracked);
                if let Err(e) = semantic.remove_file_snippets(&tracked) {
                    warn!(
                        "Failed to prune snippets for deleted file {}: {}",
                        tracked, e
                    );
                }
                if let Err(e) = semantic.remove_file_hash(&tracked) {
                    warn!(
                        "Failed to remove file hash for deleted file {}: {}",
                        tracked, e
                    );
                }
            }
        }

        candidate_paths
            .into_iter()
            .filter(|path| {
                let Ok(content) = crate::util::fs::read_to_string_with_encoding(path) else {
                    return true; // re-try unreadable files
                };
                let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
                !semantic.is_file_hash_current(path, &hash)
            })
            .collect()
    } else {
        // Full index: prune snippets for files that no longer exist
        semantic.prune_deleted_snippets(repo_root)?;
        candidate_paths
    };

    if files_to_process.is_empty() {
        println!("Semantic index is up to date. No files changed.");
        return Ok(());
    }

    // ── Phase 2: Configure Rayon thread pool (HP2) ─────────────────────────
    let threads = concurrency_override
        .or(config.local_model.concurrency)
        .unwrap_or(0); // 0 = rayon's automatic default
    let pool = if threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(|e| miette::miette!("Failed to build Rayon thread pool: {}", e))?
    } else {
        rayon::ThreadPoolBuilder::new()
            .build()
            .map_err(|e| miette::miette!("Failed to build Rayon thread pool: {}", e))?
    };

    // ── Phase 3: Parallel parse + embed with progress bar (HP2 + HP4) ──────
    let total = files_to_process.len();
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan} Embedding [{bar:40.green/dim}] {pos}/{len} files  {elapsed_precise}",
        )
        .unwrap_or_else(|_| ProgressStyle::with_template("{pos}/{len}").unwrap())
        .progress_chars("█▓░"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    // Wrap accumulators for cross-thread writes
    #[allow(clippy::type_complexity)]
    let results: Arc<Mutex<Vec<(Vec<crate::semantic::chunker::AstChunk>, Vec<Vec<f32>>)>>> =
        Arc::new(Mutex::new(Vec::new()));
    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let files_indexed = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    // HP3: Track successfully processed files on parallel threads to write hashes/prune on the main thread
    let successful_files: Arc<Mutex<Vec<(std::path::PathBuf, String)>>> =
        Arc::new(Mutex::new(Vec::new()));

    let pb_ref = pb.clone();
    let files_clone = files_to_process.clone();

    pool.install(|| {
        files_clone.into_par_iter().for_each(|path| {
            match crate::util::fs::read_to_string_with_encoding(&path) {
                Ok(content) => {
                    match semantic.process_file(&path, &content) {
                        Ok((chunks, embeddings)) => {
                            if !chunks.is_empty() {
                                files_indexed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                let mut acc = results.lock().unwrap_or_else(|p| p.into_inner());
                                acc.push((chunks, embeddings));
                            }
                            // Success path: collect path & hash to write on the main thread
                            let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
                            let mut succ =
                                successful_files.lock().unwrap_or_else(|p| p.into_inner());
                            succ.push((path, hash));
                        }
                        Err(e) => {
                            let mut errs = errors.lock().unwrap_or_else(|p| p.into_inner());
                            errs.push(format!("{}: {}", path.display(), e));
                        }
                    }
                }
                Err(e) => {
                    let mut errs = errors.lock().unwrap_or_else(|p| p.into_inner());
                    errs.push(format!("{}: {}", path.display(), e));
                }
            }
            pb_ref.inc(1);
        });
    });

    pb.finish_and_clear();

    // Report any per-file errors
    let errs = errors.lock().unwrap_or_else(|p| p.into_inner());
    for e in errs.iter() {
        warn!("Semantic indexing skipped: {}", e);
    }
    drop(errs);

    // ── Phase 4: Batch ingest into CozoDB (single-threaded for safety) ─────
    // Extract successfully processed files
    let succ_mutex = Arc::try_unwrap(successful_files).unwrap_or_else(|arc| {
        let guard = arc.lock().unwrap_or_else(|p| p.into_inner());
        std::sync::Mutex::new(guard.clone())
    });
    let processed_files = succ_mutex.into_inner().unwrap_or_else(|p| p.into_inner());

    // 1. Remove stale snippet rows for all successfully processed files
    if !processed_files.is_empty() {
        info!("Pruning stale semantic database rows...");
        for (path, _) in &processed_files {
            let path_str = path.to_string_lossy();
            if let Err(e) = semantic.remove_file_snippets(&path_str) {
                warn!(
                    "Failed to prune stale snippets for {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    // 2. Ingest the new snippets
    // SAFETY: pool.install() has completed and all rayon tasks have joined,
    // so `results` has exactly one strong count here.
    let mutex = Arc::try_unwrap(results).unwrap_or_else(|arc| {
        // Fallback: clone out the data (should never be reached after pool.install)
        let guard = arc.lock().unwrap_or_else(|p| p.into_inner());
        std::sync::Mutex::new(guard.clone())
    });
    let batches = mutex.into_inner().unwrap_or_else(|p| p.into_inner());

    let all_chunks: Vec<_> = batches.iter().flat_map(|(c, _)| c.clone()).collect();
    let all_embeddings: Vec<_> = batches.into_iter().flat_map(|(_, e)| e).collect();

    if !all_chunks.is_empty() {
        // HP4: spinner during HNSW rebuild
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template(
                "  {spinner:.yellow} Building HNSW index… {elapsed_precise}",
            )
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        info!(
            "Ingesting {} snippets into vector store...",
            all_chunks.len()
        );
        semantic.index_chunks_batched(all_chunks, all_embeddings)?;

        spinner.finish_and_clear();
    }

    // 3. Record new hashes only for successfully processed files
    for (path, hash) in processed_files {
        if let Err(e) = semantic.record_file_hash(&path, &hash) {
            warn!("Failed to record file hash for {}: {}", path.display(), e);
        }
    }

    let indexed = files_indexed.load(std::sync::atomic::Ordering::Relaxed);
    println!(
        "Semantic indexing complete: {indexed}/{total} files produced embeddings{}.",
        if incremental { " (incremental)" } else { "" }
    );
    Ok(())
}

fn execute_scip_index(
    layout: &Layout,
    mut storage: StorageManager,
    scip_path: std::path::PathBuf,
) -> Result<()> {
    use crate::index::orchestrator::{get_file_id_by_path, insert_symbol_row};
    use crate::scip::{
        ScipIndex, ScipSymbolMapper, is_scip_stale, normalize_scip_path, register_scip_index,
    };

    info!("Ingesting SCIP index from {:?}", scip_path);
    let scip_index = ScipIndex::load(&scip_path)?;

    let conn = storage.get_connection();
    if !is_scip_stale(conn, &scip_path, &scip_index.file_hash)? {
        info!("SCIP index is up to date, skipping ingestion.");
        return Ok(());
    }

    let conn_mut = storage.get_connection_mut();
    let tx = conn_mut.unchecked_transaction().into_diagnostic()?;

    let mut symbols_ingested = 0usize;
    let mut files_skipped = 0usize;

    for document in &scip_index.index.documents {
        let relative_path =
            match normalize_scip_path(layout.root.as_std_path(), &document.relative_path) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(e) => {
                    warn!(
                        "Failed to normalize SCIP path {}: {}",
                        document.relative_path, e
                    );
                    continue;
                }
            };

        let file_id = match get_file_id_by_path(&tx, &relative_path) {
            Ok(id) => id,
            Err(_) => {
                // If file not in project_files, we might want to skip or add it.
                // For SCIP, we expect the file to be discovered by ProjectIndexer first.
                files_skipped += 1;
                continue;
            }
        };

        // Create a map of symbol name -> SymbolInformation for easy lookup
        let symbol_info_map: std::collections::HashMap<_, _> = scip_index
            .index
            .external_symbols
            .iter()
            .chain(scip_index.index.documents.iter().flat_map(|d| &d.symbols))
            .map(|s| (&s.symbol, s))
            .collect();

        for occurrence in &document.occurrences {
            if occurrence.symbol.is_empty() || occurrence.symbol.starts_with("local ") {
                continue;
            }

            if let Some(symbol_info) = symbol_info_map.get(&occurrence.symbol) {
                let project_symbol =
                    ScipSymbolMapper::map_to_project_symbol(file_id, symbol_info, occurrence);
                insert_symbol_row(&tx, &project_symbol, file_id)?;
                symbols_ingested += 1;
            }
        }
    }

    register_scip_index(&tx, &scip_path, &scip_index.file_hash)?;
    tx.commit().into_diagnostic()?;

    info!(
        "SCIP ingestion complete: {} symbols ingested, {} files skipped (not in project index).",
        symbols_ingested, files_skipped
    );

    Ok(())
}

fn execute_contracts_index(
    layout: &Layout,
    db_path: &std::path::Path,
) -> miette::Result<crate::contracts::index::ContractsIndexSummary> {
    use crate::contracts::index::index_contracts;
    use rusqlite::Connection;

    let config = match load_config(layout) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to load config: {:#}", e);
            println!("No contracts config — skipping contract index.");
            return Ok(Default::default());
        }
    };

    if config.contracts.spec_paths.is_empty() {
        println!("No spec paths configured in [contracts].spec_paths — skipping contract index.");
        return Ok(Default::default());
    }

    let conn = Connection::open(db_path).into_diagnostic()?;
    let summary = index_contracts(&config.contracts, &conn, &config.local_model, &layout.root)
        .map_err(|e| miette::miette!("Contract index failed: {}", e))?;

    Ok(summary)
}

pub fn execute_index_check(
    path: &std::path::Path,
    threshold: u64,
    json: bool,
    strict: bool,
) -> Result<()> {
    let root = camino::Utf8PathBuf::from_path_buf(path.to_path_buf())
        .map_err(|_| miette::miette!("Invalid UTF-8 in path"))?;
    let layout = Layout::new(root.as_str());

    let storage_res = StorageManager::open_read_only(&layout.root);
    let mut warning = match storage_res {
        Ok(ref storage) => crate::index::staleness::check_index_staleness(storage, threshold),
        Err(_) => {
            // Storage missing = index missing (definitely stale)
            Some(crate::index::staleness::StalenessWarning {
                days_since_indexed: 999,
                stale_files: 0,
                unindexed_files: 0,
                sample_paths: vec![],
                last_indexed_at: None,
                is_missing: true,
            })
        }
    };

    // Check for unindexed files if we haven't already marked it as missing
    if let Ok(repo) = crate::git::repo::open_repo(path)
        && let Ok(files) = crate::git::status::get_repo_status(&repo)
    {
        let indexed_files = if let Ok(ref storage) = storage_res {
            storage.get_active_file_id_map().unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

        let mut unindexed = 0;
        for file in &files {
            let rel_path = &file.path;
            if !indexed_files.contains_key(rel_path) {
                let ext = rel_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if matches!(ext, "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "md") {
                    unindexed += 1;
                }
            }
        }

        if unindexed > 0 {
            if let Some(ref mut w) = warning {
                w.unindexed_files = unindexed;
            } else {
                warning = Some(crate::index::staleness::StalenessWarning {
                    days_since_indexed: 0,
                    stale_files: 0,
                    unindexed_files: unindexed,
                    sample_paths: vec![],
                    last_indexed_at: None,
                    is_missing: false,
                });
            }
        }
    }

    if let Some(warning) = warning {
        if json {
            println!("{}", serde_json::to_string(&warning).unwrap_or_default());
        } else {
            crate::index::staleness::print_staleness_warning(&warning);
        }

        if warning.is_missing {
            // If missing, only fail if there are actually files to index
            if warning.unindexed_files > 0 {
                std::process::exit(1);
            }
        } else if strict && (warning.days_since_indexed > threshold || warning.unindexed_files > 0)
        {
            std::process::exit(1);
        }
    } else if json {
        println!(r#"{{"status": "fresh"}}"#);
    } else {
        println!("Index is fresh.");
    }

    Ok(())
}
