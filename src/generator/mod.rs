//! Core SnowID generator implementation

mod base62_methods;

use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::SnowIDConfig;
use crate::error::SnowIDError;
use crate::extractor::SnowIDExtractor;

/// Main ID generator with cache-line alignment to prevent false sharing
#[derive(Debug)]
#[repr(align(64))]
pub struct SnowID {
    /// Node ID for this generator
    pub node_id: u16,

    /// Configuration for this generator
    pub config: SnowIDConfig,

    /// Extractor for decomposing IDs
    pub extract: SnowIDExtractor,

    /// Last timestamp used to generate an ID (hot atomic)
    last_timestamp: AtomicU64,

    /// Sequence counter for IDs generated in the same millisecond
    sequence: AtomicU16,
}

impl SnowID {
    pub const TIMESTAMP_BITS: u32 = 42;
    pub const TOTAL_NODE_AND_SEQUENCE_BITS: u8 = 22;
    const MAX_BACKOFF_MS: u64 = 100;

    /// Create a new SnowID generator with default configuration
    pub fn new(node_id: u16) -> Result<Self, SnowIDError> {
        Self::with_config(node_id, SnowIDConfig::default())
    }

    /// Create a new SnowID generator with custom configuration
    pub fn with_config(node_id: u16, config: SnowIDConfig) -> Result<Self, SnowIDError> {
        let max_node_id = config.max_node_id();
        if node_id > max_node_id {
            return Err(SnowIDError::InvalidNodeId {
                node_id,
                max: max_node_id,
            });
        }
        Ok(Self {
            node_id,
            config,
            extract: SnowIDExtractor::new(config),
            last_timestamp: AtomicU64::new(0),
            sequence: AtomicU16::new(0),
        })
    }

    /// Generate a new SnowID
    #[inline]
    pub fn generate(&self) -> u64 {
        let now = self.get_time_since_epoch();
        let last_ts = self.last_timestamp.load(Ordering::Acquire);

        // Fast path: time has advanced - try to update timestamp inline
        if now > last_ts {
            if let Some(id) = self.try_advance_timestamp(last_ts, now) {
                return id;
            }
            return self.generate_slow_path();
        }

        // Same millisecond - fast path: try to get next sequence slot
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        if seq < self.config.max_sequence_id() {
            return self.create_snowid(last_ts, seq + 1);
        }

        self.generate_slow_path()
    }

    /// Slow path for ID generation when fast path fails
    #[cold]
    #[inline(never)]
    fn generate_slow_path(&self) -> u64 {
        let mut backoff_ms = 1u64;

        loop {
            let now = self.get_time_since_epoch();
            let last_ts = self.last_timestamp.load(Ordering::Acquire);
            let ts = now.max(last_ts);

            if ts > last_ts {
                if let Some(id) = self.try_advance_timestamp(last_ts, ts) {
                    return id;
                }
                continue;
            }

            let seq_prev = self.sequence.fetch_add(1, Ordering::AcqRel);
            if seq_prev < self.config.max_sequence_id() {
                return self.create_snowid(ts, seq_prev + 1);
            }

            let next_ts = self.wait_next_millis(ts, backoff_ms);
            backoff_ms = (backoff_ms.saturating_mul(2)).min(Self::MAX_BACKOFF_MS);

            loop {
                let current_last = self.last_timestamp.load(Ordering::Acquire);
                if next_ts <= current_last {
                    break;
                }
                if let Some(id) = self.try_advance_timestamp(current_last, next_ts) {
                    return id;
                }
            }
        }
    }

    /// Get current time in milliseconds since epoch
    #[inline(always)]
    pub(crate) fn get_time_since_epoch(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before Unix epoch!");
        now.as_millis() as u64 - self.config.epoch()
    }

    /// Wait until next millisecond with optional spin/yield before sleeping
    pub(crate) fn wait_next_millis(&self, from_timestamp: u64, mut backoff_ms: u64) -> u64 {
        loop {
            if self.config.spin_enabled() && self.config.spin_loops() > 0 {
                let yield_every = self.config.spin_yield_every();
                for i in 0..self.config.spin_loops() {
                    if let Some(new_ts) = self.check_timestamp_advanced(from_timestamp) {
                        return new_ts;
                    }
                    std::hint::spin_loop();
                    if yield_every != 0 && i % yield_every == yield_every - 1 {
                        thread::yield_now();
                    }
                }
            }

            thread::sleep(Duration::from_millis(backoff_ms));
            if let Some(new_ts) = self.check_timestamp_advanced(from_timestamp) {
                return new_ts;
            }
            backoff_ms = backoff_ms.saturating_mul(2).min(Self::MAX_BACKOFF_MS);
        }
    }

    #[inline]
    fn check_timestamp_advanced(&self, from_timestamp: u64) -> Option<u64> {
        let new_ts = self.get_time_since_epoch();
        (new_ts > from_timestamp).then_some(new_ts)
    }

    #[inline(always)]
    fn try_advance_timestamp(&self, old_ts: u64, new_ts: u64) -> Option<u64> {
        match self.last_timestamp.compare_exchange_weak(
            old_ts,
            new_ts,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                self.sequence.store(0, Ordering::Release);
                Some(self.create_snowid(new_ts, 0))
            }
            Err(_) => None,
        }
    }

    #[inline(always)]
    fn create_snowid(&self, timestamp: u64, sequence: u16) -> u64 {
        self.create_snowid_with_node(timestamp, self.node_id, sequence)
    }

    #[inline(always)]
    pub(crate) fn create_snowid_with_node(
        &self,
        timestamp: u64,
        node_id: u16,
        sequence: u16,
    ) -> u64 {
        ((timestamp & self.config.timestamp_mask()) << self.config.timestamp_shift())
            | ((node_id as u64) << self.config.node_shift())
            | (sequence as u64)
    }
}
