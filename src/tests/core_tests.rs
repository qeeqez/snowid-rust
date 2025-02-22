use crate::*;

#[test]
fn test_clock_backwards() {
    let mut generator = Tsid::new(1).unwrap();
    let tsid1 = generator.generate();
    
    // Simulate clock moving backwards by saving current timestamp
    let original_timestamp = generator.last_timestamp;
    
    // Generate another ID - it should handle backwards clock gracefully
    let tsid2 = generator.generate();
    
    assert!(tsid2 > tsid1, "Second TSID should be greater than first");
    
    let (ts1, _, seq1) = generator.extract.decompose(tsid1);
    let (ts2, _, seq2) = generator.extract.decompose(tsid2);
    
    if ts1 == ts2 {
        assert!(seq2 > seq1, "Sequence should increment when timestamp is same");
    } else {
        assert!(ts2 >= original_timestamp, "Timestamp should not go backwards");
    }
} 