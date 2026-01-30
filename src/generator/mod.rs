//! Core SnowID generator implementation
//!
//! Optimized for high performance with:
//! - Combined atomic state (timestamp + sequence in single AtomicU64)
//! - Precomputed shifts and masks
//! - Modular wait/backoff strategies

mod base62_methods;
mod state;
mod time;
mod wait;

use std::sync::atomic::{AtomicU64, Ordering};

use crate::config::SnowIDConfig;
use crate::error::SnowIDError;
use crate::extractor::SnowIDExtractor;

use state::State;
use time::time_since_epoch;
use wait::{next_backoff, sleep_until_next_ms, spin_wait, MAX_BACKOFF_MS};

/// Main ID generator with cache-line alignment
#[derive(Debug)]
#[repr(align(64))]
pub struct SnowID {
    // === Hot path fields ===
    state: AtomicU64,
    node_prefix: u64,
    max_seq: u16,
    ts_shift: u8,
    ts_mask: u64,
    epoch: u64,

    // === Cold path fields ===
    pub node_id: u16,
    pub config: SnowIDConfig,
    pub extract: SnowIDExtractor,
}

impl SnowID {
    pub const TIMESTAMP_BITS: u32 = 42;
    pub const TOTAL_NODE_AND_SEQUENCE_BITS: u8 = 22;

    /// Create with default configuration
    pub fn new(node_id: u16) -> Result<Self, SnowIDError> {
        Self::with_config(node_id, SnowIDConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(node_id: u16, config: SnowIDConfig) -> Result<Self, SnowIDError> {
        Self::validate_node_id(node_id, &config)?;
        Ok(Self::build(node_id, config))
    }

    /// Validate node_id against config limits
    fn validate_node_id(node_id: u16, config: &SnowIDConfig) -> Result<(), SnowIDError> {
        let max = config.max_node_id();
        if node_id > max {
            return Err(SnowIDError::InvalidNodeId { node_id, max });
        }
        Ok(())
    }

    /// Build generator from validated inputs
    fn build(node_id: u16, config: SnowIDConfig) -> Self {
        Self {
            state: AtomicU64::new(0),
            node_prefix: Self::compute_node_prefix(node_id, &config),
            max_seq: config.max_sequence_id(),
            ts_shift: config.timestamp_shift(),
            ts_mask: config.timestamp_mask(),
            epoch: config.epoch(),
            node_id,
            config,
            extract: SnowIDExtractor::new(config),
        }
    }

    /// Precompute node prefix for fast ID assembly
    #[inline(always)]
    fn compute_node_prefix(node_id: u16, config: &SnowIDConfig) -> u64 {
        (node_id as u64) << config.node_shift()
    }

    /// Get current time since epoch
    #[inline(always)]
    fn now_ms(&self) -> u64 {
        time_since_epoch(self.epoch)
    }

    #[inline(always)]
    pub(crate) fn get_time_since_epoch(&self) -> u64 {
        self.now_ms()
    }

    /// Generate a new SnowID
    #[inline]
    pub fn generate(&self) -> u64 {
        let now = self.now_ms();
        let current = State::from_raw(self.state.load(Ordering::Acquire));

        // Fast path 1: time advanced
        if now > current.timestamp() {
            if let Some(id) = self.try_claim_millisecond(current, now) {
                return id;
            }
            return self.generate_slow_path();
        }

        // Fast path 2: same millisecond, sequence available
        if let Some(id) = self.try_increment_sequence(current) {
            return id;
        }

        self.generate_slow_path()
    }

    /// Try to claim new millisecond with sequence 0
    #[inline]
    fn try_claim_millisecond(&self, current: State, new_ts: u64) -> Option<u64> {
        let new_state = State::new(new_ts, 0);
        self.cas_state(current, new_state)
            .then(|| self.assemble_id(new_ts, 0))
    }

    /// Try to increment sequence within current millisecond
    #[inline]
    fn try_increment_sequence(&self, current: State) -> Option<u64> {
        if current.sequence() >= self.max_seq {
            return None;
        }
        let new_seq = current.sequence() + 1;
        let new_state = State::new(current.timestamp(), new_seq);
        self.cas_state(current, new_state)
            .then(|| self.assemble_id(current.timestamp(), new_seq))
    }

    /// Atomic compare-and-swap on state
    #[inline(always)]
    fn cas_state(&self, expected: State, new: State) -> bool {
        self.state
            .compare_exchange_weak(expected.raw(), new.raw(), Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Slow path for contended generation
    #[cold]
    #[inline(never)]
    fn generate_slow_path(&self) -> u64 {
        let mut backoff_ms = 1u64;

        loop {
            let now = self.now_ms();
            let current = State::from_raw(self.state.load(Ordering::Acquire));

            // Time advanced
            if now > current.timestamp() {
                if let Some(id) = self.try_claim_millisecond(current, now) {
                    return id;
                }
                continue;
            }

            // Try sequence increment
            if let Some(id) = self.try_increment_sequence(current) {
                return id;
            }

            // Sequence exhausted - wait
            self.wait_next_millis(current.timestamp(), backoff_ms);
            backoff_ms = next_backoff(backoff_ms);
        }
    }

    /// Wait for next millisecond using spin + sleep strategy
    pub(crate) fn wait_next_millis(&self, from_ts: u64, backoff_ms: u64) -> u64 {
        // Try spin-wait first
        if let Some(new_ts) = spin_wait(from_ts, &self.config, || self.now_ms()) {
            return new_ts;
        }
        // Fall back to sleep
        sleep_until_next_ms(from_ts, backoff_ms, || self.now_ms())
    }

    /// Assemble final SnowID from components
    #[inline(always)]
    fn assemble_id(&self, timestamp: u64, sequence: u16) -> u64 {
        ((timestamp & self.ts_mask) << self.ts_shift) | self.node_prefix | (sequence as u64)
    }

    #[inline(always)]
    pub(crate) fn create_snowid_with_node(&self, ts: u64, node: u16, seq: u16) -> u64 {
        ((ts & self.config.timestamp_mask()) << self.config.timestamp_shift())
            | ((node as u64) << self.config.node_shift())
            | (seq as u64)
    }
}
