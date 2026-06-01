# Track U2 Plan: AI-Brains Daemon Status Subcommand

- [ ] Task U2.1: Add `status` option to the daemon command dispatch mapping in `crates/ai-brains-cli/src/commands/daemon.rs`.
- [ ] Task U2.2: Implement process connectivity check to probe if completions and embedding ports are actively listening.
- [ ] Task U2.3: Read state files or PID indicators (e.g. from app data directory) to output the daemon's active process ID.
- [ ] Task U2.4: Test status reporting when daemon is stopped vs running.
