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
    let cli = parse_ok(&["changeguard", "ledger", "start", "src/main.rs"]);
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
        "--issue",
        "JIRA-123",
    ]);
    let Commands::Ledger { command, .. } = cli.command else {
        panic!("expected Ledger");
    };
    let LedgerCommands::Start {
        entity,
        message,
        issue,
        ..
    } = command
    else {
        panic!("expected Start");
    };
    assert_eq!(entity, "src/main.rs");
    assert!(message.as_deref() == Some("fix crash"));
    assert!(issue.as_deref() == Some("JIRA-123"));
}

#[test]
fn test_start_missing_entity() {
    let text = parse_err(&["changeguard", "ledger", "start"]);
    assert!(
        text.contains("entity") || text.contains("required"),
        "expected entity requirement: {text}"
    );
}

#[test]
fn test_start_extra_positionals() {
    let text = parse_err(&["changeguard", "ledger", "start", "a", "b"]);
    assert!(
        text.contains("unexpected") || text.contains("trailing") || text.contains("found"),
        "expected rejection of extra positional: {text}"
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
        "abc12345",
        "--summary",
        "fix bug",
        "--reason",
        "it was broken",
    ]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_commit_missing_summary() {
    let text = parse_err(&["changeguard", "ledger", "commit", "abc123", "--reason", "r"]);
    assert!(text.contains("summary"), "expected summary required");
}

#[test]
fn test_commit_missing_reason() {
    let text = parse_err(&[
        "changeguard",
        "ledger",
        "commit",
        "abc123",
        "--summary",
        "s",
    ]);
    assert!(text.contains("reason"), "expected reason required");
}

// ===========================================================================
// Rollback
// ===========================================================================
#[test]
fn test_rollback_success() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "rollback",
        "abc123",
        "--reason",
        "wrong",
    ]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_rollback_missing_reason() {
    let text = parse_err(&["changeguard", "ledger", "rollback", "abc123"]);
    assert!(text.contains("reason"), "expected reason required");
}

// ===========================================================================
// Reconcile
// ===========================================================================
#[test]
fn test_reconcile_success_with_tx_id() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "reconcile",
        "--tx-id",
        "abc",
        "--reason",
        "r",
    ]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_reconcile_success_with_all() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "reconcile",
        "--all",
        "--reason",
        "cleanup",
    ]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_reconcile_missing_reason() {
    let text = parse_err(&["changeguard", "ledger", "reconcile", "--all"]);
    assert!(text.contains("reason"), "expected reason required");
}

// ===========================================================================
// Adopt   (reason is now required per schema rule)
// ===========================================================================
#[test]
fn test_adopt_success() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "adopt",
        "--all",
        "--reason",
        "adopting",
    ]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_adopt_missing_reason() {
    let text = parse_err(&["changeguard", "ledger", "adopt", "--all"]);
    assert!(
        text.contains("reason"),
        "expected reason to be required for adopt"
    );
}

// ===========================================================================
// Atomic
// ===========================================================================
#[test]
fn test_atomic_success_required_only() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "atomic",
        "src/lib.rs",
        "--summary",
        "add fn",
        "--reason",
        "needed",
    ]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_atomic_missing_summary() {
    let text = parse_err(&["changeguard", "ledger", "atomic", "entity", "--reason", "r"]);
    assert!(text.contains("summary"));
}

#[test]
fn test_atomic_missing_reason() {
    let text = parse_err(&[
        "changeguard",
        "ledger",
        "atomic",
        "entity",
        "--summary",
        "s",
    ]);
    assert!(text.contains("reason"));
}

// ===========================================================================
// Note   (entity positional + --message flag required; old positional deprecated)
// ===========================================================================
#[test]
fn test_note_success_with_message_flag() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "note",
        "src/main.rs",
        "--message",
        "something learned",
    ]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_note_parses_with_neither_flag() {
    // During deprecation grace period, both --message and positional note are optional
    // at the clap level. The handler enforces that at least one is provided.
    let cli = parse_ok(&["changeguard", "ledger", "note", "src/main.rs"]);
    let Commands::Ledger { command, .. } = cli.command else {
        panic!("expected Ledger");
    };
    let LedgerCommands::Note {
        entity,
        message,
        note: deprecated_note,
    } = command
    else {
        panic!("expected Note");
    };
    assert_eq!(entity, "src/main.rs");
    assert!(message.is_none());
    assert!(deprecated_note.is_none());
}

#[test]
fn test_note_extra_positional_accepted_during_deprecation() {
    // During deprecation grace period, extra positionals are accepted as the
    // deprecated `note` field.
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "note",
        "src/main.rs",
        "--message",
        "msg",
        "extra",
    ]);
    let Commands::Ledger { command, .. } = cli.command else {
        panic!("expected Ledger");
    };
    let LedgerCommands::Note {
        entity,
        message,
        note: deprecated_note,
    } = command
    else {
        panic!("expected Note");
    };
    assert_eq!(entity, "src/main.rs");
    assert_eq!(message.as_deref(), Some("msg"));
    assert_eq!(deprecated_note.as_deref(), Some("extra"));
}

#[test]
fn test_note_deprecated_positional_accepted() {
    // During deprecation grace period, the old positional form should still parse.
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "note",
        "src/main.rs",
        "old-style note",
    ]);
    let Commands::Ledger { command, .. } = cli.command else {
        panic!("expected Ledger");
    };
    let LedgerCommands::Note {
        entity,
        message,
        note: deprecated_note,
    } = command
    else {
        panic!("expected Note");
    };
    assert_eq!(entity, "src/main.rs");
    // The new --message flag should be unset when using deprecated positional
    assert!(message.is_none());
    // The deprecated positional value is in the `note` field
    assert_eq!(deprecated_note.as_deref(), Some("old-style note"));
}

// ===========================================================================
// Status
// ===========================================================================
#[test]
fn test_status_success_no_args() {
    let cli = parse_ok(&["changeguard", "ledger", "status"]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_status_success_with_entity() {
    let cli = parse_ok(&["changeguard", "ledger", "status", "--entity", "src/lib.rs"]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_status_compact_and_exit_code() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "status",
        "--compact",
        "--exit-code",
    ]);
    assert!(is_ledger(&cli.command));
}

// ===========================================================================
// Resume
// ===========================================================================
#[test]
fn test_resume_success_no_args() {
    let cli = parse_ok(&["changeguard", "ledger", "resume"]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_resume_success_with_tx_id() {
    let cli = parse_ok(&["changeguard", "ledger", "resume", "abc12345"]);
    assert!(is_ledger(&cli.command));
}

// ===========================================================================
// Register
// ===========================================================================
#[test]
fn test_register_success() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "register",
        "--rule-type",
        "TECH_STACK",
        "--payload",
        r#"{"key":"val"}"#,
    ]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_register_missing_rule_type() {
    let text = parse_err(&["changeguard", "ledger", "register", "--payload", r#"{}"#]);
    assert!(text.contains("rule-type"));
}

#[test]
fn test_register_missing_payload() {
    let text = parse_err(&[
        "changeguard",
        "ledger",
        "register",
        "--rule-type",
        "TECH_STACK",
    ]);
    assert!(text.contains("payload"));
}

// ===========================================================================
// Stack
// ===========================================================================
#[test]
fn test_stack_success_no_args() {
    let cli = parse_ok(&["changeguard", "ledger", "stack"]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_stack_success_with_category() {
    let cli = parse_ok(&["changeguard", "ledger", "stack", "--category", "security"]);
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
fn test_ledger_audit_with_entity() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "audit",
        "--entity",
        "src/main.rs",
        "--include-unaudited",
    ]);
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

#[test]
fn test_adr_with_days() {
    let cli = parse_ok(&["changeguard", "ledger", "adr", "--days", "30"]);
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

#[test]
fn test_search_with_filters() {
    let cli = parse_ok(&[
        "changeguard",
        "ledger",
        "search",
        "panic",
        "--category",
        "BUGFIX",
        "--days",
        "7",
        "--breaking",
        "--limit",
        "10",
    ]);
    assert!(is_ledger(&cli.command));
}

#[test]
fn test_search_missing_query() {
    let text = parse_err(&["changeguard", "ledger", "search"]);
    assert!(text.contains("QUERY"), "expected query required");
}

// ===========================================================================
// --help snapshot tests: each subcommand help includes key indicators
// ===========================================================================
#[test]
fn test_help_start_mentions_entity_and_category() {
    let h = parse_err(&["changeguard", "ledger", "start", "--help"]);
    assert!(h.contains("entity"));
    assert!(h.contains("--category"));
}

#[test]
fn test_help_commit_mentions_required_flags() {
    let h = parse_err(&["changeguard", "ledger", "commit", "--help"]);
    assert!(h.contains("TX_ID"));
    assert!(h.contains("--summary"));
    assert!(h.contains("--reason"));
}

#[test]
fn test_help_rollback_mentions_reason() {
    let h = parse_err(&["changeguard", "ledger", "rollback", "--help"]);
    assert!(h.contains("--reason"));
}

#[test]
fn test_help_reconcile_mentions_reason() {
    let h = parse_err(&["changeguard", "ledger", "reconcile", "--help"]);
    assert!(h.contains("--reason"));
    assert!(h.contains("--all"));
}

#[test]
fn test_help_adopt_mentions_reason_required() {
    let h = parse_err(&["changeguard", "ledger", "adopt", "--help"]);
    assert!(h.contains("--reason"));
}

#[test]
fn test_help_note_mentions_message_flag() {
    let h = parse_err(&["changeguard", "ledger", "note", "--help"]);
    assert!(h.contains("entity"));
    assert!(h.contains("--message"));
}

#[test]
fn test_help_atomic_mentions_summary_reason() {
    let h = parse_err(&["changeguard", "ledger", "atomic", "--help"]);
    assert!(h.contains("--summary"));
    assert!(h.contains("--reason"));
}

#[test]
fn test_help_status_mentions_options() {
    let h = parse_err(&["changeguard", "ledger", "status", "--help"]);
    assert!(h.contains("--compact"));
    assert!(h.contains("--exit-code"));
}

#[test]
fn test_help_resume_mentions_tx_id() {
    let h = parse_err(&["changeguard", "ledger", "resume", "--help"]);
    assert!(h.contains("TX_ID"));
}

#[test]
fn test_help_register_mentions_rule_type() {
    let h = parse_err(&["changeguard", "ledger", "register", "--help"]);
    assert!(h.contains("--rule-type"));
}

#[test]
fn test_help_search_mentions_query() {
    let h = parse_err(&["changeguard", "ledger", "search", "--help"]);
    assert!(h.contains("QUERY"));
    assert!(h.contains("--limit"));
}

// ===========================================================================
// LedgerGlobalOpts: --dry-run is accepted before the subcommand
// ===========================================================================
#[test]
fn test_ledger_global_dry_run_flag() {
    let cli = parse_ok(&["changeguard", "ledger", "--dry-run", "status"]);
    assert!(is_ledger(&cli.command));
}
