use crate::config::model::LocalModelConfig;
use std::time::Duration;

pub const MAX_BATCH_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct Dimensions {
    pub dimensions: usize,
    pub model_name: String,
    pub active: bool,
}

impl Dimensions {
    pub fn new(dims: usize) -> Self {
        Self {
            dimensions: dims,
            model_name: String::new(),
            active: dims > 0,
        }
    }
}

pub fn check_local_model(config: &LocalModelConfig) -> Result<Dimensions, String> {
    if config.base_url.is_empty() {
        return Ok(Dimensions {
            dimensions: 0,
            model_name: String::new(),
            active: false,
        });
    }
    Ok(Dimensions {
        dimensions: 0,
        model_name: String::new(),
        active: false,
    })
}

pub fn embed_long_text(_config: &LocalModelConfig, _text: &str) -> Result<Vec<f32>, String> {
    Err("not implemented".to_string())
}

pub fn embed_batch(
    base_url: &str,
    model: &str,
    texts: &[&str],
    timeout_secs: u64,
) -> Result<Vec<Vec<f32>>, String> {
    if base_url.is_empty() {
        return Ok(Vec::new());
    }
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    if texts.len() > MAX_BATCH_SIZE {
        return Err(format!(
            "Batch size {} exceeds maximum {MAX_BATCH_SIZE}",
            texts.len()
        ));
    }

    let url = format!("{base_url}/v1/embeddings");
    let body = serde_json::json!({
        "model": model,
        "input": texts,
    });

    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(timeout_secs))
        .timeout_write(Duration::from_secs(30))
        .build();

    let response = agent
        .post(&url)
        .set("Content-Type", "application/json")
        .send_json(&body)
        .map_err(|e| match e {
            ureq::Error::Status(code, response) => {
                let msg = response.into_string().unwrap_or_default();
                format!(
                    "Embedding server returned {code}: {}",
                    msg.chars().take(200).collect::<String>()
                )
            }
            ureq::Error::Transport(inner) => {
                format!("Embedding server unreachable: {inner}")
            }
        })?;

    let value: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("Failed to parse embedding response: {e}"))?;

    let data = value["data"]
        .as_array()
        .ok_or_else(|| "Embedding response missing 'data' array".to_string())?;

    let mut results: Vec<Vec<f32>> = Vec::with_capacity(data.len());
    for item in data {
        let embedding = item["embedding"]
            .as_array()
            .ok_or_else(|| "Missing 'embedding' array in response data item".to_string())?;

        let floats: Vec<f32> = embedding
            .iter()
            .map(|v| {
                v.as_f64()
                    .ok_or_else(|| "Non-numeric value in embedding array".to_string())
                    .map(|f| f as f32)
            })
            .collect::<Result<Vec<f32>, String>>()?;

        results.push(floats);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::LocalModelConfig;
    use httpmock::prelude::*;

    #[test]
    fn test_dimensions_new_active() {
        let d = Dimensions::new(768);
        assert_eq!(d.dimensions, 768);
        assert!(d.active);

        let d = Dimensions::new(0);
        assert_eq!(d.dimensions, 0);
        assert!(!d.active);
    }

    #[test]
    fn test_check_local_model_empty_url() {
        let config = LocalModelConfig::default();
        let result = check_local_model(&config).unwrap();
        assert!(!result.active);
        assert_eq!(result.dimensions, 0);
    }

    #[test]
    fn test_check_local_model_unreachable() {
        let config = LocalModelConfig {
            base_url: "http://127.0.0.1:1".to_string(),
            embedding_model: "test-model".to_string(),
            timeout_secs: 1,
            ..LocalModelConfig::default()
        };
        let result = check_local_model(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_embed_long_text_short_delegates_to_batch() {
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
            context_window: 8192,
            timeout_secs: 30,
            ..LocalModelConfig::default()
        };

        let result = embed_long_text(&config, "hello world").unwrap();
        assert_eq!(result.len(), 3);
        assert!((result[0] - 0.1).abs() < 1e-6);
        assert!((result[1] - 0.2).abs() < 1e-6);
        assert!((result[2] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_embed_long_text_long_text_chunked_and_pooled() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/embeddings");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "data": [
                        {"embedding": [1.0, 0.0, 0.0]},
                        {"embedding": [0.0, 1.0, 0.0]}
                    ]
                }));
        });

        // context_window = 2 means max_chars = 8, so a 20-char text splits into 3 chunks
        let config = LocalModelConfig {
            base_url: server.base_url(),
            embedding_model: "test-model".to_string(),
            context_window: 2,
            timeout_secs: 30,
            ..LocalModelConfig::default()
        };

        let long_text = "abcdefghijklmnopqrstuvwxyz"; // 26 chars
        let result = embed_long_text(&config, long_text).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_embed_batch_empty_url_returns_empty() {
        let result = embed_batch("", "model", &["hello"], 30).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_embed_batch_empty_texts_returns_empty() {
        let result = embed_batch("http://localhost:1234", "model", &[], 30).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_embed_batch_exceeds_max_batch() {
        let texts: Vec<String> = (0..=MAX_BATCH_SIZE).map(|i| format!("text{i}")).collect();
        let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let result = embed_batch("http://localhost:1234", "model", &refs, 30);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum"));
    }

    #[test]
    fn test_embed_batch_success() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/embeddings");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "data": [
                        {"embedding": [0.1, 0.2, 0.3]},
                        {"embedding": [0.4, 0.5, 0.6]}
                    ]
                }));
        });

        let result =
            embed_batch(&server.base_url(), "test-model", &["hello", "world"], 30).unwrap();
        mock.assert();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec![0.1_f32, 0.2, 0.3]);
        assert_eq!(result[1], vec![0.4_f32, 0.5, 0.6]);
    }

    #[test]
    fn test_embed_batch_server_error() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/embeddings");
            then.status(503).body("Service Unavailable");
        });

        let result = embed_batch(&server.base_url(), "test-model", &["hello"], 30);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("503"));
    }

    #[test]
    fn test_embed_batch_connection_refused() {
        let result = embed_batch("http://127.0.0.1:1", "model", &["hello"], 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unreachable"));
    }
}
