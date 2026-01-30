//! SnowIDConfig builder for constructing configuration

use super::{SnowIDConfig, SnowIDConfigError};

/// Default configuration values
pub(super) const DEFAULT_NODE_BITS: u8 = 10;
pub(super) const DEFAULT_CUSTOM_EPOCH: u64 = 1704067200000; // January 1, 2024 UTC
pub(super) const DEFAULT_SPIN_ENABLED: bool = true;
pub(super) const DEFAULT_SPIN_LOOPS: u32 = 64;
pub(super) const DEFAULT_SPIN_YIELD_EVERY: u32 = 16;

/// Builder for SnowIDConfig
#[derive(Debug)]
pub struct SnowIDConfigBuilder {
    pub(super) node_bits: u8,
    pub(super) custom_epoch: u64,
    pub(super) spin_enabled: bool,
    pub(super) spin_loops: u32,
    pub(super) spin_yield_every: u32,
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

    /// Set the number of bits for node ID (6-16)
    /// Sequence bits will be automatically set to (22 - node_bits)
    pub fn node_bits(mut self, bits: u8) -> Result<Self, SnowIDConfigError> {
        if !(6..=16).contains(&bits) {
            return Err(SnowIDConfigError::InvalidNodeBits { bits });
        }
        self.node_bits = bits;
        Ok(self)
    }

    /// Set a custom epoch timestamp in milliseconds
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
    pub const fn spin_loops(mut self, loops: u32) -> Self {
        self.spin_loops = loops;
        self
    }

    /// Set spin yield cadence. Yield every N spin iterations; 0 disables yielding
    pub const fn spin_yield_every(mut self, n: u32) -> Self {
        self.spin_yield_every = n;
        self
    }

    /// Build the final SnowIDConfig
    pub fn build(self) -> SnowIDConfig {
        SnowIDConfig::from_builder(self)
    }
}

impl Default for SnowIDConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
