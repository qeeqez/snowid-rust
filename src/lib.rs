#![cfg_attr(test, deny(warnings))]

use std::time::{SystemTime, UNIX_EPOCH};

mod config;
mod error;
mod extractor;

#[cfg(test)]
mod tests;

pub use config::SnowIDConfig;
pub use error::SnowIDError;
pub use extractor::SnowIDExtractor;

/// Main ID generator
#[derive(Debug)]
pub struct SnowID {
    node_id: u16,
    config: SnowIDConfig,
    pub extract: SnowIDExtractor,
    last_timestamp: u64,
    sequence: u16,
}

impl SnowID {
    pub const TIMESTAMP_BITS: u32 = 42;
    pub const TOTAL_NODE_AND_SEQUENCE_BITS: u8 = 22;

    /// Create a new SnowID generator with default configuration
    ///
    /// # Arguments
    ///
    /// * `node_id` - Node ID to use in generated IDs
    ///
    /// # Returns
    ///
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
    ///
    /// * `Result<SnowID, Error>` - New SnowID generator or error if node_id is invalid
    pub fn with_config(node_id: u16, config: SnowIDConfig) -> Result<Self, SnowIDError> {
        if node_id >= (1 << config.node_bits()) {
            return Err(SnowIDError::InvalidNodeId {
                node_id,
                max: (1 << config.node_bits()) - 1,
            });
        }

        Ok(Self {
            node_id,
            extract: SnowIDExtractor::new(config),
            config,
            last_timestamp: 0,
            sequence: 0,
        })
    }

    /// Generate a new SnowID
    ///
    /// # Returns
    /// * `u64` - New SnowID value
    pub fn generate(&mut self) -> u64 {
        let timestamp = self.get_time_since_epoch();

        if timestamp > self.last_timestamp as u64 {
            self.last_timestamp = timestamp;
            self.sequence = 0;
        } else {
            // For same timestamp or backwards clock, increment sequence
            self.sequence = self.sequence.wrapping_add(1);
            if self.sequence > self.config.max_sequence() {
                // Sequence exhausted, wait for next millisecond
                // If clock moved backwards, wait from last timestamp
                let wait_from = if timestamp == self.last_timestamp {
                    timestamp
                } else {
                    self.last_timestamp
                };
                self.last_timestamp = self.wait_next_millis(wait_from);
                self.sequence = 0;
            }
        }

        self.create_snowid(self.last_timestamp as u64, self.sequence)
    }

    /// Get the number of bits used for node ID in the current configuration
    #[inline]
    pub fn node_bits(&self) -> u8 {
        self.config.node_bits()
    }

    /// Get the number of bits used for sequence in the current configuration
    #[inline]
    pub fn sequence_bits(&self) -> u8 {
        self.config.sequence_bits()
    }

    /// Get the maximum node ID supported by the current configuration
    #[inline]
    pub fn max_node_id(&self) -> u16 {
        (1 << self.config.node_bits()) - 1
    }

    /// Get the maximum sequence number supported by the current configuration
    #[inline]
    pub fn max_sequence(&self) -> u16 {
        self.config.max_sequence()
    }

    #[inline]
    fn get_time_since_epoch(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        now - self.config.custom_epoch()
    }

    #[inline]
    fn wait_next_millis(&self, timestamp: u64) -> u64 {
        let mut now = timestamp;
        while now <= timestamp {
            now = self.get_time_since_epoch();
        }
        now
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
}
