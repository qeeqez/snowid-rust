# ❄️ SnowID

A collection of Snowflake-like ID generator implementations in multiple languages.

**Generate 64-bit unique identifiers that are:**
- ⚡️ Fast
- 📈 Time-sorted
- 🔄 Monotonic
- 🔒 Thread-safe
- 🌐 Distributed-ready

## Available Implementations

- [Rust](./rust/README.md) - Fast Rust implementation with zero dependencies
- Golang (Coming soon)
- JavaScript (Coming soon)

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

## 📜 License

MIT - See [LICENSE](LICENSE) for details
