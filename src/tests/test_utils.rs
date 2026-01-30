//! Shared test utilities for SnowID tests

use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current wall-clock time in ms since custom epoch
pub fn wall_clock_ms(epoch: u64) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before Unix epoch!");
    now.as_millis() as u64 - epoch
}

/// Assert that all IDs in the collection are unique
pub fn assert_unique_ids(ids: &[u64], expected_count: usize) {
    let set: HashSet<_> = ids.iter().copied().collect();
    assert_eq!(
        set.len(),
        expected_count,
        "Expected {} unique IDs, but got {} (duplicates detected)",
        expected_count,
        set.len()
    );
}

/// Assert that IDs are strictly monotonically increasing (in order)
pub fn assert_ids_monotonic(ids: &[u64]) {
    for i in 1..ids.len() {
        assert!(
            ids[i] > ids[i - 1],
            "ID at position {} ({}) should be > previous ({})",
            i,
            ids[i],
            ids[i - 1]
        );
    }
}

/// Assert that IDs are monotonically increasing when sorted
pub fn assert_monotonic_sorted(ids: &mut [u64]) {
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

/// Assert collection has expected unique count and is monotonically increasing
pub fn assert_unique_and_monotonic(mut ids: Vec<u64>, expected_count: usize) {
    assert_unique_ids(&ids, expected_count);
    assert_monotonic_sorted(&mut ids);
}

/// Assert timestamp is accurate within tolerance (ms)
pub fn assert_timestamp_accurate(ts: u64, epoch: u64, tolerance_ms: u64) {
    let wall_ts = wall_clock_ms(epoch);
    let diff = if wall_ts >= ts {
        wall_ts - ts
    } else {
        ts - wall_ts
    };
    assert!(
        diff <= tolerance_ms,
        "Timestamp drift: ts={}, wall={}, diff={}ms (max {}ms)",
        ts,
        wall_ts,
        diff,
        tolerance_ms
    );
}
