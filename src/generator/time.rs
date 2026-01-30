//! Time utilities for SnowID generation
//!
//! Provides wall-clock time in milliseconds since custom epoch

use std::time::{SystemTime, UNIX_EPOCH};

/// Get current wall-clock time in milliseconds since Unix epoch
#[inline(always)]
pub fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before Unix epoch!")
        .as_millis() as u64
}

/// Get current time in milliseconds since custom epoch
#[inline(always)]
pub fn time_since_epoch(epoch: u64) -> u64 {
    unix_time_ms() - epoch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_time_is_reasonable() {
        let now = unix_time_ms();
        // Should be after 2024-01-01
        assert!(now > 1704067200000);
        // Should be before 2100-01-01
        assert!(now < 4102444800000);
    }

    #[test]
    fn test_time_since_epoch() {
        let epoch = 1704067200000u64; // 2024-01-01
        let ts = time_since_epoch(epoch);
        // Should be positive (after 2024)
        assert!(ts > 0);
        // Should be less than 100 years in ms
        assert!(ts < 100 * 365 * 24 * 60 * 60 * 1000);
    }
}
