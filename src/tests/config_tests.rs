//! Configuration tests

#[cfg(test)]
mod tests {
    use crate::SnowID;
    use crate::config::{SnowIDConfig, SnowIDConfigError};

    #[test]
    fn test_valid_node_bits() {
        for bits in 6..=16 {
            let config = SnowIDConfig::builder().node_bits(bits).unwrap().build();
            assert_eq!(config.node_bits(), bits);
            assert_eq!(
                config.sequence_bits(),
                SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - bits
            );
        }
    }

    #[test]
    fn test_node_bits_ok() {
        let cfg = SnowIDConfig::builder().node_bits(12).unwrap().build();
        assert_eq!(cfg.node_bits(), 12);
    }

    #[test]
    fn test_node_bits_err() {
        let err = SnowIDConfig::builder().node_bits(5).unwrap_err();
        assert_eq!(err, SnowIDConfigError::InvalidNodeBits { bits: 5 });
    }

    #[test]
    fn test_custom_config() {
        let config = SnowIDConfig::builder()
            .node_bits(12)
            .unwrap()
            .epoch(1640995200000)
            .build();

        assert_eq!(config.node_bits(), 12);
        assert_eq!(config.sequence_bits(), 10);
        assert_eq!(config.epoch(), 1640995200000);
    }

    #[test]
    fn test_default_config() {
        let config = SnowIDConfig::default();
        assert_eq!(config.node_bits(), 10);
        assert_eq!(config.sequence_bits(), 12);
        assert!(config.spin_enabled());
    }

    #[test]
    fn test_bit_config() {
        let config = SnowIDConfig::default();
        assert_eq!(config.max_sequence_id(), 0xFFF);
        assert_eq!(config.max_node_id(), 0x3FF);
    }

    #[test]
    fn test_spin_tuning_builder() {
        let cfg = SnowIDConfig::builder()
            .enable_spin(false)
            .spin_loops(0)
            .spin_yield_every(0)
            .build();
        assert!(!cfg.spin_enabled());
        assert_eq!(cfg.spin_loops(), 0);
        assert_eq!(cfg.spin_yield_every(), 0);

        let cfg2 = SnowIDConfig::builder()
            .enable_spin(true)
            .spin_loops(128)
            .spin_yield_every(8)
            .build();
        assert!(cfg2.spin_enabled());
        assert_eq!(cfg2.spin_loops(), 128);
        assert_eq!(cfg2.spin_yield_every(), 8);
    }
}
