use changeguard::bridge::ipc::IpcClient;
use std::time::Duration;

#[test]
fn test_ipc_client_timeout_on_nonexistent_pipe() {
    // This should not hang
    let client = IpcClient::connect_with_timeout(Duration::from_millis(100));
    assert!(client.is_err());
}
