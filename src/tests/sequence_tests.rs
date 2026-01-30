//! Sequence rollover and uniqueness tests

#[cfg(test)]
mod tests {
    use crate::tests::test_utils::{assert_ids_monotonic, assert_unique_ids};
    use crate::*;
    use std::collections::HashSet;
    use std::thread;

    #[test]
    fn test_sequence_rollover() {
        let generator = SnowID::new(1).unwrap();
        let initial_id = generator.generate();
        let initial_ts = generator.extract.timestamp(initial_id);
        let mut max_seq = 0;

        for i in 0..10000 {
            let id = generator.generate();
            let (ts, _, seq) = generator.extract.decompose(id);
            max_seq = max_seq.max(seq);

            if ts == initial_ts && seq < max_seq {
                assert!(max_seq <= generator.config.max_sequence_id());
                return;
            } else if ts != initial_ts && i > 0 {
                assert!(seq <= 1, "Sequence should reset on ts change");
                return;
            }

            if i % 100 == 0 {
                thread::yield_now();
            }
        }
        panic!("Sequence did not rollover in 10000 iterations");
    }

    #[test]
    fn test_sequence_overflow_handling() {
        let generator = SnowID::new(1).unwrap();
        let mut last_ts = 0;

        for _ in 0..100000 {
            let id = generator.generate();
            let (ts, _, seq) = generator.extract.decompose(id);

            if ts == last_ts && last_ts > 0 {
                if seq >= generator.config.max_sequence_id() {
                    let next = generator.generate();
                    let (next_ts, _, next_seq) = generator.extract.decompose(next);
                    assert!(next_ts > ts, "Timestamp should advance on overflow");
                    assert_eq!(next_seq, 0, "Sequence should reset");
                    return;
                }
            }

            last_ts = ts;
            if seq % 10 == 0 {
                thread::yield_now();
            }
        }
        panic!("Sequence overflow not handled");
    }

    #[test]
    fn test_sequence_monotonicity() {
        let generator = SnowID::new(1).unwrap();
        let ids: Vec<u64> = (0..1000).map(|_| generator.generate()).collect();
        assert_ids_monotonic(&ids);
    }

    #[test]
    fn test_10k_unique_ids() {
        const COUNT: usize = 10_000;
        let generator = SnowID::new(1).unwrap();
        let ids: Vec<u64> = (0..COUNT).map(|_| generator.generate()).collect();

        assert_unique_ids(&ids, COUNT);
        assert_ids_monotonic(&ids);

        // Analyze timestamp distribution
        let timestamps: HashSet<_> = ids
            .iter()
            .map(|id| generator.extract.timestamp(*id))
            .collect();
        assert!(timestamps.len() >= 1, "Should have at least one timestamp");
    }
}
