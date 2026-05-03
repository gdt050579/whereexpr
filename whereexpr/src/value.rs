use crate::Error;
use std::net::IpAddr;

/// A runtime value returned by [`Attributes::get`] for a specific field of type `T`.
///
/// `Value` is a tagged union that wraps the actual data without owning it
/// (string-like variants borrow from the source object). It is used both when
/// returning field data during expression evaluation and when constructing a
/// [`Predicate`](crate::Predicate) programmatically.
///
/// # Choosing the right variant
///
/// Match the variant to the Rust type you store in your struct and declare in
/// your [`Attributes`] implementation. The [`ValueKind`] mirror enum describes
/// the same set of types but without carrying data, and is used for type
/// declarations and introspection.
#[derive(Debug, Clone)]
pub enum Value<'a> {
    /// A UTF-8 string slice. Supports all string operations: `is`, `contains`,
    /// `starts-with`, `ends-with`, `glob`, and their negated / list variants.
    ///
    /// ```text
    /// name is Alice
    /// message contains error
    /// filename ends-with .log
    /// mime-type glob image/*
    /// ```
    String(&'a str),

    /// A list of UTF-8 string slices
    StringList(&'a [&'a str]),

    /// A UTF-8 string slice representing a filesystem path. Supports the same
    /// pattern operations as [`String`](Value::String), including `{ignore-case}`.
    /// Path attributes also support glob-regex match (`glob-re-match`).
    ///
    /// ```text
    /// path starts-with /home/user
    /// path ends-with-one-of [.rs, .toml]
    /// path glob /var/log/**/*.log
    /// ```
    Path(&'a str),

    /// A raw byte-slice for arbitrary binary data.
    ///
    /// > **Note:** full predicate support for `Bytes` is not yet implemented.
    Bytes(&'a [u8]),

    /// An unsigned 8-bit integer (0 – 255). Supports all numeric operations:
    /// `>`, `>=`, `<`, `<=`, `in-range`, `not-in-range`, `is`, `is-not`,
    /// `is-one-of`, and `is-not-one-of`.
    ///
    /// ```text
    /// priority is 1
    /// flags > 0
    /// ttl in-range [1, 64]
    /// ```
    U8(u8),

    /// An unsigned 16-bit integer (0 – 65 535). Supports all numeric operations.
    ///
    /// ```text
    /// port is 8080
    /// port in-range [1024, 65535]
    /// port is-not-one-of [0, 1, 22]
    /// ```
    U16(u16),

    /// An unsigned 32-bit integer (0 – 4 294 967 295). Supports all numeric operations.
    ///
    /// ```text
    /// age > 18
    /// user-id is 42
    /// score in-range [100, 999]
    /// ```
    U32(u32),

    /// An unsigned 64-bit integer. Supports all numeric operations.
    ///
    /// ```text
    /// file-size > 1048576
    /// timestamp >= 1700000000
    /// inode is-not 0
    /// ```
    U64(u64),

    /// A signed 8-bit integer (−128 – 127). Supports all numeric operations.
    ///
    /// ```text
    /// offset > 0
    /// delta is-not 0
    /// level in-range [-10, 10]
    /// ```
    I8(i8),

    /// A signed 16-bit integer. Supports all numeric operations.
    ///
    /// ```text
    /// altitude > -500
    /// diff in-range [-1000, 1000]
    /// ```
    I16(i16),

    /// A signed 32-bit integer. Supports all numeric operations.
    ///
    /// ```text
    /// balance >= 0
    /// temperature < -20
    /// offset not-in-range [-100, 100]
    /// ```
    I32(i32),

    /// A signed 64-bit integer. Supports all numeric operations.
    ///
    /// ```text
    /// unix-time > 0
    /// profit >= -1000000
    /// ```
    I64(i64),

    /// A 32-bit floating-point number. Supports all numeric operations.
    ///
    /// ```text
    /// ratio > 0.5
    /// confidence in-range [0.0, 1.0]
    /// ```
    F32(f32),

    /// A 64-bit floating-point number. Supports all numeric operations.
    ///
    /// ```text
    /// price > 9.99
    /// load in-range [0.0, 1.0]
    /// discount < 0.15
    /// ```
    F64(f64),

    /// A 128-bit hash stored as a fixed-size 16-byte array (e.g. MD5).
    /// Supports equality and membership operations: `is`, `is-not`,
    /// `is-one-of`, `is-not-one-of`.
    ///
    /// ```text
    /// md5 is d41d8cd98f00b204e9800998ecf8427e
    /// checksum is-not-one-of [aabbcc..., 112233...]
    /// ```
    Hash128(&'a [u8; 16]),

    /// A 160-bit hash stored as a fixed-size 20-byte array (e.g. SHA-1).
    /// Supports equality and membership operations.
    ///
    /// ```text
    /// sha1 is da39a3ee5e6b4b0d3255bfef95601890afd80709
    /// commit is-one-of [abc123..., def456...]
    /// ```
    Hash160(&'a [u8; 20]),

    /// A 256-bit hash stored as a fixed-size 32-byte array (e.g. SHA-256).
    /// Supports equality and membership operations.
    ///
    /// ```text
    /// sha256 is e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    /// digest is-not 0000...0000
    /// ```
    Hash256(&'a [u8; 32]),

    /// An IP address (either IPv4 or IPv6 via [`std::net::IpAddr`]).
    /// Supports all numeric / comparison operations, enabling subnet-style
    /// range checks.
    ///
    /// ```text
    /// client-ip is 127.0.0.1
    /// source-ip is-not-one-of [10.0.0.1, 192.168.1.1]
    /// peer-ip in-range [10.0.0.0, 10.255.255.255]
    /// ```
    IpAddr(IpAddr),

    /// A point in time represented as a Unix timestamp (seconds since the
    /// Unix epoch, stored as `u64`). Supports all numeric operations.
    ///
    /// ```text
    /// created-at > 1700000000
    /// modified-at in-range [1690000000, 1700000000]
    /// expires-at < 9999999999
    /// ```
    DateTime(u64),

    /// A boolean value. Supports only equality: `is` and `is-not`.
    /// List operations (`is-one-of`, etc.) are not supported for `Bool`.
    ///
    /// ```text
    /// is-active is true
    /// has-errors is-not false
    /// ```
    Bool(bool),

    /// Signals that the attribute is absent or not applicable. When
    /// [`Attributes::get`] returns `Value::Unknown` (or `None`) the condition
    /// that references the attribute evaluates to `None`, causing the whole
    /// expression to yield `None` as well.
    Unknown,
}

/// The type tag of a [`Value`], without carrying any data.
///
/// `ValueKind` is used in two places:
///
/// 1. **[`Attributes::kind`]** — declares the type of each attribute so that the
///    expression builder can validate operations and parse string literals into
///    the correct type at build time.
/// 2. **[`Predicate`](crate::Predicate) constructors** (`with_str`, `with_str_list`)
///    — tells the predicate how to interpret a string value.
///
/// Each variant corresponds 1-to-1 with a [`Value`] variant and has a canonical
/// string token accepted by [`ValueKind::parse_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    /// UTF-8 text. Token: `string`
    ///
    /// ```text
    /// name is Alice
    /// ```
    String,

    /// A list of UTF-8 string slices. Token: `string-list`
    StringList,

    /// UTF-8 filesystem path as `&str`. Token: `path`
    ///
    /// ```text
    /// path starts-with /home
    /// ```
    Path,

    /// Raw binary data. Token: `bytes`
    ///
    /// > **Note:** full predicate support for `Bytes` is not yet implemented.
    Bytes,

    /// Unsigned 8-bit integer. Token: `u8`
    ///
    /// ```text
    /// priority is 1
    /// ```
    U8,

    /// Unsigned 16-bit integer. Token: `u16`
    ///
    /// ```text
    /// port in-range [1024, 65535]
    /// ```
    U16,

    /// Unsigned 32-bit integer. Token: `u32`
    ///
    /// ```text
    /// age >= 18
    /// ```
    U32,

    /// Unsigned 64-bit integer. Token: `u64`
    ///
    /// ```text
    /// file-size > 1048576
    /// ```
    U64,

    /// Signed 8-bit integer. Token: `i8`
    ///
    /// ```text
    /// level in-range [-10, 10]
    /// ```
    I8,

    /// Signed 16-bit integer. Token: `i16`
    ///
    /// ```text
    /// altitude > -500
    /// ```
    I16,

    /// Signed 32-bit integer. Token: `i32`
    ///
    /// ```text
    /// balance >= 0
    /// ```
    I32,

    /// Signed 64-bit integer. Token: `i64`
    ///
    /// ```text
    /// unix-time > 0
    /// ```
    I64,

    /// 32-bit floating-point number. Token: `f32`
    ///
    /// ```text
    /// confidence in-range [0.0, 1.0]
    /// ```
    F32,

    /// 64-bit floating-point number. Token: `f64`
    ///
    /// ```text
    /// price > 9.99
    /// ```
    F64,

    /// 128-bit hash (16 bytes, e.g. MD5). Token: `hash128`
    ///
    /// ```text
    /// md5 is d41d8cd98f00b204e9800998ecf8427e
    /// ```
    Hash128,

    /// 160-bit hash (20 bytes, e.g. SHA-1). Token: `hash160`
    ///
    /// ```text
    /// sha1 is da39a3ee5e6b4b0d3255bfef95601890afd80709
    /// ```
    Hash160,

    /// 256-bit hash (32 bytes, e.g. SHA-256). Token: `hash256`
    ///
    /// ```text
    /// sha256 is e3b0c44298fc1c149afbf4c8996fb924...
    /// ```
    Hash256,

    /// IPv4 or IPv6 address. Tokens: `ip`, `ipaddr`
    ///
    /// ```text
    /// client-ip is 127.0.0.1
    /// peer-ip in-range [10.0.0.0, 10.255.255.255]
    /// ```
    IpAddr,

    /// Unix timestamp (`u64` seconds since epoch). Tokens: `datetime`, `datetim`
    ///
    /// ```text
    /// created-at > 1700000000
    /// modified-at in-range [1690000000, 1700000000]
    /// ```
    DateTime,

    /// Boolean. Token: `bool`. Only `is` and `is-not` are supported; list
    /// operations are not valid for this type.
    ///
    /// ```text
    /// is-active is true
    /// has-errors is-not false
    /// ```
    Bool,

    /// Absent / not applicable. Token: `none`
    Unknown,
}

/// An opaque, zero-cost index that uniquely identifies an attribute within a
/// type that implements [`Attributes`].
///
/// Declare one constant per attribute in your type and use them consistently in
/// all three [`Attributes`] methods:
///
/// ```text
/// const NAME: AttributeIndex = AttributeIndex::new(0);
/// const AGE:  AttributeIndex = AttributeIndex::new(1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttributeIndex(u16);

impl AttributeIndex {
    /// Creates a new `AttributeIndex` with the given numeric index.
    ///
    /// Indices are arbitrary unsigned integers — the only requirement is that
    /// each attribute within a type has a unique index and that the same index
    /// is used consistently across all three [`Attributes`] methods.
    ///
    /// This is a `const fn`, so indices can be defined as associated constants:
    ///
    /// ```text
    /// const NAME: AttributeIndex = AttributeIndex::new(0);
    /// const AGE:  AttributeIndex = AttributeIndex::new(1);
    /// const EMAIL: AttributeIndex = AttributeIndex::new(2);
    /// ```
    pub const fn new(index: u16) -> Self {
        Self(index)
    }

    /// Returns the raw numeric index value.
    ///
    /// ```text
    /// AttributeIndex::new(3).index()  →  3
    /// ```
    pub const fn index(&self) -> u16 {
        self.0
    }
}

/// Exposes the fields of a type `T` to the expression engine.
///
/// Implement this trait on any struct whose fields you want to filter or match
/// against. Three methods must be provided, each covering one direction of the
/// name ↔ index ↔ value relationship:
///
/// | Method | Direction | Called by |
/// |---|---|---|
/// | [`index`](Attributes::index) | name → `AttributeIndex` | `ExpressionBuilder::build` (string resolution) |
/// | [`kind`](Attributes::kind) | `AttributeIndex` → `ValueKind` | `ExpressionBuilder::build` (type validation) |
/// | [`get`](Attributes::get) | `AttributeIndex` → `Value` | Expression evaluation at runtime |
///
/// # Example
///
/// ```text
/// struct Person { name: String, age: u32 }
///
/// impl Person {
///     const NAME: AttributeIndex = AttributeIndex::new(0);
///     const AGE:  AttributeIndex = AttributeIndex::new(1);
/// }
///
/// impl Attributes for Person {
///     fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> { … }
///     fn kind(idx: AttributeIndex)       -> Option<ValueKind> { … }
///     fn index(name: &str)               -> Option<AttributeIndex> { … }
/// }
/// ```
///
/// Once implemented, you can build expressions like:
///
/// ```text
/// name is Alice
/// age > 30
/// name is-one-of [Alice, Bob] {ignore-case}
/// ```
pub trait Attributes {
    /// Returns the runtime value of the attribute identified by `idx` for `self`.
    ///
    /// Return `None` (or `Value::Unknown`) when the attribute is absent or not
    /// applicable for this particular object. A `None` result causes the
    /// condition that references the attribute to evaluate to `None`, which
    /// propagates through the whole expression.
    ///
    /// ```text
    /// // Person { name: "Alice", age: 30 }
    /// get(NAME)  →  Some(Value::String("Alice"))
    /// get(AGE)   →  Some(Value::U32(30))
    /// get(99)    →  None  (unknown index)
    /// ```
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>>;

    /// Returns the [`ValueKind`] (type tag) for the attribute identified by `idx`.
    ///
    /// This is a **static** method called once during
    /// [`ExpressionBuilder::build`](crate::ExpressionBuilder::build) to validate
    /// that the operation in each condition string is compatible with the
    /// attribute's type. Return `None` for unknown indices.
    ///
    /// ```text
    /// kind(NAME)  →  Some(ValueKind::String)
    /// kind(AGE)   →  Some(ValueKind::U32)
    /// kind(99)    →  None
    /// ```
    fn kind(idx: AttributeIndex) -> Option<ValueKind>;

    /// Resolves an attribute **name string** to its [`AttributeIndex`].
    ///
    /// This is a **static** method called once during
    /// [`ExpressionBuilder::build`](crate::ExpressionBuilder::build) when a
    /// condition references an attribute by name (e.g. from
    /// [`Condition::from_str`](crate::Condition::from_str) or
    /// [`Condition::new`](crate::Condition::new)). Return `None` if the name is
    /// not recognised; `build` will then return
    /// [`Error::UnknownAttribute`](crate::Error::UnknownAttribute).
    ///
    /// ```text
    /// index("name")    →  Some(AttributeIndex(0))
    /// index("age")     →  Some(AttributeIndex(1))
    /// index("missing") →  None
    /// ```
    fn index(name: &str) -> Option<AttributeIndex>;

    /// A constant that uniquely identifies the type of the object that implements this trait.
    /// This constant must be set (and unique) by the implementer of the trait.
    const TYPE_ID: u64;

    /// The name of the type that implements this trait (it will be used for error/panic messages)
    const TYPE_NAME: &'static str;
}

impl<'a> Value<'a> {
    pub(crate) fn kind(&self) -> ValueKind {
        match self {
            Value::String(_) => ValueKind::String,
            Value::StringList(_) => ValueKind::StringList,
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
            Value::Unknown => ValueKind::Unknown,
        }
    }
}
impl ValueKind {
    /// Parses a `ValueKind` from its string token.
    ///
    /// Parsing is **case-insensitive**. Returns `None` for unknown tokens.
    /// For a `Result`-returning alternative see the [`FromStr`](std::str::FromStr)
    /// implementation on `ValueKind`.
    ///
    /// # Recognised tokens
    ///
    /// ```text
    /// "string"   → ValueKind::String
    /// "path"     → ValueKind::Path
    /// "bytes"    → ValueKind::Bytes
    /// "u8"       → ValueKind::U8
    /// "u16"      → ValueKind::U16
    /// "u32"      → ValueKind::U32
    /// "u64"      → ValueKind::U64
    /// "i8"       → ValueKind::I8
    /// "i16"      → ValueKind::I16
    /// "i32"      → ValueKind::I32
    /// "i64"      → ValueKind::I64
    /// "f32"      → ValueKind::F32
    /// "f64"      → ValueKind::F64
    /// "hash128"  → ValueKind::Hash128
    /// "hash160"  → ValueKind::Hash160
    /// "hash256"  → ValueKind::Hash256
    /// "ip"       → ValueKind::IpAddr
    /// "ipaddr"   → ValueKind::IpAddr
    /// "datetime" → ValueKind::DateTime
    /// "bool"     → ValueKind::Bool
    /// "none"     → ValueKind::None
    /// "unknown"  → None
    /// ```
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
                [b'n', b'o', b'n', b'e'] => Some(Self::Unknown),
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
                let low = [b[0] | 32, b[1] | 32, b[2] | 32, b[3] | 32, b[4] | 32, b[5] | 32, b[6] | 32, b[7] | 32];
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
            ValueKind::StringList => Value::StringList(&[]),
            ValueKind::Path => Value::Path(""),
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
            ValueKind::Unknown => Value::Unknown,
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

#[cfg(feature = "error_description")]
impl std::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::String => write!(f, "String"),
            ValueKind::StringList => write!(f, "StringList"),
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
            ValueKind::Unknown => write!(f, "None"),
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
