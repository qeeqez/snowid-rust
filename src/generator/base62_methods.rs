//! Base62 convenience methods for SnowID generator

use crate::base62::{decode as base62_decode, encode as base62_encode};
use crate::base62::{encode_array as base62_encode_array, encode_into as base62_encode_into};
use crate::base62::{DecodeError as Base62DecodeError, MAX_LEN as BASE62_MAX_LEN};

use super::SnowID;

impl SnowID {
    /// Generate a new base62 encoded SnowID (zero-allocation, array-based)
    #[inline]
    pub fn generate_base62_array(&self) -> ([u8; BASE62_MAX_LEN], usize) {
        let id = self.generate();
        base62_encode_array(id)
    }

    /// Generate a new base62 encoded SnowID into caller-provided buffer
    #[inline]
    pub fn generate_base62_into<'a>(&self, buf: &'a mut [u8; BASE62_MAX_LEN]) -> (&'a str, u64) {
        let id = self.generate();
        (base62_encode_into(id, buf), id)
    }

    /// Generate a new base62 encoded SnowID (allocates String)
    pub fn generate_base62(&self) -> String {
        let id = self.generate();
        base62_encode(id)
    }

    /// Generate base62 encoded SnowID with raw u64 value
    pub fn generate_base62_with_raw(&self) -> (String, u64) {
        let id = self.generate();
        (base62_encode(id), id)
    }

    /// Decode a base62 encoded SnowID back to its raw u64 value
    pub fn decode_base62(&self, encoded: &str) -> Result<u64, Base62DecodeError> {
        base62_decode(encoded)
    }

    /// Decompose a base62 encoded SnowID into its components
    pub fn decompose_base62(&self, encoded: &str) -> Result<(u64, u16, u16), Base62DecodeError> {
        let id = self.decode_base62(encoded)?;
        Ok(self.extract.decompose(id))
    }
}
