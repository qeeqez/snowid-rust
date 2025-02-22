#[cfg(test)]
mod tests {
    use crate::config::TsidConfig;
    use crate::Tsid;
    use crate::TsidError;
    use crate::config::TIMESTAMP_BITS;

    #[test]
    fn test_invalid_node_id() {
        match Tsid::new(1024) {
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
        let gen0 = Tsid::new(0).unwrap();
        let tsid0 = gen0.generate().unwrap();
        let (_, node0, _) = gen0.extract.decompose(tsid0);
        assert_eq!(node0, 0);

        // Test maximum node ID
        let gen1023 = Tsid::new(1023).unwrap();
        let tsid1023 = gen1023.generate().unwrap();
        let (_, node1023, _) = gen1023.extract.decompose(tsid1023);
        assert_eq!(node1023, 1023);
    }

    #[test]
    fn test_component_boundaries() {
        let config = TsidConfig::builder()
            .node_bits(10)
            .custom_epoch(0)
            .build();

        let generator = Tsid::with_config(1023, config).unwrap();

        // Test timestamp boundaries
        let tsid = generator.generate().unwrap();
        let (timestamp, _, _) = generator.extract.decompose(tsid);
        assert!(timestamp > 0);
        assert!(timestamp <= (1u64 << TIMESTAMP_BITS) - 1);

        // Test node boundaries
        let (_, node, _) = generator.extract.decompose(tsid);
        assert!(node <= 1023);

        // Test sequence boundaries
        let (_, _, sequence) = generator.extract.decompose(tsid);
        assert!(sequence <= 4095);

        // Test custom bit layout boundaries
        let custom_config = TsidConfig::builder()
            .node_bits(12)
            .custom_epoch(0)
            .build();

        let custom_gen = Tsid::with_config(4095, custom_config).unwrap();
        let tsid = custom_gen.generate().unwrap();
        let (_, node, sequence) = custom_gen.extract.decompose(tsid);

        assert!(node <= 4095);
        assert!(sequence <= 1023);
    }

    #[test]
    fn test_zero_node_id() {
        let generator = Tsid::new(0).unwrap();
        let tsid = generator.generate().unwrap();
        let (_, node, _) = generator.extract.decompose(tsid);
        assert_eq!(node, 0);
    }

    #[test]
    fn test_component_max_values() {
        let generator = Tsid::new(1023).unwrap();
        let tsid = generator.generate().unwrap();
        let (timestamp, node, sequence) = generator.extract.decompose(tsid);

        assert!(timestamp <= generator.config.timestamp_mask());
        assert!(node <= generator.config.max_node_id());
        assert!(sequence <= generator.config.max_sequence());
    }
}
