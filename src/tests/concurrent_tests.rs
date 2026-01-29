#[cfg(test)]
mod tests {
    use crate::tests::test_utils::assert_unique_and_monotonic;
    use crate::*;
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_concurrent_generation() {
        let generator = Arc::new(Mutex::new(SnowID::new(1).unwrap()));
        let mut handles = vec![];
        let num_threads = 4;
        let ids_per_thread = 250;

        // Generate IDs concurrently
        for _ in 0..num_threads {
            let generator_clone = Arc::clone(&generator);
            handles.push(thread::spawn(move || {
                (0..ids_per_thread)
                    .map(|_| {
                        let generator_lock = generator_clone.lock().unwrap();
                        generator_lock.generate()
                    })
                    .collect::<Vec<_>>()
            }));
        }

        // Collect all generated IDs
        let mut all_ids = Vec::with_capacity(num_threads * ids_per_thread);
        for handle in handles {
            all_ids.extend(handle.join().unwrap());
        }

        // Use shared utility for uniqueness and monotonicity checks
        assert_unique_and_monotonic(all_ids, num_threads * ids_per_thread);
    }

    #[test]
    fn test_rapid_generation() {
        let generator = SnowID::new(1).unwrap();
        let mut ids = HashSet::new();
        let iterations = 1000;

        // Generate IDs as fast as possible
        for _ in 0..iterations {
            let id = generator.generate();
            assert!(ids.insert(id), "Duplicate ID generated: {id}");
        }

        // Verify expected number of unique IDs
        assert_eq!(
            ids.len(),
            iterations,
            "Expected {} unique IDs, but got {}",
            iterations,
            ids.len()
        );
    }

    #[test]
    fn test_timestamp_monotonicity() {
        let generator = SnowID::new(1).unwrap();
        let mut last_timestamp: u64 = 0;

        for _ in 0..100 {
            let snowid = generator.generate();
            let (timestamp, _, _) = generator.extract.decompose(snowid);
            assert!(timestamp >= last_timestamp);
            last_timestamp = timestamp;

            // Add small delay to ensure timestamp changes
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn test_concurrent_generation_lockfree() {
        let generator = Arc::new(SnowID::new(7).unwrap());
        let num_threads = 8;
        let ids_per_thread = 500;
        let mut handles = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let generator_clone = Arc::clone(&generator);
            handles.push(thread::spawn(move || {
                let mut v = Vec::with_capacity(ids_per_thread);
                for _ in 0..ids_per_thread {
                    v.push(generator_clone.generate());
                }
                v
            }));
        }

        let mut all_ids = Vec::with_capacity(num_threads * ids_per_thread);
        for h in handles {
            all_ids.extend(h.join().expect("thread panicked"));
        }

        // Use shared utility for uniqueness and monotonicity checks
        assert_unique_and_monotonic(all_ids, num_threads * ids_per_thread);
    }
}

