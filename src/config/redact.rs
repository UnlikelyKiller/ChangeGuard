use serde_json::Value;

/// Field name patterns that indicate a secret value (lowercase comparison).
const SECRET_FIELD_PATTERNS: &[&str] = &[
    "api_key",
    "apikey",
    "api-key",
    "token",
    "secret",
    "password",
    "credential",
    "ollama_key",
    "ollama_cloud_api_key",
    "gemini_api_key",
];

/// Check whether a field name looks like it holds a secret value.
fn is_secret_field_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    SECRET_FIELD_PATTERNS
        .iter()
        .any(|&p| lower == p || lower.ends_with(p))
}

/// Recursively walk a `serde_json::Value` and replace any field whose name
/// matches a known secret pattern with the string `"[REDACTED]"`.
///
/// Operates on both human and JSON config display paths so that secret values
/// are never leaked through `config view`, `config verify`, or any other
/// config serialization surface.
pub fn redact_config_value(value: &mut Value) {
    match value {
        Value::Object(obj) => {
            // Recurse into children first so nested objects are also redacted.
            for val in obj.values_mut() {
                redact_config_value(val);
            }
            // Then redact secret-named fields at this level.
            let secret_keys: Vec<String> = obj
                .keys()
                .filter(|k| is_secret_field_name(k))
                .cloned()
                .collect();
            for key in secret_keys {
                if let Some(inner) = obj.get(&key) {
                    let display = if inner.is_string() && !inner.as_str().unwrap_or("").is_empty() {
                        "[REDACTED]"
                    } else if inner.is_null() || inner.as_str().is_some_and(|s| s.is_empty()) {
                        "(not set)"
                    } else {
                        "[REDACTED]"
                    };
                    obj.insert(key, Value::String(display.to_string()));
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                redact_config_value(item);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_redact_api_key() {
        let mut val = json!({"ollama_cloud_api_key": "sk-abc123"});
        redact_config_value(&mut val);
        assert_eq!(val["ollama_cloud_api_key"], "[REDACTED]");
    }

    #[test]
    fn test_redact_gemini_key() {
        let mut val = json!({"gemini_api_key": "AIzaSyA1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q"});
        redact_config_value(&mut val);
        assert_eq!(val["gemini_api_key"], "[REDACTED]");
    }

    #[test]
    fn test_redact_ollama_key() {
        let mut val = json!({"ollama_key": "some-key-value"});
        redact_config_value(&mut val);
        assert_eq!(val["ollama_key"], "[REDACTED]");
    }

    #[test]
    fn test_redact_nested_secret() {
        let mut val = json!({
            "local_model": {
                "base_url": "http://localhost:11434",
                "ollama_cloud_api_key": "sk-secret",
                "ollama_cloud_model": "minimax-m3:cloud"
            }
        });
        redact_config_value(&mut val);
        assert_eq!(val["local_model"]["ollama_cloud_api_key"], "[REDACTED]");
        assert_eq!(val["local_model"]["base_url"], "http://localhost:11434");
        assert_eq!(val["local_model"]["ollama_cloud_model"], "minimax-m3:cloud");
    }

    #[test]
    fn test_redact_token_field() {
        let mut val = json!({"access_token": "ghp_abc123"});
        redact_config_value(&mut val);
        assert_eq!(val["access_token"], "[REDACTED]");
    }

    #[test]
    fn test_redact_secret_field() {
        let mut val = json!({"client_secret": "s3cr3t"});
        redact_config_value(&mut val);
        assert_eq!(val["client_secret"], "[REDACTED]");
    }

    #[test]
    fn test_redact_not_set_shows_placeholder() {
        let mut val = json!({"ollama_cloud_api_key": null});
        redact_config_value(&mut val);
        assert_eq!(val["ollama_cloud_api_key"], "(not set)");
    }

    #[test]
    fn test_redact_empty_shows_placeholder() {
        let mut val = json!({"ollama_cloud_api_key": ""});
        redact_config_value(&mut val);
        assert_eq!(val["ollama_cloud_api_key"], "(not set)");
    }

    #[test]
    fn test_no_redact_on_normal_fields() {
        let mut val = json!({
            "base_url": "http://localhost:11434",
            "embedding_model": "nomic-embed-text",
            "generation_model": "minimax-m3:cloud",
            "timeout_secs": 60
        });
        redact_config_value(&mut val);
        assert_eq!(val["base_url"], "http://localhost:11434");
        assert_eq!(val["embedding_model"], "nomic-embed-text");
    }

    #[test]
    fn test_redact_array_with_secret() {
        let mut val = json!([
            {"name": "config1", "api_key": "abc123"},
            {"name": "config2", "token": "def456"}
        ]);
        redact_config_value(&mut val);
        assert_eq!(val[0]["api_key"], "[REDACTED]");
        assert_eq!(val[1]["token"], "[REDACTED]");
        assert_eq!(val[0]["name"], "config1");
    }

    #[test]
    fn test_sentinel_secret_never_appears() {
        let sentinel = "TEST-SENTINEL-SECRET-VALUE-NEVER-LEAKS";
        let mut val = json!({
            "local_model": {
                "ollama_cloud_api_key": sentinel,
                "ollama_key": sentinel,
            },
            "gemini": {
                "api_key": sentinel,
            }
        });
        redact_config_value(&mut val);
        let serialized = serde_json::to_string(&val).unwrap();
        assert!(
            !serialized.contains(sentinel),
            "Sentinel secret leaked in: {serialized}"
        );
        assert!(serialized.contains("[REDACTED]"));
    }

    #[test]
    fn test_secret_in_map_values() {
        let mut val = json!({
            "providers": {
                "openai": {"api_key": "sk-openai-key"},
                "anthropic": {"api_key": "sk-anthropic-key"}
            }
        });
        redact_config_value(&mut val);
        assert_eq!(val["providers"]["openai"]["api_key"], "[REDACTED]");
        assert_eq!(val["providers"]["anthropic"]["api_key"], "[REDACTED]");
    }
}
