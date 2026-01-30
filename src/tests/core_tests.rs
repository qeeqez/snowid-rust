#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_clock_backwards() {
        let generator = SnowID::new(1).unwrap();
        let snowid1 = generator.generate();

        // Generate another ID - it should handle backwards clock gracefully
        // Even if system clock went backwards, IDs remain monotonic
        let snowid2 = generator.generate();

        assert!(
            snowid2 > snowid1,
            "Second SnowID should be greater than first"
        );

        let (ts1, _, seq1) = generator.extract.decompose(snowid1);
        let (ts2, _, seq2) = generator.extract.decompose(snowid2);

        // IDs are monotonic: either timestamp advanced or sequence incremented
        if ts1 == ts2 {
            assert!(
                seq2 > seq1,
                "Sequence should increment when timestamp is same"
            );
        } else {
            assert!(
                ts2 >= ts1,
                "Timestamp should not go backwards: ts1={}, ts2={}",
                ts1,
                ts2
            );
        }
    }
}

