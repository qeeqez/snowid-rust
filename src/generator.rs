//! Core SnowID generator implementation

use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::base62::{decode as base62_decode, encode as base62_encode};
use crate::base62::{encode_array as base62_encode_array, encode_into as base62_encode_into};
use crate::base62::{DecodeError as Base62DecodeError, MAX_LEN as BASE62_MAX_LEN};
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

    /// Last timestamp used to generate an ID (hot atomic, cache-line aligned)
    last_timestamp: AtomicU64,

    /// Sequence counter for IDs generated in the same millisecond (hot atomic)
    sequence: AtomicU16,
}

impl SnowID {
    pub const TIMESTAMP_BITS: u32 = 42;
    pub const TOTAL_NODE_AND_SEQUENCE_BITS: u8 = 22;
    const MAX_BACKOFF_MS: u64 = 100;

    /// Create a new SnowID generator with default configuration
    ///
    /// # Arguments
    ///
    /// * `node_id` - Node ID to use in generated IDs
    ///
    /// # Returns
    /// * `Result<SnowID, Error>` - New SnowID generator or error if node_id is invalid
    pub fn new(node_id: u16) -> Result<Self, SnowIDError> {
        Self::with_config(node_id, SnowIDConfig::default())
    }

    /// Create a new SnowID generator with custom configuration
    ///
    /// # Arguments
    ///
    /// * `node_id` - Node ID to use in generated IDs
    /// * `config` - Custom configuration
    ///
    /// # Returns
    /// * `Result<SnowID, Error>` - New SnowID generator or error if node_id is invalid
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
    ///
    /// # Returns
    /// * `u64` - New SnowID value
    #[inline]
    pub fn generate(&self) -> u64 {
        // Get current wall-clock time
        let now = self.get_time_since_epoch();
        let last_ts = self.last_timestamp.load(Ordering::Acquire);

        // Fast path: time has advanced - try to update timestamp inline
        if now > last_ts {
            // Try to claim the new millisecond
            if let Some(id) = self.try_advance_timestamp(last_ts, now) {
                return id;
            }
            // Lost the race, fall through to slow path
            return self.generate_slow_path();
        }

        // Same millisecond - fast path: try to get next sequence slot
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        if seq < self.config.max_sequence_id() {
            return self.create_snowid(last_ts, seq + 1);
        }

        // Sequence exhausted - slow path to wait for next millisecond
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

            // Clamp to last seen to ensure monotonic timestamp under clock regression
            let ts = now.max(last_ts);

            if ts > last_ts {
                if let Some(id) = self.try_advance_timestamp(last_ts, ts) {
                    return id;
                }
                continue;
            }

            // Same millisecond: increment sequence atomically
            let seq_prev = self.sequence.fetch_add(1, Ordering::AcqRel);
            if seq_prev < self.config.max_sequence_id() {
                return self.create_snowid(ts, seq_prev + 1);
            }

            // Sequence exhausted: wait for next millisecond
            let next_ts = self.wait_next_millis(ts, backoff_ms);
            backoff_ms = (backoff_ms.saturating_mul(2)).min(Self::MAX_BACKOFF_MS);

            // Try to publish the advanced timestamp
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

    /// Check if timestamp has advanced beyond the given value
    #[inline]
    fn check_timestamp_advanced(&self, from_timestamp: u64) -> Option<u64> {
        let new_ts = self.get_time_since_epoch();
        (new_ts > from_timestamp).then_some(new_ts)
    }

    /// Try to advance timestamp and reset sequence, returning ID if successful
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

// Base62 convenience methods
impl SnowID {
    /// Generate a new base62 encoded SnowID (zero-allocation, array-based)
    #[inline]
    pub fn generate_base62_array(&self) -> ([u8; BASE62_MAX_LEN], usize) {
        let id = self.generate();
        base62_encode_array(id)
    }

    /// Generate a new base62 encoded SnowID into caller-provided buffer
    #[inline]
    pub fn generate_base62_into<'a>(&self, buf: &'a mut [u8; BASE62_MAX_LEN]) -> (&'a str, u64) {
        let id = self.generate();
        (base62_encode_into(id, buf), id)
    }

    /// Generate a new base62 encoded SnowID (allocates String)
    pub fn generate_base62(&self) -> String {
        let id = self.generate();
        base62_encode(id)
    }

    /// Generate base62 encoded SnowID with raw u64 value
    pub fn generate_base62_with_raw(&self) -> (String, u64) {
        let id = self.generate();
        (base62_encode(id), id)
    }

    /// Decode a base62 encoded SnowID back to its raw u64 value
    pub fn decode_base62(&self, encoded: &str) -> Result<u64, Base62DecodeError> {
        base62_decode(encoded)
    }

    /// Decompose a base62 encoded SnowID into its components
    pub fn decompose_base62(&self, encoded: &str) -> Result<(u64, u16, u16), Base62DecodeError> {
        let id = self.decode_base62(encoded)?;
        Ok(self.extract.decompose(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base62_generate() {
        let generator = SnowID::new(1).unwrap();
        let id = generator.generate_base62();

        assert!(!id.is_empty());

        let decoded = generator.decode_base62(&id).unwrap();
        let (timestamp, node_id, sequence) = generator.extract.decompose(decoded);

        assert_eq!(node_id, 1);
        assert!(timestamp > 0);
        assert!(sequence <= generator.config.max_sequence_id());
    }

    #[test]
    fn test_base62_with_raw() {
        let generator = SnowID::new(1).unwrap();
        let (id, raw) = generator.generate_base62_with_raw();
        assert_eq!(base62_decode(&id).unwrap(), raw);
    }

    #[test]
    fn test_base62_decompose() {
        let generator = SnowID::new(1).unwrap();
        let id = generator.generate_base62();
        let (timestamp, node_id, sequence) = generator.decompose_base62(&id).unwrap();

        assert_eq!(node_id, 1);
        assert!(timestamp > 0);
        assert!(sequence <= generator.config.max_sequence_id());
    }
}
