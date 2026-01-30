//! # SnowID
//!
//! A Rust implementation of a Snowflake-like ID generator with 42-bit timestamp.
//!
//! Generate 64-bit unique identifiers that are:
//! - ⚡️ Fast (~325ns per ID)
//! - 📈 Time-sorted
//! - 🔄 Monotonic
//! - 🔒 Thread-safe
//! - 🌐 Distributed-ready

#![forbid(unsafe_code)]

pub mod base62;
mod config;
mod error;
mod extractor;
mod generator;

#[cfg(test)]
pub mod tests;

// Re-export main types
pub use config::SnowIDConfig;
pub use error::SnowIDError;
pub use extractor::SnowIDExtractor;
pub use generator::SnowID;

// Re-export base62 types at crate root for backward compatibility
pub use base62::DecodeError as Base62DecodeError;
pub use base62::MAX_LEN as BASE62_MAX_LEN;
pub use base62::{decode as base62_decode, encode as base62_encode};
pub use base62::{encode_array as base62_encode_array, encode_into as base62_encode_into};
