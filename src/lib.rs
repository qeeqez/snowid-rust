use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

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
    pub config: SnowIDConfig,
    pub extract: SnowIDExtractor,
    last_timestamp: AtomicU64,
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
        if node_id > config.max_node_id() {
            return Err(SnowIDError::InvalidNodeId {
                node_id,
                max: (1 << config.node_bits()) - 1,
            });
        }

        Ok(Self {
            node_id,
            extract: SnowIDExtractor::new(config),
            config,
            last_timestamp: AtomicU64::new(0),
            sequence: AtomicU16::new(0),
        })
    }

    /// Generate a new SnowID
    ///
    /// # Returns
    /// * `u64` - New SnowID value
    pub fn generate(&self) -> u64 {
        let mut timestamp = self.get_time_since_epoch();
        let mut last_ts = self.last_timestamp.load(Ordering::Acquire);
        let mut backoff = 1;

        loop {
            if timestamp > last_ts {
                // Try to update last_timestamp atomically
                match self.last_timestamp.compare_exchange(
                    last_ts,
                    timestamp,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        self.sequence.store(0, Ordering::Release);
                        break;
                    }
                    Err(actual) => {
                        last_ts = actual;
                        continue;
                    }
                }
            } else {
                // For same timestamp or backwards clock
                let current_sequence = self.sequence.fetch_add(1, Ordering::AcqRel);
                
                if current_sequence < self.config.max_sequence_id() {
                    // We got a valid sequence number
                    break;
                }
                
                // Sequence exhausted, wait for next millisecond with exponential backoff
                let wait_from = timestamp.max(last_ts);
                timestamp = self.wait_next_millis(wait_from, backoff);
                backoff = (backoff * 2).min(Self::MAX_BACKOFF_MS);
                
                // Update last_ts for next iteration
                last_ts = self.last_timestamp.load(Ordering::Acquire);
            }
        }

        self.create_snowid(timestamp, self.sequence.load(Ordering::Acquire))
    }

    /// Get current time in milliseconds since epoch
    fn get_time_since_epoch(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
            
        let current_time = now.as_millis() as u64;
        let epoch_time = self.config.epoch();
        
        if current_time <= epoch_time {
            panic!("Current time {} is before epoch {}", current_time, epoch_time);
        }
        
        current_time - epoch_time
    }

    /// Wait until next millisecond with exponential backoff
    fn wait_next_millis(&self, timestamp: u64, backoff_ms: u64) -> u64 {
        thread::sleep(Duration::from_millis(backoff_ms));
        let mut new_timestamp = self.get_time_since_epoch();
        
        while new_timestamp <= timestamp {
            thread::yield_now();
            new_timestamp = self.get_time_since_epoch();
        }
        
        new_timestamp
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
