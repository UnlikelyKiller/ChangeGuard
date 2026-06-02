# Track U10 Spec: Flexible Local Completion Model Fallback

## Background
The `ask` command currently fails if the local completions server is unreachable. To improve usability and support cloud-based fallbacks (such as the Ollama Pro Cloud API), we need a flexible fallback option.

## Objective
Support a customizable fallback LLM completions path using environment variables (such as `OLLAMA_CLOUD_URL`, `OLLAMA_CLOUD_API_KEY`, and `OLLAMA_CLOUD_MODEL`) when the local offline model fails.

## Proposed Design
* Load `OLLAMA_CLOUD_URL` (e.g. `https://api.ollama.com`), `OLLAMA_CLOUD_API_KEY`, and `OLLAMA_CLOUD_MODEL` (e.g. `minimax-m3:cloud`) from the `.env` configuration.
* Update `LocalModelClient` to automatically switch to the Ollama Cloud URL and Model (with authorization headers) if the local completions endpoint fails to respond (or if explicitly configured to use Ollama Cloud).
* Maintain complete flexibility to allow fallback to Gemini completions as well.
