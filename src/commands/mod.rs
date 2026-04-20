pub mod ask;
pub mod doctor;
pub mod federate;
pub mod hotspots;
pub mod impact;
pub mod init;
pub mod reset;
pub mod scan;
pub mod verify;
pub mod watch;
#[cfg(feature = "daemon")]
pub mod daemon;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum CommandError {
    #[error("Failed to discover repository root")]
    RepoDiscoveryFailed,

    #[error("I/O error during command execution")]
    IoError(#[from] std::io::Error),

    #[error("Verification failed: {0}")]
    Verify(String),
}
