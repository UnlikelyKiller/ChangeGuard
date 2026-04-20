pub mod types;
pub mod python;
pub mod rust;
pub mod typescript;

pub use self::types::Language;
use crate::index::symbols::Symbol;
use miette::Result;
use std::path::Path;

pub fn parse_symbols(path: &Path, content: &str) -> Result<Option<Vec<Symbol>>> {
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match extension {
        "rs" => rust::extract_symbols(content),
        "ts" | "tsx" | "js" | "jsx" => typescript::extract_symbols(content),
        "py" => python::extract_symbols(content),
        _ => Ok(None),
    }
}
