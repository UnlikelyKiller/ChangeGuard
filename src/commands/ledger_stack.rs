use crate::ledger::*;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use camino::Utf8PathBuf;
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

pub fn execute_ledger_stack(category: Option<String>) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let db = LedgerDb::new(storage.get_connection());

    println!(
        "{}",
        "ChangeGuard Tech Stack & Validators".bold().underline()
    );

    let rules = db
        .get_tech_stack_rules(category.as_deref())
        .map_err(|e| miette::miette!("{}", e))?;
    println!("\n{}", "TECH STACK RULES".cyan().bold());
    if rules.is_empty() {
        println!("  None.");
    } else {
        for rule in rules {
            let locked_str = if rule.locked {
                " [LOCKED]".red().bold().to_string()
            } else {
                "".to_string()
            };
            println!(
                "  {} ({}): {}{}",
                rule.category.yellow(),
                rule.name,
                rule.status,
                locked_str
            );
            for r in rule.rules {
                println!("    - {}", r);
            }
        }
    }

    let validators = db
        .get_commit_validators(category.as_deref())
        .map_err(|e| miette::miette!("{}", e))?;
    println!("\n{}", "COMMIT VALIDATORS".magenta().bold());
    if validators.is_empty() {
        println!("  None.");
    } else {
        for v in validators {
            let enabled_str = if !v.enabled {
                " [DISABLED]".dimmed().to_string()
            } else {
                "".to_string()
            };
            println!(
                "  {} ({:?}): {} {}{}",
                v.name.yellow(),
                v.validation_level,
                v.executable,
                v.args.join(" "),
                enabled_str
            );
            if let Some(desc) = v.description {
                println!("    Description: {}", desc.dimmed());
            }
            if let Some(glob) = v.glob {
                println!("    Scope: {}", glob);
            }
        }
    }

    let mappings = db
        .get_category_mappings(category.as_deref())
        .map_err(|e| miette::miette!("{}", e))?;
    println!("\n{}", "CATEGORY MAPPINGS".blue().bold());
    if mappings.is_empty() {
        println!("  None.");
    } else {
        for m in mappings {
            println!(
                "  {} -> {}",
                m.ledger_category.yellow(),
                m.stack_category.cyan()
            );
            if let Some(desc) = m.description {
                println!("    {}", desc.dimmed());
            }
        }
    }

    Ok(())
}
