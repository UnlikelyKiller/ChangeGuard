use crate::state::layout::Layout;
use camino::{Utf8Path, Utf8PathBuf};
use miette::{Result, miette};
use std::env;
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
enum RemovalOutcome {
    Removed,
    Absent,
    Preserved,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResetItem {
    path: Utf8PathBuf,
    outcome: RemovalOutcome,
}

pub fn execute_reset(
    remove_config: bool,
    remove_rules: bool,
    remove_all: bool,
    confirm_destructive: bool,
) -> Result<()> {
    if (remove_all || remove_config || remove_rules) && !confirm_destructive {
        return Err(miette!(
            "Destructive reset options require confirmation. Re-run with '--yes'."
        ));
    }

    let current_dir = env::current_dir()
        .map_err(|e| miette!("Failed to get current directory: {e}"))?;
    let root = Utf8PathBuf::from_path_buf(current_dir)
        .map_err(|path| miette!("Current directory is not valid UTF-8: {:?}", path))?;
    let layout = Layout::new(root.as_str());

    let mut items = if remove_all {
        vec![remove_path(layout.state_dir.clone(), &layout.state_dir)]
    } else {
        default_reset_items(&layout, remove_config, remove_rules)
    };
    items.sort_by(|a, b| a.path.cmp(&b.path));

    print_summary(&items);

    let failures: Vec<String> = items
        .iter()
        .filter_map(|item| match &item.outcome {
            RemovalOutcome::Failed(reason) => Some(format!("{}: {}", item.path, reason)),
            _ => None,
        })
        .collect();

    if failures.is_empty() {
        Ok(())
    } else {
        Err(miette!(
            "Reset completed with failures:\n{}",
            failures.join("\n")
        ))
    }
}

fn default_reset_items(
    layout: &Layout,
    remove_config: bool,
    remove_rules: bool,
) -> Vec<ResetItem> {
    let mut items = vec![
        remove_path(layout.logs_dir(), &layout.state_dir),
        remove_path(layout.tmp_dir(), &layout.state_dir),
        remove_path(layout.reports_dir(), &layout.state_dir),
        remove_path(layout.state_subdir(), &layout.state_dir),
    ];

    items.push(maybe_preserve_or_remove(
        layout.config_file(),
        &layout.state_dir,
        remove_config,
    ));
    items.push(maybe_preserve_or_remove(
        layout.rules_file(),
        &layout.state_dir,
        remove_rules,
    ));

    items
}

fn maybe_preserve_or_remove(path: Utf8PathBuf, state_root: &Utf8Path, should_remove: bool) -> ResetItem {
    if should_remove {
        remove_path(path, state_root)
    } else {
        ResetItem {
            path,
            outcome: RemovalOutcome::Preserved,
        }
    }
}

fn remove_path(path: Utf8PathBuf, state_root: &Utf8Path) -> ResetItem {
    let outcome = match validate_target(&path, state_root) {
        Err(err) => RemovalOutcome::Failed(err),
        Ok(()) => match fs::symlink_metadata(&path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => RemovalOutcome::Absent,
            Err(err) => RemovalOutcome::Failed(err.to_string()),
            Ok(metadata) => {
                if let Err(err) = clear_readonly_recursive(&path) {
                    RemovalOutcome::Failed(err.to_string())
                } else {
                    let result = if metadata.file_type().is_dir() {
                        fs::remove_dir_all(&path)
                    } else {
                        fs::remove_file(&path)
                    };

                    match result {
                        Ok(()) => RemovalOutcome::Removed,
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                            RemovalOutcome::Absent
                        }
                        Err(err) => RemovalOutcome::Failed(err.to_string()),
                    }
                }
            }
        },
    };

    ResetItem { path, outcome }
}

fn validate_target(path: &Utf8Path, state_root: &Utf8Path) -> std::result::Result<(), String> {
    if path == state_root || path.starts_with(state_root) {
        Ok(())
    } else {
        Err(format!("refusing to remove path outside {}", state_root))
    }
}

fn clear_readonly_recursive(path: &Utf8Path) -> std::io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };

    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    let mut permissions = metadata.permissions();
    if permissions.readonly() {
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions)?;
    }

    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let child = Utf8PathBuf::from_path_buf(entry.path())
                .map_err(|_| std::io::Error::other("encountered non-UTF-8 path during reset"))?;
            clear_readonly_recursive(&child)?;
        }
    }

    Ok(())
}

fn print_summary(items: &[ResetItem]) {
    for item in items {
        let label = match &item.outcome {
            RemovalOutcome::Removed => "removed",
            RemovalOutcome::Absent => "absent",
            RemovalOutcome::Preserved => "preserved",
            RemovalOutcome::Failed(_) => "failed",
        };
        println!("{label}: {}", item.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_target_rejects_outside_path() {
        let state_root = Utf8Path::new("repo/.changeguard");
        let other = Utf8Path::new("repo/src");
        assert!(validate_target(other, state_root).is_err());
    }
}
