## Spec: Fix ask Gemini Model Name (Track CG-F1)

### Acceptance Criteria

1. **No 404**: `changeguard ask "test" --backend gemini` does not return HTTP 404 for model not found
2. **Model configurable**: Model name can be overridden via env var or config key
3. **Sensible default**: Default model is a currently-supported Gemini model (e.g., `gemini-2.5-flash`)
4. **Graceful fallback**: If Gemini API key is missing, falls back to local model with a clear message
5. **Bridge query works**: The AI-Brains bridge query within `ask` context gathering does not produce FTS5 syntax errors
6. **CI gate passes**: `cargo fmt --check`, `cargo clippy`, `cargo test` all pass
