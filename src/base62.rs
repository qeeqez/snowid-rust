/// Base62 encoding and decoding for SnowID
///
/// This module provides optimized functionality to encode u64 SnowIDs to base62 strings
/// and decode base62 strings back to u64 SnowIDs using lookup tables.
use once_cell::sync::Lazy;

/// Character set for base62 encoding (0-9, A-Z, a-z)
const BASE62_CHARS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Lookup table for decoding base62 characters to their values
static DECODE_MAP: Lazy<[i8; 256]> = Lazy::new(|| {
    let mut map = [-1i8; 256];
    for (i, &c) in BASE62_CHARS.iter().enumerate() {
        map[c as usize] = i as i8;
    }
    map
});

/// Maximum length of a base62 encoded u64 (11 characters)
const MAX_BASE62_LEN: usize = 11;

/// Encode a u64 SnowID to a base62 string using lookup tables
///
/// # Arguments
/// * `id` - The u64 SnowID to encode
///
/// # Returns
/// * `String` - The base62 encoded string
pub fn encode(mut id: u64) -> String {
    if id == 0 {
        return "0".to_string();
    }

    // Pre-allocate buffer with maximum possible size
    let mut buffer = [0u8; MAX_BASE62_LEN];
    let mut position = MAX_BASE62_LEN;

    while id > 0 && position > 0 {
        position -= 1;
        let remainder = (id % 62) as usize;
        buffer[position] = BASE62_CHARS[remainder];
        id /= 62;
    }

    // Convert only the used portion of the buffer to a string
    String::from_utf8_lossy(&buffer[position..]).into_owned()
}

/// Decode a base62 string to a u64 SnowID using lookup tables
///
/// # Arguments
/// * `encoded` - The base62 encoded string
///
/// # Returns
/// * `Result<u64, Error>` - The decoded u64 SnowID or an error
pub fn decode(encoded: &str) -> Result<u64, DecodeError> {
    if encoded.is_empty() {
        return Err(DecodeError::EmptyString);
    }

    let mut result: u64 = 0;
    for &c in encoded.as_bytes() {
        let value = DECODE_MAP[c as usize];
        if value == -1 {
            return Err(DecodeError::InvalidCharacter(c as char));
        }

        // Check for potential overflow
        if let Some(new_result) = result.checked_mul(62) {
            result = new_result;
        } else {
            return Err(DecodeError::Overflow);
        }

        if let Some(new_result) = result.checked_add(value as u64) {
            result = new_result;
        } else {
            return Err(DecodeError::Overflow);
        }
    }

    Ok(result)
}

/// Errors that can occur during base62 decoding
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    /// The input string is empty
    #[error("Cannot decode an empty string")]
    EmptyString,

    /// The input string contains an invalid character
    #[error("Invalid base62 character: {0}")]
    InvalidCharacter(char),

    /// The decoded value would overflow a u64
    #[error("Decoded value would overflow u64")]
    Overflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let test_cases = [
            0u64,
            1,
            10,
            62,
            100,
            1000,
            1_000_000,
            u64::MAX / 2,
            u64::MAX,
        ];

        for &id in &test_cases {
            let encoded = encode(id);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, id, "Failed roundtrip for {}", id);
        }
    }

    #[test]
    fn test_encode_known_values() {
        assert_eq!(encode(0), "0");
        assert_eq!(encode(10), "A");
        assert_eq!(encode(35), "Z");
        assert_eq!(encode(36), "a");
        assert_eq!(encode(61), "z");
        assert_eq!(encode(62), "10");
        assert_eq!(encode(1000), "G8");
    }

    #[test]
    fn test_decode_errors() {
        assert_eq!(decode(""), Err(DecodeError::EmptyString));
        assert_eq!(decode("!"), Err(DecodeError::InvalidCharacter('!')));
        assert_eq!(decode("a!b"), Err(DecodeError::InvalidCharacter('!')));
    }
}
