pub mod rust;
pub mod typescript;
pub mod python;

use std::path::Path;
use miette::Result;
use crate::index::symbols::Symbol;

pub fn parse_symbols(path: &Path, content: &str) -> Result<Option<Vec<Symbol>>> {
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match extension {
        "rs" => rust::extract_symbols(content),
        "ts" | "tsx" | "js" | "jsx" => typescript::extract_symbols(content),
        "py" => python::extract_symbols(content),
        _ => Ok(None),
    }
}
