use crate::config::TsidConfig;

/// TSID component extractor
pub struct TsidExtractor {
    config: TsidConfig,
}

impl TsidExtractor {
    /// Create a new TSID extractor with the given configuration
    pub(crate) fn new(config: TsidConfig) -> Self {
        Self { config }
    }

    /// Decompose TSID into its components: timestamp, node ID, and sequence
    pub fn decompose(&self, tsid: u64) -> (u64, u16, u16) {
        (
            self.extract_timestamp(tsid),
            self.extract_node(tsid),
            self.extract_sequence(tsid)
        )
    }

    /// Extract timestamp from a TSID
    #[inline]
    pub fn extract_timestamp(&self, tsid: u64) -> u64 {
        (tsid >> self.config.timestamp_shift()) & self.config.timestamp_mask()
    }

    /// Extract node ID from a TSID
    #[inline]
    pub fn extract_node(&self, tsid: u64) -> u16 {
        ((tsid >> self.config.node_shift()) & self.config.node_mask() as u64) as u16
    }

    /// Extract sequence from a TSID
    #[inline]
    pub fn extract_sequence(&self, tsid: u64) -> u16 {
        (tsid & self.config.sequence_mask() as u64) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TsidConfig;

    #[test]
    fn test_extract_components() {
        let config = TsidConfig::default();
        let extractor = TsidExtractor::new(config);

        // Create a known TSID value with specific components
        let timestamp: u64 = 0x1234567;
        let node: u16 = 42;
        let sequence: u16 = 123;

        let tsid = ((timestamp & config.timestamp_mask()) << config.timestamp_shift())
            | ((node as u64 & config.node_mask() as u64) << config.node_shift())
            | (sequence as u64 & config.sequence_mask() as u64);

        // Test individual component extraction
        assert_eq!(extractor.extract_timestamp(tsid), timestamp);
        assert_eq!(extractor.extract_node(tsid), node);
        assert_eq!(extractor.extract_sequence(tsid), sequence);

        // Test combined extraction
        let (ext_timestamp, ext_node, ext_sequence) = extractor.decompose(tsid);
        assert_eq!(ext_timestamp, timestamp);
        assert_eq!(ext_node, node);
        assert_eq!(ext_sequence, sequence);
    }

    #[test]
    fn test_component_boundaries() {
        let config = TsidConfig::default();
        let extractor = TsidExtractor::new(config);

        // Test maximum values
        let max_timestamp = config.timestamp_mask();
        let max_node_id = config.max_node_id();
        let max_sequence = config.max_sequence();

        let tsid = (max_timestamp << config.timestamp_shift())
            | ((max_node_id as u64) << config.node_shift())
            | max_sequence as u64;

        let (timestamp, node, sequence) = extractor.decompose(tsid);
        assert_eq!(timestamp, max_timestamp);
        assert_eq!(node, max_node_id);
        assert_eq!(sequence, max_sequence);
    }
}
