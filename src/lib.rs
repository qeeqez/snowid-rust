#![cfg_attr(test, deny(warnings))]

use std::time::{SystemTime, UNIX_EPOCH};

mod config;
mod error;
mod extractor;

pub use config::TsidConfig;
pub use error::TsidError;
pub use extractor::TsidExtractor;

/// Time-Sorted ID Generator
pub struct Tsid {
    node_id: u16,
    config: TsidConfig,
    pub extract: TsidExtractor,
    last_timestamp: u64,
    last_sequence: u16,
}

impl Tsid {
    /// Create a new TSID generator with default configuration
    ///
    /// # Arguments
    /// * `node_id` - Node ID for this generator
    ///
    /// # Returns
    /// * `Result<Tsid, Error>` - New TSID generator or error if node_id is invalid
    pub fn new(node_id: u16) -> Result<Self, TsidError> {
        Self::with_config(node_id, TsidConfig::default())
    }

    /// Create a new TSID generator with custom configuration
    ///
    /// # Arguments
    /// * `node_id` - Node ID for this generator
    /// * `config` - Custom configuration for the generator
    ///
    /// # Returns
    /// * `Result<Tsid, Error>` - New TSID generator or error if node_id is invalid
    pub fn with_config(node_id: u16, config: TsidConfig) -> Result<Self, TsidError> {
        if node_id > config.max_node_id() {
            return Err(TsidError::InvalidNodeId {
                node_id,
                max_allowed: config.max_node_id(),
            });
        }

        Ok(Self {
            node_id,
            extract: TsidExtractor::new(config.clone()),
            config,
            last_timestamp: 0,
            last_sequence: config.max_sequence(),
        })
    }

    /// Generate a new TSID
    ///
    /// # Returns
    /// * `u64` - New TSID value
    pub fn generate(&mut self) -> u64 {
        let timestamp = self.get_time_since_epoch();

        if timestamp > self.last_timestamp {
            self.last_timestamp = timestamp;
            self.last_sequence = 0;
        } else {
            // For same timestamp or backwards clock, increment sequence
            self.last_sequence = self.last_sequence.wrapping_add(1);
            if self.last_sequence > self.config.max_sequence() {
                // Sequence exhausted, wait for next millisecond
                // If clock moved backwards, wait from last timestamp
                let wait_from = if timestamp == self.last_timestamp {
                    timestamp
                } else {
                    self.last_timestamp
                };
                self.last_timestamp = self.wait_next_millis(wait_from);
                self.last_sequence = 0;
            }
        }

        self.create_tsid(self.last_timestamp, self.last_sequence)
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
        self.config.max_node_id()
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
    fn create_tsid(&self, timestamp: u64, sequence: u16) -> u64 {
        self.create_tsid_with_node(timestamp, self.node_id, sequence)
    }

    #[inline]
    fn create_tsid_with_node(&self, timestamp: u64, node_id: u16, sequence: u16) -> u64 {
        ((timestamp & self.config.timestamp_mask()) << self.config.timestamp_shift())
            | ((node_id as u64 & self.config.node_mask() as u64) << self.config.node_shift())
            | (sequence as u64 & self.config.sequence_mask() as u64)
    }
}

#[cfg(test)]
mod tests;
