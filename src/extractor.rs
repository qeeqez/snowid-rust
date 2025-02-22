use crate::config::BitConfig;

/// TSID component extractor
pub struct TsidExtractor {
    bit_config: BitConfig,
}

impl TsidExtractor {
    /// Create a new TSID extractor with the given bit configuration
    pub(crate) fn new(bit_config: BitConfig) -> Self {
        Self { bit_config }
    }

    /// Extract timestamp, node ID, and sequence from a TSID
    pub fn extract_from_tsid(&self, tsid: u64) -> (u64, u16, u16) {
        (
            self.extract_timestamp(tsid),
            self.extract_node(tsid),
            self.extract_sequence(tsid)
        )
    }

    /// Extract timestamp from a TSID
    #[inline]
    pub fn extract_timestamp(&self, tsid: u64) -> u64 {
        (tsid >> self.bit_config.timestamp_shift) & self.bit_config.timestamp_mask
    }

    /// Extract node ID from a TSID
    #[inline]
    pub fn extract_node(&self, tsid: u64) -> u16 {
        ((tsid >> self.bit_config.node_shift) & self.bit_config.node_mask as u64) as u16
    }

    /// Extract sequence from a TSID
    #[inline]
    pub fn extract_sequence(&self, tsid: u64) -> u16 {
        (tsid & self.bit_config.sequence_mask as u64) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TsidConfig;

    #[test]
    fn test_extract_components() {
        let config = TsidConfig::default();
        let bit_config = config.create_bit_config();
        let extractor = TsidExtractor::new(bit_config);

        // Create a known TSID value with specific components
        let timestamp: u64 = 0x1234567;
        let node: u16 = 42;
        let sequence: u16 = 123;

        let tsid = ((timestamp & bit_config.timestamp_mask) << bit_config.timestamp_shift)
            | ((node as u64 & bit_config.node_mask as u64) << bit_config.node_shift)
            | (sequence as u64 & bit_config.sequence_mask as u64);

        // Test individual component extraction
        assert_eq!(extractor.extract_timestamp(tsid), timestamp);
        assert_eq!(extractor.extract_node(tsid), node);
        assert_eq!(extractor.extract_sequence(tsid), sequence);

        // Test combined extraction
        let (ext_timestamp, ext_node, ext_sequence) = extractor.extract_from_tsid(tsid);
        assert_eq!(ext_timestamp, timestamp);
        assert_eq!(ext_node, node);
        assert_eq!(ext_sequence, sequence);
    }

    #[test]
    fn test_component_boundaries() {
        let config = TsidConfig::default();
        let bit_config = config.create_bit_config();
        let extractor = TsidExtractor::new(bit_config);

        // Test maximum values
        let max_timestamp = bit_config.timestamp_mask;
        let max_node = bit_config.max_node;
        let max_sequence = bit_config.max_sequence;

        let tsid = (max_timestamp << bit_config.timestamp_shift)
            | ((max_node as u64) << bit_config.node_shift)
            | max_sequence as u64;

        let (timestamp, node, sequence) = extractor.extract_from_tsid(tsid);
        assert_eq!(timestamp, max_timestamp);
        assert_eq!(node, max_node);
        assert_eq!(sequence, max_sequence);
    }
}
