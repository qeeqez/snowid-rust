use crate::SnowID;

/// Default configuration values
const DEFAULT_NODE_BITS: u8 = 10;
const DEFAULT_CUSTOM_EPOCH: u64 = 1704067200000; // January 1, 2024 UTC

/// Configuration for SnowID generator
#[derive(Debug, Clone, Copy)]
pub struct SnowIDConfig {
    node_bits: u8,
    custom_epoch: u64,
    timestamp_shift: u8,
    node_shift: u8,
    timestamp_mask: u64,
    node_mask: u16,
    sequence_mask: u16,
}

impl SnowIDConfig {
    /// Create new SnowIDConfig with given node bits
    fn new(node_bits: u8, custom_epoch: u64) -> Self {
        let sequence_bits = SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - node_bits;
        Self {
            node_bits,
            custom_epoch,
            timestamp_shift: SnowID::TOTAL_NODE_AND_SEQUENCE_BITS,
            node_shift: sequence_bits,
            timestamp_mask: (1 << SnowID::TIMESTAMP_BITS) - 1,
            node_mask: ((1u32 << node_bits) - 1) as u16,
            sequence_mask: ((1u32 << sequence_bits) - 1) as u16,
        }
    }

    /// Create a new configuration builder
    pub fn builder() -> SnowIDConfigBuilder {
        SnowIDConfigBuilder::new()
    }

    /// Get epoch timestamp
    #[inline]
    pub fn epoch(&self) -> u64 {
        self.custom_epoch
    }

    /// Get node bits configuration
    #[inline]
    pub fn node_bits(&self) -> u8 {
        self.node_bits
    }

    /// Get sequence bits derived from node bits
    #[inline]
    pub fn sequence_bits(&self) -> u8 {
        SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - self.node_bits
    }

    /// Get the maximum node ID supported by the current configuration
    #[inline]
    pub fn max_node_id(&self) -> u16 {
        self.node_mask
    }

    /// Get the maximum sequence number supported by the current configuration
    #[inline]
    pub fn max_sequence_id(&self) -> u16 {
        self.sequence_mask
    }

    // Internal methods used by SnowID and SnowIDExtractor
    #[inline]
    pub(crate) fn timestamp_shift(&self) -> u8 {
        self.timestamp_shift
    }

    #[inline]
    pub(crate) fn node_shift(&self) -> u8 {
        self.node_shift
    }

    #[inline]
    pub(crate) fn timestamp_mask(&self) -> u64 {
        self.timestamp_mask
    }

    #[inline]
    pub(crate) fn node_mask(&self) -> u16 {
        self.node_mask
    }

    #[inline]
    pub(crate) fn sequence_mask(&self) -> u16 {
        self.sequence_mask
    }

    #[inline]
    pub(crate) fn custom_epoch(&self) -> u64 {
        self.custom_epoch
    }
}

impl Default for SnowIDConfig {
    fn default() -> Self {
        Self::new(DEFAULT_NODE_BITS, DEFAULT_CUSTOM_EPOCH)
    }
}

/// Builder for SnowIDConfig
#[derive(Debug)]
pub struct SnowIDConfigBuilder {
    node_bits: u8,
    custom_epoch: u64,
}

impl SnowIDConfigBuilder {
    /// Create a new SnowIDConfigBuilder with default values
    pub fn new() -> Self {
        Self {
            node_bits: DEFAULT_NODE_BITS,
            custom_epoch: DEFAULT_CUSTOM_EPOCH,
        }
    }

    /// Set the number of bits for node ID (6-16)
    /// Sequence bits will be automatically set to (22 - node_bits)
    ///
    /// # Arguments
    /// * `bits` - Number of bits for node ID (6-16)
    ///
    /// # Returns
    /// * `Self` - Builder instance for chaining
    ///
    /// # Panics
    /// Panics if bits is not between 6 and 16 (inclusive)
    ///
    /// # Note
    /// The range is limited to 6-16 bits due to u16 constraints:
    /// - Minimum 6 bits = 64 nodes (reasonable minimum for distributed systems)
    /// - Maximum 16 bits = 65,536 nodes (u16 max value)
    pub fn node_bits(mut self, bits: u8) -> Self {
        assert!(
            (6..=16).contains(&bits),
            "Node bits must be between 6 and 16"
        );
        self.node_bits = bits;
        self
    }

    /// Set a custom epoch timestamp in milliseconds
    ///
    /// # Arguments
    /// * `epoch` - Custom epoch timestamp in milliseconds since Unix epoch
    ///
    /// # Returns
    /// * `Self` - Builder instance for chaining
    pub fn epoch(mut self, epoch: u64) -> Self {
        self.custom_epoch = epoch;
        self
    }

    /// Build the final SnowIDConfig
    ///
    /// # Returns
    /// * `SnowIDConfig` - The configured SnowIDConfig instance
    pub fn build(self) -> SnowIDConfig {
        SnowIDConfig::new(self.node_bits, self.custom_epoch)
    }
}

impl Default for SnowIDConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod node_bits_validation {
        use super::*;

        #[test]
        fn test_valid_node_bits() {
            // Test all valid node bits from 6 to 16
            for bits in 6..=16 {
                let config = SnowIDConfig::builder().node_bits(bits).build();
                assert_eq!(config.node_bits(), bits);
                assert_eq!(
                    config.sequence_bits(),
                    SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - bits
                );
                assert_eq!(config.max_node_id(), ((1u32 << bits) - 1) as u16);
            }
        }

        #[test]
        #[should_panic(expected = "Node bits must be between 6 and 16")]
        fn test_too_few_node_bits() {
            SnowIDConfig::builder().node_bits(5).build();
        }

        #[test]
        #[should_panic(expected = "Node bits must be between 6 and 16")]
        fn test_too_many_node_bits() {
            SnowIDConfig::builder().node_bits(17).build();
        }

        #[test]
        #[should_panic(expected = "Node bits must be between 6 and 16")]
        fn test_max_u8_node_bits() {
            SnowIDConfig::builder().node_bits(u8::MAX).build();
        }
    }

    #[test]
    fn test_custom_config() {
        let config = SnowIDConfig::builder()
            .node_bits(12)
            .epoch(1640995200000) // 2022-01-01
            .build();

        assert_eq!(config.node_bits(), 12);
        assert_eq!(config.sequence_bits(), 10); // 22 - 12
        assert_eq!(config.custom_epoch(), 1640995200000);
    }

    #[test]
    fn test_default_config() {
        let config = SnowIDConfig::default();
        assert_eq!(config.node_bits(), DEFAULT_NODE_BITS);
        assert_eq!(
            config.sequence_bits(),
            SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - DEFAULT_NODE_BITS
        );
        assert_eq!(config.custom_epoch(), DEFAULT_CUSTOM_EPOCH);
    }

    #[test]
    #[should_panic(expected = "Node bits must be between 6 and 16")]
    fn test_invalid_node_bits() {
        SnowIDConfig::builder().node_bits(21).build();
    }

    #[test]
    fn test_bit_config() {
        let config = SnowIDConfig::default();
        assert_eq!(config.node_shift(), 12);
        assert_eq!(config.timestamp_shift(), 22);
        assert_eq!(config.sequence_mask(), 0xFFF);
        assert_eq!(config.node_mask(), 0x3FF);
        assert_eq!(config.timestamp_mask(), (1u64 << 42) - 1);
        assert_eq!(config.max_sequence_id(), 0xFFF);
        assert_eq!(config.max_node_id(), 0x3FF);
    }
}
