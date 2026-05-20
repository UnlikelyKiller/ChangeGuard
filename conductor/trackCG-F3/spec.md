## Spec: Fix ledger note Deprecation Warning (Track CG-F3)

### Acceptance Criteria

1. **No deprecation warning**: `changeguard ledger note docs/test.md "a note"` prints no deprecation warning
2. **Positional note works**: `changeguard ledger note docs/test.md "a note"` records the note (visible in `changeguard ledger search`)
3. **--message flag works**: `changeguard ledger note docs/test.md --message "a note"` also works (if flag added)
4. **Backward compat**: Existing ledger note entries are unaffected
5. **CI gate passes**: `cargo fmt --check`, `cargo clippy`, `cargo test` all pass
