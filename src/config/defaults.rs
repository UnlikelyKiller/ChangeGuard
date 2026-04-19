pub const DEFAULT_CONFIG: &str = r#"[core]
strict = false
auto_fix = false

[watch]
debounce_ms = 1000
ignore_patterns = ["target/**", ".git/**", "node_modules/**"]

[gemini]
# api_key = "..."
# model = "gemini-2.0-flash-exp"
"#;
