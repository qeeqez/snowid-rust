/// Default configuration values
pub const TIMESTAMP_BITS: u8 = 42; // Fixed timestamp bits
const TOTAL_NODE_AND_SEQUENCE_BITS: u8 = 22; // Fixed total for node + sequence
pub const DEFAULT_NODE_BITS: u8 = 10;
pub const DEFAULT_CUSTOM_EPOCH: u64 = 1704067200000; // January 1, 2024 UTC

/// Configuration for TSID Generator
#[derive(Debug, Clone, Copy)]
pub struct TsidConfig {
    pub(crate) node_bits: u8,
    pub(crate) custom_epoch: u64,
    timestamp_shift: u8,
    node_shift: u8,
    timestamp_mask: u64,
    node_mask: u16,
    sequence_mask: u16,
}

impl TsidConfig {
    /// Create new TsidConfig with given node bits
    fn new(node_bits: u8, custom_epoch: u64) -> Self {
        let sequence_bits = TOTAL_NODE_AND_SEQUENCE_BITS - node_bits;
        Self {
            node_bits,
            custom_epoch,
            timestamp_shift: TOTAL_NODE_AND_SEQUENCE_BITS,
            node_shift: sequence_bits,
            timestamp_mask: (1 << TIMESTAMP_BITS) - 1,
            node_mask: (1 << node_bits) - 1,
            sequence_mask: (1 << sequence_bits) - 1,
        }
    }

    /// Create a new configuration builder
    pub fn builder() -> TsidConfigBuilder {
        TsidConfigBuilder::new()
    }

    /// Get sequence bits derived from node bits
    #[inline]
    pub fn sequence_bits(&self) -> u8 {
        TOTAL_NODE_AND_SEQUENCE_BITS - self.node_bits
    }

    /// Get the maximum node ID supported by the current configuration
    #[inline]
    pub fn max_node_id(&self) -> u16 {
        self.node_mask
    }

    /// Get the maximum sequence number supported by the current configuration
    #[inline]
    pub fn max_sequence(&self) -> u16 {
        self.sequence_mask
    }

    /// Get timestamp shift for internal use
    #[inline]
    pub(crate) fn timestamp_shift(&self) -> u8 {
        self.timestamp_shift
    }

    /// Get node shift for internal use
    #[inline]
    pub(crate) fn node_shift(&self) -> u8 {
        self.node_shift
    }

    /// Get timestamp mask for internal use
    #[inline]
    pub(crate) fn timestamp_mask(&self) -> u64 {
        self.timestamp_mask
    }

    /// Get node mask for internal use
    #[inline]
    pub(crate) fn node_mask(&self) -> u16 {
        self.node_mask
    }

    /// Get sequence mask for internal use
    #[inline]
    pub(crate) fn sequence_mask(&self) -> u16 {
        self.sequence_mask
    }
}

impl Default for TsidConfig {
    fn default() -> Self {
        Self::new(DEFAULT_NODE_BITS, DEFAULT_CUSTOM_EPOCH)
    }
}

/// Builder for TsidConfig
#[derive(Debug)]
pub struct TsidConfigBuilder {
    node_bits: u8,
    custom_epoch: u64,
}

impl TsidConfigBuilder {
    /// Create a new TsidConfigBuilder with default values
    pub fn new() -> Self {
        Self {
            node_bits: DEFAULT_NODE_BITS,
            custom_epoch: DEFAULT_CUSTOM_EPOCH,
        }
    }

    /// Set the number of bits for node ID (1-20)
    /// Sequence bits will be automatically set to (22 - node_bits)
    /// 
    /// # Arguments
    /// * `bits` - Number of bits for node ID (1-20)
    /// 
    /// # Returns
    /// * `Self` - Builder instance for chaining
    /// 
    /// # Panics
    /// Panics if bits is not between 1 and 20
    pub fn node_bits(mut self, bits: u8) -> Self {
        assert!(bits > 0 && bits <= 20, "Node bits must be between 1 and 20");
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
    pub fn custom_epoch(mut self, epoch: u64) -> Self {
        self.custom_epoch = epoch;
        self
    }

    /// Build the final TsidConfig
    /// 
    /// # Returns
    /// * `TsidConfig` - The configured TsidConfig instance
    pub fn build(self) -> TsidConfig {
        TsidConfig::new(self.node_bits, self.custom_epoch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_config() {
        let config = TsidConfig::builder()
            .node_bits(12)
            .custom_epoch(1640995200000) // 2022-01-01
            .build();

        assert_eq!(config.node_bits, 12);
        assert_eq!(config.sequence_bits(), 10); // 22 - 12
        assert_eq!(config.custom_epoch, 1640995200000);
    }

    #[test]
    fn test_default_config() {
        let config = TsidConfig::default();
        assert_eq!(config.node_bits, DEFAULT_NODE_BITS);
        assert_eq!(config.sequence_bits(), TOTAL_NODE_AND_SEQUENCE_BITS - DEFAULT_NODE_BITS);
        assert_eq!(config.custom_epoch, DEFAULT_CUSTOM_EPOCH);
    }

    #[test]
    #[should_panic(expected = "Node bits must be between 1 and 20")]
    fn test_invalid_node_bits() {
        TsidConfig::builder().node_bits(21).build();
    }

    #[test]
    fn test_bit_config() {
        let config = TsidConfig::default();
        assert_eq!(config.node_shift(), 12);
        assert_eq!(config.timestamp_shift(), 22);
        assert_eq!(config.sequence_mask(), 0xFFF);
        assert_eq!(config.node_mask(), 0x3FF);
        assert_eq!(config.timestamp_mask(), (1u64 << 42) - 1);
        assert_eq!(config.max_sequence(), 0xFFF);
        assert_eq!(config.max_node_id(), 0x3FF);
    }
}
