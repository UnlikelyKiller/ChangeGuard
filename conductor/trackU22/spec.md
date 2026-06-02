# Track U22: ChangeGuard LLM Query Timeout Guardrails

**Status:** ⏳ **Pending**
**Started:** None
**Owner:** None
**Priority:** P1 — usability / prevents pipeline blocks.

---

## Problem Statement

When using `changeguard ask`, the client connects to LLM backends (either local/fallback models or external APIs). If a backend is slow, loading a new model, or completely unresponsive, the client can hang indefinitely. This blocks automated verification pipelines and scripts in non-interactive CI environments. Introducing client-side timeouts ensures that ChangeGuard fails gracefully with a fallback or error message rather than hanging forever.

## Acceptance Criteria

**AC1:** A configurable timeout threshold (defaulting to 15 seconds) is enforced on LLM connection and retrieval requests in `changeguard ask`.

**AC2:** If a request exceeds this timeout, the operation fails gracefully, outputs a descriptive warning to stderr, and aborts the request.

**AC3:** A `--timeout <seconds>` CLI flag or corresponding environment variable allows users to override the default timeout limit.

## Design Notes

- Leverage asynchronous futures wrapping or client configuration options (e.g. `reqwest` client builder timeouts, or `tokio::time::timeout` on futures) to abort long-running network operations.
- Handle timeout errors gracefully and print a clear error to stderr.

## Verification

- Simulate or mock a non-responsive LLM backend and execute `changeguard ask --timeout 2`. Verify it terminates and exits cleanly with a timeout message in under 3 seconds.
