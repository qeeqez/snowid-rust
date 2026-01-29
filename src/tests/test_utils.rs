//! Shared test utilities for SnowID tests

use std::collections::HashSet;

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
