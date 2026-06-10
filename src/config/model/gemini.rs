use serde::{Deserialize, Serialize};

pub const DEFAULT_GEMINI_FAST_MODEL: &str = "gemini-3.5-flash";
pub const DEFAULT_GEMINI_DEEP_MODEL: &str = "gemini-3.1-pro";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GeminiConfig {
    pub api_key: Option<String>,
    /// Optional override used for every Gemini request.
    pub model: Option<String>,
    /// Default for routine, low-latency ChangeGuard ask modes.
    pub fast_model: Option<String>,
    /// Default for high-risk or review-heavy ChangeGuard ask modes.
    pub deep_model: Option<String>,
    pub timeout_secs: Option<u64>,
    #[serde(default = "default_context_window")]
    pub context_window: usize,
}

fn default_context_window() -> usize {
    128_000
}
