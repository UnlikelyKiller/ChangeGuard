use changeguard::commands::ask::execute_ask;
use changeguard::gemini::modes::GeminiMode;
use changeguard::impact::packet::ImpactPacket;
use changeguard::state::layout::Layout;
use changeguard::state::storage::StorageManager;
use std::fs;
use tempfile::tempdir;

use crate::common::{DirGuard, cwd_lock};

#[test]
fn test_ask_command_no_packet() {
    let _lock = cwd_lock().lock().unwrap();
    unsafe {
        std::env::set_var("CHANGEGUARD_NON_INTERACTIVE", "1");
    }
    let old_gemini_key = std::env::var("GEMINI_API_KEY");
    unsafe {
        std::env::remove_var("GEMINI_API_KEY");
    }

    let tmp = tempdir().unwrap();
    let root = camino::Utf8Path::from_path(tmp.path()).unwrap();
    let _guard = DirGuard::from_utf8(root);

    // Initialize a minimal git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let layout = Layout::new(root);
    layout.ensure_state_dir().unwrap();

    // We must init the storage so it can be queried, even if there's no packet.
    // If it's totally missing, execute_ask will try to create it.
    // We expect it to succeed now (fallback to global mode) instead of erroring out.
    // However, it will fail when trying to connect to Gemini/Local if no config is set.
    // We'll write a dummy config to trigger a specific error later in the chain,
    // proving it got past the "No impact report" check.

    fs::write(layout.config_file(), "[gemini]\nfast_model = \"dummy\"\n").unwrap();

    let result = execute_ask(
        Some("What's up?".into()),
        false, // semantic
        10,    // limit
        GeminiMode::Analyze,
        false, // narrative
        None,  // backend
        false, // auto_index
        15,    // timeout_secs
        false, // no_kg_fallback
    );

    // It should NOT fail with "No impact report found" anymore.
    // Depending on the test environment, it might fail to reach Gemini or the local model.
    if let Err(e) = result {
        let err_str = e.to_string();
        assert!(
            !err_str.contains("No impact report found"),
            "Should fallback to global mode"
        );
    }

    unsafe {
        if let Ok(val) = old_gemini_key {
            std::env::set_var("GEMINI_API_KEY", val);
        }
        std::env::remove_var("CHANGEGUARD_NON_INTERACTIVE");
    }
}

#[test]
fn test_ask_invalid_config_fails_before_query_execution() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = camino::Utf8Path::from_path(tmp.path()).unwrap();
    let _guard = DirGuard::from_utf8(root);

    // Initialize a minimal git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let layout = Layout::new(root);
    layout.ensure_state_dir().unwrap();
    fs::write(layout.config_file(), "[watch]\ndebounce_ms = 0\n").unwrap();

    {
        let storage =
            StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path()).unwrap();
        storage.save_packet(&ImpactPacket::default()).unwrap();
        // storage is dropped here, releasing the CozoDB lock
    }

    let err = execute_ask(
        Some("What's up?".into()),
        false, // semantic
        10,    // limit
        GeminiMode::Analyze,
        false, // narrative
        None,  // backend
        false, // auto_index
        15,    // timeout_secs
        false, // no_kg_fallback
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("debounce_ms"));
}

/// U22: end-to-end timeout test. The local model completion client must
/// respect the per-call `timeout_secs_override` parameter, return an
/// error, and abort well before the server's mocked response delay.
///
/// Server delay is intentionally small (3s) so that the httpmock
/// listener thread exits promptly after the assertion fires — a 15s
/// delay held the test binary open for an extra 13s, producing false
/// "test running for over 60 seconds" reports.
#[test]
fn test_ask_respects_cli_timeout_override() {
    use changeguard::config::model::LocalModelConfig;
    use changeguard::local_model::client::{ChatMessage, CompletionOptions, complete};
    use std::time::Instant;

    let server = httpmock::MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(httpmock::Method::POST)
            .path("/v1/chat/completions");
        then.status(200)
            .delay(std::time::Duration::from_secs(3))
            .header("Content-Type", "application/json")
            .json_body(serde_json::json!({
                "choices": [{"message": {"content": "too late"}}]
            }));
    });

    let config = LocalModelConfig {
        base_url: server.base_url(),
        embedding_url: None,
        generation_url: None,
        ollama_cloud_url: None,
        ollama_cloud_api_key: None,
        ollama_cloud_model: None,
        embedding_model: String::new(),
        generation_model: "test-model".to_string(),
        rerank_model: String::new(),
        dimensions: 0,
        context_window: 38000,
        timeout_secs: 60, // not used — override takes precedence
        prefer_local: false,
        chunk_top_k: 10,
        chunk_min_similarity: 0.3,
        chunk_dedup_threshold: 0.95,
        disable_hnsw: false,
        concurrency: None,
    };

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "hello".to_string(),
    }];

    let start = Instant::now();
    let result = complete(&config, &messages, &CompletionOptions::default(), Some(1));
    let elapsed = start.elapsed();

    assert!(result.is_err(), "expected timeout error, got: {result:?}");
    let err = result.unwrap_err();
    assert!(
        err.contains("timed out"),
        "expected 'timed out' in error, got: {err}"
    );
    assert!(
        elapsed < std::time::Duration::from_secs(2),
        "expected <2s, got {elapsed:?}"
    );
    assert_eq!(mock.hits(), 1, "the mock should have been hit exactly once");
}
