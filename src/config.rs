/// Default configuration values
pub const TIMESTAMP_BITS: u8 = 42; // Fixed timestamp bits
const TOTAL_NODE_AND_SEQUENCE_BITS: u8 = 22; // Fixed total for node + sequence
pub const DEFAULT_NODE_BITS: u8 = 10;
pub const DEFAULT_CUSTOM_EPOCH: u64 = 1704067200000; // January 1, 2024 UTC

/// Configuration for TSID Generator
#[derive(Debug, Clone, Copy)]
pub struct TsidConfig {
    pub(crate) node_bits: u8,
    pub(crate) sequence_bits: u8,
    pub(crate) custom_epoch: u64,
}

impl Default for TsidConfig {
    fn default() -> Self {
        Self {
            node_bits: DEFAULT_NODE_BITS,
            sequence_bits: TOTAL_NODE_AND_SEQUENCE_BITS - DEFAULT_NODE_BITS,
            custom_epoch: DEFAULT_CUSTOM_EPOCH,
        }
    }
}

/// Builder for TsidConfig
#[derive(Debug)]
pub struct TsidConfigBuilder {
    config: TsidConfig,
}

impl TsidConfigBuilder {
    /// Create a new TsidConfigBuilder with default values
    pub fn new() -> Self {
        Self {
            config: TsidConfig::default(),
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
        self.config.node_bits = bits;
        self.config.sequence_bits = TOTAL_NODE_AND_SEQUENCE_BITS - bits;
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
        self.config.custom_epoch = epoch;
        self
    }

    /// Build the final TsidConfig
    /// 
    /// # Returns
    /// * `TsidConfig` - The configured TsidConfig instance
    pub fn build(self) -> TsidConfig {
        self.config
    }
}

impl TsidConfig {
    /// Create a new configuration builder
    pub fn builder() -> TsidConfigBuilder {
        TsidConfigBuilder::new()
    }

    /// Create masks and shifts based on configuration
    pub(crate) fn create_bit_config(&self) -> BitConfig {
        let node_shift = self.sequence_bits;
        let timestamp_shift = self.node_bits + self.sequence_bits;

        let sequence_mask = (1 << self.sequence_bits) - 1;
        let node_mask = (1 << self.node_bits) - 1;
        let timestamp_mask = (1u64 << TIMESTAMP_BITS) - 1;

        BitConfig {
            node_shift,
            timestamp_shift,
            sequence_mask,
            node_mask,
            timestamp_mask,
            max_sequence: sequence_mask,
            max_node: node_mask,
        }
    }
}

/// Internal bit configuration
#[derive(Debug, Clone, Copy)]
pub(crate) struct BitConfig {
    pub(crate) node_shift: u8,
    pub(crate) timestamp_shift: u8,
    pub(crate) sequence_mask: u16,
    pub(crate) node_mask: u16,
    pub(crate) timestamp_mask: u64,
    pub(crate) max_sequence: u16,
    pub(crate) max_node: u16,
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
        assert_eq!(config.sequence_bits, 10); // 22 - 12
        assert_eq!(config.custom_epoch, 1640995200000);
    }

    #[test]
    fn test_default_config() {
        let config = TsidConfig::default();
        assert_eq!(config.node_bits, DEFAULT_NODE_BITS);
        assert_eq!(config.sequence_bits, TOTAL_NODE_AND_SEQUENCE_BITS - DEFAULT_NODE_BITS);
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
        let bit_config = config.create_bit_config();

        assert_eq!(bit_config.node_shift, 12);
        assert_eq!(bit_config.timestamp_shift, 22);
        assert_eq!(bit_config.sequence_mask, 0xFFF);
        assert_eq!(bit_config.node_mask, 0x3FF);
        assert_eq!(bit_config.timestamp_mask, (1u64 << 42) - 1);
        assert_eq!(bit_config.max_sequence, 0xFFF);
        assert_eq!(bit_config.max_node, 0x3FF);
    }
}
