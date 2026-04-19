use clap::{Args, Parser, Subcommand};
use miette::Result;
use crate::commands::init::execute_init;

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
    Init(InitArgs),
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

#[derive(Args)]
pub struct InitArgs {
    /// Do not add .changeguard/ to .gitignore
    #[arg(long)]
    pub no_gitignore: bool,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => execute_init(args.no_gitignore)?,
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
