use std::thread;
use std::time::Duration;
use crate::*;

#[test]
fn test_sequence_rollover() {
    let generator = TsidGenerator::new(1).unwrap();
    let mut last_sequence = None;
    
    // Generate IDs until sequence rolls over
    for _ in 0..1025 {
        let tsid = generator.generate().unwrap();
        let (_, _, sequence) = generator.extract_from_tsid(tsid);
        
        // Sequence should never exceed max
        assert!(sequence <= generator.max_sequence(), 
            "Sequence {} exceeded maximum {}", sequence, generator.max_sequence());
        
        // If we have a last sequence and current is less, we've rolled over
        if let Some(last) = last_sequence {
            if sequence < last {
                return; // Test passed - we found a rollover
            }
        }
        
        last_sequence = Some(sequence);
    }
    
    panic!("Sequence did not roll over as expected");
}

#[test]
fn test_sequence_overflow_handling() {
    let generator = TsidGenerator::new(1).unwrap();
    let mut last_sequence = None;
    let mut sequence_rollovers = 0;
    let mut last_timestamp = None;
    
    // Generate enough IDs to force multiple sequence rollovers
    for _ in 0..5000 {
        let tsid = generator.generate().unwrap();
        let (timestamp, _, sequence) = generator.extract_from_tsid(tsid);
        
        // Check for sequence rollover within the same timestamp
        if let (Some(last_seq), Some(last_ts)) = (last_sequence, last_timestamp) {
            if timestamp == last_ts && sequence < last_seq {
                sequence_rollovers += 1;
            }
        }
        
        last_sequence = Some(sequence);
        last_timestamp = Some(timestamp);
        
        // Add a small delay occasionally to avoid overwhelming the system
        if sequence_rollovers == 0 && sequence % 100 == 0 {
            thread::sleep(Duration::from_micros(1));
        }
    }
    
    // We should have seen at least one sequence rollover
    assert!(sequence_rollovers > 0, 
        "No sequence rollovers detected after 5000 generations"
    );
}

#[test]
fn test_sequence_restart() {
    let generator = TsidGenerator::new(1).unwrap();
    
    // Generate multiple IDs in the same millisecond
    let tsid1 = generator.generate().unwrap();
    let tsid2 = generator.generate().unwrap();
    let tsid3 = generator.generate().unwrap();
    
    let (ts1, _, seq1) = generator.extract_from_tsid(tsid1);
    let (ts2, _, seq2) = generator.extract_from_tsid(tsid2);
    let (ts3, _, seq3) = generator.extract_from_tsid(tsid3);
    
    // If timestamps are the same, sequences should increment
    if ts1 == ts2 {
        assert!(seq2 > seq1, "Sequence should increment within same millisecond");
    }
    if ts2 == ts3 {
        assert!(seq3 > seq2, "Sequence should increment within same millisecond");
    }
}

#[test]
fn test_sequence_restart_on_overflow() {
    let generator = TsidGenerator::new(1).unwrap();
    let mut last_sequence = None;
    let mut last_timestamp = None;
    let mut sequence_restarts = 0;
    
    // Generate IDs until we see sequence restarts
    for _ in 0..5000 {
        let tsid = generator.generate().unwrap();
        let (timestamp, _, sequence) = generator.extract_from_tsid(tsid);
        
        if let (Some(last_seq), Some(last_ts)) = (last_sequence, last_timestamp) {
            if timestamp > last_ts {
                // If timestamp changed, sequence should restart from 0
                assert_eq!(sequence, 0, 
                    "Sequence did not restart from 0 on timestamp change"
                );
                sequence_restarts += 1;
            } else if sequence < last_seq {
                // If timestamp is same but sequence decreased, we've rolled over
                sequence_restarts += 1;
            }
        }
        
        last_sequence = Some(sequence);
        last_timestamp = Some(timestamp);
        
        // Add a small delay occasionally to force timestamp changes
        if sequence_restarts == 0 && sequence % 100 == 0 {
            thread::sleep(Duration::from_millis(1));
        }
    }
    
    // We should have seen some sequence restarts
    assert!(sequence_restarts > 0, 
        "No sequence restarts detected after 5000 generations"
    );
}
