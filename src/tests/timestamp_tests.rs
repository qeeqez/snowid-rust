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
        let g = SnowID::new(1).unwrap();
        let ts = g.extract.timestamp(g.generate());
        assert_timestamp_accurate(ts, g.config.epoch(), 10);
    }

    #[test]
    fn test_timestamp_advances_with_real_sleep() {
        let g = SnowID::new(1).unwrap();
        let ts1 = g.extract.timestamp(g.generate());
        thread::sleep(Duration::from_millis(100));
        let ts2 = g.extract.timestamp(g.generate());

        let diff = ts2 - ts1;
        assert!(diff >= 80 && diff <= 150, "Expected ~100ms, got {}ms", diff);
    }

    #[test]
    fn test_timestamps_across_generator_restart() {
        let g1 = SnowID::new(1).unwrap();
        let ts1 = g1.extract.timestamp(g1.generate());
        thread::sleep(Duration::from_millis(50));

        let g2 = SnowID::new(1).unwrap();
        let ts2 = g2.extract.timestamp(g2.generate());

        assert!(ts2 > ts1 && ts2 - ts1 >= 40, "Expected ~50ms diff");
    }

    #[test]
    fn test_timestamp_accuracy_under_load() {
        let g = SnowID::new(1).unwrap();
        let epoch = g.config.epoch();
        let mut max_drift: i64 = 0;

        for _ in 0..1000 {
            let before = wall_clock_ms(epoch);
            let ts = g.extract.timestamp(g.generate());
            let after = wall_clock_ms(epoch);

            if ts < before {
                max_drift = max_drift.max((before - ts) as i64);
            }
            if ts > after {
                max_drift = max_drift.max((ts - after) as i64);
            }
        }
        assert!(max_drift <= 5, "Max drift {}ms", max_drift);
    }

    #[test]
    fn test_multiple_generators_same_time() {
        let gens: Vec<_> = (0..5).map(|i| SnowID::new(i).unwrap()).collect();
        let tss: Vec<_> = gens
            .iter()
            .map(|g| g.extract.timestamp(g.generate()))
            .collect();

        let (min, max) = (*tss.iter().min().unwrap(), *tss.iter().max().unwrap());
        assert!(max - min <= 10, "Timestamps spread too wide");
    }

    #[test]
    fn test_ids_sortable_by_time() {
        let g = SnowID::new(1).unwrap();
        let ids: Vec<u64> = (0..10)
            .map(|i| {
                if i % 3 == 0 && i > 0 {
                    thread::sleep(Duration::from_millis(5));
                }
                g.generate()
            })
            .collect();
        assert_ids_monotonic(&ids);
    }

    #[test]
    fn test_same_millisecond_generation() {
        let g = SnowID::new(1).unwrap();
        let ids: Vec<u64> = (0..100).map(|_| g.generate()).collect();
        assert_unique_ids(&ids, 100);
        assert_ids_monotonic(&ids);
    }

    #[test]
    fn test_different_node_bits_configs() {
        for node_bits in [6u8, 10, 14, 16] {
            let cfg = SnowIDConfig::builder()
                .node_bits(node_bits)
                .unwrap()
                .build();
            let g = SnowID::with_config(0, cfg).unwrap();
            assert_timestamp_accurate(g.extract.timestamp(g.generate()), g.config.epoch(), 10);
        }
    }

    #[test]
    fn test_custom_epoch_timestamp() {
        let epoch = 1577836800000u64;
        let cfg = SnowIDConfig::builder().epoch(epoch).build();
        let g = SnowID::with_config(1, cfg).unwrap();
        assert_timestamp_accurate(g.extract.timestamp(g.generate()), epoch, 10);
    }

    #[test]
    fn test_various_sleep_intervals() {
        let g = SnowID::new(1).unwrap();
        for ms in [1u64, 5, 10, 25] {
            let ts1 = g.extract.timestamp(g.generate());
            thread::sleep(Duration::from_millis(ms));
            let ts2 = g.extract.timestamp(g.generate());

            let diff = ts2 - ts1;
            assert!(
                diff >= (ms * 7 / 10) && diff <= ms * 2 + 5,
                "Sleep {}ms: got {}ms",
                ms,
                diff
            );
        }
    }

    #[test]
    fn test_mixed_sleep_and_burst() {
        let g = SnowID::new(1).unwrap();
        let mut ids: Vec<u64> = Vec::new();

        for round in 0..3 {
            for _ in 0..50 {
                ids.push(g.generate());
            }
            if round < 2 {
                thread::sleep(Duration::from_millis(10));
            }
        }

        assert_ids_monotonic(&ids);
        let tss: HashSet<_> = ids.iter().map(|id| g.extract.timestamp(*id)).collect();
        assert!(tss.len() > 1, "Should cross ms boundaries");
    }
}
