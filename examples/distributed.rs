use rand::{rng, Rng};
use snowid::SnowID;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // Create a thread-safe generator with Mutex for mutable access
    let generator = Arc::new(Mutex::new(SnowID::new(1).unwrap()));
    let mut handles = vec![];

    // Spawn multiple threads simulating distributed ID generation
    for thread_id in 0..4 {
        let generator_clone = Arc::clone(&generator);
        handles.push(thread::spawn(move || {
            let mut ids = HashSet::new();
            let mut rng = rng();

            // Generate some IDs with random delays
            for i in 0..5 {
                // Lock the generator to generate ID
                let id = {
                    let generator_lock = generator_clone.lock().unwrap();
                    generator_lock.generate()
                };

                // Extract components (doesn't need mutable access)
                let (ts, node, seq) = {
                    let generator_lock = generator_clone.lock().unwrap();
                    generator_lock.extract.decompose(id)
                };

                println!("Thread {thread_id} generated ID {i} (ts={ts}, node={node}, seq={seq})");

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
        assert!(ids[i] > ids[i - 1], "IDs not monotonically increasing!");
    }
    println!("All IDs are unique and monotonically increasing!");
}
