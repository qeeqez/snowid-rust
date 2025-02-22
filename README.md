# 🆔 SnowID Rust

[![Crates.io](https://img.shields.io/crates/v/snowid-rust.svg)](https://crates.io/crates/snowid-rust)
[![Documentation](https://docs.rs/snowid-rust/badge.svg)](https://docs.rs/snowid-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> A Rust implementation of a Snowflake-like ID generator with 42-bit timestamp.

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
snowid-rust = "0.1.0"
```

```rust
use snowid::SnowID;

fn main() {
    let mut gen = SnowID::new(1).unwrap();
    let id = gen.generate();
    println!("Generated ID: {}", id);
}
```

## 🔧 Configuration

```rust
let config = SnowIDConfig::builder()
    .custom_epoch(1577836800000) // 2020-01-01 00:00:00 UTC
    .node_bits(10)
    .sequence_bits(12)
    .build();

let gen = SnowID::with_config(1, config).unwrap();
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
