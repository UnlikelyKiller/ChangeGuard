use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;
use std::process::Command;
use tracing::info;

pub fn execute_update(migrate: bool, binary: bool, force: bool) -> Result<()> {
    if binary {
        update_binary()?;
    }

    if migrate {
        migrate_state(force)?;
    }

    if !binary && !migrate {
        println!(
            "{}",
            "Please specify --binary, --migrate, or both.".yellow()
        );
    }

    Ok(())
}

fn update_binary() -> Result<()> {
    println!("{}", "Updating ChangeGuard binary...".bold().cyan());

    let current_dir = env::current_dir().into_diagnostic()?;
    let cargo_toml = current_dir.join("Cargo.toml");

    if cargo_toml.exists() {
        info!("Detected local source repository. Running 'cargo install --path .'");
        let status = Command::new("cargo")
            .args(["install", "--path", "."])
            .status()
            .into_diagnostic()?;

        if status.success() {
            println!("{}", "Binary updated successfully.".green());
        } else {
            return Err(miette::miette!(
                "Failed to update binary via cargo install."
            ));
        }
    } else {
        println!("{}", "Not in a ChangeGuard source repository. Binary update via CLI is only supported from the source root.".yellow());
        println!("Try running: cargo install --git https://github.com/UnlikelyKiller/ChangeGuard");
    }

    Ok(())
}

fn migrate_state(force: bool) -> Result<()> {
    println!("{}", "Migrating repository state...".bold().cyan());

    let current_dir = env::current_dir().into_diagnostic()?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    if !force {
        println!(
            "{}",
            "Warning: This will clear existing Knowledge Graph and Semantic indices.".yellow()
        );
        println!("Are you sure? Use --force to proceed.");
        return Ok(());
    }

    let state_dir = layout.state_subdir();
    let cozo_path = state_dir.join("ledger.cozo");

    if cozo_path.exists() {
        info!("Removing old Knowledge Graph at {:?}", cozo_path);
        std::fs::remove_file(&cozo_path).into_diagnostic()?;
    }

    let db_path = state_dir.join("ledger.db");
    info!("Re-initializing storage at {:?}", db_path);
    let _storage = StorageManager::init(db_path.as_std_path())?;

    println!(
        "{}",
        "State cleared. You should now run 'changeguard index --semantic' to rebuild the indices."
            .green()
    );

    Ok(())
}
