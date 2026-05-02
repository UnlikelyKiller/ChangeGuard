use crate::config::load;
use crate::policy::load as policy_load;
use crate::state::layout::Layout;
use miette::Result;

pub fn execute_config_verify() -> Result<()> {
    let current_dir = std::env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {e}"))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let mut success = true;

    println!("Verifying ChangeGuard configuration...");

    // Verify config.toml
    match load::load_config(&layout) {
        Ok(_) => {
            println!("  ✅ config.toml is valid");
        }
        Err(e) => {
            println!("  ❌ config.toml is invalid:\n    {e}");
            success = false;
        }
    }

    // Verify rules.toml
    match policy_load::load_rules(&layout) {
        Ok(_) => {
            println!("  ✅ rules.toml is valid");
        }
        Err(e) => {
            println!("  ❌ rules.toml is invalid:\n    {e}");
            success = false;
        }
    }

    if success {
        println!("\nAll configurations are valid.");
        Ok(())
    } else {
        Err(miette::miette!("Configuration verification failed."))
    }
}
