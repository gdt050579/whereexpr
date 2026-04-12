use std::fmt;
use std::str::FromStr;
use super::{IntoValueKind, FromRepr};
use crate::{ValueKind, Value};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash<const N: usize> {
    bytes: [u8; N],
}

pub type Hash128 = Hash<16>;
pub type Hash160 = Hash<20>;
pub type Hash256 = Hash<32>;

#[derive(Debug)]
pub struct InvalidHash;

impl fmt::Display for InvalidHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid hash: expected lowercase hex string")
    }
}

impl<const N: usize> Hash<N> {
    pub fn new(bytes: [u8; N]) -> Self {
        Self { bytes }
    }

    pub fn as_bytes(&self) -> &[u8; N] {
        &self.bytes
    }
}

impl<const N: usize> fmt::Display for Hash<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl<const N: usize> FromStr for Hash<N> {
    type Err = InvalidHash;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.len() != N * 2 {
            return Err(InvalidHash);
        }
        let mut bytes = [0u8; N];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let hi = hex_char_to_u8(chunk[0]).ok_or(InvalidHash)?;
            let lo = hex_char_to_u8(chunk[1]).ok_or(InvalidHash)?;
            bytes[i] = (hi << 4) | lo;
        }
        Ok(Self { bytes })
    }
}

fn hex_char_to_u8(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

impl Default for Hash128 {
    fn default() -> Self {
        Self::new([0u8; 16])
    }
}

impl Default for Hash160 {
    fn default() -> Self {
        Self::new([0u8; 20])
    }
}

impl IntoValueKind for Hash128 {
    const VALUE_KIND: ValueKind = ValueKind::Hash128;
}

impl IntoValueKind for Hash160 {
    const VALUE_KIND: ValueKind = ValueKind::Hash160;
}

impl IntoValueKind for Hash256 {
    const VALUE_KIND: ValueKind = ValueKind::Hash256;
}
impl FromRepr<Hash128> for Hash128 {
    fn from_repr(repr: &str) -> Result<Self, crate::Error> {
        Self::from_str(repr).map_err(|_| crate::Error::FailToParseValue(repr.to_string(), ValueKind::Hash128))
    }
}
impl FromRepr<Hash160> for Hash160 {
    fn from_repr(repr: &str) -> Result<Self, crate::Error> {
        Self::from_str(repr).map_err(|_| crate::Error::FailToParseValue(repr.to_string(), ValueKind::Hash160))
    }
}
impl FromRepr<Hash256> for Hash256 {
    fn from_repr(repr: &str) -> Result<Self, crate::Error> {
        Self::from_str(repr).map_err(|_| crate::Error::FailToParseValue(repr.to_string(), ValueKind::Hash256))
    }
}
impl TryFrom<Value<'_>> for Hash128 {
    type Error = crate::Error;
    fn try_from(value: Value<'_>) -> Result<Self, Self::Error> {
        match value {
            Value::Hash128(v) => Ok(Hash128::new(*v)),
            _ => Err(crate::Error::ExpectingADifferentValueKind(value.kind(), ValueKind::Hash128)),
        }
    }
} 
impl TryFrom<Value<'_>> for Hash160 {
    type Error = crate::Error;
    fn try_from(value: Value<'_>) -> Result<Self, Self::Error> {
        match value {
            Value::Hash160(v) => Ok(Hash160::new(*v)),
            _ => Err(crate::Error::ExpectingADifferentValueKind(value.kind(), ValueKind::Hash160)),
        }
    }
} 
impl TryFrom<Value<'_>> for Hash256 {
    type Error = crate::Error;
    fn try_from(value: Value<'_>) -> Result<Self, Self::Error> {
        match value {
            Value::Hash256(v) => Ok(Hash256::new(*v)),
            _ => Err(crate::Error::ExpectingADifferentValueKind(value.kind(), ValueKind::Hash256)),
        }
    }
}