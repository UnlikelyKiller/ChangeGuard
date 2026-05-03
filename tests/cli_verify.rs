use changeguard::commands::verify::execute_verify;

#[test]
fn test_verify_command_pass() {
    let cmd = "echo hello";
    let result = execute_verify(Some(cmd.into()), 5, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_verify_command_fail() {
    let cmd = "exit 1";
    let result = execute_verify(Some(cmd.into()), 5, false, false);
    assert!(result.is_err());
}

#[test]
fn test_verify_command_timeout() {
    let cmd = if cfg!(target_os = "windows") {
        "ping -n 10 127.0.0.1 >nul"
    } else {
        "sleep 10"
    };
    let result = execute_verify(Some(cmd.into()), 1, false, false);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Timed out"));
}

#[test]
fn test_verify_command_not_found() {
    let result = execute_verify(Some("nonexistent_command_9999".into()), 5, false, false);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Command not found"));
}
