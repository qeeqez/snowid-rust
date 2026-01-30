//! Wait and backoff strategies for sequence exhaustion
//!
//! Implements spin-wait and exponential backoff for waiting until next millisecond

use std::thread;
use std::time::Duration;

use crate::config::SnowIDConfig;

/// Maximum backoff duration in milliseconds
pub const MAX_BACKOFF_MS: u64 = 100;

/// Perform spin-wait loop, checking for timestamp advancement
///
/// Returns Some(new_ts) if timestamp advanced, None if spin loops exhausted
#[inline]
pub fn spin_wait<F>(from_timestamp: u64, config: &SnowIDConfig, get_time: F) -> Option<u64>
where
    F: Fn() -> u64,
{
    if !config.spin_enabled() || config.spin_loops() == 0 {
        return None;
    }

    let yield_every = config.spin_yield_every();

    for i in 0..config.spin_loops() {
        let new_ts = get_time();
        if new_ts > from_timestamp {
            return Some(new_ts);
        }

        std::hint::spin_loop();

        if yield_every != 0 && i % yield_every == yield_every - 1 {
            thread::yield_now();
        }
    }

    None
}

/// Sleep with exponential backoff, returning new timestamp once advanced
#[inline]
pub fn sleep_until_next_ms<F>(from_timestamp: u64, mut backoff_ms: u64, get_time: F) -> u64
where
    F: Fn() -> u64,
{
    loop {
        thread::sleep(Duration::from_millis(backoff_ms));
        let new_ts = get_time();
        if new_ts > from_timestamp {
            return new_ts;
        }
        backoff_ms = next_backoff(backoff_ms);
    }
}

/// Calculate next backoff duration with exponential growth capped at MAX_BACKOFF_MS
#[inline(always)]
pub const fn next_backoff(current: u64) -> u64 {
    let next = current.saturating_mul(2);
    if next > MAX_BACKOFF_MS {
        MAX_BACKOFF_MS
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_backoff() {
        assert_eq!(next_backoff(1), 2);
        assert_eq!(next_backoff(50), 100);
        assert_eq!(next_backoff(100), 100); // Capped at MAX_BACKOFF_MS
        assert_eq!(next_backoff(200), 100); // Already over, still capped
    }

    #[test]
    fn test_spin_wait_disabled() {
        let config = SnowIDConfig::builder().enable_spin(false).build();
        let result = spin_wait(100, &config, || 200);
        assert!(result.is_none());
    }

    #[test]
    fn test_spin_wait_immediate_advance() {
        let config = SnowIDConfig::builder()
            .enable_spin(true)
            .spin_loops(10)
            .build();
        let result = spin_wait(100, &config, || 200);
        assert_eq!(result, Some(200));
    }
}
