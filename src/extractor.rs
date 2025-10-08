use crate::config::SnowIDConfig;

/// SnowID component extractor
#[derive(Debug, Copy, Clone)]
pub struct SnowIDExtractor {
    config: SnowIDConfig,
}

impl SnowIDExtractor {
    /// Create a new SnowID extractor with the given configuration
    pub(crate) fn new(config: SnowIDConfig) -> Self {
        Self { config }
    }

    /// Extract timestamp component from a SnowID
    #[inline(always)]
    pub fn timestamp(&self, id: u64) -> u64 {
        (id >> self.config.timestamp_shift()) & self.config.timestamp_mask()
    }

    /// Extract node component from a SnowID
    #[inline(always)]
    pub fn node(&self, id: u64) -> u16 {
        ((id >> self.config.node_shift()) & self.config.node_mask() as u64) as u16
    }

    /// Extract sequence component from a SnowID
    #[inline(always)]
    pub fn sequence(&self, id: u64) -> u16 {
        (id & self.config.sequence_mask() as u64) as u16
    }

    /// Decompose SnowID into its components: timestamp, node ID, and sequence
    /// Optimized to extract all components in a single pass
    #[inline]
    pub fn decompose(&self, id: u64) -> (u64, u16, u16) {
        let timestamp = (id >> self.config.timestamp_shift()) & self.config.timestamp_mask();
        let node = ((id >> self.config.node_shift()) & self.config.node_mask() as u64) as u16;
        let sequence = (id & self.config.sequence_mask() as u64) as u16;
        (timestamp, node, sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SnowID;

    fn create_snow_id(config: SnowIDConfig, timestamp: u64, node: u16, sequence: u16) -> u64 {
        ((timestamp & config.timestamp_mask()) << config.timestamp_shift())
            | ((node as u64 & config.node_mask() as u64) << config.node_shift())
            | (sequence as u64 & config.sequence_mask() as u64)
    }

    #[test]
    fn test_decompose() {
        let config = SnowIDConfig::default();
        let snowid_gen = SnowID::with_config(42, config).unwrap();

        // Create a known SnowID value with specific components
        let timestamp: u64 = 0x1234567;
        let node: u16 = 42;
        let sequence: u16 = 123;

        // Create SnowID using the generator's internal method
        let id = create_snow_id(config, timestamp, node, sequence);

        // Test individual component extraction
        assert_eq!(snowid_gen.extract.timestamp(id), timestamp);
        assert_eq!(snowid_gen.extract.node(id), node);
        assert_eq!(snowid_gen.extract.sequence(id), sequence);

        // Test combined extraction
        let (ext_timestamp, ext_node, ext_sequence) = snowid_gen.extract.decompose(id);
        assert_eq!(ext_timestamp, timestamp);
        assert_eq!(ext_node, node);
        assert_eq!(ext_sequence, sequence);
    }

    #[test]
    fn test_component_boundaries() {
        let config = SnowIDConfig::default();
        let snowid_gen = SnowID::with_config(1, config).unwrap();

        // Test maximum values
        let max_timestamp = (1u64 << 42) - 1;
        let max_node_id = config.max_node_id();
        let max_sequence = config.max_sequence_id();

        // Create SnowID using maximum values
        let id = create_snow_id(config, max_timestamp, max_node_id, max_sequence);

        assert_eq!(snowid_gen.extract.timestamp(id), max_timestamp);
        assert_eq!(snowid_gen.extract.node(id), max_node_id);
        assert_eq!(snowid_gen.extract.sequence(id), max_sequence);
    }
}
