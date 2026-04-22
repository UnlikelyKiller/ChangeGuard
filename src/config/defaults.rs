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
min_shared_commits = 3
min_revisions = 5
decay_half_life = 100

[hotspots]
max_commits = 500
limit = 10

# [verify]
# default_timeout_secs = 300
# Steps to run when `changeguard verify` is invoked without -c.
# Each step has a description, command, and optional timeout_secs (defaults to 300).
# [[verify.steps]]
# description = "Run project tests"
# command = "cargo test -j 1 -- --test-threads=1"
# timeout_secs = 300
# [[verify.steps]]
# description = "Check formatting"
# command = "cargo fmt --check"

[gemini]
# Prefer GEMINI_API_KEY in the environment or local .env.
# api_key = "..."
# Optional override for every ask mode:
# model = "gemini-3.1-pro-preview"
fast_model = "gemini-3.1-flash-lite-preview"
deep_model = "gemini-3.1-pro-preview"
timeout_secs = 120
context_window = 128000
"#;
