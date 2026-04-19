use changeguard::commands::verify::execute_verify;

#[test]
fn test_verify_command_pass() {
    let result = execute_verify(Some("powershell -Command echo 'pass'".into()), 5);
    assert!(result.is_ok());
}

#[test]
fn test_verify_command_fail() {
    let result = execute_verify(Some("powershell -Command exit 1".into()), 5);
    assert!(result.is_err());
}

#[test]
fn test_verify_command_timeout() {
    let result = execute_verify(Some("powershell -Command Start-Sleep -Seconds 10".into()), 1);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Timed out"));
}

#[test]
fn test_verify_command_not_found() {
    let result = execute_verify(Some("nonexistent_command_9999".into()), 5);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Command not found"));
}
