# Track I4-1 Plan: VRAM Pressure Monitoring in Doctor

## Phase 1 — Red (Failing Tests)

- [ ] Create `src/platform/gpu.rs` with `VramInfo` struct and `query_vram_usage` stub (returns `Err("not implemented")`).
- [ ] Create `src/platform/mod.rs` if absent; add `pub mod gpu;`.
- [ ] Write unit tests (no GPU required — test only the threshold logic):
  - `vram_no_warning_below_threshold`: `VramInfo { budget: 12GB, current: 10GB }` → no warning.
  - `vram_warning_at_875`: `VramInfo { budget: 12GB, current: 10_500_000_000 }` → yellow `⚠`.
  - `vram_critical_at_95`: `VramInfo { budget: 12GB, current: 11_400_000_000 }` → red `✗`.
- [ ] Commit: `test(gpu): red — VRAM warning threshold logic`

## Phase 2 — Green (Implementation)

- [ ] Add `windows` crate dependency to `Cargo.toml` under `[target.'cfg(target_os = "windows")'.dependencies]`:
  ```toml
  windows = { version = "0.58", features = ["Win32_Graphics_Dxgi", "Win32_Graphics_Dxgi_Common", "Win32_System_Com"] }
  ```
- [ ] Implement `query_vram_usage` in `src/platform/gpu.rs`:
  - Use `CreateDXGIFactory2`, `EnumAdapters1(0)`, cast to `IDXGIAdapter3`, call `QueryVideoMemoryInfo` with `DXGI_MEMORY_SEGMENT_GROUP_LOCAL`.
  - Wrap in `unsafe { ... }`. Map `windows::core::Error` to `String` via `.map_err(|e| e.message().to_string())`.
- [ ] Add warning-level classifier:
  ```rust
  pub enum VramPressure { Ok, High, Critical }
  pub fn classify(info: &VramInfo) -> VramPressure { ... }
  ```
- [ ] In `src/commands/doctor.rs`:
  - Call `query_vram_usage()` (Windows only via `#[cfg]`).
  - Format and print the VRAM line with appropriate icon/color.
  - On non-Windows, print the `n/a` line.
- [ ] Run CI gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.
- [ ] Commit: `feat(doctor): VRAM pressure monitoring via DXGI for Intel Arc (I4-1)`

## Verification

- [ ] `changeguard doctor` on Windows with router running shows VRAM line with actual numbers.
- [ ] With both models loaded (~11 GB): yellow `⚠` warning appears.
- [ ] With only one model loaded (~9 GB): no warning.
- [ ] Build on Linux (cross-compile or CI): VRAM section absent, no compile errors.
