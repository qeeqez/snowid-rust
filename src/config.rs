use crate::SnowID;
use thiserror::Error;

/// Default configuration values
const DEFAULT_NODE_BITS: u8 = 10;
const DEFAULT_CUSTOM_EPOCH: u64 = 1704067200000; // January 1, 2024 UTC
const DEFAULT_SPIN_ENABLED: bool = true;
const DEFAULT_SPIN_LOOPS: u32 = 64;
const DEFAULT_SPIN_YIELD_EVERY: u32 = 16;

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
    // Throughput tuning
    spin_enabled: bool,
    spin_loops: u32,
    spin_yield_every: u32,
}

/// Errors related to `SnowIDConfig` builder validation
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SnowIDConfigError {
    /// Provided node bits are out of the supported range [6, 16]
    #[error("Node bits {bits} must be between 6 and 16")]
    InvalidNodeBits { bits: u8 },
}

impl SnowIDConfig {
    /// Calculate mask for given number of bits
    #[inline]
    const fn calculate_mask(bits: u8) -> u16 {
        ((1u32 << bits) - 1) as u16
    }

    /// Create new SnowIDConfig with given node bits
    fn new(node_bits: u8, custom_epoch: u64) -> Self {
        let sequence_bits = SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - node_bits;
        Self {
            node_bits,
            custom_epoch,
            timestamp_shift: SnowID::TOTAL_NODE_AND_SEQUENCE_BITS,
            node_shift: sequence_bits,
            timestamp_mask: (1u64 << SnowID::TIMESTAMP_BITS) - 1,
            node_mask: Self::calculate_mask(node_bits),
            sequence_mask: Self::calculate_mask(sequence_bits),
            spin_enabled: DEFAULT_SPIN_ENABLED,
            spin_loops: DEFAULT_SPIN_LOOPS,
            spin_yield_every: DEFAULT_SPIN_YIELD_EVERY,
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

    /// Whether micro spin is enabled before sleeping on overflow
    #[inline]
    pub fn spin_enabled(&self) -> bool {
        self.spin_enabled
    }

    /// Number of spin loops to attempt before sleeping
    #[inline]
    pub fn spin_loops(&self) -> u32 {
        self.spin_loops
    }

    /// Yield frequency during spin: yield every N iterations; 0 disables yielding
    #[inline]
    pub fn spin_yield_every(&self) -> u32 {
        self.spin_yield_every
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
    spin_enabled: bool,
    spin_loops: u32,
    spin_yield_every: u32,
}

impl SnowIDConfigBuilder {
    /// Create a new SnowIDConfigBuilder with default values
    pub fn new() -> Self {
        Self {
            node_bits: DEFAULT_NODE_BITS,
            custom_epoch: DEFAULT_CUSTOM_EPOCH,
            spin_enabled: DEFAULT_SPIN_ENABLED,
            spin_loops: DEFAULT_SPIN_LOOPS,
            spin_yield_every: DEFAULT_SPIN_YIELD_EVERY,
        }
    }

    /// Set the number of bits for node ID (6-16) in a fallible way.
    /// Sequence bits will be automatically set to (22 - node_bits)
    ///
    /// # Arguments
    /// * `bits` - Number of bits for node ID (6-16)
    ///
    /// # Returns
    /// * `Result<Self, SnowIDConfigError>` - Builder instance or validation error
    pub fn node_bits(mut self, bits: u8) -> Result<Self, SnowIDConfigError> {
        let true = (6..=16).contains(&bits) else {
            return Err(SnowIDConfigError::InvalidNodeBits { bits });
        };
        self.node_bits = bits;
        Ok(self)
    }

    /// Set a custom epoch timestamp in milliseconds
    ///
    /// # Arguments
    /// * `epoch` - Custom epoch timestamp in milliseconds since Unix epoch
    pub const fn epoch(mut self, epoch: u64) -> Self {
        self.custom_epoch = epoch;
        self
    }

    /// Enable or disable micro spin before sleep on overflow
    pub const fn enable_spin(mut self, enable: bool) -> Self {
        self.spin_enabled = enable;
        self
    }

    /// Set number of spin loops attempted before falling back to sleep
    /// A value of 0 disables spinning.
    pub const fn spin_loops(mut self, loops: u32) -> Self {
        self.spin_loops = loops;
        self
    }

    /// Set spin yield cadence. Yield every N spin iterations; 0 disables yielding.
    pub const fn spin_yield_every(mut self, n: u32) -> Self {
        self.spin_yield_every = n;
        self
    }

    /// Build the final SnowIDConfig
    ///
    /// # Returns
    /// * `SnowIDConfig` - The configured SnowIDConfig instance
    pub fn build(self) -> SnowIDConfig {
        let mut cfg = SnowIDConfig::new(self.node_bits, self.custom_epoch);
        cfg.spin_enabled = self.spin_enabled;
        cfg.spin_loops = self.spin_loops;
        cfg.spin_yield_every = self.spin_yield_every;
        cfg
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
                let config = SnowIDConfig::builder().node_bits(bits).unwrap().build();
                assert_eq!(config.node_bits(), bits);
                assert_eq!(
                    config.sequence_bits(),
                    SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - bits
                );
                assert_eq!(config.max_node_id(), SnowIDConfig::calculate_mask(bits));
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
    }

    #[test]
    fn test_custom_config() {
        let config = SnowIDConfig::builder()
            .node_bits(12)
            .unwrap()
            .epoch(1640995200000) // 2022-01-01
            .build();

        assert_eq!(config.node_bits(), 12);
        assert_eq!(config.sequence_bits(), 10); // 22 - 12
        assert_eq!(config.epoch(), 1640995200000);
    }

    #[test]
    fn test_default_config() {
        let config = SnowIDConfig::default();
        assert_eq!(config.node_bits(), DEFAULT_NODE_BITS);
        assert_eq!(
            config.sequence_bits(),
            SnowID::TOTAL_NODE_AND_SEQUENCE_BITS - DEFAULT_NODE_BITS
        );
        assert_eq!(config.epoch(), DEFAULT_CUSTOM_EPOCH);
        assert_eq!(config.spin_enabled(), DEFAULT_SPIN_ENABLED);
        assert_eq!(config.spin_loops(), DEFAULT_SPIN_LOOPS);
        assert_eq!(config.spin_yield_every(), DEFAULT_SPIN_YIELD_EVERY);
    }

    // Panicking builder has been removed; validation is error-based.

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
