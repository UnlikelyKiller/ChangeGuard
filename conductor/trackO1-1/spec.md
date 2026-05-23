# Track O1-1: Intent Capture TUI Scaffold

## Objective
Build the foundation for the "Interactive Confirmation" (Tier B) and "Explicit Structured Path" (Tier C) of the ChangeGuard Intent Capture mechanism. This track focuses strictly on the terminal user interface (TUI) using `ratatui` and `crossterm`.

## Requirements
*   Implement a new `changeguard intent demo` command to launch the TUI with mock data.
*   Use `ratatui` for rendering and `crossterm` for terminal backend.
*   Single-screen layout (no scrolling) fitting within 80x24 standard terminal bounds.

## Layout Specifications
*   **WHAT (3 lines):** Pre-filled diff summary.
*   **WHY (3 lines):** The primary editable intent field.
*   **RISK (1 line):** Dropdown/Enum selection (Trivial, Low, Medium, High, Critical).
*   **RELATED (1 line):** Chips for related ADRs or tickets.
*   **CONFIDENCE (1 line):** Read-only confidence score.
*   **Status Bar (Bottom):** Hotkeys `Enter` (Accept), `Tab` (Next Field), `e` (Edit), `s` (Skip), `Esc` (Abort).

## Color Coding
*   Green: LLM filled with high confidence.
*   Yellow: LLM filled with low confidence (needs review).
*   Red: Required field is empty.
*   Cyan/Gray: Standard borders and unselected fields.

## Dependencies
*   Add `ratatui = "0.30.0"`
*   Add `crossterm = "0.29.0"`