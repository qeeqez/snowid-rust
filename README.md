# 🆔 TSID Rust

[![Crates.io](https://img.shields.io/crates/v/tsid-rust.svg)](https://crates.io/crates/tsid-rust)
[![Documentation](https://docs.rs/tsid-rust/badge.svg)](https://docs.rs/tsid-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> 🚀 Lightning-fast, thread-safe, time-sorted unique ID generator for distributed systems

Generate 64-bit unique identifiers that are:
- ⚡️ Fast (~244ns per ID)
- 📈 Time-sorted
- 🔄 Monotonic
- 🔒 Thread-safe
- 🌐 Distributed-ready
- 🎯 Zero dependencies

## 🎯 Quick Start

```toml
[dependencies]
tsid-rust = "0.1.0"
```

```rust
use tsid_rust::Tsid;

// Create generator for node 1
let mut gen = Tsid::new(1)?;

// Generate unique IDs
let id = gen.generate();

// Extract components
let (ts, node, seq) = gen.extract.decompose(id);
```

## 🛠 Features

- **Time-sorted**: IDs sort chronologically by creation time
- **Configurable**: Customize bits for timestamp, node ID, and sequence
- **Clock-safe**: Handles clock drift and sequence overflow
- **Distributed**: Safe for multi-node and multi-thread use
- **Fast**: ~244ns per ID in single thread, scales well with concurrency

## 🔧 Configuration

Default setup provides:
- 42 bits timestamp (~139 years)
- 10 bits node ID (1,024 nodes)
- 12 bits sequence (4,096 IDs/ms/node)

Customize with builder:
```rust
let config = TsidConfig::builder()
    .node_bits(12)          // 4096 nodes
    .custom_epoch(1704067200000) // Custom epoch
    .build();

let gen = Tsid::with_config(1, config)?;
```

## 🧮 ID Structure

```text
|------------------------------------------|------------|------------|
|              TIMESTAMP                    |   NODE     |  SEQUENCE  |
|------------------------------------------|------------|------------|
```

## 🚀 Examples

Check out [examples](examples/) for:
- Basic usage
- Custom configuration
- Distributed generation
- Performance benchmarks

## 📊 Performance

| Operation | Time |
|-----------|------|
| Single ID | ~244ns |
| 100 IDs | ~24µs |
| Extract | ~836ps |
| 8 threads (800 IDs) | ~66µs |

## 📜 License

MIT - See [LICENSE](LICENSE) for details
