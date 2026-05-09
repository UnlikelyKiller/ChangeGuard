use camino::Utf8PathBuf;
use miette::{IntoDiagnostic, Result};
use std::env;
use crate::state::layout::Layout;
use crate::search::{TantivySearchEngine, StreamIndexer, RegexFilter};
use tracing::info;

pub fn execute_search(
    query: String,
    regex: bool,
    limit: usize,
    index: bool,
) -> Result<()> {
    let root = get_repo_root()?;
    let layout = Layout::new(&root);
    layout.ensure_state_dir()?;

    let index_path = layout.search_index_dir();
    let engine = TantivySearchEngine::open_or_create(index_path.as_std_path())?;

    if index || is_index_empty(&index_path) {
        info!("Indexing repository for search...");
        engine.clear()?;
        let indexer = StreamIndexer::new(engine);
        indexer.index_repository(&root)?;
        // Re-open engine to pick up new index
        let engine = TantivySearchEngine::open_or_create(index_path.as_std_path())?;
        perform_search(engine, &root, query, regex, limit)?;
    } else {
        perform_search(engine, &root, query, regex, limit)?;
    }

    Ok(())
}

fn perform_search(
    engine: TantivySearchEngine,
    root: &camino::Utf8Path,
    query: String,
    regex: bool,
    limit: usize,
) -> Result<()> {
    if regex {
        let filter = RegexFilter::new(&engine);
        let matches = filter.search(root, &query, limit)?;
        if matches.is_empty() {
            println!("No matches found.");
        } else {
            for m in matches {
                println!("{}:{}: {}", m.path, m.line_number, m.content);
            }
        }
    } else {
        let results = engine.search(&query, limit)?;
        if results.is_empty() {
            println!("No matches found.");
        } else {
            for r in results {
                println!("{} (score: {:.2})", r.path, r.score);
            }
        }
    }
    Ok(())
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

fn is_index_empty(path: &camino::Utf8Path) -> bool {
    if !path.exists() {
        return true;
    }
    std::fs::read_dir(path)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(true)
}
