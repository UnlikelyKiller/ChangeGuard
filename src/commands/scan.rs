use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::git::{ChangeType, RepoSnapshot};
use crate::ui::print_header;
use comfy_table::Table;
use miette::Result;
use owo_colors::OwoColorize;
use std::env;

pub fn execute_scan() -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

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
    print_header("ChangeGuard Git Scan Summary");

    let branch = snapshot.branch_name.as_deref().unwrap_or("DETACHED");
    let head = snapshot.head_hash.as_deref().unwrap_or("None");

    println!("{:<15} {}", "Branch:".bold().cyan(), branch);
    println!("{:<15} {}", "HEAD:".bold().cyan(), head);
    println!(
        "{:<15} {}",
        "State:".bold().cyan(),
        if snapshot.is_clean {
            "CLEAN".green().bold().to_string()
        } else {
            "DIRTY".yellow().bold().to_string()
        }
    );

    if !snapshot.is_clean {
        println!("\n{}", "Changes:".bold());

        let mut table = Table::new();
        table.set_header(vec!["State", "Action", "File Path"]);

        for change in &snapshot.changes {
            let status_indicator = if change.is_staged {
                "Staged".green().to_string()
            } else {
                "Unstaged".dimmed().to_string()
            };
            let (change_label, color_path) = match &change.change_type {
                ChangeType::Added => (
                    "Added".green().to_string(),
                    change.path.display().to_string().green().to_string(),
                ),
                ChangeType::Modified => (
                    "Modified".yellow().to_string(),
                    change.path.display().to_string().yellow().to_string(),
                ),
                ChangeType::Deleted => (
                    "Deleted".red().to_string(),
                    change.path.display().to_string().red().to_string(),
                ),
                ChangeType::Renamed { old_path } => (
                    "Renamed".blue().to_string(),
                    format!("{} -> {}", old_path.display(), change.path.display())
                        .blue()
                        .to_string(),
                ),
            };

            table.add_row(vec![status_indicator, change_label, color_path]);
        }

        println!("{table}");
    }
}
