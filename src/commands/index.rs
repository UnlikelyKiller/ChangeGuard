use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::env;

use crate::index::project_index::ProjectIndexer;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;

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

pub fn execute_index(incremental: bool, check: bool, json: bool) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path())?;
    let repo_path = layout.root.clone();

    let mut indexer = ProjectIndexer::new(storage, repo_path);

    if check {
        let status = indexer.check_status()?;
        if json {
            let output = serde_json::to_string_pretty(&status).into_diagnostic()?;
            println!("{}", output);
        } else {
            println!("Index Status:");
            println!("  Files indexed:   {}", status.total_files);
            println!("  Symbols indexed: {}", status.total_symbols);
            println!("  Stale files:     {}", status.stale_files);
            if let Some(last) = &status.last_indexed_at {
                println!("  Last indexed:    {}", last);
            } else {
                println!("  Last indexed:     never");
            }
        }
        if status.stale_files > 0 {
            std::process::exit(1);
        }
        return Ok(());
    }

    let stats = if incremental {
        indexer.incremental_index()?
    } else {
        indexer.full_index()?
    };

    // Index documentation files
    let doc_stats = indexer.index_docs()?;

    if json {
        let mut output = serde_json::to_value(&stats).into_diagnostic()?;
        let doc_obj = serde_json::to_value(&doc_stats).into_diagnostic()?;
        if let (Some(map), Some(doc)) = (output.as_object_mut(), doc_obj.as_object()) {
            for (k, v) in doc {
                map.insert(format!("doc_{}", k), v.clone());
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&output).into_diagnostic()?
        );
    } else {
        println!("Indexing complete:");
        println!("  Files indexed:   {}", stats.files_indexed);
        println!("  Symbols indexed: {}", stats.symbols_indexed);
        if stats.parse_failures > 0 {
            println!("  Parse failures:  {}", stats.parse_failures);
        }
        if stats.skipped_binary > 0 {
            println!("  Skipped binary:  {}", stats.skipped_binary);
        }
        if stats.skipped_unsupported > 0 {
            println!("  Skipped unsupported: {}", stats.skipped_unsupported);
        }
        println!("  Duration:        {}ms", stats.duration_ms);
        println!();
        println!("Documentation:");
        println!("  Docs indexed:    {}", doc_stats.docs_indexed);
        if doc_stats.parse_failures > 0 {
            println!("  Doc parse failures: {}", doc_stats.parse_failures);
        }
        if doc_stats.missing_readme {
            println!("  README:          not found");
        } else {
            println!("  README:          found");
        }
    }

    Ok(())
}
