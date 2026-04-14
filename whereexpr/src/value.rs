use std::net::IpAddr;
use crate::Error;

#[derive(Debug, Clone)]
pub enum Value<'a> {
    String(&'a str),
    Path(&'a [u8]),
    Bytes(&'a [u8]),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Hash128(&'a [u8; 16]),
    Hash160(&'a [u8; 20]),
    Hash256(&'a [u8; 32]),
    IpAddr(IpAddr),
    DateTime(u64),
    Bool(bool),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    String,
    Path,
    Bytes,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Hash128,
    Hash160,
    Hash256,
    IpAddr,
    DateTime,
    Bool,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttributeIndex(u16);
impl AttributeIndex {
    pub const fn new(index: u16) -> Self {
        Self(index)
    }
    pub const fn index(&self) -> u16 {
        self.0
    }
}
pub trait Attributes {
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>>;
    fn kind(idx: AttributeIndex) -> Option<ValueKind>;
    fn index(name: &str) -> Option<AttributeIndex>; 
}



impl<'a> Value<'a> {
    pub(crate) fn kind(&self) -> ValueKind {
        match self {
            Value::String(_) => ValueKind::String,
            Value::Path(_) => ValueKind::Path,
            Value::Bytes(_) => ValueKind::Bytes,
            Value::U8(_) => ValueKind::U8,
            Value::U16(_) => ValueKind::U16,
            Value::U32(_) => ValueKind::U32,
            Value::U64(_) => ValueKind::U64,
            Value::I8(_) => ValueKind::I8,
            Value::I16(_) => ValueKind::I16,
            Value::I32(_) => ValueKind::I32,
            Value::I64(_) => ValueKind::I64,
            Value::F32(_) => ValueKind::F32,
            Value::F64(_) => ValueKind::F64,
            Value::Hash128(_) => ValueKind::Hash128,
            Value::Hash160(_) => ValueKind::Hash160,
            Value::Hash256(_) => ValueKind::Hash256,
            Value::IpAddr(_) => ValueKind::IpAddr,
            Value::DateTime(_) => ValueKind::DateTime,
            Value::Bool(_) => ValueKind::Bool,
            Value::None => ValueKind::None,
        }
    }
}
impl ValueKind {
    pub fn parse_str(repr: &str) -> Option<ValueKind> {
        let b = repr.as_bytes();
        match b.len() {
            2 => match [b[0] | 32, b[1]] {
                [b'u', b'8'] => Some(Self::U8),
                [b'i', b'8'] => Some(Self::I8),
                [b'i', b'p'] => Some(Self::IpAddr),
                _ => None,
            },
            3 => match [b[0] | 32, b[1] | 32, b[2]] {
                [b'u', b'1', b'6'] => Some(Self::U16),
                [b'u', b'3', b'2'] => Some(Self::U32),
                [b'u', b'6', b'4'] => Some(Self::U64),
                [b'i', b'1', b'6'] => Some(Self::I16),
                [b'i', b'3', b'2'] => Some(Self::I32),
                [b'i', b'6', b'4'] => Some(Self::I64),
                [b'f', b'3', b'2'] => Some(Self::F32),
                [b'f', b'6', b'4'] => Some(Self::F64),
                _ => None,
            },
            4 => match [b[0] | 32, b[1] | 32, b[2] | 32, b[3] | 32] {
                [b'b', b'o', b'o', b'l'] => Some(Self::Bool),
                [b'n', b'o', b'n', b'e'] => Some(Self::None),
                [b'p', b'a', b't', b'h'] => Some(Self::Path),               
                _ => None,
            },
            5 => match [b[0] | 32, b[1] | 32, b[2] | 32, b[3] | 32, b[4] | 32] {
                [b'b', b'y', b't', b'e', b's'] => Some(Self::Bytes),
                _ => None,
            },
            6 => match [b[0] | 32, b[1] | 32, b[2] | 32, b[3] | 32, b[4] | 32, b[5] | 32] {
                [b's', b't', b'r', b'i', b'n', b'g'] => Some(Self::String),
                [b'i', b'p', b'a', b'd', b'd', b'r'] => Some(Self::IpAddr),
                _ => None,
            },
            7 => match [b[0] | 32, b[1] | 32, b[2] | 32, b[3] | 32, b[4] | 32, b[5] | 32, b[6] | 32] {
                [b'h', b'a', b's', b'h', b'1', b'2', b'8'] => Some(Self::Hash128),
                [b'h', b'a', b's', b'h', b'1', b'6', b'0'] => Some(Self::Hash160),
                [b'h', b'a', b's', b'h', b'2', b'5', b'6'] => Some(Self::Hash256),
                [b'd', b'a', b't', b'e', b't', b'i', b'm'] => Some(Self::DateTime),
                _ => None,
            },
            8 => {
                // "datetime" with 'e' at end = 8 chars
                let low = [b[0]|32, b[1]|32, b[2]|32, b[3]|32, b[4]|32, b[5]|32, b[6]|32, b[7]|32];
                match low {
                    [b'd', b'a', b't', b'e', b't', b'i', b'm', b'e'] => Some(Self::DateTime),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    // fortez validare la compile time pe campurile din FieldValueKind si FieldValue (sa fie la fel - 1:1)
    pub(crate) fn _default_value(&self) -> Value<'static> {
        match self {
            ValueKind::String => Value::String(""),
            ValueKind::Path => Value::Path(b""),
            ValueKind::Bytes => Value::Bytes(b""),
            ValueKind::U8 => Value::U8(0),
            ValueKind::U16 => Value::U16(0),
            ValueKind::U32 => Value::U32(0),
            ValueKind::U64 => Value::U64(0),
            ValueKind::I8 => Value::I8(0),
            ValueKind::I16 => Value::I16(0),
            ValueKind::I32 => Value::I32(0),
            ValueKind::I64 => Value::I64(0),
            ValueKind::F32 => Value::F32(0.0),
            ValueKind::F64 => Value::F64(0.0),
            ValueKind::Hash128 => Value::Hash128(&[0u8; 16]),
            ValueKind::Hash160 => Value::Hash160(&[0u8; 20]),
            ValueKind::Hash256 => Value::Hash256(&[0u8; 32]),
            ValueKind::IpAddr => Value::IpAddr(IpAddr::from([0, 0, 0, 0])),
            ValueKind::DateTime => Value::DateTime(0),
            ValueKind::Bool => Value::Bool(false),
            ValueKind::None => Value::None,
        }
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(s: &'a str) -> Self {
        Value::String(s)
    }
}

impl<'a> TryFrom<Value<'a>> for &'a str {
    type Error = Error;

    fn try_from(value: Value<'a>) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(Error::ExpectingADifferentValueKind(value.kind(), ValueKind::String)),
        }
    }
}

impl<'a> TryFrom<Value<'a>> for &'a [u8] {
    type Error = Error;

    fn try_from(value: Value<'a>) -> Result<Self, Self::Error> {
        match value {
            Value::Path(p) => Ok(p),
            _ => Err(Error::ExpectingADifferentValueKind(value.kind(), ValueKind::Path)),
        }
    }
}

#[cfg(feature = "error_description")]
impl std::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::String => write!(f, "String"),
            ValueKind::Path => write!(f, "Path"),
            ValueKind::Bytes => write!(f, "Bytes"),
            ValueKind::U8 => write!(f, "U8"),
            ValueKind::U16 => write!(f, "U16"),
            ValueKind::U32 => write!(f, "U32"),
            ValueKind::U64 => write!(f, "U64"),
            ValueKind::I8 => write!(f, "I8"),
            ValueKind::I16 => write!(f, "I16"),
            ValueKind::I32 => write!(f, "I32"),
            ValueKind::I64 => write!(f, "I64"),
            ValueKind::F32 => write!(f, "F32"),
            ValueKind::F64 => write!(f, "F64"),
            ValueKind::Hash128 => write!(f, "Hash128"),
            ValueKind::Hash160 => write!(f, "Hash160"),
            ValueKind::Hash256 => write!(f, "Hash256"),
            ValueKind::IpAddr => write!(f, "IP address"),
            ValueKind::DateTime => write!(f, "DateTime"),
            ValueKind::Bool => write!(f, "Bool"),
            ValueKind::None => write!(f, "None"),
        }
    }
}
impl std::str::FromStr for ValueKind {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = Self::parse_str(s).ok_or(Error::UnknownValueKind(0, s.len() as u32, s.to_string()))?;
        Ok(kind)
    }
}