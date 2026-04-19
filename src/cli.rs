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
    Scan,
    /// Watch the repository for changes and batch them
    Watch {
        /// The interval in milliseconds to batch events
        #[arg(long, short, default_value_t = 1000)]
        interval: u64,
    },
    /// Analyze the impact of changes and generate a report
    Impact,
    /// Plan and run targeted verification
    Verify {
        /// The command to run for verification
        #[arg(long, short)]
        command: Option<String>,
        /// Timeout in seconds
        #[arg(long, short, default_value_t = 60)]
        timeout: u64,
    },
    /// Ask Gemini for assistance based on the current context
    Ask {
        /// The query to ask Gemini
        query: String,
    },
    /// Reset the local state
    Reset,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { no_gitignore } => crate::commands::init::execute_init(no_gitignore),
        Commands::Doctor => crate::commands::doctor::execute_doctor(),
        Commands::Scan => crate::commands::scan::execute_scan(),
        Commands::Watch { interval } => crate::commands::watch::execute_watch(interval),
        Commands::Impact => crate::commands::impact::execute_impact(),
        Commands::Verify { command, timeout } => crate::commands::verify::execute_verify(command, timeout),
        Commands::Ask { query } => crate::commands::ask::execute_ask(query),
        Commands::Reset => {
            println!("Resetting local state...");
            Ok(())
        }
    }
}
