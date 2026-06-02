pub mod ask;
pub mod bridge;
pub mod config;
pub mod config_verify;
#[cfg(feature = "daemon")]
pub mod daemon;
pub mod dead_code;
pub mod doctor;
pub mod federate;
pub mod helpers;
pub mod hook_commit_msg;
pub mod hook_post_commit;
pub mod hotspots;
pub mod impact;
pub mod index;
pub mod init;
pub mod intent;
pub mod ledger;
pub mod ledger_adr;
pub mod ledger_audit;
pub mod ledger_register;
pub mod ledger_search;
pub mod ledger_stack;
pub mod reset;
pub mod scan;
pub mod search;
pub mod update;
pub mod verify;
pub mod viz;
#[cfg(feature = "viz-server")]
pub mod viz_server;
pub mod watch;

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
