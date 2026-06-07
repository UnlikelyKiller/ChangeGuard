use changeguard::cli::{Cli, Commands, LedgerCommands};
use clap::Parser;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse CLI args and return Ok(Cli) on success.
fn parse_ok(args: &[&str]) -> Cli {
    match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => panic!("expected Ok but got error: {e}"),
    }
}

/// Parse CLI args and return the error string on failure.
fn parse_err(args: &[&str]) -> String {
    match Cli::try_parse_from(args) {
        Ok(_cli) => panic!("expected Err but got Ok"),
        Err(e) => e.to_string(),
    }
}

fn is_ledger(cmd: &Commands) -> bool {
    matches!(cmd, Commands::Ledger { .. })
}

// ===========================================================================
// Start
// ===========================================================================
#[test]
fn test_start_success_required_only() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "start",
        "src/main.rs",
        "--category",
        "FEATURE",
        "--message",
        "msg",
    ]);
    let Commands::Ledger { command, .. } = cli.command else {
        panic!("expected Ledger command");
    };
    assert!(matches!(command, LedgerCommands::Start { entity, .. } if entity == "src/main.rs"));
}

#[test]
fn test_start_success_with_all_args() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "start",
        "src/main.rs",
        "--category",
        "BUGFIX",
        "--message",
        "fix crash",
    ]);
    let Commands::Ledger { command, .. } = cli.command else {
        panic!("expected Ledger");
    };
    let LedgerCommands::Start {
        entity,
        category,
        message,
    } = command
    else {
        panic!("expected Start");
    };
    assert_eq!(entity, "src/main.rs");
    assert_eq!(category, "BUGFIX");
    assert_eq!(message, "fix crash");
}

#[test]
fn test_start_accepts_invalid_category_for_runtime_correction() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "start",
        "src/main.rs",
        "--category",
        "doc",
        "--message",
        "update docs",
    ]);
    let Commands::Ledger { command, .. } = cli.command else {
        panic!("expected Ledger");
    };
    let LedgerCommands::Start { category, .. } = command else {
        panic!("expected Start");
    };
    assert_eq!(category, "doc");
}

#[test]
fn test_start_missing_entity() {
    let text = parse_err(&[
        "changeguard",
        "ledger",
        "start",
        "--category",
        "FEATURE",
        "--message",
        "m",
    ]);
    assert!(
        text.contains("entity") || text.contains("required"),
        "expected entity requirement: {text}"
    );
}

// ===========================================================================
// Commit
// ===========================================================================
#[test]
fn test_commit_success_required_only() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "commit",
        "--summary",
        "fix bug",
        "--reason",
        "it was broken",
    ]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_commit_accepts_git_flags() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "commit",
        "tx-123",
        "--summary",
        "fix bug",
        "--reason",
        "it was broken",
        "--with-git",
        "--git-message",
        "custom message",
        "--no-signoff",
        "--dry-run",
    ]);
    let Commands::Ledger { command, .. } = cli.command else {
        panic!("expected Ledger command");
    };
    let LedgerCommands::Commit {
        tx_id,
        with_git,
        git_message,
        no_signoff,
        dry_run,
        ..
    } = command
    else {
        panic!("expected Commit");
    };
    assert_eq!(tx_id.as_deref(), Some("tx-123"));
    assert!(with_git);
    assert_eq!(git_message.as_deref(), Some("custom message"));
    assert!(no_signoff);
    assert!(dry_run);
}

#[test]
fn test_commit_rejects_git_flags_without_with_git() {
    for flag in ["--git-message", "--no-signoff", "--dry-run"] {
        let mut args = vec![
            "changeguard",
            "ledger",
            "commit",
            "--summary",
            "fix bug",
            "--reason",
            "it was broken",
            flag,
        ];
        if flag == "--git-message" {
            args.push("custom message");
        }

        let text = parse_err(&args);
        assert!(
            text.contains("--with-git") || text.contains("with-git"),
            "expected --with-git requirement for {flag}: {text}"
        );
    }
}

// ===========================================================================
// Atomic
// ===========================================================================
#[test]
fn test_atomic_success() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "atomic",
        "src/lib.rs",
        "--category",
        "FEATURE",
        "--summary",
        "add fn",
        "--reason",
        "needed",
    ]);
    assert!(is_ledger(&cli.command));
}

// ===========================================================================
// Status
// ===========================================================================
#[test]
fn test_status_success_no_args() {
    let cli = parse_ok(&["changeguard", "ledger", "status"]);
    assert!(is_ledger(&cli.command));
}

// ===========================================================================
// Ledger Audit
// ===========================================================================
#[test]
fn test_ledger_audit_success() {
    let cli = parse_ok(&["changeguard", "ledger", "audit"]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_ledger_audit_with_json() {
    let cli = parse_ok(&["changeguard", "ledger", "audit", "--json"]);
    let Commands::Ledger { command, .. } = cli.command else {
        panic!("expected Ledger command");
    };
    assert!(matches!(command, LedgerCommands::Audit { json: true, .. }));
}

// ===========================================================================
// ADR
// ===========================================================================
#[test]
fn test_adr_success_no_args() {
    let cli = parse_ok(&["changeguard", "ledger", "adr"]);
    assert!(is_ledger(&cli.command));
}

// ===========================================================================
// Search
// ===========================================================================
#[test]
fn test_search_success() {
    let cli = parse_ok(&["changeguard", "ledger", "search", "my query"]);
    assert!(is_ledger(&cli.command));
}

// ===========================================================================
// Reconcile
// ===========================================================================
#[test]
fn test_reconcile_success() {
    let cli = parse_ok(&["changeguard", "ledger", "reconcile", "--all"]);
    assert!(is_ledger(&cli.command));
}

// ===========================================================================
// Adopt
// ===========================================================================
#[test]
fn test_adopt_success() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "adopt",
        "--all",
        "--category",
        "FEATURE",
        "--summary",
        "s",
        "--reason",
        "r",
    ]);
    assert!(is_ledger(&cli.command));
}
