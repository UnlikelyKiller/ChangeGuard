use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum LedgerError {
    #[error("Database error: {0}")]
    #[diagnostic(code(ledger::db_error))]
    Database(#[from] rusqlite::Error),

    #[error("Entity '{0}' already has a PENDING transaction")]
    #[diagnostic(
        code(ledger::conflict),
        help("Commit or rollback the existing transaction first.")
    )]
    Conflict(String),

    #[error("Transaction '{0}' not found")]
    #[diagnostic(code(ledger::not_found))]
    NotFound(String),

    #[error("Transaction '{0}' is already {1}")]
    #[diagnostic(code(ledger::invalid_state))]
    InvalidState(String, String),

    #[error("Category '{0}' requires verification")]
    #[diagnostic(
        code(ledger::verification_required),
        help("Run 'changeguard verify' or provide verification status/basis.")
    )]
    VerificationRequired(String),

    #[error("Empty entity path is not allowed")]
    #[diagnostic(code(ledger::empty_entity))]
    EmptyEntity,

    #[error("IO error: {0}")]
    #[diagnostic(code(ledger::io_error))]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    #[diagnostic(code(ledger::config_error))]
    Config(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_formatting() {
        let err = LedgerError::Conflict("src/main.rs".to_string());
        assert_eq!(
            format!("{}", err),
            "Entity 'src/main.rs' already has a PENDING transaction"
        );

        let err = LedgerError::NotFound("abc-123".to_string());
        assert_eq!(format!("{}", err), "Transaction 'abc-123' not found");
    }
}
