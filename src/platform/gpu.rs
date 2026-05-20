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
