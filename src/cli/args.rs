use crate::commands::ask::Backend;
use crate::commands::bridge::BridgeCommands;
use crate::ledger::types::Category;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "changeguard")]
#[command(about = "Change Intelligence and Transactional Provenance for Software Engineering", long_about = None)]
#[command(version)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging output
    #[arg(long, short, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize ChangeGuard in the current repository
    Init {
        /// Force re-initialization (overwrites existing config)
        #[arg(short, long)]
        force: bool,
    },
    /// Scan git changes and identify affected symbols
    Scan {
        /// Run impact analysis on changes
        #[arg(short, long)]
        impact: bool,
        /// Output a high-level summary only
        #[arg(short, long)]
        summary: bool,
        /// Output as JSON (requires --impact)
        #[arg(short, long)]
        json: bool,
        /// Write JSON output to file
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Analyze impact of current changes
    Impact {
        /// Traverse all parent commits for temporal coupling
        #[arg(long)]
        all_parents: bool,
        /// Output a concise summary
        #[arg(short, long)]
        summary: bool,
        /// Enable telemetry coverage analysis
        #[arg(long)]
        telemetry: bool,
        /// Run dead-code analysis on affected files
        #[arg(long)]
        dead_code: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Write output to file
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Index the project for search and discovery
    Index {
        /// Perform incremental index (only changed files)
        #[arg(long, short)]
        incremental: bool,
        /// Force a full re-index
        #[arg(long, short)]
        full: bool,
        /// Refresh the knowledge graph (analyze structure)
        #[arg(long)]
        analyze_graph: bool,
        /// Index documentation files
        #[arg(long)]
        docs: bool,
        /// Index API contract files (OpenAPI/Swagger)
        #[arg(long)]
        contracts: bool,
        /// Index code snippets for semantic search (local embeddings)
        #[arg(long)]
        semantic: bool,
        /// Ingest an external SCIP index (Protobuf)
        #[arg(long)]
        scip: Option<std::path::PathBuf>,
        /// Export knowledge graph data to passive documentation
        #[arg(long)]
        export_docs: bool,
        /// Filter exported documentation by type (e.g. mermaid, markdown)
        #[arg(long)]
        doc_type: Option<String>,
        /// Check index freshness
        #[arg(long)]
        check: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Strict mode for check (exit 1 if stale)
        #[arg(long)]
        strict: bool,
        /// Number of parallel threads for semantic indexing (default: logical CPUs)
        #[arg(long, short = 'j')]
        concurrency: Option<usize>,
        /// Print resolved semantic settings and exit. Optionally takes a path for JSON output.
        #[arg(long, value_name = "OUTPUT_PATH", num_args = 0..=1)]
        semantic_dry_run: Option<Option<std::path::PathBuf>>,
        /// Use Gemini for semantic extraction (fast, large context) instead of local model
        #[arg(long)]
        fast: bool,
    },
    /// Search the codebase using high-performance regex or semantic search
    Search {
        /// The query string
        query: String,
        /// Use regular expression search
        #[arg(short, long)]
        regex: bool,
        /// Use semantic search (requires local model and indexed snippets)
        #[arg(short, long)]
        semantic: bool,
        /// Limit the number of results
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        /// Force re-index before searching
        #[arg(short, long)]
        index: bool,
        /// Output results as NDJSON BridgeRecord entries
        #[arg(long)]
        json: bool,
        /// Automatically run incremental index before searching if the index is stale
        #[arg(long)]
        auto_index: bool,
        /// Use hybrid search (combines regex and BM25 results)
        #[arg(long)]
        hybrid: bool,
    },
    /// Rank files by change frequency and complexity (Hotspots)
    Hotspots {
        #[command(flatten)]
        args: HotspotArgs,
    },
    /// List and filter API endpoints
    Endpoints(crate::commands::endpoints::EndpointsArgs),
    /// Manage cross-repo federation
    Federate {
        #[command(subcommand)]
        command: FederateCommands,
    },
    /// Service boundary and topology commands
    Services {
        #[command(subcommand)]
        command: ServiceSubcommands,
    },
    /// Manage data models and schema migrations
    #[command(name = "data-models")]
    DataModels(crate::commands::data_models::DataModelsArgs),
    /// CI configuration and gate commands
    Ci(crate::commands::deploy::CiArgs),
    /// Deployment manifest and surface commands
    Deploy(crate::commands::deploy::DeployArgs),
    /// Manage project dependencies and security advisories
    Dependencies(crate::commands::dependencies::DependenciesArgs),
    /// Manage runtime observability and SLOs
    Observability(crate::commands::observability::ObservabilityArgs),
    /// Manage security boundaries and policies
    Security(crate::commands::security::SecurityArgs),
    /// List tests validating a specific entity
    Tests(crate::commands::test_mapping::TestsForEntityArgs),
    /// Manage ChangeGuard bridge (AI-Brains integration)
    Bridge {
        #[command(subcommand)]
        subcommand: BridgeCommands,
    },
    /// Manage project ledger and transactional provenance
    Ledger {
        #[command(subcommand)]
        command: LedgerCommands,
    },
    /// Run verification plan (predictive Bayesian testing)
    Verify {
        /// Optional specific command or step to run
        command: Option<String>,
        /// Timeout in seconds
        #[arg(long, short, default_value_t = 600)]
        timeout: u64,
        /// Disable Bayesian failure prediction
        #[arg(long)]
        no_predict: bool,
        /// Explain failure probability via local LLM for a specific entity
        #[arg(long)]
        explain: bool,
        /// Entity path for verification explanation (use with --explain; does not narrow executed steps)
        #[arg(long, short)]
        entity: Option<String>,
        /// Show detailed health of the verification system
        #[arg(long)]
        health: bool,
        /// Mathematically verify all transaction signatures in the ledger
        #[arg(long)]
        signatures: bool,
        /// Show the verification plan without executing any commands
        #[arg(long)]
        dry_run: bool,
    },
    /// Ask Gemini or a local model for assistance based on the current context
    Ask {
        /// The query to ask
        query: Option<String>,
        /// Use semantic search for code snippets instead of full impact context
        #[arg(long, short)]
        semantic: bool,
        /// Maximum number of code snippets to include in context
        #[arg(long, short, default_value_t = 10)]
        limit: usize,
        /// Gemini interaction mode
        #[arg(long, short, default_value = "analyze")]
        mode: crate::gemini::modes::GeminiMode,
        /// Enable narrative mode (Senior Architect summary)
        #[arg(long)]
        narrative: bool,
        /// Backend to use (local, gemini, or auto)
        #[arg(long)]
        backend: Option<Backend>,
        /// Automatically run incremental index before querying if the index is stale
        #[arg(long)]
        auto_index: bool,
        /// Per-request timeout in seconds for LLM backend calls (default: 15).
        /// Prevents `changeguard ask` from hanging when a backend is slow or unresponsive.
        #[arg(long, default_value_t = 15)]
        timeout: u64,
        /// Disable Knowledge Graph BM25 fallback when semantic index is empty
        #[arg(long)]
        no_kg_fallback: bool,
    },
    /// Manage ChangeGuard intent capture and TUI interaction
    Intent {
        #[command(subcommand)]
        command: IntentCommands,
    },
    /// Reset ChangeGuard state or configuration
    Reset {
        /// Remove configuration file
        #[arg(long)]
        remove_config: bool,
        /// Remove local rules
        #[arg(long)]
        remove_rules: bool,
        /// Reset the ledger (history and pending transactions)
        #[arg(long)]
        include_ledger: bool,
        /// Remove all state and configuration (total reset)
        #[arg(long, short)]
        all: bool,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
        /// Show what files/directories would be deleted without deleting them
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Health check for ChangeGuard and local model stack
    Doctor,
    /// Quick status check of the project ledger and pending transactions
    Status,
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Detect likely dead code across the repository
    DeadCode {
        /// Minimum confidence threshold to report a finding
        #[arg(long, default_value_t = 0.75)]
        threshold: f64,
        /// Maximum number of findings to display
        #[arg(long, default_value_t = 50)]
        limit: usize,
        /// Automatically run incremental index before detection if the index is stale
        #[arg(long)]
        auto_index: bool,
    },
    /// Perform a holistic project audit or history for an entity
    Audit {
        /// Entity path to audit (e.g. src/main.rs)
        #[arg(short, long)]
        entity: Option<String>,
        /// Entity path to audit (positional fallback)
        #[arg(hide = true)]
        pos_entity: Option<String>,
        /// Include unaudited drift in the report
        #[arg(long, short)]
        include_unaudited: bool,
        /// Maximum number of entries to display
        #[arg(long, short, default_value_t = 10)]
        limit: usize,
        /// Offset for pagination
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Generate an interactive visualization of the knowledge graph
    Viz {
        /// Custom output path for the HTML file
        #[arg(long, short, alias = "out")]
        output: Option<String>,
        /// Maximum number of nodes to include
        #[arg(long, short, default_value_t = 1000)]
        limit: usize,
        /// Maximum depth for relationship traversal
        #[arg(long, short, default_value_t = 2)]
        depth: usize,
        /// Filter by specific entity (root of the graph)
        #[arg(long, short)]
        entity: Option<String>,
        /// Visualization view: "graph" (default) or "services" (K4 service connectivity)
        #[arg(long, default_value = "graph")]
        view: String,
    },
    /// Update ChangeGuard binary or migrate repository state
    #[command(alias = "upgrade")]
    Update {
        /// Perform repository state migration (re-index and schema upgrade)
        #[arg(long)]
        migrate: bool,
        /// Update ChangeGuard binary to the latest version
        #[arg(long)]
        binary: bool,
        /// Skip confirmation prompts
        #[arg(long, short)]
        force: bool,
        /// Force unlock CozoDB by terminating other running ChangeGuard processes
        #[arg(long = "force-unlock")]
        force_unlock: bool,
        /// Use fast semantic index bypass (skip LLM semantic extraction during migration)
        #[arg(long)]
        fast: bool,
        /// Show what update actions would be performed without executing them
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Watch repository for changes and run incremental graph sync
    Watch {
        /// Throttle interval in milliseconds for debouncing file events.
        /// Defaults to `watch.debounce_ms` from config when not specified.
        #[arg(long, short, default_value_t = 0)]
        interval: u64,
        /// Output watch events as JSON
        #[arg(long, short)]
        json: bool,
        /// Disable Knowledge Graph sync during watch
        #[arg(long = "no-graph-sync")]
        no_graph_sync: bool,
    },
    /// High-performance trigram-based search (low-level)
    #[command(hide = true)]
    SearchTrigrams {
        /// Trigrams to search for (space separated)
        trigrams: Vec<String>,
        /// Limit results
        #[arg(long, short, default_value_t = 100)]
        limit: usize,
    },
    #[cfg(feature = "daemon")]
    Daemon {
        /// The interval in milliseconds to batch events
        #[arg(long, short, default_value_t = 1000)]
        interval: u64,
    },
    /// Knowledge graph visualization server
    #[cfg(feature = "viz-server")]
    VizServer {
        /// Port to listen on
        #[arg(long, short, default_value_t = 9000)]
        port: u16,
        /// Address to bind to
        #[arg(long, short, default_value = "127.0.0.1")]
        bind: String,
        /// Open the visualization in the default browser
        #[arg(long)]
        open: bool,
        /// Stop a running visualization server
        #[arg(long)]
        stop: bool,
    },
    /// Internal helper commands for git hooks and lifecycle management
    #[command(hide = true)]
    Internal {
        #[command(subcommand)]
        command: InternalCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum AdrSubcommands {
    /// Export MADR files from ledger history
    Export {
        /// Output path for ADR files
        #[arg(short, long, alias = "output-dir", default_value = "docs/adr")]
        output: String,
        /// Filter entries from the last N days
        #[arg(short, long)]
        days: Option<u64>,
    },
    /// Update lifecycle status of an ADR
    UpdateStatus {
        /// ADR ID (transaction ID or prefix)
        adr_id: String,
        /// New status
        #[arg(value_enum)]
        status: crate::ledger::types::AdrStatus,
    },
    /// Link an ADR as superseding another
    Link {
        /// Current ADR ID
        adr_id: String,
        /// ID of the ADR being superseded
        #[arg(short, long)]
        supersedes: String,
    },
    /// Record a review for an ADR
    Review {
        /// ADR ID
        adr_id: String,
        /// Optional review notes
        #[arg(short, long)]
        message: Option<String>,
    },
    /// List all ADRs in the ledger
    List,
}

#[derive(Subcommand, Debug)]
pub enum ServiceSubcommands {
    /// Show service boundary changes and topology
    Diff(crate::commands::services_diff::ServicesDiffArgs),
}

#[derive(Subcommand, Debug)]
pub enum ValidatorSubcommands {
    /// List all registered commit validators
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Enable a commit validator
    Enable {
        /// Name of the validator
        name: String,
    },
    /// Disable a commit validator
    Disable {
        /// Name of the validator
        name: String,
    },
    /// Remove a commit validator from the registry
    Remove {
        /// Name of the validator
        name: String,
    },
    /// Check validator executables and report health
    Doctor,
}

#[derive(Subcommand, Debug)]
pub enum InternalCommands {
    /// Internal git hook command for commit message validation
    #[command(name = "hook-commit-msg")]
    HookCommitMsg {
        /// The file containing the commit message
        msg_file: PathBuf,
    },
    /// Internal git hook command for post-commit processing
    #[command(name = "hook-post-commit")]
    HookPostCommit,
}

#[derive(Args, Debug)]
pub struct HotspotArgs {
    #[command(subcommand)]
    pub command: Option<HotspotSubcommands>,

    /// Limit the number of hotspots displayed
    #[arg(short, long)]
    pub limit: Option<usize>,

    /// Number of commits to analyze
    #[arg(short, long)]
    pub commits: Option<usize>,

    /// Number of days to analyze
    #[arg(short, long)]
    pub days: Option<u32>,

    /// Specific commit to start from
    #[arg(long)]
    pub since: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Automatically run incremental index before calculation if the index is stale
    #[arg(long)]
    pub auto_index: bool,

    /// Traverse all parent commits (useful for branch merges)
    #[arg(long)]
    pub all_parents: bool,

    /// Include centrality data (requires prior `index --analyze-graph`)
    #[arg(long)]
    pub centrality: bool,

    /// Filter by entity path
    #[arg(short, long)]
    pub entity: Option<String>,

    /// Find semantically similar code clusters (duplication hotspots)
    #[arg(long, short)]
    pub semantic: bool,

    /// Persist the results as a snapshot in the history tables
    #[arg(long)]
    pub snapshot: bool,
}

#[derive(Subcommand, Debug)]
pub enum HotspotSubcommands {
    /// Show hotspot and temporal coupling trends over time
    Trend {
        /// Entity path to filter by
        #[arg(short, long)]
        entity: Option<String>,
        /// Number of days to look back
        #[arg(short, long, default_value_t = 30)]
        days: u32,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Explain why a file is a hotspot or highly coupled
    Explain {
        /// Entity path to explain
        entity: String,
    },
    /// Check hotspot and coupling budgets
    Budget {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum IntentCommands {
    /// Launch the interactive intent confirmation UI with mock data
    Demo,
}

#[derive(Subcommand, Debug)]
pub enum FederateCommands {
    /// Export public interfaces for other repositories to consume
    Export {
        /// Preview the schema without writing to .changeguard/state/schema.json
        #[arg(long, short = 'd')]
        dry_run: bool,
        /// Custom output path for the schema file
        #[arg(long, short)]
        out: Option<String>,
    },
    /// Scan sibling directories for ChangeGuard schemas
    Scan,
    /// Show status of federated links
    Status,
}

#[derive(Subcommand, Debug)]
pub enum LedgerCommands {
    /// Start a new change transaction
    Start {
        /// Entity path to track
        entity: String,
        /// Category of change (ARCHITECTURE, FEATURE, BUGFIX, REFACTOR, INFRA, SECURITY, DOCS, CHORE, TOOLING)
        #[arg(short, long)]
        category: Category,
        /// Intent message for the change
        #[arg(short, long)]
        message: String,
    },
    /// Finalize and commit a change transaction
    Commit {
        /// Transaction ID to commit (optional, defaults to current)
        tx_id: Option<String>,
        /// Summary of the change
        #[arg(short, long)]
        summary: String,
        /// Reason for the change (Architecture Decision)
        #[arg(short, long)]
        reason: String,
        /// Mark as a breaking change
        #[arg(long)]
        breaking: bool,
        /// Bypass verification gate enforcement
        #[arg(long)]
        force: bool,
        /// Create a git commit after the ledger commit succeeds
        #[arg(long)]
        with_git: bool,
        /// Override the generated git commit message
        #[arg(long, requires = "with_git")]
        git_message: Option<String>,
        /// Skip adding a git Signed-off-by trailer
        #[arg(long, requires = "with_git")]
        no_signoff: bool,
        /// Print the git commit command without executing it
        #[arg(long, requires = "with_git")]
        dry_run: bool,
    },
    /// Roll back an active transaction
    Rollback {
        /// Transaction ID to rollback (optional, defaults to current)
        tx_id: Option<String>,
        /// Reason for the rollback
        #[arg(short, long)]
        reason: String,
    },
    /// Record a surgical atomic change without a full session
    Atomic {
        /// Entity path
        entity: String,
        /// Category of change
        #[arg(short, long)]
        category: Category,
        /// Summary
        #[arg(short, long)]
        summary: String,
        /// Reason
        #[arg(short, long)]
        reason: String,
        /// Bypass verification gate enforcement
        #[arg(long)]
        force: bool,
    },
    /// Show status of active transactions and uncommitted drift
    Status {
        /// Filter status by entity path
        #[arg(short, long)]
        entity: Option<String>,
        /// Output a compact view
        #[arg(short, long)]
        compact: bool,
        /// Exit with 1 if there is unaudited drift
        #[arg(long)]
        exit_code: bool,
        /// Perform signature verification and exit with 1 if signatures are invalid
        #[arg(long = "verify-signatures")]
        verify_signatures: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Register a new tech stack rule or commit validator
    Register {
        #[command(subcommand)]
        command: RegisterCommands,
    },
    /// Show active tech stack enforcement rules
    Stack {
        /// Filter by category (e.g. Database, Auth)
        category: Option<Category>,
    },
    /// Architectural Decision Records (MADR format)
    Adr {
        #[command(subcommand)]
        command: AdrSubcommands,
    },
    /// Manage commit validators
    Validator {
        #[command(subcommand)]
        command: ValidatorSubcommands,
    },
    /// Show the entity graph neighborhood governed by a transaction
    Graph(crate::commands::ledger_graph::LedgerGraphArgs),
    /// Full-text search across ledger history
    Search {
        /// Search query
        query: String,
        /// Filter by category
        #[arg(short, long)]
        category: Option<Category>,
        /// Number of days to look back
        #[arg(short, long)]
        days: Option<u64>,
        /// Filter by breaking changes only
        #[arg(short, long)]
        breaking: bool,
        /// Limit results
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        /// Offset for pagination
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Reconcile detected drift with a transaction or pattern
    Reconcile {
        /// Transaction ID to associate drift with
        #[arg(short, long)]
        tx_id: Option<String>,
        /// File pattern to reconcile (glob)
        #[arg(short, long)]
        pattern: Option<String>,
        /// Reconcile all current drift
        #[arg(long)]
        all: bool,
        /// Reason for reconciliation
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// Adopt drift as a new committed transaction
    Adopt {
        /// File pattern to adopt
        #[arg(short, long)]
        pattern: Option<String>,
        /// Adopt all current drift
        #[arg(long)]
        all: bool,
        /// Category for the new transaction
        #[arg(short, long)]
        category: Category,
        /// Summary for the new transaction
        #[arg(short, long)]
        summary: String,
        /// Reason for the new transaction
        #[arg(short, long)]
        reason: String,
    },
    /// Perform a holistic project audit or history for an entity
    Audit {
        /// Entity path to audit (e.g. src/main.rs)
        #[arg(short, long)]
        entity: Option<String>,
        /// Entity path to audit (positional fallback)
        #[arg(hide = true)]
        pos_entity: Option<String>,
        /// Include unaudited drift in the report
        #[arg(long, short)]
        include_unaudited: bool,
        /// Maximum number of entries to display
        #[arg(long, short, default_value_t = 10)]
        limit: usize,
        /// Offset for pagination
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Garbage collect orphaned or stale ledger entries
    Gc {
        /// Remove PENDING transactions older than TTL
        #[arg(long)]
        stale: bool,
        /// Remove transactions with no corresponding git commit
        #[arg(long)]
        orphans: bool,
        /// Time-to-live for PENDING transactions in hours (used with --stale)
        #[arg(long, default_value_t = 72)]
        ttl_hours: u64,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum RegisterCommands {
    /// Register a forbidden term (tech stack enforcement)
    Rule {
        /// Forbidden term or technology name
        term: String,
        /// Category (e.g. Database, ORM)
        #[arg(short, long)]
        category: Category,
        /// Reason for prohibition
        #[arg(short, long)]
        reason: String,
    },
    /// Register a commit validator script
    Validator {
        /// Name of the validator
        name: String,
        /// Command to execute (supports {entity} placeholder)
        #[arg(short = 'x', long)]
        command: String,
        /// Category this validator applies to (or 'ALL')
        #[arg(short, long)]
        category: String,
        /// Timeout in seconds
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Verify current configuration and environment health
    Verify {
        /// Output results as JSON
        #[arg(long)]
        json: bool,
        /// Filter by specific section name (e.g. backend, semantic)
        #[arg(long, short)]
        section: Option<String>,
        /// Include defaults that are normally hidden
        #[arg(long, short)]
        verbose: bool,
    },
    /// View resolved project configuration
    View {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Filter view by section (e.g. local_model)
        #[arg(long, short)]
        section: Option<String>,
        /// Filter view by key within section (requires --section, or searches top-level)
        #[arg(long, short)]
        key: Option<String>,
    },
    /// Manage environment and config schemas
    Schema {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show differences between declared and inferred config
    Diff {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
