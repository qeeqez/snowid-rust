#[cfg(test)]
mod tests {
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
            let gen = Arc::clone(&generator);
            handles.push(thread::spawn(move || {
                (0..ids_per_thread)
                    .map(|_| {
                        let mut gen = gen.lock().unwrap();
                        gen.generate()
                    })
                    .collect::<Vec<_>>()
            }));
        }

        // Collect all generated IDs
        let mut all_ids = HashSet::new();
        for handle in handles {
            let ids = handle.join().unwrap();
            all_ids.extend(ids);
        }

        // Verify no duplicates were generated
        assert_eq!(
            all_ids.len(),
            num_threads * ids_per_thread,
            "Expected {} unique IDs, but got {}",
            num_threads * ids_per_thread,
            all_ids.len()
        );

        // Verify all IDs are monotonically increasing
        let mut ids: Vec<_> = all_ids.into_iter().collect();
        ids.sort_unstable();
        for i in 1..ids.len() {
            assert!(
                ids[i] > ids[i - 1],
                "ID at position {} ({}) is not greater than previous ID ({})",
                i,
                ids[i],
                ids[i - 1]
            );
        }
    }

    #[test]
    fn test_rapid_generation() {
        let mut generator = SnowID::new(1).unwrap();
        let mut ids = HashSet::new();
        let iterations = 1000;

        // Generate IDs as fast as possible
        for _ in 0..iterations {
            let id = generator.generate();
            assert!(ids.insert(id), "Duplicate ID generated: {}", id);
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
        let mut generator = SnowID::new(1).unwrap();
        let mut last_timestamp = 0;

        for _ in 0..100 {
            let snowid = generator.generate();
            let (timestamp, _, _) = generator.extract.decompose(snowid);
            assert!(timestamp >= last_timestamp);
            last_timestamp = timestamp;

            // Add small delay to ensure timestamp changes
            thread::sleep(Duration::from_millis(1));
        }
    }
}
