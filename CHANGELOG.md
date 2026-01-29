# Changelog

## [1.0.1](https://github.com/qeeqez/snowid-rust/compare/v1.0.0...v1.0.1) (2026-01-29)


### Bug Fixes

* update readme ([58fbcc4](https://github.com/qeeqez/snowid-rust/commit/58fbcc4afa3096511a5e2239c61fdf735e6a66c4))

## [1.0.0](https://github.com/qeeqez/snowid-rust/compare/v0.3.0...v1.0.0) (2026-01-29)

### ⚠ BREAKING CHANGES

*   **deps:** Removed `thiserror` dependency to reduce binary size and compile times. Error types now implement `std::error::Error` directly.
*   **api:** Base62 encoding now encourages zero-allocation patterns.

### Features

*   **base62:** Added zero-allocation APIs `base62_encode_array` and `base62_encode_into` for high-performance encoding without heap allocation.
*   **core:** Integrated `coarsetime` for ~20x faster time queries (hybrid monotonic/wall-clock approach).
*   **concurrency:** Improved thread-safety and performance using optimized lock-free patterns for high-contention scenarios.
*   **config:** Enhanced spin-wait configuration for finer control over latency during sequence overflow.
*   **modernization:** Updated code to use Rust 2024 edition features.

### Performance Improvements

*   **optimization:** Significant reduction in generation latency (sub-350ns/op) via hot-path optimizations.
*   **memory:** Elimination of heap allocations in core generation paths.

### Bug Fixes

*   **ci:** Fixed formatting issues and streamlined CI workflows.
*   **deps:** Updated `chrono`, `base62`, and `criterion` to latest stable versions.

### Miscellaneous

*   **ci:** Migrated to `release-please` for automated semantic versioning and changelog generation.
*   **docs:** Comprehensive documentation updates including new zero-allocation examples and benchmark results.
