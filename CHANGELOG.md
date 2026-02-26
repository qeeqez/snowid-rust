# Changelog

## [2.0.0](https://github.com/qeeqez/snowid-rust/compare/v1.0.1...v2.0.0) (2026-01-30)


### ⚠ BREAKING CHANGES

* **perf:** optimize generator with combined atomic state and precomputed fields
* 1.0.0
* removed thiserror dependency

### release

* 1.0.0 ([fec917c](https://github.com/qeeqez/snowid-rust/commit/fec917c29455a25626a789c33e21bb042d410921))


### Features

* **perf:** optimize generate() with inline timestamp advancement ([693bbc2](https://github.com/qeeqez/snowid-rust/commit/693bbc2a0e5e4715bd04045aa71469b4d609d8d1))
* **perf:** optimize generator with combined atomic state and precomputed fields ([f081941](https://github.com/qeeqez/snowid-rust/commit/f081941eb0822805a783cfd7c8542537e3b29155))
* performance optimizations and zero-allocation base62 API ([f4090fc](https://github.com/qeeqez/snowid-rust/commit/f4090fc5905c0f385bda6042d98da9649fd24e9b))


### Bug Fixes

* replace absurd u16::MAX comparison with config max value ([6d91c8b](https://github.com/qeeqez/snowid-rust/commit/6d91c8b47fc6afb9f5a3f2b5dd1ba79c3c921019))
* revert coarsetime optimization to fix timestamp accuracy ([0179c89](https://github.com/qeeqez/snowid-rust/commit/0179c8919eeeec33baa549ada77c7d0d0611055f))
* silence false positive dead_code warnings for test-used methods ([766ec68](https://github.com/qeeqez/snowid-rust/commit/766ec6844a899b46ddfb578b747b2966f0f112f7))
* update readme ([58fbcc4](https://github.com/qeeqez/snowid-rust/commit/58fbcc4afa3096511a5e2239c61fdf735e6a66c4))


### Performance Improvements

* optimize ID generation with advanced techniques and rust 2024 features (~5-10% faster) ([23dcc2f](https://github.com/qeeqez/snowid-rust/commit/23dcc2fea9aa34fbe1bfc21d2bc9e4ca11c61b35))

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
