use changeguard::commands::ask::execute_ask;
use changeguard::gemini::modes::GeminiMode;
use std::env;
use tempfile::tempdir;

#[test]
fn test_ask_command_no_packet() {
    let tmp = tempdir().unwrap();
    let old_dir = env::current_dir().unwrap();
    env::set_current_dir(tmp.path()).unwrap();

    // Should fail because no .changeguard/state/ledger.db exists
    let result = execute_ask("What's up?".into(), GeminiMode::Analyze);
    assert!(result.is_err());

    env::set_current_dir(old_dir).unwrap();
}
