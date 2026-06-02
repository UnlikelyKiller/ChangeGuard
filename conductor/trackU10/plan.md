# Track U10 Plan: Flexible Local Completion Model Fallback

- [x] Task U10.1: Add `ollama_cloud` options (API key, URL, Model) to config loaders and `.env` parser logic in `src/config/`.
- [x] Task U10.2: Implement authorization header support and routing in `src/local_model/client.rs`.
- [x] Task U10.3: Update fallback completions logic in `execute_ask` to attempt Ollama Cloud completions if local server is unreachable.
- [x] Task U10.4: Write integration tests using `httpmock` or mock payloads to verify the fallback completions workflow.
