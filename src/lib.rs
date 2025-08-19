#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

mod config;
mod error;
mod extractor;
#[cfg(test)]
pub mod tests;

pub use config::SnowIDConfig;
pub use error::SnowIDError;
pub use extractor::SnowIDExtractor;

// Re-export base62 functions from the external crate with appropriate type conversions
pub fn base62_encode(id: u64) -> String {
    // Maximum size needed for u64 in base62 is 11 bytes
    let mut buf = [0u8; 11];
    let len = base62::encode_bytes(id, &mut buf).unwrap();
    // base62 output is always valid ASCII
    std::str::from_utf8(&buf[..len]).unwrap().to_owned()
}

// Decode a base62 string to a u64, handling potential overflow
pub fn base62_decode(encoded: &str) -> Result<u64, Base62DecodeError> {
    let decoded = base62::decode(encoded).map_err(Base62DecodeError::from)?;

    // Check if the decoded value fits in a u64
    if decoded > u64::MAX as u128 {
        return Err(Base62DecodeError::Overflow);
    }

    Ok(decoded as u64)
}

// Define our own error type that wraps the external crate's error
#[derive(Debug, thiserror::Error)]
pub enum Base62DecodeError {
    #[error("Invalid base62 character")]
    InvalidCharacter,

    #[error("Decoded value would overflow u64")]
    Overflow,

    #[error("Base62 decode error: {0}")]
    Other(#[from] base62::DecodeError),
}

/// Main ID generator
#[derive(Debug)]
pub struct SnowID {
    /// Node ID for this generator
    pub node_id: u16,

    /// Configuration for this generator
    pub config: SnowIDConfig,

    /// Extractor for decomposing IDs
    pub extract: SnowIDExtractor,

    /// Last timestamp used to generate an ID
    last_timestamp: AtomicU64,

    /// Sequence counter for IDs generated in the same millisecond
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
        // Validate node ID
        if node_id > config.max_node_id() {
            return Err(SnowIDError::InvalidNodeId {
                node_id,
                max: config.max_node_id(),
            });
        }

        // Create extractor with the same configuration
        let extract = SnowIDExtractor::new(config);

        Ok(Self {
            node_id,
            config,
            extract,
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
        let mut backoff_ms = 1u64;

        loop {
            // Read current time and last seen timestamp
            let now = self.get_time_since_epoch();
            let last_ts = self.last_timestamp.load(Ordering::Acquire);

            // Clamp to last seen to ensure monotonic timestamp under clock regression
            let ts = now.max(last_ts);

            if ts > last_ts {
                // Try to move the generator to the new millisecond
                match self
                    .last_timestamp
                    .compare_exchange(last_ts, ts, Ordering::AcqRel, Ordering::Acquire)
                {
                    Ok(_) => {
                        // New millisecond: start sequence at 0
                        self.sequence.store(0, Ordering::Release);
                        return self.create_snowid(ts, 0);
                    }
                    Err(_actual) => {
                        // Someone else advanced the timestamp; retry
                        // fall-through to same-ts handling below on next loop iteration
                        continue;
                    }
                }
            }

            // Same millisecond: increment sequence atomically and use the returned slot
            let seq_prev = self.sequence.fetch_add(1, Ordering::AcqRel);
            if seq_prev < self.config.max_sequence_id() {
                let seq_to_use = seq_prev + 1;
                return self.create_snowid(ts, seq_to_use);
            }

            // Sequence exhausted: wait for the next millisecond with exponential backoff
            let wait_from = ts;
            let next_ts = self.wait_next_millis(wait_from, backoff_ms);
            backoff_ms = (backoff_ms.saturating_mul(2)).min(Self::MAX_BACKOFF_MS);

            // Try to publish the advanced timestamp and reset sequence
            loop {
                let current_last = self.last_timestamp.load(Ordering::Acquire);
                if next_ts <= current_last {
                    // Another thread already advanced; restart outer loop
                    break;
                }
                match self.last_timestamp.compare_exchange(
                    current_last,
                    next_ts,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        self.sequence.store(0, Ordering::Release);
                        return self.create_snowid(next_ts, 0);
                    }
                    Err(_) => {
                        // Lost the race; retry inner publish or restart
                        continue;
                    }
                }
            }
        }
    }

    /// Get current time in milliseconds since epoch
    fn get_time_since_epoch(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before Unix epoch!");

        // Convert to milliseconds and subtract the custom epoch
        let millis = now.as_millis() as u64;
        millis.saturating_sub(self.config.epoch())
    }

    /// Wait until next millisecond with exponential backoff (iterative)
    fn wait_next_millis(&self, from_timestamp: u64, mut backoff_ms: u64) -> u64 {
        loop {
            thread::sleep(Duration::from_millis(backoff_ms));
            let new_ts = self.get_time_since_epoch();
            if new_ts > from_timestamp {
                return new_ts;
            }
            backoff_ms = (backoff_ms.saturating_mul(2)).min(Self::MAX_BACKOFF_MS);
        }
    }

    #[inline]
    fn create_snowid(&self, timestamp: u64, sequence: u16) -> u64 {
        self.create_snowid_with_node(timestamp, self.node_id, sequence)
    }

    #[inline]
    fn create_snowid_with_node(&self, timestamp: u64, node_id: u16, sequence: u16) -> u64 {
        ((timestamp & self.config.timestamp_mask()) << self.config.timestamp_shift())
            | ((node_id as u64 & self.config.node_mask() as u64) << self.config.node_shift())
            | (sequence as u64 & self.config.sequence_mask() as u64)
    }

    /// Generate a new base62 encoded SnowID
    ///
    /// # Returns
    /// * `String` - New base62 encoded SnowID value
    pub fn generate_base62(&self) -> String {
        let id = self.generate();
        base62_encode(id)
    }

    /// Generate a new base62 encoded SnowID and return both the encoded string and the raw u64 value
    ///
    /// # Returns
    /// * `(String, u64)` - Tuple containing the base62 encoded SnowID and the raw u64 value
    pub fn generate_base62_with_raw(&self) -> (String, u64) {
        let id = self.generate();
        (base62_encode(id), id)
    }

    /// Decode a base62 encoded SnowID back to its raw u64 value
    ///
    /// # Arguments
    /// * `encoded` - The base62 encoded SnowID string
    ///
    /// # Returns
    /// * `Result<u64, Base62DecodeError>` - The decoded u64 SnowID or an error
    pub fn decode_base62(&self, encoded: &str) -> Result<u64, Base62DecodeError> {
        base62_decode(encoded)
    }

    /// Decompose a base62 encoded SnowID into its components: timestamp, node ID, and sequence
    ///
    /// # Arguments
    /// * `encoded` - The base62 encoded SnowID string
    ///
    /// # Returns
    /// * `Result<(u64, u16, u16), Base62DecodeError>` - Tuple containing the components or an error
    pub fn decompose_base62(&self, encoded: &str) -> Result<(u64, u16, u16), Base62DecodeError> {
        let id = self.decode_base62(encoded)?;
        Ok(self.extract.decompose(id))
    }
}

#[cfg(test)]
mod base62_tests {
    use super::*;

    #[test]
    fn test_base62_generate() {
        let generator = SnowID::new(1).unwrap();

        // Generate a base62 ID
        let id = generator.generate_base62();

        // It should be a non-empty string
        assert!(!id.is_empty());

        // It should be decodable
        let decoded = generator.decode_base62(&id).unwrap();

        // The decoded value should be a valid SnowID
        let (timestamp, node_id, sequence) = generator.extract.decompose(decoded);

        // Check that the node ID is correct
        assert_eq!(node_id, 1);

        // Check that the timestamp is reasonable (just verify it's not zero)
        assert!(timestamp > 0);

        // Sequence should be within bounds
        assert!(sequence <= generator.config.max_sequence_id());
    }

    #[test]
    fn test_base62_with_raw() {
        let generator = SnowID::new(1).unwrap();

        // Generate a base62 ID with raw value
        let (id, raw) = generator.generate_base62_with_raw();

        // Check that the encoded ID decodes to the raw value
        assert_eq!(base62_decode(&id).unwrap(), raw);
    }

    #[test]
    fn test_base62_decompose() {
        let generator = SnowID::new(1).unwrap();

        // Generate a base62 ID
        let id = generator.generate_base62();

        // Decompose it
        let (timestamp, node_id, sequence) = generator.decompose_base62(&id).unwrap();

        // Check that the node ID is correct
        assert_eq!(node_id, 1);

        // Check that the timestamp is reasonable (just verify it's not zero)
        assert!(timestamp > 0);

        // Sequence should be within bounds
        assert!(sequence <= generator.config.max_sequence_id());
    }
}
