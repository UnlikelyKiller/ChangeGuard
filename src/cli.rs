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
    Watch,
    /// Analyze the impact of changes and generate a report
    Impact,
    /// Plan and run targeted verification
    Verify,
    /// Ask Gemini for assistance based on the current context
    Ask,
    /// Reset the local state
    Reset,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { no_gitignore } => crate::commands::init::execute_init(no_gitignore),
        Commands::Doctor => crate::commands::doctor::execute_doctor(),
        Commands::Scan => crate::commands::scan::execute_scan(),
        Commands::Watch => {
            println!("Watching for changes...");
            Ok(())
        }
        Commands::Impact => {
            println!("Analyzing impact...");
            Ok(())
        }
        Commands::Verify => {
            println!("Running verification...");
            Ok(())
        }
        Commands::Ask => {
            println!("Asking Gemini...");
            Ok(())
        }
        Commands::Reset => {
            println!("Resetting local state...");
            Ok(())
        }
    }
}
