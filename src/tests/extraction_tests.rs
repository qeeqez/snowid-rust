#[cfg(test)]
mod tests {
    use crate::*;
    use std::collections::HashSet;

    #[test]
    fn test_tsid_generation_and_extraction() {
        // Test basic generation and extraction
        let mut generator = Tsid::new(42).unwrap();
        let tsid1 = generator.generate();
        
        assert_eq!(generator.extract.node(tsid1), 42);
        assert_eq!(generator.extract.sequence(tsid1), 0);
        assert!(generator.extract.timestamp(tsid1) > 0);

        // Test sequential generation
        let tsid2 = generator.generate();
        assert!(tsid2 > tsid1);
        
        assert_eq!(generator.extract.node(tsid2), 42);
        assert!(generator.extract.sequence(tsid2) > 0);
        assert!(generator.extract.timestamp(tsid2) >= generator.extract.timestamp(tsid1));
    }

    #[test]
    fn test_custom_configuration() {
        let config = TsidConfig::builder()
            .node_bits(12)
            .build();

        let mut generator = Tsid::with_config(1023, config).unwrap();
        
        // Verify configuration limits
        assert_eq!(generator.max_node_id(), 4095);
        assert_eq!(generator.max_sequence(), 1023);

        // Generate and verify components
        let tsid = generator.generate();
        
        assert!(generator.extract.node(tsid) <= 4095, "Node ID exceeds maximum");
        assert!(generator.extract.sequence(tsid) <= 1023, "Sequence exceeds maximum");
    }

    #[test]
    fn test_unique_ids_across_nodes() {
        let mut gen1 = Tsid::new(1).unwrap();
        let mut gen2 = Tsid::new(2).unwrap();
        
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
    fn test_epoch_handling() {
        let custom_epoch = 1577836800000; // 2020-01-01 00:00:00 UTC
        let config = TsidConfig::builder()
            .custom_epoch(custom_epoch)
            .build();

        let mut generator = Tsid::with_config(1, config).unwrap();
        let tsid = generator.generate();
        let timestamp = generator.extract.timestamp(tsid);

        // The extracted timestamp should be relative to custom epoch
        assert!(timestamp > 0);
        
        // Convert back to Unix timestamp
        let unix_ts = timestamp + custom_epoch;
        assert!(unix_ts > custom_epoch);
        assert!(unix_ts < (custom_epoch + (1u64 << 41))); // Should be within ~69 years of epoch
    }
}
