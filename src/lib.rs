#![cfg_attr(test, deny(warnings))]

use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const TSID_TIMESTAMP_BITS: u8 = 42;
const TSID_NODE_BITS: u8 = 12;
const TSID_SEQUENCE_BITS: u8 = 10;

const TSID_NODE_SHIFT: u8 = TSID_SEQUENCE_BITS;
const TSID_TIMESTAMP_SHIFT: u8 = TSID_NODE_BITS + TSID_SEQUENCE_BITS;

const TSID_SEQUENCE_MASK: u16 = (1 << TSID_SEQUENCE_BITS) - 1;
const TSID_NODE_MASK: u16 = (1 << TSID_NODE_BITS) - 1;
const TSID_TIMESTAMP_MASK: u64 = (1 << TSID_TIMESTAMP_BITS) - 1;

const TSID_MAX_SEQUENCE: u16 = TSID_SEQUENCE_MASK;
const TSID_MAX_NODE: u16 = TSID_NODE_MASK;

// Custom epoch (January 1, 2024 UTC)
const TSID_EPOCH: u64 = 1704067200000;

/// TSID Generator for creating unique, time-sorted IDs
pub struct TsidGenerator {
    node_id: u16,
    sequence: AtomicU16,
    last_timestamp: AtomicU64,
}

impl TsidGenerator {
    /// Create a new TSID generator with the given node ID
    ///
    /// # Arguments
    /// * `node_id` - Node identifier (0-4095)
    ///
    /// # Panics
    /// Panics if node_id is greater than 4095
    pub fn new(node_id: u16) -> Self {
        assert!(node_id <= TSID_MAX_NODE, "Node ID must be between 0 and 4095");
        
        Self {
            node_id,
            sequence: AtomicU16::new(0),
            last_timestamp: AtomicU64::new(0),
        }
    }

    /// Generate a new TSID
    pub fn generate(&self) -> u64 {
        loop {
            let timestamp = self.current_time();
            let last = self.last_timestamp.load(Ordering::Acquire);
            
            let sequence = if timestamp == last {
                let seq = self.sequence.fetch_add(1, Ordering::AcqRel);
                if seq < TSID_MAX_SEQUENCE {
                    seq + 1
                } else {
                    // Sequence exhausted, retry with new timestamp
                    self.sequence.store(0, Ordering::Release);
                    continue;
                }
            } else {
                self.sequence.store(0, Ordering::Release);
                self.last_timestamp.store(timestamp, Ordering::Release);
                0
            };

            return self.create_tsid(timestamp, sequence);
        }
    }

    /// Get the current timestamp in milliseconds since the custom epoch
    fn current_time(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        
        now.saturating_sub(TSID_EPOCH)
    }

    /// Create a TSID from components
    fn create_tsid(&self, timestamp: u64, sequence: u16) -> u64 {
        ((timestamp & TSID_TIMESTAMP_MASK) << TSID_TIMESTAMP_SHIFT)
            | ((self.node_id as u64 & TSID_NODE_MASK as u64) << TSID_NODE_SHIFT)
            | (sequence as u64 & TSID_SEQUENCE_MASK as u64)
    }
}

/// Extract components from a TSID
pub fn extract_from_tsid(tsid: u64) -> (u64, u16, u16) {
    let timestamp = (tsid >> TSID_TIMESTAMP_SHIFT) & TSID_TIMESTAMP_MASK;
    let node = ((tsid >> TSID_NODE_SHIFT) & TSID_NODE_MASK as u64) as u16;
    let sequence = (tsid & TSID_SEQUENCE_MASK as u64) as u16;
    
    (timestamp, node, sequence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_tsid_generation() {
        let generator = TsidGenerator::new(1);
        let tsid = generator.generate();
        assert!(tsid > 0);
    }

    #[test]
    fn test_tsid_components() {
        let generator = TsidGenerator::new(42);
        let tsid = generator.generate();
        let (timestamp, node, sequence) = extract_from_tsid(tsid);
        
        assert_eq!(node, 42);
        assert_eq!(sequence, 0);
        assert!(timestamp > 0);
    }

    #[test]
    fn test_sequential_generation() {
        let generator = TsidGenerator::new(1);
        let tsid1 = generator.generate();
        let tsid2 = generator.generate();
        assert!(tsid2 > tsid1);
    }

    #[test]
    #[should_panic(expected = "Node ID must be between 0 and 4095")]
    fn test_invalid_node_id() {
        TsidGenerator::new(4096);
    }
}
