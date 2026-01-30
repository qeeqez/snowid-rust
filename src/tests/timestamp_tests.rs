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

    // ============== NEW COMPREHENSIVE TESTS ==============

    #[test]
    fn test_same_millisecond_generation() {
        // Test rapid generation within the same millisecond
        let generator = SnowID::new(1).unwrap();

        // Generate multiple IDs as fast as possible
        let ids: Vec<u64> = (0..100).map(|_| generator.generate()).collect();

        // All IDs should be unique
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "All IDs should be unique");

        // IDs should be monotonically increasing
        for i in 1..ids.len() {
            assert!(
                ids[i] > ids[i - 1],
                "IDs should be monotonically increasing"
            );
        }

        // Check that same-ms IDs have incrementing sequences
        let first_ts = generator.extract.timestamp(ids[0]);
        let same_ms_ids: Vec<_> = ids
            .iter()
            .filter(|id| generator.extract.timestamp(**id) == first_ts)
            .collect();

        if same_ms_ids.len() > 1 {
            // Verify sequences are incrementing for same-ms IDs
            let mut prev_seq = generator.extract.sequence(*same_ms_ids[0]);
            for id in same_ms_ids.iter().skip(1) {
                let seq = generator.extract.sequence(**id);
                assert!(
                    seq > prev_seq,
                    "Sequence should increment within same millisecond"
                );
                prev_seq = seq;
            }
        }
    }

    #[test]
    fn test_different_node_bits_configs() {
        // Test with different node_bits configurations
        for node_bits in [6u8, 8, 10, 12, 14, 16] {
            let config = SnowIDConfig::builder().node_bits(node_bits).unwrap().build();
            let max_node = config.max_node_id();
            let generator = SnowID::with_config(0, config).unwrap();

            // Generate IDs and verify timestamp accuracy
            let before = wall_clock_ms(generator.config.epoch());
            let id = generator.generate();
            let after = wall_clock_ms(generator.config.epoch());

            let ts = generator.extract.timestamp(id);

            assert!(
                ts >= before && ts <= after + 1,
                "node_bits={}: timestamp {} should be between {} and {}",
                node_bits,
                ts,
                before,
                after
            );

            // Verify node ID extraction
            let (_, node, _) = generator.extract.decompose(id);
            assert_eq!(node, 0, "Node ID should be 0");

            // Test max node ID
            let max_gen = SnowID::with_config(max_node, config);
            assert!(max_gen.is_ok(), "Should allow max node ID {}", max_node);
        }
    }

    #[test]
    fn test_custom_epoch_timestamp() {
        // Test with custom epoch (2020-01-01)
        let custom_epoch = 1577836800000u64;
        let config = SnowIDConfig::builder().epoch(custom_epoch).build();
        let generator = SnowID::with_config(1, config).unwrap();

        let id = generator.generate();
        let ts = generator.extract.timestamp(id);

        // Calculate expected timestamp range
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let expected_ts = now_ms - custom_epoch;

        let diff = if ts > expected_ts {
            ts - expected_ts
        } else {
            expected_ts - ts
        };

        assert!(
            diff <= 10,
            "Custom epoch timestamp drift: ts={}, expected={}, diff={}ms",
            ts,
            expected_ts,
            diff
        );
    }

    #[test]
    fn test_various_sleep_intervals() {
        let generator = SnowID::new(1).unwrap();

        // Test with various sleep intervals
        for sleep_ms in [1, 5, 10, 25, 50] {
            let id1 = generator.generate();
            let ts1 = generator.extract.timestamp(id1);

            thread::sleep(Duration::from_millis(sleep_ms));

            let id2 = generator.generate();
            let ts2 = generator.extract.timestamp(id2);

            // Verify timestamp advanced by approximately sleep_ms
            let diff = ts2 - ts1;
            let min_expected = (sleep_ms as f64 * 0.8) as u64;
            let max_expected = (sleep_ms as f64 * 1.5) as u64 + 5; // Allow some jitter

            assert!(
                diff >= min_expected && diff <= max_expected,
                "Sleep {}ms: expected diff in [{}, {}], got {}",
                sleep_ms,
                min_expected,
                max_expected,
                diff
            );
        }
    }

    #[test]
    fn test_mixed_sleep_and_burst() {
        let generator = SnowID::new(1).unwrap();
        let mut all_ids: Vec<u64> = Vec::new();

        // Burst -> Sleep -> Burst -> Sleep pattern
        for round in 0..3 {
            // Burst: generate 50 IDs rapidly
            for _ in 0..50 {
                all_ids.push(generator.generate());
            }

            if round < 2 {
                thread::sleep(Duration::from_millis(10));
            }
        }

        // All IDs should be unique and monotonic
        for i in 1..all_ids.len() {
            assert!(
                all_ids[i] > all_ids[i - 1],
                "IDs should be monotonically increasing at position {}",
                i
            );
        }

        // Verify we have some timestamp diversity (crossed millisecond boundaries)
        let timestamps: Vec<_> = all_ids
            .iter()
            .map(|id| generator.extract.timestamp(*id))
            .collect();
        let unique_ts: std::collections::HashSet<_> = timestamps.iter().collect();

        assert!(
            unique_ts.len() > 1,
            "Should have multiple unique timestamps, got {}",
            unique_ts.len()
        );
    }
}

