use crate::git::ignore::add_to_gitignore;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;
use std::fs;
use std::process::Command;
use tracing::info;

pub fn execute_update(
    migrate: bool,
    binary: bool,
    force: bool,
    force_unlock: bool,
) -> Result<()> {
    if !migrate && !binary {
        println!(
            "{} Specify what to update (e.g. --migrate or --binary)",
            "HINT:".yellow().bold()
        );
        return Ok(());
    }

    if migrate {
        execute_migration()?;
    }

    if binary {
        execute_binary_update(force, force_unlock)?;
    }

    Ok(())
}

fn execute_migration() -> Result<()> {
    println!("{} Migrating repository state...", "INIT".cyan().bold());

    // 1. Re-index
    crate::commands::index::execute_index(crate::commands::index::IndexArgs {
        incremental: false,
        analyze_graph: true,
        docs: true,
        contracts: true,
        semantic: true,
        scip: None,
        export_docs: false,
        doc_type: None,
        check: false,
        json: false,
        strict: false,
        concurrency: None,
    })?;

    println!("{} Migration complete.", "DONE".green().bold());
    Ok(())
}

fn execute_binary_update(force: bool, force_unlock: bool) -> Result<()> {
    if force_unlock {
        crate::platform::process_policy::force_unlock_processes()?;
    }

    let is_local_repo = fs::metadata("Cargo.toml").is_ok();
    if !is_local_repo {
        return Err(miette::miette!(
            "Binary update currently only supported from within the ChangeGuard source repository."
        ));
    }

    info!("Detected local source repository. Running 'cargo install --path .'");

    // Check if the target binary is locked before starting the build
    if let Ok(bin_path) = env::current_exe()
        && std::fs::OpenOptions::new()
            .write(true)
            .open(&bin_path)
            .is_err()
    {
        println!(
            "{}",
            "Warning: ChangeGuard binary is currently locked by another process.".yellow()
        );
        println!(
            "Please close any other running instances or daemon processes before continuing."
        );
        println!("(Attempting shadow-copy anyway...)");
    }

    // --- H4: Windows shadow-copy ---
    // On Windows, we cannot overwrite a running executable. Even if
    // `cargo install` builds a new one, it will fail to copy it to the
    // destination if the destination executable is locked by the OS. We rename it to
    // `<name>.old` so that the file handle is still valid (Windows allows
    // renaming locked files) but the path is free for the new binary.
    let old_path_opt = shadow_copy_current_exe();

    let mut cmd = Command::new("cargo");
    cmd.args(["install", "--path", "."]);

    if force {
        cmd.arg("--force");
    }

    let status = cmd.status().into_diagnostic()?;

    if status.success() {
        println!("{} ChangeGuard updated successfully.", "DONE".green().bold());
        if let Some(old_path) = old_path_opt {
            info!("Stale binary moved to: {}", old_path.display());
            println!(
                "{} Stale binary will be cleaned up on next startup.",
                "INFO:".blue().bold()
            );
        }
    } else {
        // If update failed, try to restore the old binary name if we moved it
        if let (Some(old_path), Ok(bin_path)) = (old_path_opt, env::current_exe()) {
            let _ = fs::rename(old_path, bin_path);
        }
        return Err(miette::miette!("Update failed. See above for errors."));
    }

    Ok(())
}

fn shadow_copy_current_exe() -> Option<std::path::PathBuf> {
    if let Ok(bin_path) = env::current_exe() {
        let mut old_path = bin_path.clone();
        let stem = bin_path.file_stem()?.to_string_lossy();
        let extension = bin_path.extension()?.to_string_lossy();
        old_path.set_file_name(format!("{}.old.{}", stem, extension));

        if fs::rename(&bin_path, &old_path).is_ok() {
            return Some(old_path);
        }
    }
    None
}

pub fn validate_gitignore(root: &camino::Utf8Path) -> Result<()> {
    let patterns = [
        ".changeguard/tmp",
        ".changeguard/logs",
        ".changeguard/state/ledger.db-shm",
        ".changeguard/state/ledger.db-wal",
        "output/",
    ];

    for pattern in patterns {
        add_to_gitignore(root, pattern)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_gitignore_adds_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = camino::Utf8Path::from_path(tmp.path()).unwrap();

        validate_gitignore(root).unwrap();

        let ignore_path = root.join(".gitignore");
        let content = fs::read_to_string(ignore_path).unwrap();
        assert!(content.contains(".changeguard/tmp"));
        assert!(content.contains("output/"));
    }

    #[test]
    fn test_validate_gitignore_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = camino::Utf8Path::from_path(tmp.path()).unwrap();
        let ignore_path = root.join(".gitignore");
        fs::write(&ignore_path, ".changeguard/tmp\n").unwrap();

        validate_gitignore(root).unwrap();

        let content = fs::read_to_string(&ignore_path).unwrap();
        let count = content.matches(".changeguard/tmp").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_execute_update_no_flags_prints_hint() {
        // Without any flag, execute_update should print the hint and not error.
        let result = execute_update(false, false, false, false);
        assert!(result.is_ok(), "no-flag invocation should succeed");
    }
}
