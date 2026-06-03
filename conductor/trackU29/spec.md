# Track U29: Intent Demo TTY Detection

**Status:** ✅ **Completed**
**Started:** 2026-06-02
**Owner:** Antigravity
**Priority:** P2 — Minor / CLI Resilience

---

## Problem Statement

Running `changeguard intent demo` launches an interactive TUI. If run without a TTY or in non-interactive CI/CD scripts, it hangs indefinitely.

## Acceptance Criteria

**AC1:** `intent demo` checks for interactive terminal (TTY) status. If non-interactive, it exits gracefully with an error/warning instead of hanging.

## Design Notes

- Use is-terminal/is_terminal checks in `src/commands/intent.rs` to detect interactive environment before launching Ratatui.

## Verification

- Run `changeguard intent demo < NUL` (or equivalent redirect) and verify it exits cleanly with a message rather than hanging.
