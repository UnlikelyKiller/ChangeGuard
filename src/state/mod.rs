pub mod layout;
pub mod locks;
pub mod migrations;
pub mod reports;
pub mod storage;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum StateError {
    #[error("Failed to create directory: {path}")]
    #[diagnostic(
        code(state::mkdir_failed),
        help("Check directory permissions at {path}")
    )]
    MkdirFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to resolve base directory")]
    #[diagnostic(code(state::resolve_base_failed))]
    ResolveBaseFailed,

    #[error("Failed to write report: {path}")]
    #[diagnostic(
        code(state::write_report_failed),
        help("Check directory permissions at {path}")
    )]
    WriteReportFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Storage error: {0}")]
    Storage(String),
}
