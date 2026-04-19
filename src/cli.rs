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
    Init,
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
        Commands::Init => println!("Initializing Changeguard..."),
        Commands::Doctor => println!("Running doctor..."),
        Commands::Scan => println!("Scanning repository..."),
        Commands::Watch => println!("Watching for changes..."),
        Commands::Impact => println!("Analyzing impact..."),
        Commands::Verify => println!("Running verification..."),
        Commands::Ask => println!("Asking Gemini..."),
        Commands::Reset => println!("Resetting local state..."),
    }

    Ok(())
}
