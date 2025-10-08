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
- 🎯 Small dependency footprint (`thiserror`, `base62` for encoding)

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
snowid = "0.2.1"
```

```rust
use snowid::SnowID;

fn main() {
    let gen = SnowID::new(1).unwrap();
    let id = gen.generate();
    println!("Generated ID: {}", id);
}
```

## 🔠 Base62 Encoded IDs

Generate base62 encoded IDs (using characters 0-9, a-z, A-Z) for more compact and URL-friendly identifiers:

```rust
use snowid::SnowID;

fn main() {
    // Create a generator
    let gen = SnowID::new(1).unwrap();

    // Generate a base62 encoded ID
    let encoded_id = gen.generate_base62();
    println!("Base62 ID: {}", encoded_id); // Example: "2qPfVQh7Jw9"

    // Generate with raw value
    let (encoded_id, raw_id) = gen.generate_base62_with_raw();
    println!("Base62: {}, Raw: {}", encoded_id, raw_id);

    // Decode a base62 ID back to u64
    let decoded = gen.decode_base62(&encoded_id).unwrap();
    assert_eq!(decoded, raw_id);

    // Extract components from a base62 ID
    let (timestamp, node, sequence) = gen.decompose_base62(&encoded_id).unwrap();
    println!("Timestamp: {}, Node: {}, Sequence: {}", timestamp, node, sequence);
}
```

### Benefits of Base62 IDs

- 🔤 More compact representation (11 chars max vs 20 digits for u64)
- 🔗 URL-friendly (no special characters)
- 👁️ Human-readable and easier to share
- 🔄 Fully compatible with original SnowID structure

## 🔧 Configuration

```rust
use snowid::{SnowID, SnowIDConfig};

fn main() {
    // Create custom configuration
    let config = SnowIDConfig::builder()
        .epoch(1577836800000) // 2020-01-01 00:00:00 UTC
        .node_bits(8).unwrap()         // Supports 255 nodes
        .build();

    // Create generator with custom config
    let gen = SnowID::with_config(1, config).unwrap();
}
```

### ℹ️ Available Methods

```rust
use snowid::SnowID;

fn main() {
    let gen = SnowID::new(1).unwrap();
    
    // Generate numeric IDs
    let id = gen.generate();
    
    // Generate Base62 encoded IDs
    let base62_id = gen.generate_base62();
    let (base62_id, raw_id) = gen.generate_base62_with_raw();
    
    // Decode Base62 IDs
    let decoded = gen.decode_base62(&base62_id).unwrap();
    let (ts, node, seq) = gen.decompose_base62(&base62_id).unwrap();

    // Extract individual components from numeric IDs
    let timestamp = gen.extract.timestamp(id);  // Get timestamp from ID
    let node = gen.extract.node(id);           // Get node ID from ID
    let sequence = gen.extract.sequence(id);    // Get sequence from ID

    // Extract all components at once
    let (ts, node, seq) = gen.extract.decompose(id);

    // Configuration information
    let max_node = gen.config.max_node_id();          // Get maximum allowed node ID
    let node_bits = gen.config.node_bits();           // Get number of bits used for node ID
    let max_seq = gen.config.max_sequence_id();    // Get maximum sequence per millisecond
    let timestamp_bits = SnowID::TIMESTAMP_BITS; // Get number of bits used for timestamp (42)
}
```

### ⏳ Tuning Overflow Wait (Spin/Yield)

When the per-millisecond sequence is exhausted, SnowID waits for the next millisecond. You can tune the short busy-wait (spin) before sleeping:

```rust
use snowid::{SnowID, SnowIDConfig};

fn main() {
    let config = SnowIDConfig::builder()
        .node_bits(10).unwrap()
        .enable_spin(true)   // default: true
        .spin_loops(64)      // default: 64 spin iterations before sleeping
        .spin_yield_every(16) // default: yield every 16 iterations (0 disables yielding)
        .build();

    let gen = SnowID::with_config(1, config).unwrap();
}
```

Notes:

- Set `enable_spin(false)` or `spin_loops(0)` to disable spinning entirely.
- Lower `spin_loops` can reduce CPU usage; higher values may reduce tail latency under overflow.

## 📊 Performance & Comparisons

### Social Media Platform Configurations

| Platform  | Timestamp | Node Bits | Sequence Bits | Max Nodes | IDs/ms/node | Time/ID |
|-----------|-----------|-----------|---------------|-----------|-------------|---------|
| Twitter   | 41        | 10        | 12            | 1,024     | 4,096       | ~242ns  |
| Instagram | 41        | 13        | 10            | 8,192     | 1,024       | ~1.94µs |
| Discord   | 42        | 10        | 12            | 1,024     | 4,096       | ~245ns  |

### Node vs Sequence Bits Trade-off

| Node Bits | Max Nodes | IDs/ms/node | Time/ID |
|-----------|-----------|-------------|---------|
| 6         | 64        | 65,536      | ~20ns   |
| 8         | 256       | 16,384      | ~60ns   |
| 10        | 1,024     | 4,096       | ~243ns  |
| 12        | 4,096     | 1,024       | ~968ns  |
| 14        | 16,384    | 256         | ~3.86µs |
| 16        | 65,536    | 64          | ~15.4µs |

Choose configuration based on your needs:

- More nodes → Increase node bits (max 16 bits = 65,536 nodes)
- More IDs per node → Increase sequence bits (min 6 node bits = 64 nodes)
- Total bits (node + sequence) is fixed at 22 bits

### Int64 vs Base62 Performance

| Variant               | Time/ID | Size         | Notes                    |
|-----------------------|---------|--------------|--------------------------|
| Int64                 | ~290 ns | 18-20 digits | Fastest option           |
| Base62                | ~295 ns | 10-11 chars  | ~2% slower, more compact |

Base62 encoding provides more compact, URL-friendly IDs with a small performance trade-off.

## 🚀 Examples

Check out [examples](examples/) for:

- Basic usage
- Custom configuration
- Base62 encoding and decoding
- Performance comparisons between Int64 and Base62
- Distributed generation
- Performance benchmarks

## 📜 License

MIT - See [LICENSE](LICENSE) for details
