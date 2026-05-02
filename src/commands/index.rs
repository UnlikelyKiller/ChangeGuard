use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::env;

use crate::index::project_index::ProjectIndexer;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use tracing::info;

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

pub fn execute_index(
    incremental: bool,
    check: bool,
    json: bool,
    analyze_graph: bool,
) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let repo_path = layout.root.clone();

    let mut indexer = ProjectIndexer::new(storage, repo_path);

    if check {
        let status = indexer.check_status()?;
        if json {
            let output = serde_json::to_string_pretty(&status).into_diagnostic()?;
            println!("{}", output);
        } else {
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
        if status.stale_files > 0 {
            std::process::exit(1);
        }
        return Ok(());
    }

    let stats = if incremental {
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

    // Compute centrality if requested
    let cent_stats = if analyze_graph {
        indexer.compute_centrality()?
    } else {
        info!("Centrality computation skipped (use --analyze-graph to enable).");
        crate::index::centrality::CentralityStats {
            entry_points_count: 0,
            symbols_computed: 0,
            max_reachable: 0,
        }
    };

    if json {
        let mut output = serde_json::to_value(&stats).into_diagnostic()?;
        let doc_obj = serde_json::to_value(&doc_stats).into_diagnostic()?;
        let topo_obj = serde_json::to_value(&topo_stats).into_diagnostic()?;
        let ep_obj = serde_json::to_value(&ep_stats).into_diagnostic()?;
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
        if analyze_graph {
            let cent_obj = serde_json::to_value(&cent_stats).into_diagnostic()?;
            if let (Some(map), Some(cent)) = (output.as_object_mut(), cent_obj.as_object()) {
                for (k, v) in cent {
                    map.insert(format!("cent_{}", k), v.clone());
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
        if analyze_graph {
            println!();
            println!("Centrality:");
            println!("  Entry points:   {}", cent_stats.entry_points_count);
            println!("  Symbols computed: {}", cent_stats.symbols_computed);
            println!("  Max reachable:  {}", cent_stats.max_reachable);
        }
    }

    Ok(())
}
