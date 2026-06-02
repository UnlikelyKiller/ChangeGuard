use std::io::IsTerminal;

/// Returns true if the current environment is interactive (STDIN is a terminal
/// and no non-interactive overrides are set).
pub fn is_interactive() -> bool {
    // Check for explicit non-interactive flag
    if std::env::var("CHANGEGUARD_NON_INTERACTIVE").is_ok() {
        return false;
    }

    // Check for common CI environments
    if std::env::var("CI").is_ok() {
        return false;
    }

    // Default to checking if stdin is a terminal
    std::io::stdin().is_terminal()
}
