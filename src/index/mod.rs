pub mod analysis;
pub mod ast_worker;
pub mod staleness;
pub mod call_graph;
pub mod centrality;
pub mod ci_gates;
pub mod data_models;
pub mod docs;
pub mod entrypoint;
pub mod env_schema;
pub mod git_worker;
pub mod graph_loader;
pub mod graph_worker;
pub mod incremental;
pub mod languages;
pub mod metrics;
pub mod normalize;
pub mod observability;
pub mod orchestrator;
pub mod references;
pub mod routes;
pub mod runtime_usage;
pub mod storage;
pub mod symbols;
pub mod test_mapping;
pub mod topology;

pub use orchestrator::{
    BATCH_SIZE, BINARY_EXTENSIONS, MAX_FILES, PARSER_VERSION, SUPPORTED_EXTENSIONS,
};
pub use orchestrator::{
    IndexStats, IndexStatus, ProjectFile, ProjectIndexer, ProjectSymbol, ServiceIndexStats,
};
pub use staleness::{
    check_index_staleness, print_staleness_warning, warn_if_stale, StalenessWarning,
};
