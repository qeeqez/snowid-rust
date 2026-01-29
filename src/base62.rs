//! Base62 encoding and decoding for SnowID values
//!
//! Provides zero-allocation variants for hot paths:
//! - `encode_array`: Returns [u8; 11] + length
//! - `encode_into`: Writes to caller buffer
//! - `encode`: Convenience String wrapper

use std::error::Error;
use std::fmt;

/// Maximum size needed for u64 in base62 encoding (11 bytes)
pub const MAX_LEN: usize = 11;

/// Zero-allocation base62 encoding to a fixed-size array
/// Returns the array and the actual length of encoded bytes
#[inline]
pub fn encode_array(id: u64) -> ([u8; MAX_LEN], usize) {
    let mut buf = [0u8; MAX_LEN];
    let len = base62::encode_bytes(id, &mut buf).unwrap();
    (buf, len)
}

/// Zero-allocation base62 encoding into caller-provided buffer
/// Returns a str slice of the encoded portion
#[inline]
pub fn encode_into(id: u64, buf: &mut [u8; MAX_LEN]) -> &str {
    let len = base62::encode_bytes(id, buf).unwrap();
    // base62 output is always valid ASCII
    std::str::from_utf8(&buf[..len]).unwrap()
}

/// Base62 encode with String allocation (convenience wrapper)
/// For hot paths, prefer `encode_array` or `encode_into`
#[inline]
pub fn encode(id: u64) -> String {
    let (buf, len) = encode_array(id);
    // base62 output is always valid ASCII
    std::str::from_utf8(&buf[..len]).unwrap().to_owned()
}

/// Decode a base62 string to a u64, handling potential overflow
pub fn decode(encoded: &str) -> Result<u64, DecodeError> {
    let decoded = base62::decode(encoded).map_err(DecodeError::from)?;

    // Check if the decoded value fits in a u64
    let Ok(value) = u64::try_from(decoded) else {
        return Err(DecodeError::Overflow);
    };

    Ok(value)
}

/// Error type for base62 decoding operations
#[derive(Debug)]
pub enum DecodeError {
    /// Invalid character in base62 string
    InvalidCharacter,
    /// Decoded value would overflow u64
    Overflow,
    /// Other decoding error from base62 crate
    Other(base62::DecodeError),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InvalidCharacter => write!(f, "Invalid base62 character"),
            DecodeError::Overflow => write!(f, "Decoded value would overflow u64"),
            DecodeError::Other(e) => write!(f, "Base62 decode error: {}", e),
        }
    }
}

impl Error for DecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DecodeError::Other(e) => Some(e),
            _ => None,
        }
    }
}

impl From<base62::DecodeError> for DecodeError {
    fn from(err: base62::DecodeError) -> Self {
        DecodeError::Other(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let values = [1u64, 1000, u64::MAX / 2, u64::MAX];
        for &value in &values {
            let encoded = encode(value);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(value, decoded);
        }
    }

    #[test]
    fn test_encode_array_matches_string() {
        let id = 12345678901234u64;
        let string_encoded = encode(id);
        let (arr, len) = encode_array(id);
        let array_str = std::str::from_utf8(&arr[..len]).unwrap();
        assert_eq!(string_encoded, array_str);
    }

    #[test]
    fn test_encode_into_matches_string() {
        let id = 98765432109876u64;
        let string_encoded = encode(id);
        let mut buf = [0u8; MAX_LEN];
        let into_str = encode_into(id, &mut buf);
        assert_eq!(string_encoded, into_str);
    }

    #[test]
    fn test_max_len() {
        // u64::MAX should fit in 11 characters
        let (_, len) = encode_array(u64::MAX);
        assert!(len <= MAX_LEN);
    }

    #[test]
    fn test_decode_error() {
        // Invalid characters
        assert!(decode("!!!").is_err());
    }
}
