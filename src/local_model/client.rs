use crate::config::model::LocalModelConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct CompletionOptions {
    pub max_tokens: usize,
    pub temperature: f32,
}

impl Default for CompletionOptions {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct CompletionResponse {
    choices: Vec<Choice>,
}

struct CompletionEndpoint<'a> {
    label: &'a str,
    base_url: &'a str,
    model: &'a str,
    authorization: Option<String>,
}

pub fn ping_completions(config: &LocalModelConfig) -> Result<String, String> {
    if config.base_url.is_empty() && config.generation_url.is_none() {
        return Err("not configured".to_string());
    }

    let check_url = config.generation_url.as_deref().unwrap_or(&config.base_url);
    // CR3: Increased from 150ms to 500ms to prevent false negatives on WSL/container hosts.
    if !crate::util::network::is_url_reachable(check_url, Duration::from_millis(500)) {
        return Err(format!(
            "Local model server at {} is unreachable",
            check_url
        ));
    }

    let url = if let Some(gen_url) = &config.generation_url {
        format!("{}/v1/chat/completions", gen_url)
    } else {
        format!("{}/v1/chat/completions", config.base_url)
    };
    tracing::debug!("Using completion URL: {}", url);

    let body = serde_json::json!({
        "model": config.generation_model,
        "messages": [{"role": "user", "content": "ping"}],
        "max_tokens": 1,
        "stream": false,
    });

    // Use config timeout: lazy-loading servers need time to load the model before responding.
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(std::cmp::min(config.timeout_secs, 5)))
        .timeout_read(Duration::from_secs(config.timeout_secs))
        .timeout_write(Duration::from_secs(30))
        .build();

    let response = match agent
        .post(&url)
        .set("Content-Type", "application/json")
        .send_json(&body)
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(code, response)) => {
            let body = response.into_string().unwrap_or_default();
            return Err(format!(
                "{} server error ({})",
                code,
                body.chars().take(100).collect::<String>()
            ));
        }
        Err(ureq::Error::Transport(inner)) => {
            if format!("{:?}", inner).to_lowercase().contains("timeout") {
                return Err(format!("timed out after {}s", config.timeout_secs));
            }
            return Err(format!("unreachable ({})", inner));
        }
    };

    // Best-effort model name: read from response, fall back to configured model
    let model_name = response
        .into_json::<serde_json::Value>()
        .ok()
        .and_then(|v| {
            v.get("model")
                .and_then(|m| m.as_str().map(|s| s.to_string()))
        })
        .unwrap_or_else(|| config.generation_model.clone());

    Ok(model_name)
}

pub fn complete(
    config: &LocalModelConfig,
    messages: &[ChatMessage],
    options: &CompletionOptions,
    timeout_secs_override: Option<u64>,
) -> Result<String, String> {
    if config.base_url.is_empty()
        && config.generation_url.is_none()
        && !has_ollama_cloud_fallback(config)
    {
        return Err(
            "Local model server is not configured. Start llama-server, configure Ollama Cloud fallback, or use --backend gemini."
                .to_string(),
        );
    }

    let local_base_url = config.generation_url.as_deref().unwrap_or(&config.base_url);
    if !local_base_url.is_empty() {
        // CR3: Fast network probe to prevent 20s TCP hangs when model server is down.
        if crate::util::network::is_url_reachable(local_base_url, Duration::from_millis(500)) {
            let endpoint = CompletionEndpoint {
                label: "Local model server",
                base_url: local_base_url,
                model: &config.generation_model,
                authorization: None,
            };
            let effective_timeout = timeout_secs_override.unwrap_or(config.timeout_secs);
            match complete_with_endpoint(&endpoint, effective_timeout, messages, options) {
                Ok(response) => return Ok(response),
                Err(error) if has_ollama_cloud_fallback(config) => {
                    tracing::warn!(
                        "Local completion failed ({error}); trying Ollama Cloud fallback"
                    );
                }
                Err(error) => return Err(error),
            }
        } else if !has_ollama_cloud_fallback(config) {
            return Err(format!(
                "Local model server at {} is unreachable. Start llama-server or use --backend gemini.",
                local_base_url
            ));
        } else {
            tracing::warn!(
                "Local model server at {} is unreachable; trying Ollama Cloud fallback",
                local_base_url
            );
        }
    }

    if let Some(endpoint) = ollama_cloud_endpoint(config) {
        let effective_timeout = timeout_secs_override.unwrap_or(config.timeout_secs);
        return complete_with_endpoint(&endpoint, effective_timeout, messages, options);
    }

    Err(format!(
        "Local model server at {} is unreachable. Start llama-server or use --backend gemini.",
        local_base_url
    ))
}

pub fn has_ollama_cloud_fallback(config: &LocalModelConfig) -> bool {
    config
        .ollama_cloud_url
        .as_deref()
        .is_some_and(|url| !url.trim().is_empty())
        && config
            .ollama_cloud_api_key
            .as_deref()
            .is_some_and(|key| !key.trim().is_empty())
        && config
            .ollama_cloud_model
            .as_deref()
            .is_some_and(|model| !model.trim().is_empty())
}

fn ollama_cloud_endpoint(config: &LocalModelConfig) -> Option<CompletionEndpoint<'_>> {
    let base_url = config.ollama_cloud_url.as_deref()?.trim();
    let api_key = config.ollama_cloud_api_key.as_deref()?.trim();
    let model = config.ollama_cloud_model.as_deref()?.trim();
    if base_url.is_empty() || api_key.is_empty() || model.is_empty() {
        return None;
    }
    Some(CompletionEndpoint {
        label: "Ollama Cloud fallback",
        base_url,
        model,
        authorization: Some(format!("Bearer {api_key}")),
    })
}

/// U22: Walk the `ureq::Transport` error source chain looking for an
/// `io::Error` of `ErrorKind::TimedOut`. ureq 2.12 normalizes both read
/// timeouts and `WouldBlock` to `TimedOut` internally, but only the inner
/// `io::Error` carries the kind — the outer `Transport::Display` string is
/// the OS-level error message, not "timeout".
fn transport_is_timeout(err: &ureq::Transport) -> bool {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(e) = source {
        if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
            return io_err.kind() == std::io::ErrorKind::TimedOut;
        }
        source = e.source();
    }
    false
}

fn complete_with_endpoint(
    endpoint: &CompletionEndpoint<'_>,
    timeout_secs: u64,
    messages: &[ChatMessage],
    options: &CompletionOptions,
) -> Result<String, String> {
    let url = format!(
        "{}/v1/chat/completions",
        endpoint.base_url.trim_end_matches('/')
    );
    tracing::debug!("Using completion URL: {}", url);

    let body = serde_json::json!({
        "model": endpoint.model,
        "messages": messages,
        "max_tokens": options.max_tokens,
        "temperature": options.temperature,
        "stream": false,
    });

    let agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(timeout_secs))
        .timeout_write(Duration::from_secs(30))
        .build();

    let mut retry = false;

    let response = loop {
        let mut request = agent.post(&url).set("Content-Type", "application/json");
        if let Some(value) = &endpoint.authorization {
            request = request.set("Authorization", value);
        }
        let result = request.send_json(&body);

        break match result {
            Ok(resp) => resp,
            Err(ureq::Error::Status(503, _response)) if !retry => {
                std::thread::sleep(Duration::from_secs(2));
                retry = true;
                continue;
            }
            Err(ureq::Error::Status(503, response)) => {
                let body = response.into_string().unwrap_or_default();
                return Err(format!(
                    "{} returned 503: {}",
                    endpoint.label,
                    body.chars().take(200).collect::<String>()
                ));
            }
            Err(ureq::Error::Status(429, _)) => return Err("rate limited".to_string()),
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();
                return Err(format!(
                    "{} returned {code}: {}",
                    endpoint.label,
                    body.chars().take(200).collect::<String>()
                ));
            }
            Err(ureq::Error::Transport(inner)) => {
                if transport_is_timeout(&inner) {
                    return Err(format!(
                        "{} timed out after {}s",
                        endpoint.label, timeout_secs
                    ));
                }
                return Err(format!(
                    "{} not reachable at {} \u{2014} {}",
                    endpoint.label, endpoint.base_url, inner
                ));
            }
        };
    };

    let parsed: CompletionResponse = response
        .into_json()
        .map_err(|e| format!("Failed to parse completion response: {e}"))?;

    parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "No completion choices returned".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::LocalModelConfig;
    use httpmock::prelude::*;

    fn test_config(base_url: &str) -> LocalModelConfig {
        LocalModelConfig {
            base_url: base_url.to_string(),
            embedding_url: None,
            generation_url: None,
            generation_model: "test-model".to_string(),
            timeout_secs: 30,
            ..LocalModelConfig::default()
        }
    }

    fn test_messages() -> Vec<ChatMessage> {
        vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello!".to_string(),
            },
        ]
    }

    #[test]
    fn complete_success() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "choices": [
                        {
                            "message": {
                                "content": "Hello! How can I help you today?"
                            }
                        }
                    ]
                }));
        });

        let config = test_config(&server.base_url());
        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            None,
        )
        .unwrap();
        assert_eq!(result, "Hello! How can I help you today?");
    }

    #[test]
    fn complete_503_retry() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions");
            then.status(503).body("Service Unavailable");
        });

        let config = test_config(&server.base_url());
        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("503"));
        // Verify retry happened: 2 calls total
        assert_eq!(mock.hits(), 2);
    }

    #[test]
    fn complete_429_rate_limited() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions");
            then.status(429).body("Too Many Requests");
        });

        let config = test_config(&server.base_url());
        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "rate limited");
    }

    #[test]
    fn complete_other_status_error() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions");
            then.status(500).body("Internal Server Error");
        });

        let config = test_config(&server.base_url());
        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            None,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("500"));
        assert!(err.contains("Internal Server Error"));
    }

    #[test]
    fn complete_connection_refused() {
        let config = test_config("http://127.0.0.1:1");
        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("is unreachable"));
    }

    #[test]
    fn complete_empty_choices() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "choices": []
                }));
        });

        let config = test_config(&server.base_url());
        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No completion choices"));
    }

    #[test]
    fn complete_empty_url() {
        let config = test_config("");
        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("is not configured"));
    }

    #[test]
    fn completions_ping_success() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "choices": [{"message": {"content": "hi"}}]
                }));
        });
        let config = test_config(&server.base_url());
        let result = ping_completions(&config);
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);
        assert_eq!(result.unwrap(), "test-model");
    }

    #[test]
    fn completions_ping_transport_failure() {
        let config = test_config("http://127.0.0.1:1");
        let result = ping_completions(&config);
        assert!(result.is_err());
        assert!(!result.unwrap_err().is_empty(), "error should not be empty");
    }

    #[test]
    fn completions_ping_non_200() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions");
            then.status(503).body("Service Unavailable");
        });
        let config = test_config(&server.base_url());
        let result = ping_completions(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("503"), "expected '503' in: {err}");
    }

    #[test]
    fn transport_error_includes_cause() {
        // Use a port that nothing is listening on
        let config = test_config("http://127.0.0.1:1");
        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            None,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("is unreachable"),
            "expected 'is unreachable' in: {err}"
        );
    }

    /// U22.1 (red): proves the timeout override is honored. The mock delays
    /// 5 seconds; with a 1-second override the call must abort with a
    /// "timed out" error and return well before the mock would have responded.
    #[test]
    fn complete_timeout_override_fires() {
        use std::time::Instant;

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions");
            then.status(200)
                .delay(std::time::Duration::from_secs(5))
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "choices": [{"message": {"content": "too late"}}]
                }));
        });

        let config = test_config(&server.base_url());
        let start = Instant::now();
        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            Some(1),
        );
        let elapsed = start.elapsed();

        assert!(result.is_err(), "expected timeout error, got: {result:?}");
        let err = result.unwrap_err();
        assert!(
            err.contains("timed out"),
            "expected 'timed out' in error, got: {err}"
        );
        assert!(
            elapsed < std::time::Duration::from_secs(3),
            "expected <3s, got {elapsed:?}"
        );
    }

    /// U22.1 (red): when the override is None the call should still succeed
    /// (and fall back to the config-provided timeout_secs, which is 30s here
    /// — long enough to outlast the mock's 100ms response).
    #[test]
    fn complete_timeout_override_none_falls_back_to_config() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "choices": [{"message": {"content": "fast"}}]
                }));
        });

        let config = test_config(&server.base_url());
        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            None,
        );
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_eq!(result.unwrap(), "fast");
    }

    #[test]
    fn complete_falls_back_to_ollama_cloud_with_auth() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/chat/completions")
                .header("Authorization", "Bearer test-token")
                .json_body_partial(r#"{"model":"minimax-m3:cloud"}"#);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(serde_json::json!({
                    "choices": [
                        {
                            "message": {
                                "content": "cloud response"
                            }
                        }
                    ]
                }));
        });

        let config = LocalModelConfig {
            base_url: "http://127.0.0.1:1".to_string(),
            ollama_cloud_url: Some(server.base_url()),
            ollama_cloud_api_key: Some("test-token".to_string()),
            ollama_cloud_model: Some("minimax-m3:cloud".to_string()),
            ..test_config("")
        };

        let result = complete(
            &config,
            &test_messages(),
            &CompletionOptions::default(),
            None,
        )
        .unwrap();
        assert_eq!(result, "cloud response");
        assert_eq!(mock.hits(), 1);
    }
}
