use std::net::IpAddr;

pub(crate) enum Value<'a> {
    String(&'a str),
    Path(&'a [u8]),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Hash128([u8; 16]),
    Hash160([u8; 20]),
    IpAddr(IpAddr),
    DateTime(u64),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueKind {
    String,
    Path,
    Unsigned,
    Signed,
    Float,
    Hash128,
    Hash160,
    IpAddr,
    DateTime,
    Bool,
}

pub(crate) trait Attributes {
    fn get(&self, attribute_index: u16) -> Option<Value<'_>>;
    fn kind(attribute_index: u16) -> Option<ValueKind>;
}


impl<'a> Value<'a> {
    pub(crate) fn kind(&self) -> ValueKind {
        match self {
            Value::String(_) => ValueKind::String,
            Value::Path(_) => ValueKind::Path,
            Value::Unsigned(_) => ValueKind::Unsigned,
            Value::Signed(_) => ValueKind::Signed,
            Value::Float(_) => ValueKind::Float,
            Value::Hash128(_) => ValueKind::Hash128,
            Value::Hash160(_) => ValueKind::Hash160,
            Value::IpAddr(_) => ValueKind::IpAddr,
            Value::DateTime(_) => ValueKind::DateTime,
            Value::Bool(_) => ValueKind::Bool,
        }
    }
}
impl ValueKind {
    // fortez validare la compile time pe campurile din FieldValueKind si FieldValue (sa fie la fel - 1:1)
    pub(crate) fn default_value(&self) -> Value<'static> {
        match self {
            ValueKind::String => Value::String(""),
            ValueKind::Path => Value::Path(b""),
            ValueKind::Unsigned => Value::Unsigned(0),
            ValueKind::Signed => Value::Signed(0),
            ValueKind::Float => Value::Float(0.0),
            ValueKind::Hash128 => Value::Hash128([0u8; 16]),
            ValueKind::Hash160 => Value::Hash160([0u8; 20]),
            ValueKind::IpAddr => Value::IpAddr(IpAddr::from([0, 0, 0, 0])),
            ValueKind::DateTime => Value::DateTime(0),
            ValueKind::Bool => Value::Bool(false),
        }
    }

    #[cfg(feature = "error_description")]
    pub(crate) fn description(&self) -> &'static str {
        match self {
            ValueKind::String => "String",
            ValueKind::Path => "Path",
            ValueKind::Unsigned => "Unsigned number",
            ValueKind::Signed => "Signed number",
            ValueKind::Float => "Float number",
            ValueKind::Hash128 => "MD5 hash",
            ValueKind::Hash160 => "SHA1 hash",
            ValueKind::IpAddr => "IP address",
            ValueKind::DateTime => "DateTime",
            ValueKind::Bool => "Bool",
        }
    }
}
