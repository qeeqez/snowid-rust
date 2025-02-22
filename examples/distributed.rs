use rand::{rng, Rng};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tsid_rust::Tsid;

fn main() {
    // Create a thread-safe generator with Mutex for mutable access
    let generator = Arc::new(Mutex::new(Tsid::new(1).unwrap()));
    let mut handles = vec![];

    // Spawn multiple threads simulating distributed ID generation
    for thread_id in 0..4 {
        let gen = Arc::clone(&generator);
        handles.push(thread::spawn(move || {
            let mut ids = HashSet::new();
            let mut rng = rng();

            // Generate some IDs with random delays
            for i in 0..5 {
                // Lock the generator to generate ID
                let id = {
                    let mut gen = gen.lock().unwrap();
                    gen.generate()
                };
                
                // Extract components (doesn't need mutable access)
                let (ts, node, seq) = {
                    let gen = gen.lock().unwrap();
                    gen.extract.decompose(id)
                };
                
                println!(
                    "Thread {} generated ID {} (ts={}, node={}, seq={})", 
                    thread_id, i, ts, node, seq
                );
                
                // Verify ID uniqueness
                assert!(ids.insert(id), "Duplicate ID generated!");
                
                // Random delay to simulate work
                let delay = rng.random_range(0..=9);
                thread::sleep(Duration::from_millis(delay));
            }
            ids
        }));
    }

    // Collect all generated IDs
    let mut all_ids = HashSet::new();
    for handle in handles {
        let thread_ids = handle.join().unwrap();
        all_ids.extend(thread_ids);
    }

    // Verify total number of unique IDs
    println!("\nTotal unique IDs generated: {}", all_ids.len());
    
    // Verify monotonic ordering
    let mut ids: Vec<_> = all_ids.into_iter().collect();
    ids.sort_unstable();
    for i in 1..ids.len() {
        assert!(ids[i] > ids[i-1], "IDs not monotonically increasing!");
    }
    println!("All IDs are unique and monotonically increasing!");
} 