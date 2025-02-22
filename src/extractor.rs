use crate::config::TsidConfig;

/// TSID component extractor
#[derive(Debug)]
pub struct TsidExtractor {
    config: TsidConfig,
}

impl TsidExtractor {
    /// Create a new TSID extractor with the given configuration
    pub(crate) fn new(config: TsidConfig) -> Self {
        Self { config }
    }

    /// Extract timestamp component from a TSID
    #[inline]
    pub fn timestamp(&self, tsid: u64) -> u64 {
        (tsid >> self.config.timestamp_shift()) & self.config.timestamp_mask()
    }

    /// Extract node component from a TSID
    #[inline]
    pub fn node(&self, tsid: u64) -> u16 {
        ((tsid >> self.config.node_shift()) & self.config.node_mask() as u64) as u16
    }

    /// Extract sequence component from a TSID
    #[inline]
    pub fn sequence(&self, tsid: u64) -> u16 {
        (tsid & self.config.sequence_mask() as u64) as u16
    }

    /// Decompose TSID into its components: timestamp, node ID, and sequence
    pub fn decompose(&self, tsid: u64) -> (u64, u16, u16) {
        (
            self.timestamp(tsid),
            self.node(tsid),
            self.sequence(tsid)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tsid;

    #[test]
    fn test_decompose() {
        let config = TsidConfig::default();
        let tsid_gen = Tsid::with_config(42, config).unwrap();

        // Create a known TSID value with specific components
        let timestamp: u64 = 0x1234567;
        let node: u16 = 42;
        let sequence: u16 = 123;

        // Create TSID using the generator's internal method
        let tsid = ((timestamp & config.timestamp_mask()) << config.timestamp_shift())
            | ((node as u64 & config.node_mask() as u64) << config.node_shift())
            | (sequence as u64 & config.sequence_mask() as u64);

        // Test individual component extraction
        assert_eq!(tsid_gen.extract.timestamp(tsid), timestamp);
        assert_eq!(tsid_gen.extract.node(tsid), node);
        assert_eq!(tsid_gen.extract.sequence(tsid), sequence);

        // Test combined extraction
        let (ext_timestamp, ext_node, ext_sequence) = tsid_gen.extract.decompose(tsid);
        assert_eq!(ext_timestamp, timestamp);
        assert_eq!(ext_node, node);
        assert_eq!(ext_sequence, sequence);
    }

    #[test]
    fn test_component_boundaries() {
        let config = TsidConfig::default();
        let tsid_gen = Tsid::with_config(1, config).unwrap();

        // Test maximum values
        let max_timestamp = (1u64 << 42) - 1;
        let max_node_id = config.max_node_id();
        let max_sequence = config.max_sequence();

        // Create TSID using maximum values
        let tsid = ((max_timestamp & config.timestamp_mask()) << config.timestamp_shift())
            | ((max_node_id as u64 & config.node_mask() as u64) << config.node_shift())
            | (max_sequence as u64 & config.sequence_mask() as u64);

        assert_eq!(tsid_gen.extract.timestamp(tsid), max_timestamp);
        assert_eq!(tsid_gen.extract.node(tsid), max_node_id);
        assert_eq!(tsid_gen.extract.sequence(tsid), max_sequence);
    }
}
