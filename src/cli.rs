use crate::commands::ask::Backend;
use crate::commands::bridge::BridgeCommands;
use crate::commands::search::SearchArgs;
use crate::ledger::types::Category;
use clap::{Args, Parser, Subcommand};
use miette::{IntoDiagnostic, Result};
use std::env;
use std::path::PathBuf;

#[derive(Parser)]
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

#[derive(Subcommand)]
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
        /// Explain failure probability via local LLM
        #[arg(long)]
        explain: bool,
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
    },
    /// Update ChangeGuard binary or migrate repository state
    #[command(visible_alias = "upgrade")]
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
        /// Show what update actions would be performed without executing them
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Watch repository for changes and run incremental graph sync
    Watch {
        /// Throttle interval in milliseconds for debouncing file events
        #[arg(long, short, default_value_t = 1000)]
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

#[derive(Subcommand)]
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
}

#[derive(Subcommand)]
pub enum IntentCommands {
    /// Launch the interactive intent confirmation UI with mock data
    Demo,
}

#[derive(Subcommand)]
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

#[derive(Subcommand)]
pub enum LedgerCommands {
    /// Start a new change transaction
    Start {
        /// Entity path to track
        entity: String,
        /// Category of change (FEATURE, BUGFIX, ARCHITECTURE, etc.)
        #[arg(short, long)]
        category: String,
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
    /// Generate Architectural Decision Records (MADR format)
    Adr {
        /// Output path for ADR files
        #[arg(short, long, alias = "output-dir", default_value = "docs/adr")]
        output: String,
    },
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
        /// Identify and remove orphaned PENDING transactions
        #[arg(long)]
        orphans: bool,
        /// Time-to-live for PENDING transactions in days
        #[arg(long, default_value_t = 7)]
        ttl_days: u64,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand)]
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

#[derive(Subcommand)]
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
}

pub fn run_with(cli: Cli) -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let layout = crate::state::layout::Layout::new(current_dir.to_string_lossy().as_ref());

    match cli.command {
        Commands::Init { force } => crate::commands::init::execute_init(force),
        Commands::Scan {
            impact,
            summary,
            json,
            out,
        } => crate::commands::scan::execute_scan(impact, summary, json, out),
        Commands::Impact {
            all_parents,
            summary,
            telemetry,
            dead_code,
        } => crate::commands::impact::execute_impact(all_parents, summary, telemetry, dead_code),
        Commands::Index {
            incremental,
            full,
            analyze_graph,
            docs,
            contracts,
            semantic,
            scip,
            export_docs,
            doc_type,
            check,
            json,
            strict,
            concurrency,
            semantic_dry_run,
        } => {
            if check {
                crate::commands::index::execute_index_check(
                    std::path::Path::new("."),
                    3,
                    json,
                    strict,
                )
            } else {
                crate::commands::index::execute_index(crate::commands::index::IndexArgs {
                    incremental: incremental && !full,
                    check: false,
                    strict,
                    json,
                    analyze_graph,
                    docs,
                    contracts,
                    semantic,
                    scip,
                    export_docs,
                    doc_type,
                    concurrency,
                    semantic_dry_run,
                })
            }
        }
        Commands::Search {
            query,
            regex,
            semantic,
            limit,
            index,
            json,
            auto_index,
        } => {
            let project_id = current_dir
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            crate::commands::search::execute_search(SearchArgs {
                query,
                regex,
                semantic,
                limit,
                index,
                json,
                auto_index,
                project_id,
            })
        }
        Commands::Hotspots { args } => crate::commands::hotspots::execute_hotspots(args),
        Commands::Endpoints(args) => crate::commands::endpoints::execute_endpoints(args),
        Commands::Federate { command } => match command {
            FederateCommands::Export { dry_run, out } => {
                crate::commands::federate::execute_federate_export(dry_run, out)
            }
            FederateCommands::Scan => crate::commands::federate::execute_federate_scan(),
            FederateCommands::Status => crate::commands::federate::execute_federate_status(),
        },
        Commands::Bridge { subcommand } => crate::commands::bridge::execute(subcommand),
        Commands::Ledger { command } => match command {
            LedgerCommands::Start {
                entity,
                category,
                message,
            } => crate::commands::ledger::execute_ledger_start(
                entity,
                &category.to_string(),
                &message,
            ),
            LedgerCommands::Commit {
                tx_id,
                summary,
                reason,
                breaking,
                with_git,
                git_message,
                no_signoff,
                dry_run,
            } => crate::commands::ledger::execute_ledger_commit(
                tx_id,
                &summary,
                &reason,
                breaking,
                crate::commands::ledger::LedgerCommitGitOptions {
                    with_git,
                    git_message,
                    signoff: !no_signoff,
                    dry_run,
                },
            ),
            LedgerCommands::Rollback { tx_id, reason } => {
                crate::commands::ledger::execute_ledger_rollback(tx_id, reason)
            }
            LedgerCommands::Atomic {
                entity,
                category,
                summary,
                reason,
            } => crate::commands::ledger::execute_ledger_atomic(
                &entity,
                &category.to_string(),
                &summary,
                &reason,
            ),
            LedgerCommands::Status {
                entity,
                compact,
                exit_code,
                verify_signatures,
            } => crate::commands::ledger::execute_ledger_status(
                entity,
                compact,
                exit_code,
                verify_signatures,
            ),
            LedgerCommands::Register { command } => match command {
                RegisterCommands::Rule {
                    term,
                    category,
                    reason,
                } => crate::commands::ledger::execute_ledger_register_rule(
                    &term,
                    &category.to_string(),
                    &reason,
                ),
                RegisterCommands::Validator {
                    name,
                    command,
                    category,
                    timeout,
                } => crate::commands::ledger::execute_ledger_register_validator(
                    &name, &command, &category, timeout,
                ),
            },
            LedgerCommands::Stack { category } => {
                crate::commands::ledger_stack::execute_ledger_stack(category.map(|c| c.to_string()))
            }
            LedgerCommands::Adr { output } => crate::commands::ledger_adr::execute_ledger_adr(
                Some(camino::Utf8PathBuf::from(output)),
                None,
            ),
            LedgerCommands::Search {
                query,
                category,
                days,
                breaking,
                limit,
                offset,
            } => crate::commands::ledger_search::execute_ledger_search(
                query, category, days, breaking, limit, offset,
            ),
            LedgerCommands::Reconcile {
                tx_id,
                pattern,
                all,
                reason,
            } => crate::commands::ledger::execute_ledger_reconcile(tx_id, pattern, all, reason),
            LedgerCommands::Adopt {
                pattern,
                all,
                category,
                summary,
                reason,
            } => crate::commands::ledger::execute_ledger_adopt(
                pattern,
                all,
                &category.to_string(),
                &summary,
                &reason,
            ),
            LedgerCommands::Audit {
                entity,
                pos_entity,
                include_unaudited,
                limit,
                offset,
                json,
            } => crate::commands::ledger_audit::execute_ledger_audit(
                entity.or(pos_entity),
                include_unaudited,
                limit,
                offset,
                json,
            ),
            LedgerCommands::Gc {
                orphans,
                ttl_days,
                force,
            } => crate::commands::ledger::execute_ledger_gc(orphans, ttl_days, force),
        },
        Commands::Verify {
            command,
            timeout,
            no_predict,
            explain,
            health,
            signatures,
            dry_run,
        } => {
            if signatures {
                crate::commands::verify::verify_ledger_signatures(&layout)
            } else {
                crate::commands::verify::execute_verify(
                    command, timeout, no_predict, explain, health, dry_run,
                )
            }
        }
        Commands::Ask {
            query,
            semantic,
            limit,
            mode,
            narrative,
            backend,
            auto_index,
            timeout,
        } => crate::commands::ask::execute_ask(
            query, semantic, limit, mode, narrative, backend, auto_index, timeout,
        ),
        Commands::Intent { command } => match command {
            IntentCommands::Demo => crate::commands::intent::execute_intent_demo(),
        },
        Commands::Reset {
            remove_config,
            remove_rules,
            include_ledger,
            all,
            yes,
            dry_run,
        } => crate::commands::reset::execute_reset(
            remove_config,
            remove_rules,
            include_ledger,
            all,
            yes,
            dry_run,
        ),
        Commands::Doctor => crate::commands::doctor::execute_doctor(),
        Commands::Config { command } => match command {
            ConfigCommands::Verify {
                json,
                section,
                verbose,
            } => crate::commands::config::execute_config_verify(json, section.as_deref(), verbose),
            ConfigCommands::View { json, section, key } => {
                crate::commands::config::execute_config_view(json, section, key)
            }
        },
        Commands::DeadCode {
            threshold,
            limit,
            auto_index,
        } => crate::commands::dead_code::execute_dead_code(threshold, limit, auto_index),
        Commands::Viz {
            output,
            limit,
            depth,
            entity,
        } => {
            let path = output.map(std::path::PathBuf::from);
            crate::commands::viz::execute_viz(path, limit, depth, entity)
        }
        Commands::Update {
            migrate,
            binary,
            force,
            force_unlock,
            dry_run,
        } => crate::commands::update::execute_update(migrate, binary, force, force_unlock, dry_run),
        Commands::Watch {
            interval,
            json,
            no_graph_sync,
        } => crate::commands::watch::execute_watch(interval, json, no_graph_sync),
        Commands::SearchTrigrams { trigrams, limit } => {
            crate::commands::search::execute_search_trigrams(trigrams, limit)
        }
        Commands::Audit {
            entity,
            pos_entity,
            include_unaudited,
            limit,
            offset,
            json,
        } => crate::commands::ledger_audit::execute_ledger_audit(
            entity.or(pos_entity),
            include_unaudited,
            limit,
            offset,
            json,
        ),
        #[cfg(feature = "daemon")]
        Commands::Daemon { interval } => crate::commands::daemon::execute_daemon(interval),
        #[cfg(feature = "viz-server")]
        Commands::VizServer {
            port,
            bind,
            open,
            stop,
        } => crate::commands::viz_server::execute_viz_server(port, bind, open, stop),
        Commands::Internal { command } => match command {
            InternalCommands::HookCommitMsg { msg_file } => {
                crate::commands::hook_commit_msg::execute_hook_commit_msg(&msg_file)
            }
            InternalCommands::HookPostCommit => {
                crate::commands::hook_post_commit::execute_hook_post_commit()
            }
        },
    }
}
