#[cfg(test)]
mod tests {
    use crate::*;
    use std::collections::HashSet;

    #[test]
    fn test_tsid_components() {
        let generator = TsidGenerator::new(42).unwrap();
        let tsid = generator.generate().unwrap();
        let (timestamp, node, sequence) = generator.extract.decompose(tsid);
        
        assert_eq!(node, 42);
        assert_eq!(sequence, 0);
        assert!(timestamp > 0);
    }

    #[test]
    fn test_bit_layout() {
        let config = TsidConfig::builder()
            .node_bits(12)
            .sequence_bits(10)
            .custom_epoch(0)
            .build();

        let generator = TsidGenerator::with_config(42, config).unwrap();
        let tsid = generator.generate().unwrap();

        // Extract components
        let (timestamp, node, sequence) = generator.extract.decompose(tsid);

        // Verify bit layout
        assert_eq!(node, 42);
        assert!(sequence <= 1023); // 10 bits = max value of 1023
        assert!(timestamp > 0);
    }

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
        let (_, node, sequence) = generator.extract.decompose(tsid);
        
        assert!(node <= 4095, "Node ID exceeds maximum");
        assert!(sequence <= 1023, "Sequence exceeds maximum");
    }

    #[test]
    fn test_sequential_generation() {
        let generator = TsidGenerator::new(1).unwrap();
        let tsid1 = generator.generate().unwrap();
        let tsid2 = generator.generate().unwrap();
        assert!(tsid2 > tsid1);
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
    fn test_epoch_handling() {
        let custom_epoch = 1577836800000; // 2020-01-01 00:00:00 UTC
        let config = TsidConfig::builder()
            .custom_epoch(custom_epoch)
            .build();

        let generator = TsidGenerator::with_config(1, config).unwrap();
        let tsid = generator.generate().unwrap();
        let (timestamp, _, _) = generator.extract.decompose(tsid);

        // The extracted timestamp should be relative to custom epoch
        assert!(timestamp > 0);
        
        // Convert back to Unix timestamp
        let unix_ts = timestamp + custom_epoch;
        assert!(unix_ts > custom_epoch);
        assert!(unix_ts < (custom_epoch + (1u64 << 41))); // Should be within ~69 years of epoch
    }
}
