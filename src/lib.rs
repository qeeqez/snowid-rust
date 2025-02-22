#![cfg_attr(test, deny(warnings))]

use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

mod config;
mod error;
mod extractor;

pub use config::{TsidConfig, DEFAULT_NODE_BITS, DEFAULT_CUSTOM_EPOCH};
pub use error::TsidError;
use config::BitConfig;
use extractor::TsidExtractor;

/// TSID Generator for creating unique, time-sorted IDs
pub struct TsidGenerator {
    node_id: u16,
    sequence: AtomicU16,
    last_timestamp: AtomicU64,
    config: TsidConfig,
    bit_config: BitConfig,
    extractor: TsidExtractor,
}

impl Clone for TsidGenerator {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id,
            sequence: AtomicU16::new(self.sequence.load(Ordering::Relaxed)),
            last_timestamp: AtomicU64::new(self.last_timestamp.load(Ordering::Relaxed)),
            config: self.config,
            bit_config: self.bit_config,
            extractor: TsidExtractor::new(self.bit_config),
        }
    }
}

impl TsidGenerator {
    /// Create a new TSID generator with the given node ID and default configuration
    ///
    /// # Arguments
    /// * `node_id` - Node identifier (0-1023 by default)
    ///
    /// # Returns
    /// * `Result<Self, TsidError>` - A new TSID generator or an error if node_id is invalid
    pub fn new(node_id: u16) -> Result<Self, TsidError> {
        Self::with_config(node_id, TsidConfig::default())
    }

    /// Create a new TSID generator with custom configuration
    ///
    /// # Arguments
    /// * `node_id` - Node identifier (range depends on configuration)
    /// * `config` - Custom configuration for TSID generation
    ///
    /// # Returns
    /// * `Result<Self, TsidError>` - A new TSID generator or an error if node_id is invalid
    pub fn with_config(node_id: u16, config: TsidConfig) -> Result<Self, TsidError> {
        let bit_config = config.create_bit_config();
        if node_id > bit_config.max_node {
            return Err(TsidError::InvalidNodeId {
                node_id,
                max_allowed: bit_config.max_node,
            });
        }

        Ok(Self {
            node_id,
            sequence: AtomicU16::new(0),
            last_timestamp: AtomicU64::new(0),
            config,
            bit_config,
            extractor: TsidExtractor::new(bit_config),
        })
    }

    /// Generate a new TSID
    ///
    /// # Returns
    /// * `Result<u64, TsidError>` - A new TSID or an error if generation fails
    pub fn generate(&self) -> Result<u64, TsidError> {
        loop {
            let timestamp = self.current_time()?;
            let last = self.last_timestamp.load(Ordering::Acquire);
            
            // If timestamp moved forward, try to update it
            if timestamp > last {
                if let Ok(_) = self.last_timestamp.compare_exchange(
                    last,
                    timestamp,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    self.sequence.store(0, Ordering::Release);
                    return Ok(self.create_tsid(timestamp, 0));
                }
                continue;
            }
            
            // Get next sequence for current timestamp (use last if clock moved backwards)
            let current_ts = if timestamp < last { 
                return Err(TsidError::ClockBackwards);
            } else { 
                timestamp 
            };
            
            let seq = self.sequence.fetch_add(1, Ordering::AcqRel);
            
            if seq < self.bit_config.max_sequence {
                return Ok(self.create_tsid(current_ts, seq + 1));
            }
            
            // Reset sequence and try next millisecond
            self.sequence.store(0, Ordering::Release);
            continue;
        }
    }

    #[inline]
    /// Get the current timestamp in milliseconds since the configured epoch
    fn current_time(&self) -> Result<u64, TsidError> {
        Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TsidError::ClockBackwards)?
            .as_millis() as u64
            - self.config.custom_epoch)
    }

    #[inline]
    /// Create a TSID from components using the configured bit layout
    fn create_tsid(&self, timestamp: u64, sequence: u16) -> u64 {
        ((timestamp & self.bit_config.timestamp_mask) << self.bit_config.timestamp_shift)
            | ((self.node_id as u64 & self.bit_config.node_mask as u64) << self.bit_config.node_shift)
            | (sequence as u64 & self.bit_config.sequence_mask as u64)
    }

    /// Extract timestamp, node ID, and sequence from a TSID
    pub fn extract_from_tsid(&self, tsid: u64) -> (u64, u16, u16) {
        self.extractor.extract_from_tsid(tsid)
    }

    /// Extract timestamp from a TSID
    #[inline]
    pub fn extract_timestamp(&self, tsid: u64) -> u64 {
        self.extractor.extract_timestamp(tsid)
    }

    /// Extract node ID from a TSID
    #[inline]
    pub fn extract_node(&self, tsid: u64) -> u16 {
        self.extractor.extract_node(tsid)
    }

    /// Extract sequence from a TSID
    #[inline]
    pub fn extract_sequence(&self, tsid: u64) -> u16 {
        self.extractor.extract_sequence(tsid)
    }

    /// Get the maximum node ID supported by the current configuration
    pub fn max_node_id(&self) -> u16 {
        self.bit_config.max_node
    }

    /// Get the maximum sequence number supported by the current configuration
    pub fn max_sequence(&self) -> u16 {
        self.bit_config.max_sequence
    }

    /// Get the current configuration
    pub fn config(&self) -> TsidConfig {
        self.config
    }
}

#[cfg(test)]
mod tests;