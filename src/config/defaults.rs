pub const DEFAULT_CONFIG: &str = r#"[core]
strict = false
auto_fix = false

[watch]
debounce_ms = 1000
ignore_patterns = ["target/**", ".git/**", "node_modules/**"]

[temporal]
max_commits = 1000
max_files_per_commit = 50
coupling_threshold = 0.75

[hotspots]
max_commits = 500
limit = 10

[gemini]
# api_key = "..."
# model = "gemini-2.0-flash-exp"
"#;
