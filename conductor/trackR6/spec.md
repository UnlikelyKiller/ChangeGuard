# Specification: GPU VRAM Reporting & Binary Lock Resilience

## Objective
Fix inaccurate VRAM reporting on systems with multiple GPUs (e.g., Intel Arc B580) and improve the resilience of `cargo install` against Windows binary file locks.

## Requirements
### GPU VRAM Fix
- Target file: `src/platform/gpu.rs`.
- Current issue: `doctor` shows 0.0 GB VRAM. `EnumAdapters1(0)` picks the integrated GPU.
- Fix: Iterate over `EnumAdapters1(i)` until a discrete GPU with significant dedicated VRAM (>0) is found, or sum them appropriately.

### Binary Lock Resilience
- Target file: `src/commands/update.rs` or generic utility (if `cargo install` is invoked via `changeguard`).
- Mitigation: Before triggering a compilation/installation that overwrites the currently running executable on Windows, check if the executable is locked. Provide a warning or suggest the user rename the old binary / use `cargo install --force` to break the lock if possible (or gracefully tell them to stop the daemon/other instances).

## Architecture
- `gpu.rs`: Use DXGI loop `while factory.EnumAdapters1(i, &mut adapter).is_ok()`. Find `DXGI_ADAPTER_DESC1`.
- `update.rs` / `install`: Use standard `std::fs::OpenOptions` with write access to test for a lock on the target binary path before spawning the `cargo` process.