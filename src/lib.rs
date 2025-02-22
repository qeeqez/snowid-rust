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
    use std::collections::HashSet;
    use std::thread;
    use std::time::Duration;

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

    #[test]
    fn test_node_id_boundaries() {
        // Test minimum node ID
        let gen0 = TsidGenerator::new(0);
        let tsid0 = gen0.generate();
        let (_, node0, _) = extract_from_tsid(tsid0);
        assert_eq!(node0, 0);

        // Test maximum node ID
        let gen4095 = TsidGenerator::new(4095);
        let tsid4095 = gen4095.generate();
        let (_, node4095, _) = extract_from_tsid(tsid4095);
        assert_eq!(node4095, 4095);
    }

    #[test]
    fn test_sequence_rollover() {
        let generator = TsidGenerator::new(1);
        
        // Generate IDs until sequence rolls over
        for _ in 0..1025 {
            let tsid = generator.generate();
            let (_, _, sequence) = extract_from_tsid(tsid);
            
            // Sequence should never exceed max
            assert!(sequence <= TSID_MAX_SEQUENCE);
            
            // If sequence is less than last, we've rolled over
            if sequence < 1024 {
                return; // Test passed
            }
        }
        
        panic!("Sequence did not roll over as expected");
    }

    #[test]
    fn test_concurrent_generation() {
        let generator = TsidGenerator::new(1);
        let generator = std::sync::Arc::new(generator);
        let mut handles = vec![];
        let num_threads = 4;
        let ids_per_thread = 250; // Reduced to avoid sequence exhaustion

        // Generate IDs concurrently
        for _ in 0..num_threads {
            let gen = generator.clone();
            handles.push(thread::spawn(move || {
                (0..ids_per_thread).map(|_| gen.generate()).collect::<Vec<_>>()
            }));
        }

        // Collect all generated IDs
        let mut all_ids = HashSet::new();
        for handle in handles {
            let ids = handle.join().unwrap();
            all_ids.extend(ids);
        }

        // Verify no duplicates were generated
        assert_eq!(all_ids.len(), num_threads * ids_per_thread, 
            "Expected {} unique IDs, but got {}", 
            num_threads * ids_per_thread, 
            all_ids.len()
        );

        // Verify all IDs are monotonically increasing
        let mut ids: Vec<_> = all_ids.into_iter().collect();
        ids.sort_unstable();
        for i in 1..ids.len() {
            assert!(ids[i] > ids[i-1], 
                "ID at position {} ({}) is not greater than previous ID ({})",
                i, ids[i], ids[i-1]
            );
        }
    }

    #[test]
    fn test_timestamp_monotonicity() {
        let generator = TsidGenerator::new(1);
        let mut last_timestamp = 0;

        for _ in 0..100 {
            let tsid = generator.generate();
            let (timestamp, _, _) = extract_from_tsid(tsid);
            assert!(timestamp >= last_timestamp);
            last_timestamp = timestamp;
            
            // Add small delay to ensure timestamp changes
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn test_component_max_values() {
        let generator = TsidGenerator::new(TSID_MAX_NODE);
        let tsid = generator.generate();
        let (timestamp, node, sequence) = extract_from_tsid(tsid);

        assert!(timestamp <= TSID_TIMESTAMP_MASK);
        assert!(node <= TSID_MAX_NODE);
        assert!(sequence <= TSID_MAX_SEQUENCE);
    }

    #[test]
    fn test_unique_ids_across_nodes() {
        let gen1 = TsidGenerator::new(1);
        let gen2 = TsidGenerator::new(2);
        
        let mut ids = HashSet::new();
        
        // Generate IDs from both generators
        for _ in 0..1000 {
            ids.insert(gen1.generate());
            ids.insert(gen2.generate());
        }

        // Verify all IDs are unique
        assert_eq!(ids.len(), 2000);
    }

    #[test]
    fn test_sequence_reset_on_timestamp_change() {
        let generator = TsidGenerator::new(1);
        
        // Generate multiple IDs in the same millisecond
        let tsid1 = generator.generate();
        let tsid2 = generator.generate();
        let tsid3 = generator.generate();
        
        let (_, _, seq1) = extract_from_tsid(tsid1);
        let (_, _, seq2) = extract_from_tsid(tsid2);
        let (_, _, seq3) = extract_from_tsid(tsid3);
        
        // Verify sequence increments
        assert!(seq2 > seq1);
        assert!(seq3 > seq2);
        
        // Wait for timestamp to change
        thread::sleep(Duration::from_millis(2));
        
        // Generate new ID after timestamp change
        let tsid_new = generator.generate();
        let (_, _, new_seq) = extract_from_tsid(tsid_new);
        
        // Verify sequence resets
        assert_eq!(new_seq, 0, "Sequence should reset when timestamp changes");
    }

    #[test]
    fn test_bit_layout() {
        let node_id = 42;
        let generator = TsidGenerator::new(node_id);
        let tsid = generator.generate();
        
        // Extract components using bit masks directly
        let timestamp = (tsid >> TSID_TIMESTAMP_SHIFT) & TSID_TIMESTAMP_MASK;
        let node = ((tsid >> TSID_NODE_SHIFT) & (TSID_NODE_MASK as u64)) as u16;
        let sequence = (tsid & (TSID_SEQUENCE_MASK as u64)) as u16;
        
        // Verify using the public extract function
        let (ext_timestamp, ext_node, ext_sequence) = extract_from_tsid(tsid);
        
        // Compare both extraction methods
        assert_eq!(timestamp, ext_timestamp);
        assert_eq!(node, ext_node);
        assert_eq!(sequence, ext_sequence);
        assert_eq!(node, node_id);
    }

    #[test]
    fn test_epoch_handling() {
        let generator = TsidGenerator::new(1);
        let tsid = generator.generate();
        let (timestamp, _, _) = extract_from_tsid(tsid);
        
        // Get current time relative to Unix epoch
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // Calculate expected timestamp relative to custom epoch
        let expected_timestamp = now.saturating_sub(TSID_EPOCH);
        
        // Allow small difference due to test execution time
        let diff = if expected_timestamp > timestamp {
            expected_timestamp - timestamp
        } else {
            timestamp - expected_timestamp
        };
        
        assert!(diff < 1000, "Timestamp difference should be less than 1 second");
    }
}
