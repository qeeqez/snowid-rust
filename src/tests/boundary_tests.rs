#[cfg(test)]
mod tests {
    use crate::config::SnowIDConfig;
    use crate::SnowID;
    use crate::SnowIDError;
    use crate::config::TIMESTAMP_BITS;

    #[test]
    fn test_invalid_node_id() {
        match SnowID::new(1024) {
            Err(SnowIDError::InvalidNodeId { node_id, max_allowed }) => {
                assert_eq!(node_id, 1024);
                assert_eq!(max_allowed, 1023);
            }
            _ => panic!("Expected InvalidNodeId error"),
        }
    }

    #[test]
    fn test_node_id_boundaries() {
        // Test minimum node ID
        let gen0 = SnowID::new(0).unwrap();
        let snowid0 = gen0.generate().unwrap();
        let (_, node0, _) = gen0.extract.decompose(snowid0);
        assert_eq!(node0, 0);

        // Test maximum node ID
        let gen1023 = SnowID::new(1023).unwrap();
        let snowid1023 = gen1023.generate().unwrap();
        let (_, node1023, _) = gen1023.extract.decompose(snowid1023);
        assert_eq!(node1023, 1023);
    }

    #[test]
    fn test_component_boundaries() {
        let config = SnowIDConfig::builder()
            .node_bits(10)
            .custom_epoch(0)
            .build();

        let generator = SnowID::with_config(1023, config).unwrap();

        // Test timestamp boundaries
        let snowid = generator.generate().unwrap();
        let (timestamp, _, _) = generator.extract.decompose(snowid);
        assert!(timestamp > 0);
        assert!(timestamp <= (1u64 << TIMESTAMP_BITS) - 1);

        // Test node boundaries
        let (_, node, _) = generator.extract.decompose(snowid);
        assert!(node <= 1023);

        // Test sequence boundaries
        let (_, _, sequence) = generator.extract.decompose(snowid);
        assert!(sequence <= 4095);

        // Test custom bit layout boundaries
        let custom_config = SnowIDConfig::builder()
            .node_bits(12)
            .custom_epoch(0)
            .build();

        let custom_gen = SnowID::with_config(4095, custom_config).unwrap();
        let snowid = custom_gen.generate().unwrap();
        let (_, node, sequence) = custom_gen.extract.decompose(snowid);

        assert!(node <= 4095);
        assert!(sequence <= 1023);
    }

    #[test]
    fn test_zero_node_id() {
        let generator = SnowID::new(0).unwrap();
        let snowid = generator.generate().unwrap();
        let (_, node, _) = generator.extract.decompose(snowid);
        assert_eq!(node, 0);
    }

    #[test]
    fn test_component_max_values() {
        let generator = SnowID::new(1023).unwrap();
        let SnowID = generator.generate().unwrap();
        let (timestamp, node, sequence) = generator.extract.decompose(SnowID);

        assert!(timestamp <= generator.config.timestamp_mask());
        assert!(node <= generator.config.max_node_id());
        assert!(sequence <= generator.config.max_sequence());
    }
}
