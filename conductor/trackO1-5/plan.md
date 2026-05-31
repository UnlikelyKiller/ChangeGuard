# Plan: Track O1-5 (Cryptographic Provenance)

- [ ] 1. Add `ed25519-dalek` to `Cargo.toml`.
- [ ] 2. Create `src/ledger/crypto.rs` for key generation, signing, and verification logic.
- [ ] 3. Update `src/commands/init.rs` to generate and persist a keypair if it does not exist.
- [ ] 4. Create SQLite migration `M16` and equivalent CozoDB schema changes for `signature` and `public_key` columns in `ledger_entry`.
- [ ] 5. Update `src/ledger/transaction.rs` and `src/ledger/db.rs` to sign payloads during `commit()`.
- [ ] 6. Update `src/commands/verify.rs` to handle `--signatures`.
- [ ] 7. Write integration tests to ensure tampered data is caught by the verification step.