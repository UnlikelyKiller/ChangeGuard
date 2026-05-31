## Plan: Advanced CLI Help Interceptor
### Phase 1: Implementation
- [x] Task 1.1: Modify `src/main.rs` (or `src/cli.rs`) to inject an argument pre-processor before `Cli::parse()`.
- [x] Task 1.2: Implement logic to scan `env::args()` for `help` as the first positional command.
- [x] Task 1.3: Transform the args array from `[exe, help, cmd1, cmd2]` to `[exe, cmd1, cmd2, --help]`.
### Phase 2: Testing & Verification
- [x] Task 2.1: Verify `changeguard help ledger` correctly shows the ledger help.
- [x] Task 2.2: Verify `changeguard help ledger audit` correctly shows the nested audit help.
- [x] Task 2.3: Ensure no regression on standard `--help` usage.