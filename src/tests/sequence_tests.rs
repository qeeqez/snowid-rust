use std::thread;
use std::collections::HashSet;
use crate::*;
use std::time::Instant;

#[test]
fn test_sequence_rollover() {
    let mut generator = SnowID::new(1).unwrap();
    let initial_timestamp = generator.generate();
    let initial_ts = generator.extract.timestamp(initial_timestamp);
    let mut max_sequence_seen = 0;
    
    // Generate IDs until we see the sequence reset
    for i in 0..10000 {
        let showid = generator.generate();
        let (ts, _, seq) = generator.extract.decompose(showid);
        
        // Track highest sequence number seen
        max_sequence_seen = max_sequence_seen.max(seq);
        
        // If we're still in the same millisecond
        if ts == initial_ts {
            // If we've seen a sequence reset within the same millisecond
            if seq < max_sequence_seen {
                assert!(max_sequence_seen > 0, "Should have seen some sequence increment");
                assert!(max_sequence_seen <= generator.max_sequence(), 
                    "Sequence {} exceeded maximum {}", max_sequence_seen, generator.max_sequence());
                return; // Test passed - we found a sequence rollover
            }
        } else if i > 0 {
            // If timestamp changed and we've generated at least one ID
            assert_eq!(seq, 0, "Sequence should reset to 0 on timestamp change");
            return; // Test passed - sequence reset on timestamp change
        }
        
        // Generate IDs as fast as possible to stay within same millisecond
        if i % 100 == 0 {
            thread::yield_now(); // Yield to other threads but don't sleep
        }
    }
    
    panic!("Sequence did not rollover as expected within 5000 iterations");
}

#[test]
fn test_sequence_overflow_handling() {
    let mut generator = SnowID::new(1).unwrap();
    let mut last_ts = None;
    let mut last_sequence = None;
    let mut overflow_handled = false;
    
    // Generate IDs rapidly to force sequence overflow
    for _ in 0..10000 {
        let snowid = generator.generate();
        let (ts, _, sequence) = generator.extract.decompose(snowid);
        
        if let (Some(prev_ts), Some(prev_seq)) = (last_ts, last_sequence) {
            if ts == prev_ts {
                // Within same millisecond, sequence should increment
                assert!(sequence > prev_seq, 
                    "Sequence should increment within same millisecond"
                );
            } else {
                // On timestamp change, check if it was due to sequence overflow
                if prev_seq >= generator.max_sequence() {
                    assert!(ts > prev_ts, "Timestamp should advance on sequence overflow");
                    assert_eq!(sequence, 0, "Sequence should reset to 0 on overflow");
                    overflow_handled = true;
                    break;
                }
            }
        }
        
        last_ts = Some(ts);
        last_sequence = Some(sequence);
        
        // Generate IDs as fast as possible
        if sequence % 10 == 0 {
            thread::yield_now();
        }
    }
    
    assert!(overflow_handled, "Sequence overflow was not handled by advancing to next millisecond");
}

#[test]
fn test_sequence_restart() {
    let mut generator = SnowID::new(1).unwrap();
    let mut last_timestamp = None;
    let mut last_sequence = None;

    // Generate IDs rapidly to force sequence increment
    for _ in 0..100 {
        let snowid = generator.generate();
        let (timestamp, _, sequence) = generator.extract.decompose(snowid);
        
        if let (Some(last_seq), Some(last_ts)) = (last_sequence, last_timestamp) {
            if timestamp == last_ts {
                // Within same millisecond, sequence should increment
                assert!(sequence > last_seq, 
                    "Sequence should increment within same millisecond"
                );
            } else {
                // On timestamp change, sequence should restart
                assert_eq!(sequence, 0, 
                    "Sequence should restart from 0 on timestamp change"
                );
            }
        }
        
        last_sequence = Some(sequence);
        last_timestamp = Some(timestamp);
    }
}

#[test]
fn test_sequence_monotonicity() {
    let mut generator = SnowID::new(1).unwrap();
    let mut last_id = None;
    
    // Generate IDs rapidly to test monotonicity
    for _ in 0..1000 {
        let id = generator.generate();
        
        if let Some(last) = last_id {
            assert!(id > last, "IDs should be monotonically increasing");
        }
        
        last_id = Some(id);
    }
}

#[test]
fn test_10k_unique_ids() {
    const NUM_IDS: usize = 10_000;
    let mut generator = SnowID::new(1).unwrap();
    let mut ids = HashSet::with_capacity(NUM_IDS);
    let mut last_id = None;
    let mut duplicates = Vec::new();
    
    let start = Instant::now();
    
    // Generate 1 million IDs
    for i in 0..NUM_IDS {
        let id = generator.generate();
        
        // Check monotonicity
        if let Some(last) = last_id {
            assert!(id > last, "ID {} is not greater than previous ID {}", id, last);
        }
        last_id = Some(id);
        
        // Check uniqueness
        if !ids.insert(id) {
            duplicates.push((i, id));
        }
        
        // Print progress every 100k IDs
        if (i + 1) % 100_000 == 0 {
            println!("Generated {}k IDs in {:?}", (i + 1) / 1000, start.elapsed());
        }
    }
    
    let elapsed = start.elapsed();
    let ids_per_sec = NUM_IDS as f64 / elapsed.as_secs_f64();
    
    println!("\nResults:");
    println!("Total time: {:?}", elapsed);
    println!("IDs per second: {:.0}", ids_per_sec);
    println!("Unique IDs: {}/{}", ids.len(), NUM_IDS);
    
    // If we found any duplicates, print details and fail
    if !duplicates.is_empty() {
        println!("\nFound {} duplicate IDs:", duplicates.len());
        for (i, id) in duplicates.iter().take(5) {
            let (ts, node, seq) = generator.extract.decompose(*id);
            println!("Duplicate at position {}: ID {} (ts={}, node={}, seq={})", 
                i, id, ts, node, seq);
        }
        if duplicates.len() > 5 {
            println!("... and {} more duplicates", duplicates.len() - 5);
        }
        panic!("Found duplicate IDs");
    }
    
    // Analyze timestamp distribution
    let mut timestamps = ids.iter()
        .map(|id| generator.extract.timestamp(*id))
        .collect::<Vec<_>>();
    timestamps.sort_unstable();
    
    let total_time_span = timestamps.last().unwrap() - timestamps.first().unwrap();
    let unique_timestamps = timestamps.iter().collect::<HashSet<_>>().len();
    
    println!("\nTimestamp Analysis:");
    println!("Time span: {}ms", total_time_span);
    println!("Unique timestamps: {}", unique_timestamps);
    println!("Average IDs per timestamp: {:.1}", 
        NUM_IDS as f64 / unique_timestamps as f64);
}
