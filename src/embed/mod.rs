pub mod budget;
pub mod client;
pub mod similarity;
pub mod storage;

use crate::config::model::LocalModelConfig;
use rusqlite::Connection;

pub fn embed_and_store(
    _config: &LocalModelConfig,
    _conn: &Connection,
    _entity_type: &str,
    _entity_id: &str,
    _text: &str,
) -> Result<bool, String> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::migrations::get_migrations;
    use httpmock::prelude::*;
    use tempfile::tempdir;

    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations = get_migrations();
        migrations.to_latest(&mut conn).unwrap();
        conn
    }

    #[test]
    fn embed_and_store_empty_base_url_returns_false() {
        let conn = setup_db();
        let config = LocalModelConfig::default();
        let result = embed_and_store(&config, &conn, "FILE", "test.rs", "some text").unwrap();
        assert!(!result);
    }

    #[test]
    fn embed_and_store_new_text_returns_true() {
        let conn = setup_db();
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/embeddings");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "data": [
                        {"embedding": [0.1, 0.2, 0.3]}
                    ]
                }));
        });

        let config = LocalModelConfig {
            base_url: server.base_url(),
            embedding_model: "test-model".to_string(),
            dimensions: 3,
            context_window: 8192,
            timeout_secs: 30,
            ..LocalModelConfig::default()
        };

        let result = embed_and_store(&config, &conn, "FILE", "test.rs", "hello world").unwrap();
        assert!(result);
    }

    #[test]
    fn embed_and_store_duplicate_skip_returns_false() {
        let conn = setup_db();
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/embeddings");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "data": [
                        {"embedding": [0.1, 0.2, 0.3]}
                    ]
                }));
        });

        let config = LocalModelConfig {
            base_url: server.base_url(),
            embedding_model: "test-model".to_string(),
            dimensions: 3,
            context_window: 8192,
            timeout_secs: 30,
            ..LocalModelConfig::default()
        };

        let text = "hello world";
        let result1 = embed_and_store(&config, &conn, "FILE", "test.rs", text).unwrap();
        assert!(result1);

        let result2 = embed_and_store(&config, &conn, "FILE", "test.rs", text).unwrap();
        assert!(!result2);

        mock.assert_hits(1); // Only one HTTP call
    }
}
