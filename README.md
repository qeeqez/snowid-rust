# 🆔 TSID Rust

[![Crates.io](https://img.shields.io/crates/v/tsid-rust.svg)](https://crates.io/crates/tsid-rust)
[![Documentation](https://docs.rs/tsid-rust/badge.svg)](https://docs.rs/tsid-rust)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-blue.svg?maxAge=3600)](https://github.com/qeeqez/tsid-rust)

> 🚀 High-performance Rust implementation of TSID (Time-Sorted Unique Identifier) - a distributed unique ID generation system inspired by Twitter's Snowflake.

## ✨ Features

- 🔢 **64-bit Integer IDs** - Efficient storage and sorting
- ⚡ **Ultra-Fast Generation** - Lock-free and thread-safe design
- 🎯 **Zero Dependencies** - No external crates required
- 🔄 **Monotonic Ordering** - Guaranteed time-based sorting
- 💻 **Distributed Ready** - Support for multiple nodes
- 🦀 **Pure Rust** - Safe, reliable, and optimized implementation

## 🏗️ TSID Structure

TSID is a 64-bit integer composed of:

```
|-- 42 bits timestamp --|-- 12 bits node --|-- 10 bits seq --|
```

### 📊 Bits Breakdown

| Component  | Bits | Description                    | Range                                    |
|------------|------|--------------------------------|------------------------------------------|
| Timestamp  | 42   | Milliseconds since custom epoch| ~139 years of unique timestamps         |
| Node ID    | 12   | Machine/shard identifier      | 4,096 unique nodes                      |
| Sequence   | 10   | Sequence number per ms        | 1,024 IDs per millisecond per node      |

## 🚀 Quick Start

Add to your `Cargo.toml`:
```toml
[dependencies]
tsid-rust = "0.1.0"
```

Basic usage:
```rust
// Coming soon...
```

## 🔍 How It Works

1. **Time-Based**: Uses 42 bits for millisecond precision timestamp
2. **Node-Aware**: Supports distributed generation with 12-bit node ID
3. **High-Throughput**: Can generate 1,024 unique IDs per millisecond per node
4. **Monotonic**: IDs within the same millisecond are guaranteed to be monotonically increasing

## 🌟 Benefits

- **Sortable**: IDs can be sorted by time, perfect for database indexing
- **Distributed**: Works across multiple nodes without coordination
- **Compact**: 64-bit integer format, more efficient than UUID
- **Predictable**: Fixed-length IDs with known characteristics

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by Twitter's Snowflake ID system
- Built with ❤️ using Rust
