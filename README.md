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

## 🧮 ID Structure

Default configuration:
```text
|------------------------------------------|------------|------------|
|           TIMESTAMP (42 bits)            | NODE (10)  |  SEQ (12)  |
|------------------------------------------|------------|------------|
```
- Timestamp: 42 bits = 139 years from 2024-01-01 (1704067200000)
- Node ID: 10 bits = 1,024 nodes (valid range: 6-16 bits)
- Sequence: 12 bits = 4,096 IDs/ms/node

## 🎯 Quick Start

```toml
[dependencies]
tsid-rust = "0.1.0"
```

```rust
use tsid_rust::Tsid;

// Create generator for node 1
let mut gen = Tsid::new(1).unwrap();

// Generate unique IDs
let id = gen.generate();

// Extract components
let (ts, node, seq) = gen.extract.decompose(id);
```

## 🔧 Configuration

```rust
let config = TsidConfig::builder()
    .node_bits(12)          // 4096 nodes (range: 6-16 bits)
    .custom_epoch(1704067200000) // Custom epoch
    .build();

let gen = Tsid::with_config(1, config).unwrap();
```

## 📊 Performance & Comparisons

### Social Media Platform Configurations

| Platform | Timestamp | Node Bits | Sequence Bits | Max Nodes | IDs/ms/node | Time/ID |
|----------|-----------|-----------|---------------|-----------|-------------|---------|
| Twitter | 41 | 10 | 12 | 1,024 | 4,096 | ~242ns |
| Instagram | 41 | 13 | 10 | 8,192 | 1,024 | ~1.94µs |
| Discord | 42 | 10 | 12 | 1,024 | 4,096 | ~245ns |

### Node vs Sequence Bits Trade-off
| Node Bits | Max Nodes | IDs/ms/node | Time/ID |
|-----------|-----------|-------------|---------|
| 6 | 64 | 65,536 | ~20ns |
| 8 | 256 | 16,384 | ~60ns |
| 10 | 1,024 | 4,096 | ~243ns |
| 12 | 4,096 | 1,024 | ~968ns |
| 14 | 16,384 | 256 | ~3.86µs |
| 16 | 65,536 | 64 | ~15.4µs |

Choose configuration based on your needs:
- More nodes → Increase node bits (max 16 bits = 65,536 nodes)
- More IDs per node → Increase sequence bits (min 6 node bits = 64 nodes)
- Total bits (node + sequence) is fixed at 22 bits

## 🚀 Examples

Check out [examples](examples/) for:
- Basic usage
- Custom configuration
- Distributed generation
- Performance benchmarks

## 📜 License

MIT - See [LICENSE](LICENSE) for details
