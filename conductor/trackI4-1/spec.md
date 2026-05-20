# Track I4-1: VRAM Pressure Monitoring in Doctor

**Milestone:** I — Issue Remediation  
**Phase:** 4 — LLM Router Hardening (Parallel)  
**Status:** In Planning

## Objective

`changeguard doctor` does not report GPU VRAM usage. The Intel Arc B580 has 12 GB VRAM. Running both `qwen3.5-9b` (Q6_K, ~9 GB) and `bge-m3` (Q8_0, ~2–3 GB) simultaneously risks spilling into system RAM, degrading generation from ~35 TPS to ~2 TPS. Add a VRAM section to `doctor` that shows used/budget/available and emits a warning when pressure is high.

## Requirements

### DXGI-Based VRAM Query (Windows Only)
Use `IDXGIAdapter3::QueryVideoMemoryInfo` (DXGI 1.4, `dxgi1_4.h`) via the `windows` crate.

**New Cargo.toml dependency:**
```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Graphics_Dxgi",
    "Win32_Graphics_Dxgi_Common",
    "Win32_System_Com",
] }
```

Note: The project already uses `windows-sys = "0.59"` for pipe IPC. The `windows` crate (COM-capable, higher-level) is distinct from `windows-sys`. Both can coexist.

**Query logic** (in `src/commands/doctor.rs` or a new `src/platform/gpu.rs`):
```rust
#[cfg(target_os = "windows")]
pub fn query_vram_usage() -> Result<VramInfo, String> {
    use windows::Win32::Graphics::Dxgi::*;

    unsafe {
        let factory: IDXGIFactory4 = CreateDXGIFactory2(0)?;
        let adapter: IDXGIAdapter3 = factory.EnumAdapters1(0)?.cast()?;
        let mut info = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
        adapter.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut info)?;
        Ok(VramInfo {
            budget_bytes:  info.Budget,
            current_usage: info.CurrentUsage,
        })
    }
}
```

`DXGI_MEMORY_SEGMENT_GROUP_LOCAL` queries dedicated VRAM (not shared system RAM).

### `VramInfo` Struct
```rust
pub struct VramInfo {
    pub budget_bytes: u64,   // OS-assigned VRAM budget
    pub current_usage: u64,  // Current process + system usage
}
```

### Warning Threshold
Emit a yellow `⚠` warning in `doctor` when `current_usage / budget_bytes > 0.875` (87.5% of 12 GB = 10.5 GB). Emit a red `✗` if > 95%.

### Doctor Output (Windows)
```
GPU VRAM (adapter 0):  9.2 GB used / 12.0 GB budget  ⚠  High pressure — avoid running both models simultaneously
```

### Non-Windows Fallback
On non-Windows targets, skip the VRAM section entirely or print:
```
GPU VRAM:  n/a (Windows-only monitoring)
```

### Module Placement
New file: `src/platform/gpu.rs` (alongside future platform utilities). Add `pub mod gpu;` to `src/platform/mod.rs` (create if absent).

## API Contract

```rust
// src/platform/gpu.rs
pub struct VramInfo { pub budget_bytes: u64, pub current_usage: u64 }

#[cfg(target_os = "windows")]
pub fn query_vram_usage() -> Result<VramInfo, String>;
```

`doctor` calls this and formats the output. `query_vram_usage` is `unsafe` internally but presents a safe public API.

## Testing Strategy

Testing `unsafe` DXGI calls in unit tests is impractical without a GPU. Instead:
- Unit test `vram_warning_threshold`: given `VramInfo { budget: 12*GB, current: 11*GB }`, assert the warning level is `⚠`. Given 95%+, assert `✗`. Given <87.5%, assert no warning.
- The DXGI query itself is verified manually via `changeguard doctor`.
- Compile-time test: ensure the `windows` feature flags compile on Windows CI.

## Out of Scope

- Per-process VRAM breakdown (DXGI only gives system-wide usage for the local memory segment).
- Non-Intel GPU support (NVIDIA / AMD use different APIs; `EnumAdapters1(0)` returns whatever adapter Windows assigns as primary).
- Router configuration changes (`--sleep-idle-seconds` tuning is a documentation recommendation, not a code change).
- The router documentation and `127.0.0.1` config comment updates are small enough to bundle with Track I1-1 or as a standalone docs commit; they are not a separate track.
