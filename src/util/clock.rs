use chrono::{DateTime, Utc};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Debug, Clone)]
pub struct FixedClock {
    fixed: DateTime<Utc>,
}

impl FixedClock {
    pub fn new(fixed: DateTime<Utc>) -> Self {
        Self { fixed }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.fixed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_clock_returns_fixed_timestamp() {
        let fixed = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let clock = FixedClock::new(fixed);
        assert_eq!(clock.now(), fixed);
    }
}
