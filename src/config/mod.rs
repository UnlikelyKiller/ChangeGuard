pub mod defaults;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ConfigError {
    #[error("Failed to write config file at {path}")]
    WriteFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
