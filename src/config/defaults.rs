pub const DEFAULT_CONFIG: &str = r#"# ChangeGuard Configuration

[project]
name = "my-project"

[analysis]
# Languages to scan
languages = ["rust", "typescript", "python"]

[gemini]
# Gemini model configuration
model = "gemini-2.0-flash-exp"
"#;

pub const DEFAULT_RULES: &str = r#"# ChangeGuard Rules

[[rules]]
id = "no-hardcoded-secrets"
description = "Do not allow hardcoded secrets in the codebase"
severity = "error"
pattern = "(?i)password|api_key|secret|token"
"#;
