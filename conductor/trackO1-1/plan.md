# Plan: Track O1-1 (Intent Capture TUI Scaffold)

- [ ] 1. Add `ratatui` and `crossterm` to `Cargo.toml`.
- [ ] 2. Create `src/commands/intent.rs` with a `demo` subcommand.
- [ ] 3. Create `src/ui/intent_tui.rs` to house the `ratatui` rendering logic.
- [ ] 4. Define the `IntentState` struct with `what`, `why`, `risk`, `related`, and `confidence` fields, plus an `active_field` enum.
- [ ] 5. Implement the main render loop with `crossterm::event::read()`.
- [ ] 6. Render the 5 required layout blocks using `ratatui::layout::Layout`.
- [ ] 7. Apply color coding based on a mock "confidence" tier (High > 0.85 = Green, Low < 0.85 = Yellow).
- [ ] 8. Implement basic keyboard navigation (Tab between fields, Esc to exit).
- [ ] 9. Wire `changeguard intent demo` in `src/cli.rs`.