pub mod init;
pub mod doctor;
pub mod scan;
pub mod impact;
pub mod watch;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum CommandError {
    #[error("Failed to discover repository root")]
    RepoDiscoveryFailed,
    
    #[error("I/O error during command execution")]
    IoError(#[from] std::io::Error),
}
