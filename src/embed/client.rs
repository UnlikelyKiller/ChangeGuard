use std::time::Duration;

pub const MAX_BATCH_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct Dimensions {
    pub current: usize,
    pub active: bool,
}

impl Dimensions {
    pub fn new(dims: usize) -> Self {
        Self {
            current: dims,
            active: dims > 0,
        }
    }
}

pub fn check_local_model(base_url: &str) -> Result<bool, String> {
    if base_url.is_empty() {
        return Ok(false);
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build();

    match agent.get(base_url).call() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
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
    use httpmock::prelude::*;

    #[test]
    fn test_dimensions_new_active() {
        let d = Dimensions::new(768);
        assert_eq!(d.current, 768);
        assert!(d.active);

        let d = Dimensions::new(0);
        assert_eq!(d.current, 0);
        assert!(!d.active);
    }

    #[test]
    fn test_check_local_model_empty_url() {
        assert!(!check_local_model("").unwrap());
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
            when.method(POST).path("/v1/embeddings");
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
            when.method(POST).path("/v1/embeddings");
            then.status(503).body("Service Unavailable");
        });

        let result = embed_batch(&server.base_url(), "test-model", &["hello"], 30);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("503"));
    }

    #[test]
    fn test_embed_batch_connection_refused() {
        // Use a port that's very unlikely to have a listener
        let result = embed_batch("http://127.0.0.1:1", "model", &["hello"], 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unreachable"));
    }
}
