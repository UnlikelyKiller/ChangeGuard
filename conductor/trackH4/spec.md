# Track H4: Windows Deployment Safety

## Objective
Fix the "Access is denied" error during `update --binary` on Windows caused by file locks on the running executable.

## Requirements
- **Shadow Copy Strategy**: Implement a "move-before-replace" strategy. Rename the existing `changeguard.exe` to `changeguard.old` before attempting to install the new version via `cargo install`.
- **Cleanup**: Attempt to delete the `.old` binary after the new one is successfully placed, or mark it for deletion on the next reboot.
- **Error Feedback**: Provide clear instructions to the user if the lock cannot be broken (e.g., "Please close other instances of ChangeGuard").

## Definition of Done (DoD)
- [ ] `changeguard update --binary` completes successfully on Windows even when a previous version is in the cargo bin folder.
- [ ] No manual file deletion is required by the user during the update process.
