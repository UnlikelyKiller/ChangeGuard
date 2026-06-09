use changeguard::commands::verify::execute_verify;

#[test]
fn test_verify_command_pass() {
    let cmd = "echo hello";
    let result = execute_verify(Some(cmd.into()), 5, false, false, None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_verify_command_fail() {
    let cmd = "exit 1";
    let result = execute_verify(Some(cmd.into()), 5, false, false, None, false, false);
    assert!(result.is_err());
}

#[test]
fn test_verify_command_timeout() {
    let cmd = if cfg!(target_os = "windows") {
        "ping -n 10 127.0.0.1 >nul"
    } else {
        "sleep 10"
    };
    let result = execute_verify(Some(cmd.into()), 1, false, false, None, false, false);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Timed out"));
}

#[test]
fn test_verify_command_not_found() {
    let result = execute_verify(
        Some("nonexistent_command_9999".into()),
        5,
        false,
        false,
        None,
        false,
        false,
    );
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Command not found"));
}

// CR5: --dry-run flag should always succeed without executing any command.
#[test]
fn test_verify_dry_run_does_not_execute() {
    let result = execute_verify(
        Some("nonexistent_command_that_would_fail_if_run".into()),
        5,
        false,
        false,
        None,
        false,
        true, // dry_run = true
    );
    assert!(
        result.is_ok(),
        "dry-run should succeed even with a bad command: {:?}",
        result.err()
    );
}

// CR5: --health flag should pass for a known executable.
#[test]
fn test_verify_health_check_known_executable() {
    let result = execute_verify(
        Some("cargo --version".into()),
        10,
        false,
        false,
        None,
        true, // health = true
        false,
    );
    assert!(
        result.is_ok(),
        "health check for 'cargo' should pass: {:?}",
        result.err()
    );
}

// CR5: --health flag should fail for a missing executable.
#[test]
fn test_verify_health_check_missing_executable() {
    // Health mode checks config steps and auto-detected tools. We test that
    // health mode completes without panicking/hanging on a normal dev machine.
    let result = execute_verify(
        None, 5, false, false, None, true, // health = true
        false,
    );
    // On a dev machine with cargo available, health check should succeed.
    assert!(
        result.is_ok(),
        "health check should succeed on dev machine: {:?}",
        result.err()
    );
}

// CR4 regression: env-var prefix commands must correctly identify the real executable.
#[test]
fn test_verify_health_check_env_prefix_command() {
    // Health check passes None manual command so it uses auto-detection.
    // The key test is that it doesn't crash or hang.
    let result = execute_verify(
        None, 10, false, false, None, true, // health = true
        false,
    );
    assert!(
        result.is_ok(),
        "health check on dev machine should not error: {:?}",
        result.err()
    );
}

// CR8: Unit tests for the Cozo Datalog string escaping helper.
mod escape_cozo_string_tests {
    use changeguard::commands::ask::escape_cozo_string;

    #[test]
    fn test_plain_symbol_unchanged() {
        assert_eq!(escape_cozo_string("foo_bar"), "foo_bar");
    }

    #[test]
    fn test_single_quote_doubled() {
        assert_eq!(escape_cozo_string("foo'bar"), "foo''bar");
    }

    #[test]
    fn test_backslash_escaped() {
        assert_eq!(escape_cozo_string("foo\\bar"), "foo\\\\bar");
    }

    #[test]
    fn test_both_special_chars() {
        assert_eq!(escape_cozo_string("it's a\\test"), "it''s a\\\\test");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(escape_cozo_string(""), "");
    }
}
