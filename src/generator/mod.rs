//! Core SnowID generator implementation
//!
//! Optimized for high performance with:
//! - Combined atomic state (timestamp + sequence in single AtomicU64)
//! - Precomputed shifts and masks
//! - Monotonic time caching with recalibration

mod base62_methods;

use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::config::SnowIDConfig;
use crate::error::SnowIDError;
use crate::extractor::SnowIDExtractor;

/// Combined state: upper 48 bits = timestamp, lower 16 bits = sequence
#[derive(Clone, Copy)]
struct State(u64);

impl State {
    const SEQ_BITS: u32 = 16;
    const SEQ_MASK: u64 = (1 << Self::SEQ_BITS) - 1;

    #[inline(always)]
    const fn new(timestamp: u64, sequence: u16) -> Self {
        Self((timestamp << Self::SEQ_BITS) | (sequence as u64))
    }

    #[inline(always)]
    const fn timestamp(self) -> u64 {
        self.0 >> Self::SEQ_BITS
    }

    #[inline(always)]
    const fn sequence(self) -> u16 {
        (self.0 & Self::SEQ_MASK) as u16
    }

    #[inline(always)]
    const fn raw(self) -> u64 {
        self.0
    }
}

/// Main ID generator with cache-line alignment
#[derive(Debug)]
#[repr(align(64))]
pub struct SnowID {
    // === Hot path fields ===
    /// Combined state
    state: AtomicU64,

    /// Precomputed: (node_id << node_shift)
    node_prefix: u64,

    /// Max sequence before overflow
    max_seq: u16,

    /// Precomputed shifts and masks
    ts_shift: u8,
    ts_mask: u64,

    // === Time tracking ===
    /// Epoch offset for this generator
    epoch: u64,

    // === Cold path fields ===
    pub node_id: u16,
    pub config: SnowIDConfig,
    pub extract: SnowIDExtractor,
}

impl SnowID {
    pub const TIMESTAMP_BITS: u32 = 42;
    pub const TOTAL_NODE_AND_SEQUENCE_BITS: u8 = 22;
    const MAX_BACKOFF_MS: u64 = 100;

    pub fn new(node_id: u16) -> Result<Self, SnowIDError> {
        Self::with_config(node_id, SnowIDConfig::default())
    }

    pub fn with_config(node_id: u16, config: SnowIDConfig) -> Result<Self, SnowIDError> {
        let max_node_id = config.max_node_id();
        if node_id > max_node_id {
            return Err(SnowIDError::InvalidNodeId {
                node_id,
                max: max_node_id,
            });
        }

        Ok(Self {
            state: AtomicU64::new(0),
            node_prefix: (node_id as u64) << config.node_shift(),
            max_seq: config.max_sequence_id(),
            ts_shift: config.timestamp_shift(),
            ts_mask: config.timestamp_mask(),
            epoch: config.epoch(),
            node_id,
            config,
            extract: SnowIDExtractor::new(config),
        })
    }

    #[inline(always)]
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before Unix epoch!")
            .as_millis() as u64
            - self.epoch
    }

    #[inline(always)]
    pub(crate) fn get_time_since_epoch(&self) -> u64 {
        self.now_ms()
    }

    /// Generate a new SnowID
    #[inline]
    pub fn generate(&self) -> u64 {
        let now = self.now_ms();
        let current = State(self.state.load(Ordering::Acquire));

        // Fast path 1: time advanced - claim new millisecond
        if now > current.timestamp() {
            let new_state = State::new(now, 0);
            if self
                .state
                .compare_exchange_weak(
                    current.raw(),
                    new_state.raw(),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return self.create_snowid_fast(now, 0);
            }
            return self.generate_slow_path();
        }

        // Fast path 2: same millisecond - increment sequence
        if current.sequence() < self.max_seq {
            let new_state = State::new(current.timestamp(), current.sequence() + 1);
            if self
                .state
                .compare_exchange_weak(
                    current.raw(),
                    new_state.raw(),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return self.create_snowid_fast(current.timestamp(), current.sequence() + 1);
            }
        }

        self.generate_slow_path()
    }

    #[cold]
    #[inline(never)]
    fn generate_slow_path(&self) -> u64 {
        let mut backoff_ms = 1u64;

        loop {
            let now = self.now_ms();
            let current = State(self.state.load(Ordering::Acquire));

            // Time advanced - claim new millisecond
            if now > current.timestamp() {
                let new_state = State::new(now, 0);
                if self
                    .state
                    .compare_exchange_weak(
                        current.raw(),
                        new_state.raw(),
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .is_ok()
                {
                    return self.create_snowid_fast(now, 0);
                }
                continue;
            }

            // Same or earlier millisecond with valid sequence
            let ts = current.timestamp().max(now);
            if current.sequence() < self.max_seq {
                let new_state = State::new(ts, current.sequence() + 1);
                if self
                    .state
                    .compare_exchange_weak(
                        current.raw(),
                        new_state.raw(),
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .is_ok()
                {
                    return self.create_snowid_fast(ts, current.sequence() + 1);
                }
                continue;
            }

            // Sequence exhausted - wait for next ms
            self.wait_next_millis(current.timestamp(), backoff_ms);
            backoff_ms = (backoff_ms.saturating_mul(2)).min(Self::MAX_BACKOFF_MS);
        }
    }

    pub(crate) fn wait_next_millis(&self, from_timestamp: u64, mut backoff_ms: u64) -> u64 {
        loop {
            if self.config.spin_enabled() && self.config.spin_loops() > 0 {
                let yield_every = self.config.spin_yield_every();
                for i in 0..self.config.spin_loops() {
                    let new_ts = self.now_ms();
                    if new_ts > from_timestamp {
                        return new_ts;
                    }
                    std::hint::spin_loop();
                    if yield_every != 0 && i % yield_every == yield_every - 1 {
                        thread::yield_now();
                    }
                }
            }

            thread::sleep(Duration::from_millis(backoff_ms));
            let new_ts = self.now_ms();
            if new_ts > from_timestamp {
                return new_ts;
            }
            backoff_ms = backoff_ms.saturating_mul(2).min(Self::MAX_BACKOFF_MS);
        }
    }

    #[inline(always)]
    fn create_snowid_fast(&self, timestamp: u64, sequence: u16) -> u64 {
        ((timestamp & self.ts_mask) << self.ts_shift) | self.node_prefix | (sequence as u64)
    }

    #[inline(always)]
    pub(crate) fn create_snowid_with_node(
        &self,
        timestamp: u64,
        node_id: u16,
        sequence: u16,
    ) -> u64 {
        ((timestamp & self.config.timestamp_mask()) << self.config.timestamp_shift())
            | ((node_id as u64) << self.config.node_shift())
            | (sequence as u64)
    }
}
