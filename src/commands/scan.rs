use miette::Result;
use crate::git::repo::{open_repo, get_head_info};
use crate::git::status::get_repo_status;
use crate::git::{RepoSnapshot, ChangeType};
use std::env;
use owo_colors::OwoColorize;

pub fn execute_scan() -> Result<()> {
    let current_dir = env::current_dir().map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    
    let repo = open_repo(&current_dir)?;
    let (head_hash, branch_name) = get_head_info(&repo)?;
    let changes = get_repo_status(&repo)?;
    
    let is_clean = changes.is_empty();
    
    let snapshot = RepoSnapshot {
        head_hash,
        branch_name,
        is_clean,
        changes,
    };

    print_summary(&snapshot);

    Ok(())
}

fn print_summary(snapshot: &RepoSnapshot) {
    println!("{}", "ChangeGuard Git Scan Summary".bold().underline());
    
    let branch = snapshot.branch_name.as_deref().unwrap_or("DETACHED");
    let head = snapshot.head_hash.as_deref().unwrap_or("None");
    
    println!("{}: {}", "Branch".cyan(), branch);
    println!("{}: {}", "HEAD".cyan(), head);
    println!("{}: {}", "State".cyan(), if snapshot.is_clean { "CLEAN".green().to_string() } else { "DIRTY".yellow().to_string() });
    
    if !snapshot.is_clean {
        println!("\n{}", "Changes:".bold());
        for change in &snapshot.changes {
            let status_indicator = if change.is_staged { "S" } else { "U" };
            let change_label = match &change.change_type {
                ChangeType::Added => "A".green().to_string(),
                ChangeType::Modified => "M".yellow().to_string(),
                ChangeType::Deleted => "D".red().to_string(),
                ChangeType::Renamed { old_path } => format!("R ({})", old_path.display()).blue().to_string(),
            };
            
            println!("[{}] {} {}", status_indicator, change_label, change.path.display());
        }
    }
}
