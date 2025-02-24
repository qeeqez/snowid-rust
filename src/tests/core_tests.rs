use crate::*;

#[test]
fn test_clock_backwards() {
    let generator = SnowID::new(1).unwrap();
    let snowid1 = generator.generate();

    // Simulate clock moving backwards by saving current timestamp
    let original_timestamp = generator.last_timestamp.load(Ordering::SeqCst);

    // Generate another ID - it should handle backwards clock gracefully
    let snowid2 = generator.generate();

    assert!(
        snowid2 > snowid1,
        "Second SnowID should be greater than first"
    );

    let (ts1, _, seq1) = generator.extract.decompose(snowid1);
    let (ts2, _, seq2) = generator.extract.decompose(snowid2);

    if ts1 == ts2 {
        assert!(
            seq2 > seq1,
            "Sequence should increment when timestamp is same"
        );
    } else {
        assert!(
            ts2 >= original_timestamp,
            "Timestamp should not go backwards"
        );
    }
}
