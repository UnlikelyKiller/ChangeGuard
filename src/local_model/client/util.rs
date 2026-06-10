use crate::local_model::client::types::{EndpointKind, EndpointTarget};

/// Detect whether a base URL should use Ollama native chat or
/// OpenAI-compatible chat completions.
pub fn detect_endpoint_kind(base_url: &str) -> EndpointKind {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.ends_with("/api") || trimmed == "https://api.ollama.com" {
        EndpointKind::OllamaNative
    } else {
        EndpointKind::OpenAICompatible
    }
}

pub fn completion_target(base_url: &str) -> EndpointTarget {
    let base = base_url.trim_end_matches('/');
    match detect_endpoint_kind(base_url) {
        EndpointKind::OllamaNative => {
            let url = if base == "https://api.ollama.com" {
                "https://api.ollama.com/api/chat".to_string()
            } else {
                format!("{}/chat", base)
            };
            EndpointTarget {
                kind: EndpointKind::OllamaNative,
                url,
            }
        }
        EndpointKind::OpenAICompatible => EndpointTarget {
            kind: EndpointKind::OpenAICompatible,
            url: format!("{}/v1/chat/completions", base),
        },
    }
}

/// Validate the base URL shape and return a diagnostic warning if it is
/// malformed enough that endpoint selection would be ambiguous.
pub fn check_base_url_warnings(base_url: &str, _kind: EndpointKind) -> Option<String> {
    let trimmed = base_url.trim_end_matches('/');
    let lower = trimmed.to_lowercase();
    if lower.ends_with("/api/v1") {
        Some(
            "Unsupported Ollama URL shape. Use https://ollama.com/api for native Ollama \
             mode or https://ollama.com for OpenAI-compatible mode."
                .to_string(),
        )
    } else {
        None
    }
}

/// U22: Walk the `ureq::Transport` error source chain looking for an
/// `io::Error` of `ErrorKind::TimedOut`. ureq 2.12 normalizes both read
/// timeouts and `WouldBlock` to `TimedOut` internally, but only the inner
/// `io::Error` carries the kind — the outer `Transport::Display` string is
/// the OS-level error message, not "timeout".
pub fn transport_is_timeout(err: &ureq::Transport) -> bool {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(e) = source {
        if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
            return io_err.kind() == std::io::ErrorKind::TimedOut;
        }
        source = e.source();
    }
    false
}
