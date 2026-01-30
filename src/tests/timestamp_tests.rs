#[cfg(test)]
mod tests {
    use crate::*;
    use std::thread;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    /// Get current wall-clock time in ms since custom epoch for comparison
    fn wall_clock_ms(epoch: u64) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before Unix epoch!");
        now.as_millis() as u64 - epoch
    }

    #[test]
    fn test_timestamp_reflects_wall_clock() {
        let generator = SnowID::new(1).unwrap();
        let epoch = generator.config.epoch();

        // Generate an ID
        let id = generator.generate();
        let ts_from_id = generator.extract.timestamp(id);

        // Get wall-clock time (allow small delay for test execution)
        let wall_ts = wall_clock_ms(epoch);

        // Timestamp should be very close to wall clock (within 10ms tolerance)
        let diff = if wall_ts >= ts_from_id {
            wall_ts - ts_from_id
        } else {
            ts_from_id - wall_ts
        };

        assert!(
            diff <= 10,
            "Timestamp drift too large: ID timestamp={}, wall clock={}, diff={}ms",
            ts_from_id,
            wall_ts,
            diff
        );
    }

    #[test]
    fn test_timestamp_advances_with_real_sleep() {
        let generator = SnowID::new(1).unwrap();

        // Generate first ID
        let id1 = generator.generate();
        let ts1 = generator.extract.timestamp(id1);

        // Sleep for 100ms
        thread::sleep(Duration::from_millis(100));

        // Generate second ID
        let id2 = generator.generate();
        let ts2 = generator.extract.timestamp(id2);

        // Timestamp difference should be approximately 100ms (allow 80-150ms tolerance)
        let diff = ts2 - ts1;
        assert!(
            diff >= 80 && diff <= 150,
            "Timestamp should advance ~100ms after sleep, but diff={}ms",
            diff
        );
    }

    #[test]
    fn test_timestamps_across_generator_restart() {
        // Create first generator
        let gen1 = SnowID::new(1).unwrap();
        let id1 = gen1.generate();
        let ts1 = gen1.extract.timestamp(id1);

        // Sleep to ensure time passes
        thread::sleep(Duration::from_millis(50));

        // Create NEW generator (simulates restart)
        let gen2 = SnowID::new(1).unwrap();
        let id2 = gen2.generate();
        let ts2 = gen2.extract.timestamp(id2);

        // Second generator's timestamp should be GREATER than first
        // This is the key property that was broken by coarsetime
        assert!(
            ts2 > ts1,
            "Timestamp from new generator should be greater than old: ts1={}, ts2={}",
            ts1,
            ts2
        );

        // And the difference should be at least 50ms
        assert!(
            ts2 - ts1 >= 40,
            "Timestamp difference should be at least ~50ms, got {}ms",
            ts2 - ts1
        );
    }

    #[test]
    fn test_timestamp_accuracy_under_load() {
        let generator = SnowID::new(1).unwrap();
        let epoch = generator.config.epoch();
        let mut max_drift: i64 = 0;

        // Generate many IDs and check timestamp accuracy
        for _ in 0..1000 {
            let before = wall_clock_ms(epoch);
            let id = generator.generate();
            let after = wall_clock_ms(epoch);

            let ts = generator.extract.timestamp(id);

            // Timestamp should be between before and after
            if ts < before {
                let drift = before as i64 - ts as i64;
                max_drift = max_drift.max(drift);
            }
            if ts > after {
                let drift = ts as i64 - after as i64;
                max_drift = max_drift.max(drift);
            }
        }

        assert!(
            max_drift <= 5,
            "Max timestamp drift under load: {}ms (should be <=5ms)",
            max_drift
        );
    }

    #[test]
    fn test_multiple_generators_same_time() {
        // Create multiple generators at the "same" time
        let gens: Vec<_> = (0..5).map(|i| SnowID::new(i).unwrap()).collect();

        // Generate IDs from each
        let ids: Vec<_> = gens.iter().map(|g| g.generate()).collect();
        let timestamps: Vec<_> = ids
            .iter()
            .zip(gens.iter())
            .map(|(id, g)| g.extract.timestamp(*id))
            .collect();

        // All timestamps should be within 10ms of each other
        let min_ts = *timestamps.iter().min().unwrap();
        let max_ts = *timestamps.iter().max().unwrap();

        assert!(
            max_ts - min_ts <= 10,
            "Timestamps from multiple generators should be within 10ms: min={}, max={}, diff={}",
            min_ts,
            max_ts,
            max_ts - min_ts
        );
    }

    #[test]
    fn test_ids_sortable_by_time() {
        let generator = SnowID::new(1).unwrap();
        let mut ids_and_times: Vec<(u64, Instant)> = Vec::new();

        // Generate IDs with small sleeps between some
        for i in 0..10 {
            if i % 3 == 0 && i > 0 {
                thread::sleep(Duration::from_millis(5));
            }
            ids_and_times.push((generator.generate(), Instant::now()));
        }

        // IDs should be strictly increasing (time-sorted)
        for i in 1..ids_and_times.len() {
            assert!(
                ids_and_times[i].0 > ids_and_times[i - 1].0,
                "IDs should be strictly increasing: id[{}]={} <= id[{}]={}",
                i - 1,
                ids_and_times[i - 1].0,
                i,
                ids_and_times[i].0
            );
        }
    }
}
