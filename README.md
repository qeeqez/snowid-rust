# TSID Rust

A Rust implementation of Time-Sorted Unique Identifiers (TSID). TSIDs are 64-bit unique identifiers that are sorted by generation time, making them ideal for distributed systems and databases.

## Features

- Thread-safe, lock-free ID generation
- Configurable bit allocation for timestamp, node ID, and sequence
- Monotonic ordering of IDs within and across nodes
- Protection against clock drift and sequence overflow
- Fast generation: ~244ns per ID in single-threaded mode
- Efficient concurrent generation with multiple threads
- Custom epoch support
- Zero external dependencies

## Default Configuration

- 42 bits for timestamp (supports ~139 years with ms precision)
- 10 bits for node ID (supports up to 1,024 nodes)
- 12 bits for sequence (up to 4,096 IDs per ms per node)
- Default epoch: January 1, 2024 UTC

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
tsid-rust = "0.1.0"
```

### Basic Usage

```rust
use tsid_rust::TsidGenerator;

// Create a generator with node ID 1
let generator = TsidGenerator::new(1);

// Generate a new TSID
let tsid = generator.generate();

// Extract all components at once
let (timestamp, node, sequence) = generator.extract_from_tsid(tsid);
println!("All components: ts={}, node={}, seq={}", timestamp, node, sequence);

// Or extract components individually
let ts = generator.extract_timestamp(tsid);
let node = generator.extract_node(tsid);
let seq = generator.extract_sequence(tsid);
println!("Individual components: ts={}, node={}, seq={}", ts, node, seq);
```

### Custom Configuration

```rust
use tsid_rust::{TsidGenerator, TsidConfig};

// Create a custom configuration using the builder
let config = TsidConfig::builder()
    .node_bits(12)          // Support up to 4096 nodes (sequence bits will be 10)
    .custom_epoch(1704067200000) // January 1, 2024 UTC
    .build();

// Create a generator with custom config
let generator = TsidGenerator::with_config(1, config);

// Or just customize node bits
let config = TsidConfig::builder()
    .node_bits(16)  // Support up to 65536 nodes (sequence bits will be 6)
    .build();       // Use default epoch

// Get maximum values for the configuration
println!("Max node ID: {}", generator.max_node_id());
println!("Max sequence per ms: {}", generator.max_sequence());
```

### Thread-Safe Usage

```rust
use std::sync::Arc;
use std::thread;
use tsid_rust::TsidGenerator;

// Create a shared generator
let generator = Arc::new(TsidGenerator::new(1));

// Spawn multiple threads
let mut handles = vec![];
for _ in 0..4 {
    let gen = Arc::clone(&generator);
    handles.push(thread::spawn(move || {
        for _ in 0..1000 {
            let id = gen.generate();
            // Use the ID...
        }
    }));
}

// Wait for all threads to finish
for handle in handles {
    handle.join().unwrap();
}
```

## Performance

Benchmark results on a typical machine:

- Single thread ID generation: ~244ns per ID
- Sequential generation: ~24µs for 100 IDs
- Component extraction: ~836ps
- Concurrent generation (8 threads): ~66µs for 800 IDs

## Implementation Details

The TSID is a 64-bit integer composed of:

```text
|------------------------------------------|------------|------------|
|              TIMESTAMP                    |   NODE     |  SEQUENCE  |
|------------------------------------------|------------|------------|
```

- Timestamp: Milliseconds since custom epoch
- Node ID: Unique identifier for the generator instance
- Sequence: Counter that resets every millisecond

The implementation ensures:
- Monotonic ordering of IDs
- Thread-safe generation without locks
- Efficient handling of sequence overflow
- Protection against clock drift
- No external dependencies

## License

This project is licensed under the MIT License - see the LICENSE file for details.
