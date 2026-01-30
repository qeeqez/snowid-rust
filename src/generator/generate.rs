//! ID generation logic
//!
//! Core generate() implementation with fast and slow paths

use std::sync::atomic::Ordering;

use super::state::State;
use super::wait::next_backoff;
use super::SnowID;

impl SnowID {
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
    pub(crate) fn try_claim_millisecond(&self, current: State, new_ts: u64) -> Option<u64> {
        let new_state = State::new(new_ts, 0);
        self.cas_state(current, new_state)
            .then(|| self.assemble_id(new_ts, 0))
    }

    /// Try to increment sequence within current millisecond
    #[inline]
    pub(crate) fn try_increment_sequence(&self, current: State) -> Option<u64> {
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
    pub(crate) fn cas_state(&self, expected: State, new: State) -> bool {
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

            if now > current.timestamp() {
                if let Some(id) = self.try_claim_millisecond(current, now) {
                    return id;
                }
                continue;
            }

            if let Some(id) = self.try_increment_sequence(current) {
                return id;
            }

            self.wait_next_millis(current.timestamp(), backoff_ms);
            backoff_ms = next_backoff(backoff_ms);
        }
    }
}
