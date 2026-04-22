use crate::ledger::db::LedgerDb;
use crate::ledger::enforcement::{
    CategoryStackMapping, CommitValidator, RuleType, TechStackRule, WatcherPattern,
};
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use camino::Utf8PathBuf;
use chrono::Utc;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;

fn get_repo_root() -> Result<Utf8PathBuf> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let discovered = gix::discover(&current_dir).into_diagnostic()?;
    let root = discovered
        .workdir()
        .ok_or_else(|| miette::miette!("Failed to find work directory for repository"))?;

    Utf8PathBuf::from_path_buf(root.to_path_buf())
        .map_err(|_| miette::miette!("Repository root is not valid UTF-8"))
}

fn get_layout() -> Result<Layout> {
    let root = get_repo_root()?;
    Ok(Layout::new(root))
}

pub fn execute_ledger_register(rule_type: RuleType, payload: String, force: bool) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let db = LedgerDb::new(storage.get_connection());

    match rule_type {
        RuleType::TechStack => {
            let mut rule: TechStackRule = serde_json::from_str(&payload)
                .map_err(|e| miette::miette!("Invalid JSON payload for TECH_STACK: {}", e))?;

            // Validation
            if rule.category.trim().is_empty() {
                return Err(miette::miette!("Category cannot be empty"));
            }
            if rule.name.trim().is_empty() {
                return Err(miette::miette!("Name cannot be empty"));
            }

            if rule.registered_at.is_empty() {
                rule.registered_at = Utc::now().to_rfc3339();
            }

            let existing = db
                .get_tech_stack_rule(&rule.category)
                .map_err(|e| miette::miette!("{}", e))?;

            if matches!(existing, Some(ref rule_info) if rule_info.locked && !force) {
                return Err(miette::miette!(
                    "Rule for category {} is locked. Use --force to override.",
                    rule.category.yellow()
                ));
            }
            db.insert_tech_stack_rule(&rule)
                .map_err(|e| miette::miette!("{}", e))?;
            println!(
                "Registered tech stack rule for category: {}",
                rule.category.cyan()
            );
        }
        RuleType::Validator => {
            let validator: CommitValidator = serde_json::from_str(&payload)
                .map_err(|e| miette::miette!("Invalid JSON payload for VALIDATOR: {}", e))?;

            // Validation
            if validator.category.trim().is_empty() {
                return Err(miette::miette!("Category cannot be empty"));
            }
            if validator.name.trim().is_empty() {
                return Err(miette::miette!("Validator name cannot be empty"));
            }
            if validator.executable.trim().is_empty() {
                return Err(miette::miette!("Executable cannot be empty"));
            }
            if validator.timeout_ms <= 0 {
                return Err(miette::miette!("timeout_ms must be positive"));
            }

            db.insert_commit_validator(&validator)
                .map_err(|e| miette::miette!("{}", e))?;
            println!("Registered commit validator: {}", validator.name.cyan());
        }
        RuleType::Mapping => {
            let mapping: CategoryStackMapping = serde_json::from_str(&payload)
                .map_err(|e| miette::miette!("Invalid JSON payload for MAPPING: {}", e))?;

            // Validation
            if mapping.ledger_category.trim().is_empty() {
                return Err(miette::miette!("ledger_category cannot be empty"));
            }
            if mapping.stack_category.trim().is_empty() {
                return Err(miette::miette!("stack_category cannot be empty"));
            }

            db.insert_category_mapping(&mapping)
                .map_err(|e| miette::miette!("{}", e))?;
            println!(
                "Registered category mapping: {} -> {}",
                mapping.ledger_category.cyan(),
                mapping.stack_category.cyan()
            );
        }
        RuleType::Watcher => {
            let pattern: WatcherPattern = serde_json::from_str(&payload)
                .map_err(|e| miette::miette!("Invalid JSON payload for WATCHER: {}", e))?;

            // Validation
            if pattern.category.trim().is_empty() {
                return Err(miette::miette!("Category cannot be empty"));
            }
            if pattern.glob.trim().is_empty() {
                return Err(miette::miette!("Watcher glob cannot be empty"));
            }

            db.insert_watcher_pattern(&pattern)
                .map_err(|e| miette::miette!("{}", e))?;
            println!("Registered watcher pattern: {}", pattern.glob.cyan());
        }
    }

    Ok(())
}
