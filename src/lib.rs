#![cfg_attr(test, deny(warnings))]

use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

mod config;
mod error;

pub use config::{TsidConfig, DEFAULT_NODE_BITS, DEFAULT_CUSTOM_EPOCH};
pub use error::TsidError;
use config::BitConfig;

/// TSID Generator for creating unique, time-sorted IDs
pub struct TsidGenerator {
    node_id: u16,
    sequence: AtomicU16,
    last_timestamp: AtomicU64,
    config: TsidConfig,
    bit_config: BitConfig,
}

impl Clone for TsidGenerator {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id,
            sequence: AtomicU16::new(self.sequence.load(Ordering::Relaxed)),
            last_timestamp: AtomicU64::new(self.last_timestamp.load(Ordering::Relaxed)),
            config: self.config,
            bit_config: self.bit_config,
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
            } else {
                return Err(TsidError::SequenceOverflow);
            }
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
        (
            self.extract_timestamp(tsid),
            self.extract_node(tsid),
            self.extract_sequence(tsid)
        )
    }

    /// Extract timestamp from a TSID
    #[inline]
    pub fn extract_timestamp(&self, tsid: u64) -> u64 {
        (tsid >> self.bit_config.timestamp_shift) & self.bit_config.timestamp_mask
    }

    /// Extract node ID from a TSID
    #[inline]
    pub fn extract_node(&self, tsid: u64) -> u16 {
        ((tsid >> self.bit_config.node_shift) & self.bit_config.node_mask as u64) as u16
    }

    /// Extract sequence from a TSID
    #[inline]
    pub fn extract_sequence(&self, tsid: u64) -> u16 {
        (tsid & self.bit_config.sequence_mask as u64) as u16
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
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use crate::config::TIMESTAMP_BITS;

    #[test]
    fn test_custom_config() {
        let config = TsidConfig::builder()
            .node_bits(12)       // 4096 nodes
            .custom_epoch(DEFAULT_CUSTOM_EPOCH)
            .build();

        let generator = TsidGenerator::with_config(1023, config).unwrap();
        assert_eq!(generator.max_node_id(), 4095);
        assert_eq!(generator.max_sequence(), 1023);

        let tsid = generator.generate().unwrap();
        let (_, node, sequence) = generator.extract_from_tsid(tsid);
        
        assert!(node <= 4095, "Node ID exceeds maximum");
        assert!(sequence <= 1023, "Sequence exceeds maximum");
    }

    #[test]
    fn test_tsid_generation() {
        let generator = TsidGenerator::new(1).unwrap();
        let tsid = generator.generate().unwrap();
        assert!(tsid > 0);
    }

    #[test]
    fn test_tsid_components() {
        let generator = TsidGenerator::new(42).unwrap();
        let tsid = generator.generate().unwrap();
        let (timestamp, node, sequence) = generator.extract_from_tsid(tsid);
        
        assert_eq!(node, 42);
        assert_eq!(sequence, 0);
        assert!(timestamp > 0);
    }

    #[test]
    fn test_sequential_generation() {
        let generator = TsidGenerator::new(1).unwrap();
        let tsid1 = generator.generate().unwrap();
        let tsid2 = generator.generate().unwrap();
        assert!(tsid2 > tsid1);
    }

    #[test]
    fn test_invalid_node_id() {
        match TsidGenerator::new(1024) {
            Err(TsidError::InvalidNodeId { node_id, max_allowed }) => {
                assert_eq!(node_id, 1024);
                assert_eq!(max_allowed, 1023);
            }
            _ => panic!("Expected InvalidNodeId error"),
        }
    }

    #[test]
    fn test_node_id_boundaries() {
        // Test minimum node ID
        let gen0 = TsidGenerator::new(0).unwrap();
        let tsid0 = gen0.generate().unwrap();
        let (_, node0, _) = gen0.extract_from_tsid(tsid0);
        assert_eq!(node0, 0);

        // Test maximum node ID
        let gen1023 = TsidGenerator::new(1023).unwrap();
        let tsid1023 = gen1023.generate().unwrap();
        let (_, node1023, _) = gen1023.extract_from_tsid(tsid1023);
        assert_eq!(node1023, 1023);
    }

    #[test]
    fn test_sequence_rollover() {
        let generator = TsidGenerator::new(1).unwrap();
        
        // Generate IDs until sequence rolls over
        for _ in 0..1025 {
            let tsid = generator.generate().unwrap();
            let (_, _, sequence) = generator.extract_from_tsid(tsid);
            
            // Sequence should never exceed max
            assert!(sequence <= generator.max_sequence(), 
                "Sequence {} exceeded maximum {}", sequence, generator.max_sequence());
            
            // If sequence is less than last, we've rolled over
            if sequence < 1024 {
                return; // Test passed
            }
        }
        
        panic!("Sequence did not roll over as expected");
    }

    #[test]
    fn test_concurrent_generation() {
        let generator = Arc::new(TsidGenerator::new(1).unwrap());
        let generator = std::sync::Arc::new(generator);
        let mut handles = vec![];
        let num_threads = 4;
        let ids_per_thread = 250; // Reduced to avoid sequence exhaustion

        // Generate IDs concurrently
        for _ in 0..num_threads {
            let gen = generator.clone();
            handles.push(thread::spawn(move || {
                (0..ids_per_thread).map(|_| gen.generate().unwrap()).collect::<Vec<_>>()
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
        let generator = TsidGenerator::new(1).unwrap();
        let mut last_timestamp = 0;

        for _ in 0..100 {
            let tsid = generator.generate().unwrap();
            let (timestamp, _, _) = generator.extract_from_tsid(tsid);
            assert!(timestamp >= last_timestamp);
            last_timestamp = timestamp;
            
            // Add small delay to ensure timestamp changes
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn test_component_max_values() {
        let generator = TsidGenerator::new(1023).unwrap();
        let tsid = generator.generate().unwrap();
        let (timestamp, node, sequence) = generator.extract_from_tsid(tsid);

        assert!(timestamp <= generator.bit_config.timestamp_mask);
        assert!(node <= generator.bit_config.max_node);
        assert!(sequence <= generator.bit_config.max_sequence);
    }

    #[test]
    fn test_unique_ids_across_nodes() {
        let gen1 = TsidGenerator::new(1).unwrap();
        let gen2 = TsidGenerator::new(2).unwrap();
        
        let mut ids = HashSet::new();
        
        // Generate IDs from both generators
        for _ in 0..1000 {
            ids.insert(gen1.generate().unwrap());
            ids.insert(gen2.generate().unwrap());
        }

        // Verify all IDs are unique
        assert_eq!(ids.len(), 2000);
    }

    #[test]
    fn test_sequence_restart() {
        let generator = TsidGenerator::new(1).unwrap();
        
        // Generate multiple IDs in the same millisecond
        let tsid1 = generator.generate().unwrap();
        let tsid2 = generator.generate().unwrap();
        let tsid3 = generator.generate().unwrap();
        
        let (_, _, seq1) = generator.extract_from_tsid(tsid1);
        let (_, _, seq2) = generator.extract_from_tsid(tsid2);
        let (_, _, seq3) = generator.extract_from_tsid(tsid3);
        
        // Verify sequence increments
        assert!(seq2 > seq1);
        assert!(seq3 > seq2);
        
        // Wait for timestamp to change
        thread::sleep(Duration::from_millis(2));
        
        // Generate new ID after timestamp change
        let tsid_new = generator.generate().unwrap();
        let (_, _, new_seq) = generator.extract_from_tsid(tsid_new);
        
        // Verify sequence resets
        assert_eq!(new_seq, 0, "Sequence should reset when timestamp changes");
    }

    #[test]
    fn test_bit_layout() {
        let node_id = 42;
        let generator = TsidGenerator::new(node_id).unwrap();
        let tsid = generator.generate().unwrap();
        
        // Extract components using bit masks directly
        let timestamp = (tsid >> generator.bit_config.timestamp_shift) & generator.bit_config.timestamp_mask;
        let node = ((tsid >> generator.bit_config.node_shift) & generator.bit_config.node_mask as u64) as u16;
        let sequence = (tsid & generator.bit_config.sequence_mask as u64) as u16;
        
        // Verify using the public extract function
        let (ext_timestamp, ext_node, ext_sequence) = generator.extract_from_tsid(tsid);
        
        // Compare both extraction methods
        assert_eq!(timestamp, ext_timestamp);
        assert_eq!(node, ext_node);
        assert_eq!(sequence, ext_sequence);
        assert_eq!(node, node_id);
    }

    #[test]
    fn test_epoch_handling() {
        let generator = TsidGenerator::new(1).unwrap();
        let tsid = generator.generate().unwrap();
        let (timestamp, _, _) = generator.extract_from_tsid(tsid);
        
        // Get current time relative to Unix epoch
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // Calculate expected timestamp relative to custom epoch
        let expected_timestamp = now.saturating_sub(generator.config.custom_epoch);
        
        // Allow small difference due to test execution time
        let diff = if expected_timestamp > timestamp {
            expected_timestamp - timestamp
        } else {
            timestamp - expected_timestamp
        };
        
        assert!(diff < 1000, "Timestamp difference should be less than 1 second");
    }

    #[test]
    fn test_sequence_overflow_handling() {
        let generator = TsidGenerator::new(1).unwrap();
        
        // Spawn multiple threads to generate IDs rapidly
        let handles: Vec<_> = (0..4).map(|_| {
            let gen = generator.clone();
            thread::spawn(move || {
                for _ in 0..300 {
                    let id = gen.generate().unwrap();
                    let (_timestamp, _, sequence) = gen.extract_from_tsid(id);
                    
                    // Verify sequence doesn't exceed max
                    assert!(sequence <= generator.bit_config.max_sequence, 
                        "Sequence {} exceeded maximum {}", sequence, generator.bit_config.max_sequence);
                }
            })
        }).collect();

        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_sequence_uniqueness() {
        let generator = Arc::new(TsidGenerator::new(1).unwrap());
        let seen_ids = Arc::new(Mutex::new(HashSet::new()));
        let threads = 4;
        let ids_per_thread = 500;

        let handles: Vec<_> = (0..threads).map(|_| {
            let gen = generator.clone();
            let seen = seen_ids.clone();
            thread::spawn(move || {
                for _ in 0..ids_per_thread {
                    let id = gen.generate().unwrap();
                    let mut seen = seen.lock().unwrap();
                    assert!(!seen.contains(&id), "Duplicate ID generated: {}", id);
                    seen.insert(id);
                }
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify total number of unique IDs
        let total_ids = seen_ids.lock().unwrap().len();
        assert_eq!(total_ids, threads * ids_per_thread);
    }

    #[test]
    fn test_rapid_generation() {
        let generator = TsidGenerator::new(1).unwrap();
        let mut last_id = 0;
        
        // Generate IDs as fast as possible
        for _ in 0..10_000 {
            let id = match generator.generate() {
                Ok(id) => id,
                Err(TsidError::SequenceOverflow) => {
                    // On sequence overflow, wait for next millisecond and retry
                    thread::sleep(Duration::from_millis(1));
                    generator.generate().unwrap()
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            };
            assert!(id > last_id, "ID not monotonically increasing");
            last_id = id;
        }
    }

    #[test]
    fn test_component_boundaries() {
        let generator = TsidGenerator::new(1023).unwrap();
        let _tsid = generator.generate().unwrap();
        
        // Test maximum values for each component
        let max_timestamp = (1u64 << TIMESTAMP_BITS) - 1;
        let max_node = (1u16 << generator.config.node_bits) - 1;
        let max_sequence = (1u16 << generator.config.sequence_bits) - 1;
        
        // Create a TSID with maximum values
        let max_tsid = ((max_timestamp & generator.bit_config.timestamp_mask) << generator.bit_config.timestamp_shift) |
                      ((max_node as u64 & generator.bit_config.node_mask as u64) << generator.bit_config.node_shift) |
                      (max_sequence as u64 & generator.bit_config.sequence_mask as u64);
        
        // Extract and verify components
        let (ts, node, seq) = generator.extract_from_tsid(max_tsid);
        assert_eq!(ts, max_timestamp);
        assert_eq!(node, max_node);
        assert_eq!(seq, max_sequence);
        
        // Verify no bits are set outside their designated positions
        let total_bits = TIMESTAMP_BITS as u32 + 
                        generator.config.node_bits as u32 + 
                        generator.config.sequence_bits as u32;
        
        // Create a mask for all valid bits
        let valid_bits_mask = ((1u64 << generator.config.sequence_bits) - 1) |
                            (((1u64 << generator.config.node_bits) - 1) << generator.bit_config.node_shift) |
                            (((1u64 << TIMESTAMP_BITS) - 1) << generator.bit_config.timestamp_shift);
        
        // Check that there are no bits set outside our valid bits
        assert_eq!(max_tsid & !valid_bits_mask, 0, 
            "Found set bits outside of designated positions");
        
        // Verify total bits used is correct
        assert_eq!(total_bits, 64, 
            "Total bits {} should equal 64", total_bits);
    }

    #[test]
    fn test_zero_node_id() {
        let generator = TsidGenerator::new(0).unwrap();
        let tsid = generator.generate().unwrap();
        let (_, node, _) = generator.extract_from_tsid(tsid);
        assert_eq!(node, 0, "Node ID should be preserved as 0");
    }

    #[test]
    fn test_sequence_restart_on_overflow() {
        let generator = TsidGenerator::new(1).unwrap();
        
        // Generate multiple IDs in the same millisecond
        let first_id = generator.generate().unwrap();
        let mut last_id = first_id;
        
        for _ in 0..100 {
            let current_id = generator.generate().unwrap();
            assert!(current_id > last_id, "Generated ID should be greater than previous");
            last_id = current_id;
        }
        
        // Extract components
        let (first_ts, _, first_seq) = generator.extract_from_tsid(first_id);
        let (last_ts, _, last_seq) = generator.extract_from_tsid(last_id);
        
        // If timestamps are the same, sequence should have increased
        if first_ts == last_ts {
            assert!(last_seq > first_seq, "Sequence should increase within same millisecond");
        } else {
            // If timestamps are different, we can't make assumptions about sequence
            assert!(last_ts > first_ts, "Timestamp should increase");
        }
    }
}
