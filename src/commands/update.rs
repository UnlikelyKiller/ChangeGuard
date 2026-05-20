use crate::git::ignore::add_to_gitignore;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;
use std::process::Command;
use tracing::info;

const IGNORE_PATTERNS: &[&str] = &[".changeguard/"];

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
        print_hint();
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
    let root = get_repo_root()?;
    let layout = Layout::new(&root);

    // Step 1: .gitignore hygiene check (always runs)
    validate_gitignore(&root);

    if force {
        hard_migrate(&layout)?;
    } else {
        soft_migrate(&layout)?;
    }

    Ok(())
}

/// Soft migration: re-initialize storage (runs pending schema migrations) without
/// wiping existing Knowledge Graph or semantic indices.
fn soft_migrate(layout: &Layout) -> Result<()> {
    println!("{}", "Running soft state migration...".bold().cyan());

    let db_path = layout.state_subdir().join("ledger.db");
    info!("Re-initializing storage at {:?}", db_path);
    let _storage = StorageManager::init(db_path.as_std_path())?;

    println!("{}", "Schema migrations applied (if any).".green());
    println!(
        "{}",
        "Use --force to also clear Knowledge Graph and semantic indices for a full rebuild."
            .yellow()
    );

    Ok(())
}

/// Hard migration: wipe non-ledger state (CozoDB, search index) then re-init.
fn hard_migrate(layout: &Layout) -> Result<()> {
    println!("{}", "Running hard state migration...".bold().cyan());

    // Wipe Knowledge Graph (CozoDB sled store)
    let cozo_path = layout.state_subdir().join("ledger.cozo");
    if cozo_path.exists() {
        info!("Removing Knowledge Graph at {:?}", cozo_path);
        if cozo_path.is_dir() {
            std::fs::remove_dir_all(&cozo_path).into_diagnostic()?;
        } else {
            std::fs::remove_file(&cozo_path).into_diagnostic()?;
        }
        println!("{}", "Knowledge Graph cleared.".green());
    }

    // Wipe Tantivy search index
    let search_dir = layout.search_index_dir();
    if search_dir.exists() {
        info!("Removing search index at {:?}", search_dir);
        std::fs::remove_dir_all(search_dir.as_std_path()).into_diagnostic()?;
        println!("{}", "Search index cleared.".green());
    }

    // Re-init storage (runs all schema migrations fresh)
    let db_path = layout.state_subdir().join("ledger.db");
    info!("Re-initializing storage at {:?}", db_path);
    let _storage = StorageManager::init(db_path.as_std_path())?;

    println!("{}", "State migration complete.".green());
    println!(
        "{}",
        "Run 'changeguard index --semantic' to rebuild Knowledge Graph and semantic indices."
            .cyan()
    );

    Ok(())
}

/// Check that `.gitignore` contains necessary patterns and add them if missing.
fn validate_gitignore(root: &camino::Utf8Path) {
    for pattern in IGNORE_PATTERNS {
        match add_to_gitignore(root, pattern) {
            Ok(true) => println!("{} Added '{}' to .gitignore", "[HYGIENE]".yellow(), pattern),
            Ok(false) => { /* already present */ }
            Err(e) => println!(
                "{} Could not check .gitignore for '{}': {}",
                "[HYGIENE]".yellow(),
                pattern,
                e
            ),
        }
    }
}

fn print_hint() {
    println!();
    println!("{}", "Usage:".bold());
    println!("  changeguard update --binary        Update the ChangeGuard binary");
    println!("  changeguard update --migrate        Run soft state migration (schema updates)");
    println!("  changeguard update --migrate --force  Hard reset: wipe KG + search index, re-init");
}

fn get_repo_root() -> Result<Utf8PathBuf> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let discovered = gix::discover(&current_dir).into_diagnostic()?;
    let root = discovered
        .workdir()
        .ok_or_else(|| miette::miette!("Failed to find work directory for repository"))?;

    Utf8PathBuf::from_path_buf(root.to_path_buf())
        .map_err(|_| miette::miette!("Repository root is not valid UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_get_repo_root_ok() {
        let root = get_repo_root().unwrap();
        assert!(
            root.as_str().contains("changeguard"),
            "should be in the changeguard repo"
        );
    }

    #[test]
    fn test_validate_gitignore_adds_missing() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();

        // No .gitignore exists yet
        validate_gitignore(&root);

        let content = fs::read_to_string(dir.path().join(".gitignore")).unwrap_or_default();
        assert!(
            content.contains(".changeguard/"),
            "should have added .changeguard/ pattern"
        );
    }

    #[test]
    fn test_validate_gitignore_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();

        // Run twice
        validate_gitignore(&root);
        validate_gitignore(&root);

        let content = fs::read_to_string(dir.path().join(".gitignore")).unwrap_or_default();
        let count = content.matches(".changeguard/").count();
        assert_eq!(count, 1, "pattern should appear only once");
    }

    #[test]
    fn test_execute_update_no_flags_prints_hint() {
        // Without any flag, execute_update should print the hint and not error.
        let result = execute_update(false, false, false);
        assert!(result.is_ok(), "no-flag invocation should succeed");
    }
}
