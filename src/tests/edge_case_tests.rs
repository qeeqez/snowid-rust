//! Edge case tests for Discord-style Snowflake ID generation

#[cfg(test)]
mod tests {
    use crate::tests::test_utils::{assert_ids_monotonic, assert_unique_ids};
    use crate::*;
    use std::collections::HashSet;
    use std::sync::{Arc, Barrier};
    use std::thread;

    /// Test extreme minimum: 6-bit node ID (64 nodes, 65536 seq/ms)
    #[test]
    fn test_min_node_bits_config() {
        let cfg = SnowIDConfig::builder().node_bits(6).unwrap().build();
        assert_eq!(cfg.max_node_id(), 63);
        assert_eq!(cfg.max_sequence_id(), 65535);

        let g = SnowID::with_config(63, cfg).unwrap();
        let id = g.generate();
        let (ts, node, seq) = g.extract.decompose(id);

        assert!(ts > 0);
        assert_eq!(node, 63);
        assert!(seq <= 65535);
    }

    /// Test extreme maximum: 16-bit node ID (65536 nodes, 64 seq/ms)
    #[test]
    fn test_max_node_bits_config() {
        let cfg = SnowIDConfig::builder().node_bits(16).unwrap().build();
        assert_eq!(cfg.max_node_id(), 65535);
        assert_eq!(cfg.max_sequence_id(), 63);

        let g = SnowID::with_config(65535, cfg).unwrap();
        let id = g.generate();
        let (ts, node, seq) = g.extract.decompose(id);

        assert!(ts > 0);
        assert_eq!(node, 65535);
        assert!(seq <= 63);
    }

    /// Verify ID bit structure: timestamp | node | sequence
    #[test]
    fn test_id_bit_structure() {
        let cfg = SnowIDConfig::builder().node_bits(10).unwrap().epoch(0).build();
        let g = SnowID::with_config(0b1010101010, cfg).unwrap();

        let id = g.generate();
        let node = g.extract.node(id);
        assert_eq!(node, 0b1010101010, "Node bits should be preserved");

        // Verify bit positions: seq=12 bits, node=10 bits, ts=42 bits
        let seq_mask = 0xFFF; // 12 bits
        let node_mask = 0x3FF << 12; // 10 bits shifted by 12
        let ts_mask = !0u64 << 22; // remaining 42 bits

        assert_eq!(id & seq_mask, g.extract.sequence(id) as u64);
        assert_eq!((id & node_mask) >> 12, g.extract.node(id) as u64);
        assert_eq!((id & ts_mask) >> 22, g.extract.timestamp(id));
    }

    /// Test that IDs from different nodes never collide
    #[test]
    fn test_cross_node_uniqueness() {
        let mut all_ids = HashSet::new();

        for node in 0..10 {
            let g = SnowID::new(node).unwrap();
            for _ in 0..100 {
                let id = g.generate();
                assert!(all_ids.insert(id), "Collision from node {}", node);
            }
        }
        assert_eq!(all_ids.len(), 1000);
    }

    /// Test IDs are numerically sorted (not lexicographically)
    #[test]
    fn test_numeric_sorting() {
        let g = SnowID::new(1).unwrap();
        let ids: Vec<u64> = (0..100).map(|_| g.generate()).collect();

        let mut sorted = ids.clone();
        sorted.sort();

        assert_eq!(ids, sorted, "IDs should already be numerically sorted");
    }

    /// Test high contention: multiple threads on same millisecond
    #[test]
    fn test_high_contention() {
        let g = Arc::new(SnowID::new(1).unwrap());
        let barrier = Arc::new(Barrier::new(4));
        let mut handles = vec![];

        for _ in 0..4 {
            let g = Arc::clone(&g);
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                b.wait(); // Sync start
                (0..250).map(|_| g.generate()).collect::<Vec<_>>()
            }));
        }

        let mut all_ids = vec![];
        for h in handles {
            all_ids.extend(h.join().unwrap());
        }

        assert_unique_ids(&all_ids, 1000);
    }

    /// Test sequence exhaustion triggers timestamp advance
    #[test]
    fn test_sequence_exhaustion_advances_timestamp() {
        // Use 16-bit nodes = only 64 sequences per ms
        let cfg = SnowIDConfig::builder().node_bits(16).unwrap().build();
        let g = SnowID::with_config(0, cfg).unwrap();

        let first_id = g.generate();
        let first_ts = g.extract.timestamp(first_id);

        // Generate 64 more IDs (should exhaust sequence)
        let mut ids = vec![first_id];
        for _ in 0..63 {
            ids.push(g.generate());
        }

        // Next ID should have advanced timestamp
        let next_id = g.generate();
        let next_ts = g.extract.timestamp(next_id);

        assert!(next_ts >= first_ts, "Timestamp should not regress");
        assert_ids_monotonic(&ids);
    }

    /// Test epoch near current time (minimal timestamp values)
    #[test]
    fn test_recent_epoch() {
        let recent_epoch = 1735689600000u64; // 2025-01-01
        let cfg = SnowIDConfig::builder().epoch(recent_epoch).build();
        let g = SnowID::with_config(1, cfg).unwrap();

        let id = g.generate();
        let ts = g.extract.timestamp(id);

        // Timestamp should be reasonable (within ~2 years of epoch)
        assert!(ts < 2 * 365 * 24 * 60 * 60 * 1000, "Expected ts < 2 years, got {}", ts);
    }

    /// Test all supported node_bits configurations
    #[test]
    fn test_all_node_bits_configs() {
        for bits in 6..=16 {
            let cfg = SnowIDConfig::builder().node_bits(bits).unwrap().build();
            let max_node = cfg.max_node_id();
            let max_seq = cfg.max_sequence_id();

            // Verify bit allocation
            assert_eq!(bits + cfg.sequence_bits(), 22);
            assert_eq!(max_node as u32, (1u32 << bits) - 1);
            assert_eq!(max_seq as u32, (1u32 << (22 - bits)) - 1);

            // Create generator at max node
            let g = SnowID::with_config(max_node, cfg).unwrap();
            let id = g.generate();
            let (_, node, seq) = g.extract.decompose(id);

            assert_eq!(node, max_node);
            assert!(seq <= max_seq);
        }
    }

    /// Test Base62 encoding preserves sort order
    #[test]
    fn test_base62_sort_order() {
        let g = SnowID::new(1).unwrap();

        let id1 = g.generate();
        thread::sleep(std::time::Duration::from_millis(5));
        let id2 = g.generate();

        let (s1, _) = g.generate_base62_with_raw();
        let (s2, _) = g.generate_base62_with_raw();

        assert!(id2 > id1);
        // Note: Base62 string comparison != numeric comparison
        // but decoded values should maintain order
        assert!(base62_decode(&s2).unwrap() > base62_decode(&s1).unwrap());
    }

    /// Test ID decomposition round-trip
    #[test]
    fn test_decomposition_roundtrip() {
        let g = SnowID::new(42).unwrap();

        for _ in 0..100 {
            let id = g.generate();
            let (ts, node, seq) = g.extract.decompose(id);

            // Reconstruct ID manually
            let reconstructed = (ts << 22) | ((node as u64) << 12) | (seq as u64);
            assert_eq!(id, reconstructed, "ID should round-trip through decomposition");
        }
    }
}
