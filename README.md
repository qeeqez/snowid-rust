# ❄️ SnowID Rust

[![Crates.io](https://img.shields.io/crates/v/snowid.svg)](https://crates.io/crates/snowid)
[![Documentation](https://docs.rs/snowid/badge.svg)](https://docs.rs/snowid)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> A Rust implementation of a Snowflake-like ID generator with 42-bit timestamp.

**Generate 64-bit unique identifiers that are:**
- ⚡️ Fast (~244ns per ID)
- 📈 Time-sorted
- 🔄 Monotonic
- 🔒 Thread-safe
- 🌐 Distributed-ready
- 🎯 Zero dependencies

## 🧮 ID Structure

**Example ID**: 151819733950271234

**Default configuration:**
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
snowid = "0.1.2"
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
use snowid::{SnowID, SnowIDConfig};

fn main() {
    // Create custom configuration
    let config = SnowIDConfig::builder()
        .epoch(1577836800000) // 2020-01-01 00:00:00 UTC
        .node_bits(8)         // Supports 255 nodes
        .build();

    // Create generator with custom config
    let mut gen = SnowID::with_config(1, config).unwrap();
}
```

### ℹ️ Available Methods
```rust
use snowid::SnowID;

fn main() {
    let mut gen = SnowID::new(1).unwrap();
    let id = gen.generate();

    // Extract individual components
    let timestamp = gen.extract.timestamp(id);  // Get timestamp from ID
    let node = gen.extract.node(id);           // Get node ID from ID
    let sequence = gen.extract.sequence(id);    // Get sequence from ID

    // Extract all components at once
    let (ts, node, seq) = gen.extract.decompose(id);

    // Configuration information
    let max_node = gen.max_node_id();          // Get maximum allowed node ID
    let node_bits = gen.node_bits();           // Get number of bits used for node ID
    let max_seq = gen.config.max_sequence_id();    // Get maximum sequence per millisecond
    let timestamp_bits = SnowID::TIMESTAMP_BITS; // Get number of bits used for timestamp (42)
}
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
