use clap::{Args, Parser, Subcommand};
use miette::Result;

#[derive(Parser)]
#[command(name = "changeguard")]
#[command(about = "ChangeGuard: Local-first change intelligence and Gemini-assisted development", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize Changeguard in the current repository
    Init {
        /// Do not update .gitignore
        #[arg(long)]
        no_gitignore: bool,
    },
    /// Check the health of the environment and tools
    Doctor,
    /// Scan the repository for changes
    Scan {
        /// Also run impact analysis after scanning
        #[arg(long)]
        impact: bool,
    },
    /// Watch the repository for changes and batch them
    Watch {
        /// The interval in milliseconds to batch events
        #[arg(long, short, default_value_t = 1000)]
        interval: u64,
        /// Output events as line-delimited JSON
        #[arg(long)]
        json: bool,
        /// Disable live Knowledge Graph updates
        #[arg(long)]
        no_graph_sync: bool,
    },
    /// Analyze the impact of changes and generate a report
    Impact {
        /// Enable full history traversal (default is first-parent only)
        #[arg(long)]
        all_parents: bool,
        /// Show condensed one-line summary instead of full analysis
        #[arg(long)]
        summary: bool,
        /// Warn about files with API routes/handlers but no telemetry instrumentation
        #[arg(long)]
        telemetry_coverage: bool,
        /// Enable dead code detection for changed files
        #[arg(long)]
        dead_code: bool,
    },
    /// Plan and run targeted verification
    Verify {
        /// The command to run for verification
        #[arg(long, short)]
        command: Option<String>,
        /// Timeout in seconds
        #[arg(long, short, default_value_t = 60)]
        timeout: u64,
        /// Disable predictive verification
        #[arg(long)]
        no_predict: bool,
        /// Show rationale for predicted verification targets
        #[arg(long)]
        explain: bool,
        /// Emit ledger health warnings even on clean verify passes
        #[arg(long)]
        health: bool,
    },
    /// Ask Gemini or a local model for assistance based on the current context
    Ask {
        /// The query to ask
        query: Option<String>,
        /// Use semantic search for code snippets instead of full impact context
        #[arg(long, short)]
        semantic: bool,
        /// Gemini interaction mode
        #[arg(long, short, default_value = "analyze")]
        mode: crate::gemini::modes::GeminiMode,
        /// Enable narrative mode (Senior Architect summary)
        #[arg(long)]
        narrative: bool,
        /// Backend to use (local, gemini, or auto)
        #[arg(long)]
        backend: Option<crate::commands::ask::Backend>,
    },
    /// Reset the local state
    Reset {
        /// Also remove .changeguard/config.toml
        #[arg(long)]
        remove_config: bool,
        /// Also remove .changeguard/rules.toml
        #[arg(long)]
        remove_rules: bool,
        /// Also remove .changeguard/state/ledger.db
        #[arg(long)]
        include_ledger: bool,
        /// Remove the entire .changeguard/ tree
        #[arg(long)]
        all: bool,
        /// Confirm destructive reset actions
        #[arg(long)]
        yes: bool,
    },
    /// Index all supported source files in the repository
    Index {
        /// Only re-index files that have changed since the last index
        #[arg(long)]
        incremental: bool,
        /// Show index status without re-indexing
        #[arg(long)]
        check: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Compute symbol centrality from the call graph (entrypoints_reachable)
        #[arg(long)]
        analyze_graph: bool,
        /// Index document files (crawl, chunk, embed) from configured docs paths
        #[arg(long)]
        docs: bool,
        /// Index API contract specs (OpenAPI/Swagger)
        #[arg(long)]
        contracts: bool,
        /// Index code snippets for semantic search (local embeddings)
        #[arg(long)]
        semantic: bool,
        /// Ingest an external SCIP index (Protobuf)
        #[arg(long)]
        scip: Option<std::path::PathBuf>,
        /// Export structural documentation from the Knowledge Graph
        #[arg(long)]
        export_docs: bool,
        /// Generate only the specified doc type (comma-separated)
        #[arg(long)]
        doc_type: Option<String>,
    },
    /// Search the codebase using ranked BM25 or trigram-accelerated regex
    Search {
        /// The search query or regex pattern
        query: String,
        /// Use regex search instead of ranked full-text
        #[arg(long, short)]
        regex: bool,
        /// Maximum number of results to return
        #[arg(long, short, default_value_t = 10)]
        limit: usize,
        /// Force re-indexing before search
        #[arg(long, short)]
        index: bool,
    },
    /// Identify high-risk hotspots in the codebase
    Hotspots {
        /// Maximum number of hotspots to show
        #[arg(long, short, default_value_t = 10)]
        limit: usize,
        /// Commit history window to analyze
        #[arg(long, short, default_value_t = 100)]
        commits: usize,
        /// Output hotspots as JSON
        #[arg(long)]
        json: bool,
        /// Filter by directory
        #[arg(long)]
        dir: Option<String>,
        /// Filter by language (extension)
        #[arg(long)]
        lang: Option<String>,
        /// Enable full history traversal (default is first-parent only)
        #[arg(long)]
        all_parents: bool,
        /// Include centrality data (requires prior `index --analyze-graph`)
        #[arg(long)]
        centrality: bool,
    },
    /// Manage federated intelligence across multiple repositories
    Federate {
        #[command(subcommand)]
        command: FederateCommands,
    },
    /// Manage the ChangeGuard Ledger (transactional provenance)
    Ledger {
        #[command(flatten)]
        global_opts: LedgerGlobalOpts,
        #[command(subcommand)]
        command: LedgerCommands,
    },
    /// Start the LSP-Lite ChangeGuard daemon
    #[cfg(feature = "daemon")]
    Daemon {
        /// The interval in milliseconds to batch events
        #[arg(long, short, default_value_t = 1000)]
        interval: u64,
    },
    /// Perform a holistic project audit or history for an entity
    Audit {
        /// Show history for a specific entity
        #[arg(long, short)]
        entity: Option<String>,
        /// Include UNAUDITED drift in global view
        #[arg(long)]
        include_unaudited: bool,
    },
    /// Manage configuration
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
    },
    /// Generate an interactive visualization of the knowledge graph
    Viz {
        /// Custom output path for the HTML file
        #[arg(long, short)]
        output: Option<std::path::PathBuf>,
    },
    /// Start a live WebSocket viz server with an Arc Diagram
    #[cfg(feature = "viz-server")]
    VizServer {
        /// WebSocket server port
        #[arg(long, short, default_value_t = 8765)]
        port: u16,
        /// Bind address
        #[arg(long, short, default_value = "127.0.0.1")]
        bind: String,
        /// Automatically open the browser on startup
        #[arg(long, short)]
        open: bool,
        /// Stop a running viz server (reads PID file and terminates process)
        #[arg(long)]
        stop: bool,
    },
}

#[derive(Subcommand)]
pub enum FederateCommands {
    /// Export public interfaces for other repositories to consume
    Export,
    /// Scan sibling directories for ChangeGuard schemas
    Scan,
    /// Show status of federated links
    Status,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Verify the configuration files
    Verify,
}

/// Shared flags available to all ledger subcommands.
#[derive(Args)]
pub struct LedgerGlobalOpts {
    /// Simulate the operation without making changes
    #[arg(long)]
    pub dry_run: bool,
}

/// Ledger subcommands follow a single schema rule:
/// **Mandatory primary subject** (entity, tx_id, query) is positional;
/// **all other mandatory fields** (summary, reason, message) are required named flags.
/// Optional identifiers and metadata always use named flags.
/// No command has more than one mandatory positional argument.
#[derive(Subcommand)]
pub enum LedgerCommands {
    /// Start a new transaction
    Start {
        /// The entity being changed (path/symbol)
        entity: String,
        /// The category of change
        #[arg(long, short, value_enum, default_value_t = crate::ledger::Category::Feature)]
        category: crate::ledger::Category,
        /// A brief description of the planned action
        #[arg(long, short)]
        message: Option<String>,
        /// Associated issue reference (e.g., JIRA-123)
        #[arg(long)]
        issue: Option<String>,
    },
    /// Commit a PENDING transaction to the ledger
    Commit {
        /// Transaction ID or unique prefix
        tx_id: String,
        /// High-level summary of the change
        #[arg(long, short)]
        summary: String,
        /// Technical reasoning for the change
        #[arg(long, short)]
        reason: String,
        /// Type of change performed
        #[arg(long, value_enum, default_value_t = crate::ledger::ChangeType::Modify)]
        change_type: crate::ledger::ChangeType,
        /// Mark as a breaking change
        #[arg(long)]
        breaking: bool,
        /// Automatically reconcile matching UNAUDITED drift
        #[arg(long, overrides_with = "no_auto_reconcile")]
        auto_reconcile: bool,
        /// Do not auto-reconcile drift (overrides config)
        #[arg(long)]
        no_auto_reconcile: bool,
        /// Skip verification gate enforcement (use with caution)
        #[arg(long)]
        force: bool,
        /// Also create a git commit after ledger commit succeeds
        #[arg(long)]
        with_git: bool,
        /// Override the auto-generated git commit message
        #[arg(long)]
        git_message: Option<String>,
        /// Do not add Signed-off-by to the git commit
        #[arg(long)]
        no_signoff: bool,
    },
    /// Roll back a PENDING transaction
    Rollback {
        /// Transaction ID or unique prefix
        tx_id: String,
        /// Reason for rolling back
        #[arg(long, short)]
        reason: String,
    },
    /// Reconcile UNAUDITED drift
    Reconcile {
        /// Specific transaction ID or unique prefix
        #[arg(long)]
        tx_id: Option<String>,
        /// Reconcile by entity pattern (glob)
        #[arg(long = "entity-pattern")]
        pattern: Option<String>,
        /// Reconcile all UNAUDITED drift
        #[arg(long)]
        all: bool,
        /// Technical reasoning for the reconciliation
        #[arg(long, short)]
        reason: String,
    },
    /// Adopt UNAUDITED drift into a PENDING transaction
    Adopt {
        /// Specific transaction ID or unique prefix
        #[arg(long)]
        tx_id: Option<String>,
        /// Adopt by entity pattern (glob)
        #[arg(long = "entity-pattern")]
        pattern: Option<String>,
        /// Adopt all UNAUDITED drift
        #[arg(long)]
        all: bool,
        /// Reason for adopting the drift (required, for audit provenance)
        #[arg(long, short)]
        reason: String,
    },
    /// Atomically start and commit a change
    Atomic {
        /// The entity being changed (path/symbol)
        entity: String,
        /// High-level summary of the change
        #[arg(long, short)]
        summary: String,
        /// Technical reasoning for the change
        #[arg(long, short)]
        reason: String,
        /// The category of change
        #[arg(long, short, value_enum, default_value_t = crate::ledger::Category::Chore)]
        category: crate::ledger::Category,
    },
    /// Add a note/lesson to the most recent transaction for an entity
    Note {
        /// The entity (path/symbol)
        entity: String,
        /// The note or lesson learned (required)
        #[arg(long, short)]
        message: Option<String>,
        /// DEPRECATED: Use --message instead. Accepted as a positional for grace period.
        #[arg(verbatim_doc_comment)]
        note: Option<String>,
    },
    /// Show the current status of the ledger and pending transactions
    Status {
        /// Show full history for an entity
        #[arg(long)]
        entity: Option<String>,
        /// Show condensed counts only
        #[arg(long)]
        compact: bool,
        /// Exit with code 1 if there are pending or unaudited entries (useful in git hooks)
        #[arg(long)]
        exit_code: bool,
    },
    /// Resume a PENDING transaction (set as active in session)
    Resume {
        /// Transaction ID or unique prefix (optional: find most recent for context)
        tx_id: Option<String>,
    },
    /// Register a tech stack rule or commit validator
    Register {
        /// Type of rule to register (TECH_STACK, VALIDATOR)
        #[arg(long, value_enum)]
        rule_type: crate::ledger::enforcement::RuleType,
        /// JSON payload for the rule/validator
        #[arg(long)]
        payload: String,
        /// Overwrite existing locked rules
        #[arg(long)]
        force: bool,
    },
    /// View the currently registered tech stack and validators
    Stack {
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
    },
    /// Perform a holistic project audit or history for an entity
    Audit {
        /// Show history for a specific entity
        #[arg(long, short)]
        entity: Option<String>,
        /// Include UNAUDITED drift in global view
        #[arg(long)]
        include_unaudited: bool,
    },
    /// Export architectural decisions as MADR-format markdown
    Adr {
        /// The directory to export ADRs to
        #[arg(long, short)]
        output_dir: Option<camino::Utf8PathBuf>,
        /// Only export ADRs from the last N days
        #[arg(long, short)]
        days: Option<u64>,
    },
    /// Search the ledger using full-text search
    Search {
        /// The search query
        query: String,
        /// Filter by category
        #[arg(long, short, value_enum)]
        category: Option<crate::ledger::Category>,
        /// Only search entries from the last N days
        #[arg(long, short)]
        days: Option<u64>,
        /// Only search for breaking changes
        #[arg(long, short)]
        breaking: bool,
        /// Limit the number of results
        #[arg(long, short, default_value_t = 50)]
        limit: usize,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { no_gitignore } => crate::commands::init::execute_init(no_gitignore),
        Commands::Doctor => crate::commands::doctor::execute_doctor(),
        Commands::Scan { impact } => crate::commands::scan::execute_scan(impact),
        Commands::Watch {
            interval,
            json,
            no_graph_sync,
        } => crate::commands::watch::execute_watch(interval, json, no_graph_sync),
        Commands::Impact {
            all_parents,
            summary,
            telemetry_coverage,
            dead_code,
        } => crate::commands::impact::execute_impact(
            all_parents,
            summary,
            telemetry_coverage,
            dead_code,
        ),
        Commands::Verify {
            command,
            timeout,
            no_predict,
            explain,
            health,
        } => crate::commands::verify::execute_verify(command, timeout, no_predict, explain, health),
        Commands::Ask {
            query,
            semantic,
            mode,
            narrative,
            backend,
        } => crate::commands::ask::execute_ask(query, semantic, mode, narrative, backend),
        Commands::Reset {
            remove_config,
            remove_rules,
            include_ledger,
            all,
            yes,
        } => crate::commands::reset::execute_reset(
            remove_config,
            remove_rules,
            include_ledger,
            all,
            yes,
        ),
        Commands::Index {
            incremental,
            check,
            json,
            analyze_graph,
            docs,
            contracts,
            semantic,
            scip,
            export_docs,
            doc_type,
        } => crate::commands::index::execute_index(crate::commands::index::IndexArgs {
            incremental,
            check,
            json,
            analyze_graph,
            docs,
            contracts,
            semantic,
            scip,
            export_docs,
            doc_type,
        }),
        Commands::Search {
            query,
            regex,
            limit,
            index,
        } => crate::commands::search::execute_search(query, regex, limit, index),
        Commands::Hotspots {
            limit,
            commits,
            json,
            dir,
            lang,
            all_parents,
            centrality,
        } => crate::commands::hotspots::execute_hotspots(
            limit,
            commits,
            json,
            dir,
            lang,
            all_parents,
            centrality,
        ),
        Commands::Federate { command } => match command {
            FederateCommands::Export => crate::commands::federate::execute_federate_export(),
            FederateCommands::Scan => crate::commands::federate::execute_federate_scan(),
            FederateCommands::Status => crate::commands::federate::execute_federate_status(),
        },
        Commands::Ledger {
            command,
            global_opts,
        } => match command {
            LedgerCommands::Start {
                entity,
                category,
                message,
                issue,
            } => crate::commands::ledger::execute_ledger_start(entity, category, message, issue),
            LedgerCommands::Commit {
                tx_id,
                summary,
                reason,
                change_type,
                breaking,
                auto_reconcile,
                no_auto_reconcile,
                force,
                with_git,
                git_message,
                no_signoff,
            } => crate::commands::ledger::execute_ledger_commit(
                tx_id,
                summary,
                reason,
                change_type,
                breaking,
                auto_reconcile,
                no_auto_reconcile,
                force,
                with_git,
                git_message,
                no_signoff,
                global_opts.dry_run,
            ),
            LedgerCommands::Rollback { tx_id, reason } => {
                crate::commands::ledger::execute_ledger_rollback(tx_id, reason)
            }
            LedgerCommands::Reconcile {
                tx_id,
                pattern,
                all,
                reason,
            } => crate::commands::ledger::execute_ledger_reconcile(tx_id, pattern, all, reason),
            LedgerCommands::Adopt {
                tx_id,
                pattern,
                all,
                reason,
            } => crate::commands::ledger::execute_ledger_adopt(tx_id, pattern, all, reason),
            LedgerCommands::Atomic {
                entity,
                summary,
                reason,
                category,
            } => crate::commands::ledger::execute_ledger_atomic(entity, summary, reason, category),
            LedgerCommands::Note {
                entity,
                message,
                note,
            } => crate::commands::ledger::execute_ledger_note(entity, message, note),
            LedgerCommands::Status {
                entity,
                compact,
                exit_code,
            } => crate::commands::ledger::execute_ledger_status(entity, compact, exit_code),
            LedgerCommands::Resume { tx_id } => {
                crate::commands::ledger::execute_ledger_resume(tx_id)
            }
            LedgerCommands::Register {
                rule_type,
                payload,
                force,
            } => {
                crate::commands::ledger_register::execute_ledger_register(rule_type, payload, force)
            }
            LedgerCommands::Stack { category } => {
                crate::commands::ledger_stack::execute_ledger_stack(category)
            }
            LedgerCommands::Audit {
                entity,
                include_unaudited,
            } => crate::commands::ledger_audit::execute_ledger_audit(entity, include_unaudited),
            LedgerCommands::Adr { output_dir, days } => {
                crate::commands::ledger_adr::execute_ledger_adr(output_dir, days)
            }

            LedgerCommands::Search {
                query,
                category,
                days,
                breaking,
                limit,
            } => crate::commands::ledger_search::execute_ledger_search(
                query, category, days, breaking, limit,
            ),
        },
        Commands::Config { command } => match command {
            ConfigCommands::Verify => crate::commands::config::execute_config_verify(),
        },
        Commands::DeadCode { threshold, limit } => {
            crate::commands::dead_code::execute_dead_code(threshold, limit)
        }
        #[cfg(feature = "daemon")]
        Commands::Daemon { interval } => crate::commands::daemon::execute_daemon(interval),
        Commands::Audit {
            entity,
            include_unaudited,
        } => crate::commands::ledger_audit::execute_ledger_audit(entity, include_unaudited),
        Commands::Viz { output } => crate::commands::viz::execute_viz(output),
        #[cfg(feature = "viz-server")]
        Commands::VizServer {
            port,
            bind,
            open,
            stop,
        } => crate::commands::viz_server::execute_viz_server(port, bind, open, stop),
    }
}
