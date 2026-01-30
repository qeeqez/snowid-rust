//! Combined atomic state for timestamp + sequence
//!
//! Packs 48-bit timestamp and 16-bit sequence into single u64 for lock-free CAS

/// Combined state: upper 48 bits = timestamp, lower 16 bits = sequence
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State(u64);

impl State {
    /// Number of bits used for sequence
    pub const SEQ_BITS: u32 = 16;

    /// Mask to extract sequence from raw value
    pub const SEQ_MASK: u64 = (1 << Self::SEQ_BITS) - 1;

    /// Create new state from timestamp and sequence
    #[inline(always)]
    pub const fn new(timestamp: u64, sequence: u16) -> Self {
        Self((timestamp << Self::SEQ_BITS) | (sequence as u64))
    }

    /// Extract timestamp from state
    #[inline(always)]
    pub const fn timestamp(self) -> u64 {
        self.0 >> Self::SEQ_BITS
    }

    /// Extract sequence from state
    #[inline(always)]
    pub const fn sequence(self) -> u16 {
        (self.0 & Self::SEQ_MASK) as u16
    }

    /// Get raw u64 value for atomic operations
    #[inline(always)]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Create state from raw u64 value
    #[inline(always)]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_roundtrip() {
        let ts = 123456789u64;
        let seq = 4095u16;
        let state = State::new(ts, seq);

        assert_eq!(state.timestamp(), ts);
        assert_eq!(state.sequence(), seq);
    }

    #[test]
    fn test_state_max_values() {
        let max_ts = (1u64 << 48) - 1;
        let max_seq = u16::MAX;
        let state = State::new(max_ts, max_seq);

        assert_eq!(state.timestamp(), max_ts);
        assert_eq!(state.sequence(), max_seq);
    }

    #[test]
    fn test_state_zero() {
        let state = State::new(0, 0);
        assert_eq!(state.raw(), 0);
        assert_eq!(state.timestamp(), 0);
        assert_eq!(state.sequence(), 0);
    }
}
