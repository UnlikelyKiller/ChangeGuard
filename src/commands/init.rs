use crate::config::ConfigError;
use crate::config::defaults::DEFAULT_CONFIG;
use crate::git::ignore::add_to_gitignore;
use crate::policy::defaults::DEFAULT_RULES;
use crate::state::layout::Layout;
use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::fs;
use tracing::info;

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
        fs::write(&config_path, DEFAULT_CONFIG).map_err(|e| ConfigError::WriteFailed {
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

    info!("ChangeGuard initialized successfully!");
    Ok(())
}
