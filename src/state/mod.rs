pub mod layout;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum StateError {
    #[error("Failed to create directory: {path}")]
    #[diagnostic(code(state::mkdir_failed), help("Check directory permissions at {path}"))]
    MkdirFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to resolve base directory")]
    #[diagnostic(code(state::resolve_base_failed))]
    ResolveBaseFailed,
}
