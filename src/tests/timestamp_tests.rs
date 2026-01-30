//! Timestamp accuracy and behavior tests

#[cfg(test)]
mod tests {
    use crate::tests::test_utils::{
        assert_ids_monotonic, assert_timestamp_accurate, assert_unique_ids, wall_clock_ms,
    };
    use crate::*;
    use std::collections::HashSet;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_timestamp_reflects_wall_clock() {
        let generator = SnowID::new(1).unwrap();
        let id = generator.generate();
        let ts = generator.extract.timestamp(id);
        assert_timestamp_accurate(ts, generator.config.epoch(), 10);
    }

    #[test]
    fn test_timestamp_advances_with_real_sleep() {
        let generator = SnowID::new(1).unwrap();

        let id1 = generator.generate();
        let ts1 = generator.extract.timestamp(id1);

        thread::sleep(Duration::from_millis(100));

        let id2 = generator.generate();
        let ts2 = generator.extract.timestamp(id2);

        let diff = ts2 - ts1;
        assert!(
            diff >= 80 && diff <= 150,
            "Expected ~100ms diff, got {}ms",
            diff
        );
    }

    #[test]
    fn test_timestamps_across_generator_restart() {
        let gen1 = SnowID::new(1).unwrap();
        let id1 = gen1.generate();
        let ts1 = gen1.extract.timestamp(id1);

        thread::sleep(Duration::from_millis(50));

        // Simulate restart with new generator
        let gen2 = SnowID::new(1).unwrap();
        let id2 = gen2.generate();
        let ts2 = gen2.extract.timestamp(id2);

        assert!(ts2 > ts1, "New generator should have later timestamp");
        assert!(ts2 - ts1 >= 40, "Expected ~50ms diff, got {}ms", ts2 - ts1);
    }

    #[test]
    fn test_timestamp_accuracy_under_load() {
        let generator = SnowID::new(1).unwrap();
        let epoch = generator.config.epoch();
        let mut max_drift: i64 = 0;

        for _ in 0..1000 {
            let before = wall_clock_ms(epoch);
            let id = generator.generate();
            let after = wall_clock_ms(epoch);
            let ts = generator.extract.timestamp(id);

            if ts < before {
                max_drift = max_drift.max((before - ts) as i64);
            }
            if ts > after {
                max_drift = max_drift.max((ts - after) as i64);
            }
        }

        assert!(max_drift <= 5, "Max drift {}ms (should be <=5ms)", max_drift);
    }

    #[test]
    fn test_multiple_generators_same_time() {
        let gens: Vec<_> = (0..5).map(|i| SnowID::new(i).unwrap()).collect();
        let timestamps: Vec<_> = gens
            .iter()
            .map(|g| g.extract.timestamp(g.generate()))
            .collect();

        let min_ts = *timestamps.iter().min().unwrap();
        let max_ts = *timestamps.iter().max().unwrap();
        assert!(max_ts - min_ts <= 10, "Timestamps spread too wide");
    }

    #[test]
    fn test_ids_sortable_by_time() {
        let generator = SnowID::new(1).unwrap();
        let ids: Vec<u64> = (0..10)
            .map(|i| {
                if i % 3 == 0 && i > 0 {
                    thread::sleep(Duration::from_millis(5));
                }
                generator.generate()
            })
            .collect();

        assert_ids_monotonic(&ids);
    }

    #[test]
    fn test_same_millisecond_generation() {
        let generator = SnowID::new(1).unwrap();
        let ids: Vec<u64> = (0..100).map(|_| generator.generate()).collect();

        assert_unique_ids(&ids, 100);
        assert_ids_monotonic(&ids);

        // Verify sequence increments within same millisecond
        let first_ts = generator.extract.timestamp(ids[0]);
        let same_ms: Vec<_> = ids
            .iter()
            .filter(|id| generator.extract.timestamp(**id) == first_ts)
            .collect();

        if same_ms.len() > 1 {
            let mut prev_seq = generator.extract.sequence(*same_ms[0]);
            for id in same_ms.iter().skip(1) {
                let seq = generator.extract.sequence(**id);
                assert!(seq > prev_seq, "Sequence should increment");
                prev_seq = seq;
            }
        }
    }

    #[test]
    fn test_different_node_bits_configs() {
        for node_bits in [6u8, 8, 10, 12, 14, 16] {
            let config = SnowIDConfig::builder()
                .node_bits(node_bits)
                .unwrap()
                .build();
            let generator = SnowID::with_config(0, config).unwrap();

            let id = generator.generate();
            let ts = generator.extract.timestamp(id);
            assert_timestamp_accurate(ts, generator.config.epoch(), 10);

            let (_, node, _) = generator.extract.decompose(id);
            assert_eq!(node, 0);
        }
    }

    #[test]
    fn test_custom_epoch_timestamp() {
        let custom_epoch = 1577836800000u64; // 2020-01-01
        let config = SnowIDConfig::builder().epoch(custom_epoch).build();
        let generator = SnowID::with_config(1, config).unwrap();

        let id = generator.generate();
        let ts = generator.extract.timestamp(id);
        assert_timestamp_accurate(ts, custom_epoch, 10);
    }

    #[test]
    fn test_various_sleep_intervals() {
        let generator = SnowID::new(1).unwrap();

        for sleep_ms in [1u64, 5, 10, 25, 50] {
            let ts1 = generator.extract.timestamp(generator.generate());
            thread::sleep(Duration::from_millis(sleep_ms));
            let ts2 = generator.extract.timestamp(generator.generate());

            let diff = ts2 - ts1;
            let min = (sleep_ms as f64 * 0.8) as u64;
            let max = (sleep_ms as f64 * 1.5) as u64 + 5;
            assert!(
                diff >= min && diff <= max,
                "Sleep {}ms: got {}ms (expected {}-{})",
                sleep_ms,
                diff,
                min,
                max
            );
        }
    }

    #[test]
    fn test_mixed_sleep_and_burst() {
        let generator = SnowID::new(1).unwrap();
        let mut ids: Vec<u64> = Vec::new();

        for round in 0..3 {
            for _ in 0..50 {
                ids.push(generator.generate());
            }
            if round < 2 {
                thread::sleep(Duration::from_millis(10));
            }
        }

        assert_ids_monotonic(&ids);

        let timestamps: HashSet<_> = ids
            .iter()
            .map(|id| generator.extract.timestamp(*id))
            .collect();
        assert!(timestamps.len() > 1, "Should cross ms boundaries");
    }
}
