use std::net::IpAddr;
use std::any::TypeId;
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
    pub fn new(index: u16) -> Self {
        Self(index)
    }
    pub fn index(&self) -> u16 {
        self.0
    }
}
pub trait Attributes {
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>>;
    fn kind(idx: AttributeIndex) -> Option<ValueKind>;
    fn index(name: &str) -> Option<AttributeIndex>;
    fn type_id() -> TypeId where Self: 'static;    
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
    // fortez validare la compile time pe campurile din FieldValueKind si FieldValue (sa fie la fel - 1:1)
    pub(crate) fn default_value(&self) -> Value<'static> {
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

    #[cfg(feature = "error_description")]
    pub(crate) fn description(&self) -> &'static str {
        match self {
            ValueKind::String => "String",
            ValueKind::Path => "Path",
            ValueKind::Bytes => "Bytes",
            ValueKind::U8 => "U8",
            ValueKind::U16 => "U16",
            ValueKind::U32 => "U32",
            ValueKind::U64 => "U64",
            ValueKind::I8 => "I8",
            ValueKind::I16 => "I16",
            ValueKind::I32 => "I32",
            ValueKind::I64 => "I64",
            ValueKind::F32 => "F32",
            ValueKind::F64 => "F64",
            ValueKind::Hash128 => "Hash128",
            ValueKind::Hash160 => "Hash160",
            ValueKind::Hash256 => "Hash256",
            ValueKind::IpAddr => "IP address",
            ValueKind::DateTime => "DateTime",
            ValueKind::Bool => "Bool",
            ValueKind::None => "None",
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
