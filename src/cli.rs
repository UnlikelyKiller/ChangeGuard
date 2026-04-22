use clap::{Parser, Subcommand};
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
    },
    /// Analyze the impact of changes and generate a report
    Impact {
        /// Enable full history traversal (default is first-parent only)
        #[arg(long)]
        all_parents: bool,
        /// Show condensed one-line summary instead of full analysis
        #[arg(long)]
        summary: bool,
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
    },
    /// Ask Gemini for assistance based on the current context
    Ask {
        /// The query to ask Gemini
        query: Option<String>,
        /// Gemini interaction mode
        #[arg(long, short, default_value = "analyze")]
        mode: crate::gemini::modes::GeminiMode,
        /// Enable narrative mode (Senior Architect summary)
        #[arg(long)]
        narrative: bool,
    },
    /// Reset the local state
    Reset {
        /// Also remove .changeguard/config.toml
        #[arg(long)]
        remove_config: bool,
        /// Also remove .changeguard/rules.toml
        #[arg(long)]
        remove_rules: bool,
        /// Remove the entire .changeguard/ tree
        #[arg(long)]
        all: bool,
        /// Confirm destructive reset actions
        #[arg(long)]
        yes: bool,
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
    },
    /// Manage federated intelligence across multiple repositories
    Federate {
        #[command(subcommand)]
        command: FederateCommands,
    },
    /// Manage the ChangeGuard Ledger (transactional provenance)
    Ledger {
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
    },
    /// Roll back a PENDING transaction
    Rollback {
        /// Transaction ID or unique prefix
        tx_id: String,
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
        /// The note or lesson learned
        note: String,
    },
    /// Show the current status of the ledger and pending transactions
    Status {
        /// Show full history for an entity
        #[arg(long)]
        entity: Option<String>,
    },
    /// Resume a PENDING transaction (set as active in session)
    Resume {
        /// Transaction ID or unique prefix
        tx_id: String,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { no_gitignore } => crate::commands::init::execute_init(no_gitignore),
        Commands::Doctor => crate::commands::doctor::execute_doctor(),
        Commands::Scan { impact } => crate::commands::scan::execute_scan(impact),
        Commands::Watch { interval, json } => crate::commands::watch::execute_watch(interval, json),
        Commands::Impact {
            all_parents,
            summary,
        } => crate::commands::impact::execute_impact(all_parents, summary),
        Commands::Verify {
            command,
            timeout,
            no_predict,
        } => crate::commands::verify::execute_verify(command, timeout, no_predict),
        Commands::Ask {
            query,
            mode,
            narrative,
        } => crate::commands::ask::execute_ask(query, mode, narrative),
        Commands::Reset {
            remove_config,
            remove_rules,
            all,
            yes,
        } => crate::commands::reset::execute_reset(remove_config, remove_rules, all, yes),
        Commands::Hotspots {
            limit,
            commits,
            json,
            dir,
            lang,
            all_parents,
        } => crate::commands::hotspots::execute_hotspots(
            limit,
            commits,
            json,
            dir,
            lang,
            all_parents,
        ),
        Commands::Federate { command } => match command {
            FederateCommands::Export => crate::commands::federate::execute_federate_export(),
            FederateCommands::Scan => crate::commands::federate::execute_federate_scan(),
            FederateCommands::Status => crate::commands::federate::execute_federate_status(),
        },
        Commands::Ledger { command } => match command {
            LedgerCommands::Start { entity, category, message, issue } => 
                crate::commands::ledger::execute_ledger_start(entity, category, message, issue),
            LedgerCommands::Commit { tx_id, summary, reason, change_type, breaking } =>
                crate::commands::ledger::execute_ledger_commit(tx_id, summary, reason, change_type, breaking),
            LedgerCommands::Rollback { tx_id } =>
                crate::commands::ledger::execute_ledger_rollback(tx_id),
            LedgerCommands::Atomic { entity, summary, reason, category } =>
                crate::commands::ledger::execute_ledger_atomic(entity, summary, reason, category),
            LedgerCommands::Note { entity, note } =>
                crate::commands::ledger::execute_ledger_note(entity, note),
            LedgerCommands::Status { entity } =>
                crate::commands::ledger::execute_ledger_status(entity),
            LedgerCommands::Resume { tx_id } =>
                crate::commands::ledger::execute_ledger_resume(tx_id),
        },
        #[cfg(feature = "daemon")]
        Commands::Daemon { interval } => crate::commands::daemon::execute_daemon(interval),
    }
}
