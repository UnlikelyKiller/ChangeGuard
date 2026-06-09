## Plan: GPU VRAM Reporting & Binary Lock Resilience
### Phase 1: VRAM Adapter Iteration
- [ ] Task 1.1: Modify `src/platform/gpu.rs` to iterate over DXGI adapters instead of hardcoding index `0`.
- [ ] Task 1.2: Select the adapter with the highest `DedicatedVideoMemory` or matching a "discrete" flag.
### Phase 2: Binary Lock Detection
- [ ] Task 2.1: Implement a check using `OpenOptions::new().write(true)` on the target ChangeGuard executable path.
- [ ] Task 2.2: If locked, log a clear warning instructing the user to close running instances or daemon processes before continuing `cargo install`.
### Phase 3: Verification
- [ ] Task 3.1: Run `changeguard doctor` on a multi-GPU machine and verify >0.0 GB VRAM is reported.
- [ ] Task 3.2: Lock the executable manually and trigger the update/install path, verifying the warning appears instead of a hard build crash at the end of compilation.