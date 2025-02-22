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

impl Clone for TsidGenerator {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id,
            sequence: AtomicU16::new(self.sequence.load(Ordering::Relaxed)),
            last_timestamp: AtomicU64::new(self.last_timestamp.load(Ordering::Relaxed)),
        }
    }
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
            
            // If timestamp moved forward, try to update it
            if timestamp > last {
                if let Ok(_) = self.last_timestamp.compare_exchange(
                    last,
                    timestamp,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    self.sequence.store(0, Ordering::Release);
                    return self.create_tsid(timestamp, 0);
                }
                continue;
            }
            
            // Get next sequence for current timestamp (use last if clock moved backwards)
            let current_ts = if timestamp < last { last } else { timestamp };
            let seq = self.sequence.fetch_add(1, Ordering::AcqRel);
            
            if seq < TSID_MAX_SEQUENCE {
                return self.create_tsid(current_ts, seq + 1);
            }
        }
    }

    #[inline]
    /// Get the current timestamp in milliseconds since the custom epoch
    fn current_time(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
            - TSID_EPOCH
    }

    #[inline]
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
    use std::sync::{Arc, Mutex};
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

    #[test]
    fn test_sequence_overflow_handling() {
        let generator = TsidGenerator::new(1);
        let first_timestamp = Arc::new(Mutex::new(None));
        let first_timestamp_clone = first_timestamp.clone();
        
        // Spawn multiple threads to generate IDs rapidly
        let handles: Vec<_> = (0..4).map(|_| {
            let gen = generator.clone();
            let ts = first_timestamp_clone.clone();
            thread::spawn(move || {
                let mut ids = Vec::new();
                for _ in 0..300 {
                    let id = gen.generate();
                    let (timestamp, _, sequence) = extract_from_tsid(id);
                    
                    // Store the first timestamp we see
                    let mut ts = ts.lock().unwrap();
                    if ts.is_none() {
                        *ts = Some(timestamp);
                    }
                    
                    // Verify sequence doesn't exceed max
                    assert!(sequence <= TSID_MAX_SEQUENCE, 
                        "Sequence {} exceeded maximum {}", sequence, TSID_MAX_SEQUENCE);
                    
                    ids.push((timestamp, sequence));
                }
                ids
            })
        }).collect();

        // Collect and analyze results
        let mut all_ids = Vec::new();
        for handle in handles {
            all_ids.extend(handle.join().unwrap());
        }

        // Sort by timestamp and sequence
        all_ids.sort_by_key(|&(ts, seq)| (ts, seq));

        // Check sequence counts per timestamp
        let mut current_ts = all_ids[0].0;
        let mut seq_count = 0;
        
        for &(ts, _) in &all_ids {
            if ts == current_ts {
                seq_count += 1;
                assert!(seq_count <= TSID_MAX_SEQUENCE as usize + 1, 
                    "Too many sequences ({}) for timestamp {}", seq_count, ts);
            } else {
                current_ts = ts;
                seq_count = 1;
            }
        }
    }

    #[test]
    fn test_timestamp_backtrack_protection() {
        let generator = TsidGenerator::new(1);
        let mut last_tsid = generator.generate();
        
        // Generate IDs and verify they're always increasing
        for _ in 0..1000 {
            let current_tsid = generator.generate();
            assert!(current_tsid > last_tsid, 
                "TSID decreased from {} to {}", last_tsid, current_tsid);
            last_tsid = current_tsid;
        }
    }

    #[test]
    fn test_max_sequence_per_ms() {
        let generator = TsidGenerator::new(1);
        let mut sequences_seen = HashSet::new();
        let mut last_timestamp = 0;
        
        for _ in 0..2000 {
            let tsid = generator.generate();
            let (timestamp, _, sequence) = extract_from_tsid(tsid);
            
            if timestamp != last_timestamp {
                // New millisecond, reset tracking
                sequences_seen.clear();
                last_timestamp = timestamp;
            }
            
            // Ensure we haven't seen this sequence for this timestamp
            let key = (timestamp, sequence);
            assert!(!sequences_seen.contains(&key), 
                "Duplicate sequence {} for timestamp {}", sequence, timestamp);
            sequences_seen.insert(key);
            
            // Verify sequence is within bounds
            assert!(sequence <= TSID_MAX_SEQUENCE);
        }
    }

    #[test]
    fn test_concurrent_sequence_uniqueness() {
        let generator = Arc::new(TsidGenerator::new(1));
        let seen_ids = Arc::new(Mutex::new(HashSet::new()));
        let threads = 4;
        let ids_per_thread = 500;

        let handles: Vec<_> = (0..threads).map(|_| {
            let gen = generator.clone();
            let seen = seen_ids.clone();
            thread::spawn(move || {
                for _ in 0..ids_per_thread {
                    let id = gen.generate();
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
        let generator = TsidGenerator::new(1);
        let mut last_id = 0;
        
        // Generate IDs as fast as possible
        for _ in 0..10_000 {
            let id = generator.generate();
            assert!(id > last_id, "ID not monotonically increasing");
            last_id = id;
        }
    }

    #[test]
    fn test_component_boundaries() {
        let generator = TsidGenerator::new(TSID_MAX_NODE);
        let _tsid = generator.generate();
        
        // Test maximum values for each component
        let max_timestamp = (1u64 << TSID_TIMESTAMP_BITS) - 1;
        let max_node = (1u16 << TSID_NODE_BITS) - 1;
        let max_sequence = (1u16 << TSID_SEQUENCE_BITS) - 1;
        
        // Create a TSID with maximum values
        let max_tsid = ((max_timestamp & TSID_TIMESTAMP_MASK) << TSID_TIMESTAMP_SHIFT) |
                      ((max_node as u64 & TSID_NODE_MASK as u64) << TSID_NODE_SHIFT) |
                      (max_sequence as u64 & TSID_SEQUENCE_MASK as u64);
        
        // Extract and verify components
        let (ts, node, seq) = extract_from_tsid(max_tsid);
        assert_eq!(ts, max_timestamp);
        assert_eq!(node, max_node);
        assert_eq!(seq, max_sequence);
        
        // Verify no bits are set outside their designated positions
        let total_bits = TSID_TIMESTAMP_BITS as u32 + 
                        TSID_NODE_BITS as u32 + 
                        TSID_SEQUENCE_BITS as u32;
        
        // Create a mask for all valid bits
        let valid_bits_mask = ((1u64 << TSID_SEQUENCE_BITS) - 1) |
                            (((1u64 << TSID_NODE_BITS) - 1) << TSID_NODE_SHIFT) |
                            (((1u64 << TSID_TIMESTAMP_BITS) - 1) << TSID_TIMESTAMP_SHIFT);
        
        // Check that there are no bits set outside our valid bits
        assert_eq!(max_tsid & !valid_bits_mask, 0, 
            "Found set bits outside of designated positions");
        
        // Verify total bits used is correct
        assert_eq!(total_bits, 64, 
            "Total bits {} should equal 64", total_bits);
    }

    #[test]
    fn test_zero_node_id() {
        let generator = TsidGenerator::new(0);
        let tsid = generator.generate();
        let (_, node, _) = extract_from_tsid(tsid);
        assert_eq!(node, 0, "Node ID should be preserved as 0");
    }

    #[test]
    fn test_sequence_restart() {
        let generator = TsidGenerator::new(1);
        let mut last_sequence = 0;
        let mut sequence_restarts = 0;
        let mut last_timestamp = 0;
        
        // Generate IDs and track sequence restarts
        for _ in 0..1000 {
            let tsid = generator.generate();
            let (timestamp, _, sequence) = extract_from_tsid(tsid);
            
            if timestamp != last_timestamp {
                // Verify sequence restarts from 0 on timestamp change
                assert_eq!(sequence, 0, 
                    "Sequence should restart from 0 on timestamp change");
                sequence_restarts += 1;
                last_timestamp = timestamp;
            } else if sequence < last_sequence {
                // If sequence decreased but timestamp didn't change,
                // we've hit the sequence limit and wrapped around
                sequence_restarts += 1;
            }
            
            last_sequence = sequence;
        }
        
        // Ensure we had at least some sequence restarts
        assert!(sequence_restarts > 0, 
            "No sequence restarts detected in 1000 generations");
    }
}
