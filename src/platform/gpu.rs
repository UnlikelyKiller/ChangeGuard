#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VramPressure {
    Ok,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy)]
pub struct VramInfo {
    pub budget_bytes: u64,
    pub current_usage: u64,
}

#[cfg(target_os = "windows")]
pub fn query_vram_usage() -> Result<VramInfo, String> {
    use windows::Win32::Graphics::Dxgi::*;
    use windows::core::Interface;
    unsafe {
        let factory: IDXGIFactory4 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))
            .map_err(|e| e.message().to_string())?;

        let mut best_info: Option<VramInfo> = None;
        let mut i = 0;

        while let Ok(adapter) = factory.EnumAdapters1(i) {
            let desc = adapter.GetDesc1().map_err(|e| e.message().to_string())?;

            // Skip software adapters
            if (desc.Flags & 2) != 0 {
                i += 1;
                continue;
            }

            let adapter3: IDXGIAdapter3 = adapter.cast().map_err(|e| e.message().to_string())?;

            // Check both LOCAL (Dedicated) and NON_LOCAL (Shared) for usage
            let groups = [
                DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
                DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL,
            ];

            for &group in &groups {
                for node in 0..4 {
                    let mut info = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
                    if adapter3
                        .QueryVideoMemoryInfo(node, group, &mut info)
                        .is_ok()
                    {
                        if info.Budget > 0 {
                            let current = VramInfo {
                                budget_bytes: info.Budget,
                                current_usage: info.CurrentUsage,
                            };

                            match best_info {
                                Some(prev) => {
                                    // Prefer the adapter reporting the most usage (likely the one being benchmarked)
                                    // or fall back to the one with the biggest budget.
                                    if current.current_usage > prev.current_usage
                                        || (current.current_usage == prev.current_usage
                                            && current.budget_bytes > prev.budget_bytes)
                                    {
                                        best_info = Some(current);
                                    }
                                }
                                None => {
                                    best_info = Some(current);
                                }
                            }
                        }
                    }
                }
            }
            i += 1;
        }

        best_info
            .ok_or_else(|| "No active GPU adapter found with reported memory budget".to_string())
    }
}

#[cfg(not(target_os = "windows"))]
pub fn query_vram_usage() -> Result<VramInfo, String> {
    Err("not implemented".to_string())
}

pub fn classify(info: &VramInfo) -> VramPressure {
    if info.budget_bytes == 0 {
        return VramPressure::Ok;
    }
    let ratio = info.current_usage as f64 / info.budget_bytes as f64;
    if ratio >= 0.95 {
        VramPressure::Critical
    } else if ratio >= 0.875 {
        VramPressure::High
    } else {
        VramPressure::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vram_no_warning_below_threshold() {
        let info = VramInfo {
            budget_bytes: 12_000_000_000,
            current_usage: 10_000_000_000,
        };
        assert_eq!(classify(&info), VramPressure::Ok);
    }

    #[test]
    fn vram_warning_at_875() {
        let info = VramInfo {
            budget_bytes: 12_000_000_000,
            current_usage: 10_500_000_000,
        };
        assert_eq!(classify(&info), VramPressure::High);
    }

    #[test]
    fn vram_critical_at_95() {
        let info = VramInfo {
            budget_bytes: 12_000_000_000,
            current_usage: 11_400_000_000,
        };
        assert_eq!(classify(&info), VramPressure::Critical);
    }
}
