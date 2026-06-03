use crate::config::ConfigError;
use crate::config::defaults::default_config_contents;
use crate::git::ignore::add_to_gitignore;
use crate::policy::defaults::DEFAULT_RULES;
use crate::state::layout::Layout;
use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::io::Write as IoWrite;
use tracing::info;

const HOOK_MARKER: &str = "# changeguard-ledger-gate";
const HOOK_BLOCK_TEMPLATE: &str = "\
# changeguard-ledger-gate: auto-installed by `changeguard init`
if command -v changeguard &>/dev/null; then
    if ! changeguard ledger status --compact --exit-code --verify-signatures 2>/dev/null; then
        echo \"\"
        echo \"  Resolve with:\"
        echo \"    Pending tx:  changeguard ledger commit <tx-id> --summary '...' --reason '...'\"
        echo \"    Drift:       changeguard ledger reconcile --all --reason '...'\"
        echo \"\"
        echo \"  Bypass (not recommended): {bypass_command}\"
        exit 1
    fi
fi
";

fn install_git_hook(root: &Utf8PathBuf, hook_name: &str, bypass_command: &str) -> Result<bool> {
    let git_dir = root.join(".git");
    if !git_dir.exists() {
        return Ok(false);
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).into_diagnostic()?;

    let hook_path = hooks_dir.join(hook_name);
    let hook_block = HOOK_BLOCK_TEMPLATE.replace("{bypass_command}", bypass_command);

    // Idempotent: skip if our block is already present
    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path).into_diagnostic()?;
        if existing.contains(HOOK_MARKER) {
            return Ok(false);
        }
        // Append to existing hook
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&hook_path)
            .into_diagnostic()?;
        let block = format!("\n{}\n", hook_block);
        file.write_all(block.as_bytes()).into_diagnostic()?;
    } else {
        // Create new hook with shebang
        let content = format!("#!/usr/bin/env bash\n\n{}\n", hook_block);
        fs::write(&hook_path, content).into_diagnostic()?;
        // Set executable bit on Unix; no-op on Windows
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path).into_diagnostic()?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms).into_diagnostic()?;
        }
    }

    Ok(true)
}

const COMMIT_MSG_MARKER: &str = "# changeguard-intent-gate";
const COMMIT_MSG_HOOK_TEMPLATE: &str = "\
# changeguard-intent-gate: auto-installed by `changeguard init`
if command -v changeguard &>/dev/null; then
    changeguard internal hook-commit-msg \"$1\"
fi
";

const POST_COMMIT_MARKER: &str = "# changeguard-post-commit-gate";
const POST_COMMIT_HOOK_TEMPLATE: &str = "\
# changeguard-post-commit-gate: auto-installed by `changeguard init`
if command -v changeguard &>/dev/null; then
    changeguard internal hook-post-commit \"$@\"
fi
";

fn install_commit_msg_hook(root: &Utf8PathBuf) -> Result<bool> {
    let git_dir = root.join(".git");
    if !git_dir.exists() {
        return Ok(false);
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).into_diagnostic()?;

    let hook_path = hooks_dir.join("commit-msg");

    // Idempotent: skip if our block is already present
    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path).into_diagnostic()?;
        if existing.contains(COMMIT_MSG_MARKER) {
            return Ok(false);
        }
        // Append to existing hook
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&hook_path)
            .into_diagnostic()?;
        let block = format!("\n{}\n", COMMIT_MSG_HOOK_TEMPLATE);
        file.write_all(block.as_bytes()).into_diagnostic()?;
    } else {
        // Create new hook with shebang
        let content = format!("#!/usr/bin/env bash\n\n{}\n", COMMIT_MSG_HOOK_TEMPLATE);
        fs::write(&hook_path, content).into_diagnostic()?;
        // Set executable bit on Unix; no-op on Windows
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path).into_diagnostic()?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms).into_diagnostic()?;
        }
    }

    Ok(true)
}

fn install_post_commit_hook(root: &Utf8PathBuf) -> Result<bool> {
    let git_dir = root.join(".git");
    if !git_dir.exists() {
        return Ok(false);
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).into_diagnostic()?;

    let hook_path = hooks_dir.join("post-commit");

    // Idempotent: skip if our block is already present
    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path).into_diagnostic()?;
        if existing.contains(POST_COMMIT_MARKER) {
            return Ok(false);
        }
        // Append to existing hook
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&hook_path)
            .into_diagnostic()?;
        let block = format!("\n{}\n", POST_COMMIT_HOOK_TEMPLATE);
        file.write_all(block.as_bytes()).into_diagnostic()?;
    } else {
        // Create new hook with shebang
        let content = format!("#!/usr/bin/env bash\n\n{}\n", POST_COMMIT_HOOK_TEMPLATE);
        fs::write(&hook_path, content).into_diagnostic()?;
        // Set executable bit on Unix; no-op on Windows
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path).into_diagnostic()?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms).into_diagnostic()?;
        }
    }

    Ok(true)
}

fn install_ledger_gate_hooks(root: &Utf8PathBuf) -> Result<Vec<&'static str>> {
    let mut installed = Vec::new();

    if install_git_hook(root, "pre-commit", "git commit --no-verify")? {
        installed.push("pre-commit");
    }

    if install_git_hook(root, "pre-push", "git push --no-verify")? {
        installed.push("pre-push");
    }

    if install_commit_msg_hook(root)? {
        installed.push("commit-msg");
    }

    if install_post_commit_hook(root)? {
        installed.push("post-commit");
    }

    Ok(installed)
}

pub fn execute_init(no_gitignore: bool) -> Result<()> {
    // 1. Discover repository root
    let root = match gix::discover(".") {
        Ok(repo) => {
            let path = repo
                .workdir()
                .ok_or(crate::commands::CommandError::RepoDiscoveryFailed)?
                .to_path_buf();
            info!("Discovered git repository root at: {:?}", path);
            Utf8PathBuf::from_path_buf(path)
                .map_err(|_| crate::commands::CommandError::RepoDiscoveryFailed)?
        }
        Err(e) => {
            info!(
                "gix::discover failed: {:?}. Using current directory as root",
                e
            );
            Utf8PathBuf::from_path_buf(std::env::current_dir().into_diagnostic()?)
                .map_err(|_| crate::commands::CommandError::RepoDiscoveryFailed)?
        }
    };

    info!("Resolved root for initialization: {}", root);
    let layout = Layout::new(&root);

    // 2. Ensure directory layout
    layout.ensure_state_dir()?;

    // 3. Generate starter configurations
    let config_path = layout.config_file();
    if !config_path.exists() {
        let config_contents = default_config_contents()?;
        fs::write(&config_path, config_contents).map_err(|e| ConfigError::WriteFailed {
            path: config_path.to_string(),
            source: e,
        })?;
        info!("Created starter config at {}", config_path);
    }

    let rules_path = layout.rules_file();
    if !rules_path.exists() {
        fs::write(&rules_path, DEFAULT_RULES).map_err(|e| ConfigError::WriteFailed {
            path: rules_path.to_string(),
            source: e,
        })?;
        info!("Created starter rules at {}", rules_path);
    }

    // 4. Update .gitignore
    if !no_gitignore {
        let changed = add_to_gitignore(&root, ".changeguard/")?;
        if changed {
            info!("Added .changeguard/ to .gitignore");
        }
    }

    // 5. Install Git ledger gate hooks
    match install_ledger_gate_hooks(&root) {
        Ok(installed) if !installed.is_empty() => {
            println!(
                "Installed ChangeGuard ledger gate hooks: {}.",
                installed.join(", ")
            );
        }
        Ok(_) => {}
        Err(e) => eprintln!("Warning: could not install Git ledger gate hooks: {e}"),
    }

    // 6. Initialize ledger storage database
    let db_path = layout.state_subdir().join("ledger.db");
    crate::state::storage::StorageManager::init(db_path.as_std_path())?;

    info!("ChangeGuard initialized successfully!");
    Ok(())
}
