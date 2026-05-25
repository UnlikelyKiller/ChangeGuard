use crate::git::ignore::add_to_gitignore;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;
use std::process::Command;
use sysinfo::System;
use tracing::info;

const IGNORE_PATTERNS: &[&str] = &[".changeguard/"];

pub fn execute_update(migrate: bool, binary: bool, force: bool, force_unlock: bool) -> Result<()> {
    if force_unlock {
        force_unlock_cozo()?;
    }
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

    let root = get_repo_root()?;
    let cargo_toml = root.as_std_path().join("Cargo.toml");

    if !cargo_toml.exists() {
        println!("{}", "Not in a ChangeGuard source repository. Binary update via CLI is only supported from the source root.".yellow());
        println!("Try running: cargo install --git https://github.com/UnlikelyKiller/ChangeGuard");
        return Ok(());
    }

    info!("Detected local source repository. Running 'cargo install --path .'");

    // --- H4: Windows shadow-copy ---
    // On Windows the running executable is locked by the OS. We rename it to
    // `<name>.old` so that the file handle is still valid (Windows allows
    // renaming locked files) but the path is free for the new binary.
    let old_path_opt = shadow_copy_current_exe();
    if let Some(ref old_path) = old_path_opt {
        info!("Shadow-copied running binary to {:?}", old_path);
    }

    let status = Command::new("cargo")
        .args(["install", "--path", root.as_str()])
        .status()
        .into_diagnostic()?;

    if status.success() {
        println!("{}", "Binary updated successfully.".green());

        // Attempt to clean up the .old file now that the new binary is in place.
        if let Some(ref old_path) = old_path_opt {
            if let Err(_e) = std::fs::remove_file(old_path) {
                // On Windows, deleting the running shadow copy usually fails with Access Denied.
                // We silently ignore it here because `sweep_stale_old_binaries()` in main.rs
                // will clean it up on the next execution.
                #[cfg(not(target_os = "windows"))]
                println!(
                    "{}",
                    format!(
                        "[CLEANUP] Could not remove old binary {:?}: {}. You may delete it manually.",
                        old_path, _e
                    )
                    .yellow()
                );
            } else {
                info!("Removed stale binary {:?}", old_path);
            }
        }
    } else {
        // Restore the old binary on failure so the user is not left without a working binary.
        if let Some(ref old_path) = old_path_opt
            && old_path.exists()
            && let Ok(cur) = env::current_exe()
        {
            let _ = std::fs::rename(old_path, &cur);
        }
        return Err(miette::miette!(
            "Failed to update binary via cargo install.\n\
             If you see 'Access is denied', close any other running ChangeGuard processes and retry."
        ));
    }

    Ok(())
}

/// On Windows, rename `current_exe` to `<same_path>.old` so that `cargo install`
/// can write the new binary without hitting the OS file-lock.
/// Returns `Some(old_path)` on success, `None` on non-Windows or on error.
fn shadow_copy_current_exe() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let current = env::current_exe().ok()?;
        let mut old_path = current.clone();
        let stem = current
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("changeguard");
        let ext = current
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("exe");

        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        old_path.set_file_name(format!("{stem}.old.{timestamp}.{ext}"));

        if let Err(e) = std::fs::rename(&current, &old_path) {
            tracing::warn!("Failed to shadow-copy binary to {:?}: {}", old_path, e);
            return None;
        }
        Some(old_path)
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
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

    layout.ensure_state_dir()?;

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

    // Step 0: Shutdown any existing handles to the database or KG
    let db_path = layout.state_subdir().join("ledger.db");
    if db_path.exists()
        && let Ok(storage) = StorageManager::init(db_path.as_std_path())
    {
        let _ = storage.shutdown();
    }

    layout.ensure_state_dir()?;

    // Wipe Knowledge Graph (CozoDB sqlite store)
    let cozo_path = layout.state_subdir().join("ledger.cozo");
    if cozo_path.exists() {
        info!("Removing Knowledge Graph at {:?}", cozo_path);
        if cozo_path.is_dir() {
            robust_remove_dir(cozo_path.as_std_path())?;
        } else {
            std::fs::remove_file(&cozo_path).into_diagnostic()?;
        }
        println!("{}", "Knowledge Graph cleared.".green());
    }

    // Wipe Tantivy search index
    let search_dir = layout.search_index_dir();
    if search_dir.exists() {
        info!("Removing search index at {:?}", search_dir);
        robust_remove_dir(search_dir.as_std_path())?;
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

/// A utility to retry directory removal on Windows, where anti-virus or lingering
/// mapped memory can cause short-lived "Permission Denied" locks.
fn robust_remove_dir(path: &std::path::Path) -> Result<()> {
    let mut attempts = 0;
    let max_attempts = 10;
    let delay = std::time::Duration::from_millis(100);

    loop {
        match std::fs::remove_dir_all(path) {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(miette::miette!(
                        "Failed to remove directory {:?} after {} attempts: {}",
                        path,
                        max_attempts,
                        e
                    ));
                }
                std::thread::sleep(delay);
            }
        }
    }
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

fn force_unlock_cozo() -> Result<()> {
    println!(
        "{}",
        "Attempting to release CozoDB file locks by terminating other ChangeGuard processes..."
            .cyan()
    );

    let mut sys = System::new_all();
    sys.refresh_all();

    let current_pid = std::process::id();
    let mut killed_count = 0;

    for (pid, process) in sys.processes() {
        let name_str = process.name().to_string_lossy();
        let name = name_str.to_lowercase();
        if (name.contains("changeguard") || name.contains("changeguard.exe"))
            && pid.as_u32() != current_pid
        {
            println!(
                "Found running ChangeGuard process: PID {} ({}) - Terminating...",
                pid, name_str
            );
            process.kill();
            killed_count += 1;
        }
    }

    if killed_count > 0 {
        println!(
            "{}",
            format!(
                "Successfully terminated {} background process(es).",
                killed_count
            )
            .green()
        );
    } else {
        println!(
            "No other running ChangeGuard processes found. File lock might be held by OS filesystem cache or another tool."
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_get_repo_root_ok() {
        let root = get_repo_root().unwrap();
        assert!(
            root.as_str().to_ascii_lowercase().contains("changeguard"),
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
        let result = execute_update(false, false, false, false);
        assert!(result.is_ok(), "no-flag invocation should succeed");
    }
}
