pub mod ignore;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum GitError {
    #[error("Failed to read .gitignore at {path}")]
    ReadIgnoreFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write .gitignore at {path}")]
    WriteIgnoreFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
