use super::predicates::*;
use super::types::*;
use super::Error;
use super::Operation;
use super::{Value, ValueKind};

enum PredicateInner {
    I8Predicate(I8Predicate),
    I16Predicate(I16Predicate),
    I32Predicate(I32Predicate),
    I64Predicate(I64Predicate),
    U8Predicate(U8Predicate),
    U16Predicate(U16Predicate),
    U32Predicate(U32Predicate),
    U64Predicate(U64Predicate),
    F32Predicate(F32Predicate),
    F64Predicate(F64Predicate),
    StringPredicate(StringPredicate),
    PathPredicate(PathPredicate),
    Hash128Predicate(Hash128Predicate),
    Hash160Predicate(Hash160Predicate),
    Hash256Predicate(Hash256Predicate),
    IpAddrPredicate(IpAddrPredicate),
    DateTimePredicate(DateTimePredicate),
    BoolPredicate(BoolPredicate),
}

/// A compiled, type-erased test that evaluates a single [`Value`] against an
/// [`Operation`] and a reference value (or list of reference values).
///
/// A `Predicate` is the lowest-level building block. It is normally created
/// indirectly through [`Condition`](crate::Condition), but you can build one
/// directly when you need fine-grained control (e.g. when constructing a
/// [`Condition::new`](crate::Condition::new) or
/// [`Condition::with_index`](crate::Condition::with_index)).
///
/// # Choosing a constructor
///
/// | Constructor | Reference value source | Type information |
/// |---|---|---|
/// | [`with_value`](Predicate::with_value) | A single `T: Into<Value>` | Inferred from the value |
/// | [`with_list`](Predicate::with_list) | A slice of `T: Into<Value> + Clone` | Inferred from the first element |
/// | [`with_value_list`](Predicate::with_value_list) | A slice of [`Value`] | Inferred from the first element |
/// | [`with_str`](Predicate::with_str) | A `&str` that is parsed at build time | Supplied explicitly as [`ValueKind`] |
/// | [`with_str_list`](Predicate::with_str_list) | A slice of `&str` parsed at build time | Supplied explicitly as [`ValueKind`] |
///
/// Negated operations (e.g. [`Operation::IsNot`], [`Operation::NotContains`]) are
/// handled transparently: the constructor stores a `negated` flag so that the
/// result of [`evaluate`](Predicate::evaluate) is automatically flipped.
pub struct Predicate {
    predicate: PredicateInner,
    negated: bool,
}

impl Predicate {
    /// Creates a predicate from a **single typed value**.
    ///
    /// The value type is determined by the `T: Into<Value>` conversion, so you can
    /// pass Rust primitives directly wherever a `From`/`Into` impl exists (e.g.
    /// `&str`, `u32`, `bool`, `IpAddr`). You can also pass a [`Value`] variant
    /// directly.
    ///
    /// Negating operations (`IsNot`, `NotContains`, …) are fully supported; the
    /// predicate stores a `negated` flag internally and flips the result at
    /// evaluation time.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the operation is incompatible with the value type
    /// (e.g. applying `Contains` to a numeric value).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use whereexpr::{Predicate, Operation, Value};
    ///
    /// // String equality
    /// let p = Predicate::with_value(Operation::Is, Value::String("hello")).unwrap();
    ///
    /// // Numeric greater-than
    /// let p = Predicate::with_value(Operation::GreaterThan, Value::U32(42)).unwrap();
    ///
    /// // Negated: "is not"
    /// let p = Predicate::with_value(Operation::IsNot, Value::U32(0)).unwrap();
    /// ```
    pub fn with_value<'a, T>(op: Operation, value: T) -> Result<Self, Error>
    where
        T: Into<Value<'a>>,
    {
        let val: Value<'a> = value.into();
        let (op, negated) = op.operation_and_negated();
        let predicate = match val {
            Value::String(s) => PredicateInner::StringPredicate(StringPredicate::with_value(op, s, false)?),
            Value::Path(v) => PredicateInner::PathPredicate(PathPredicate::with_value(op, v)?),
            Value::Bytes(items) => todo!(),
            Value::U8(v) => PredicateInner::U8Predicate(U8Predicate::with_value(op, v)?),
            Value::U16(v) => PredicateInner::U16Predicate(U16Predicate::with_value(op, v)?),
            Value::U32(v) => PredicateInner::U32Predicate(U32Predicate::with_value(op, v)?),
            Value::U64(v) => PredicateInner::U64Predicate(U64Predicate::with_value(op, v)?),
            Value::I8(v) => PredicateInner::I8Predicate(I8Predicate::with_value(op, v)?),
            Value::I16(v) => PredicateInner::I16Predicate(I16Predicate::with_value(op, v)?),
            Value::I32(v) => PredicateInner::I32Predicate(I32Predicate::with_value(op, v)?),
            Value::I64(v) => PredicateInner::I64Predicate(I64Predicate::with_value(op, v)?),
            Value::F32(v) => PredicateInner::F32Predicate(F32Predicate::with_value(op, v)?),
            Value::F64(v) => PredicateInner::F64Predicate(F64Predicate::with_value(op, v)?),
            Value::Hash128(v) => PredicateInner::Hash128Predicate(Hash128Predicate::with_value(op, Hash128::new(*v))?),
            Value::Hash160(v) => PredicateInner::Hash160Predicate(Hash160Predicate::with_value(op, Hash160::new(*v))?),
            Value::Hash256(v) => PredicateInner::Hash256Predicate(Hash256Predicate::with_value(op, Hash256::new(*v))?),
            Value::IpAddr(ip_addr) => PredicateInner::IpAddrPredicate(IpAddrPredicate::with_value(op, ip_addr)?),
            Value::DateTime(v) => PredicateInner::DateTimePredicate(DateTimePredicate::with_value(op, v)?),
            Value::Bool(v) => PredicateInner::BoolPredicate(BoolPredicate::with_value(op, v)?),
            Value::None => todo!(),
        };
        Ok(Predicate { predicate, negated })
    }
    /// Creates a predicate from a **slice of typed values**.
    ///
    /// This is a convenience wrapper around [`with_value_list`](Predicate::with_value_list)
    /// for use when the values are stored as `T: Into<Value> + Clone` rather than as
    /// [`Value`] directly. Elements are cloned and converted internally; for lists
    /// already in [`Value`] form prefer `with_value_list` to avoid the extra copy.
    ///
    /// The value type of the predicate is inferred from the **first element**; all
    /// subsequent elements must be of the same type.
    ///
    /// # Errors
    ///
    /// - [`Error::EmptyListForOperation`] – `values` is empty.
    /// - [`Error::InvalidOperationForValue`] – the operation is incompatible with
    ///   the inferred type (e.g. `Bool` cannot be used with list operations).
    /// - Any other type-specific parse or range error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use whereexpr::{Predicate, Operation, Value};
    ///
    /// // "is one of" check against a list of u32 values
    /// let allowed: &[u32] = &[1, 2, 3];
    /// // u32 implements Into<Value> via Value::U32
    /// let values: Vec<Value> = allowed.iter().map(|v| Value::U32(*v)).collect();
    /// let p = Predicate::with_value_list(Operation::IsOneOf, &values).unwrap();
    ///
    /// // Convenience form using with_list (same result)
    /// let p2 = Predicate::with_list(Operation::IsOneOf, &values).unwrap();
    /// ```
    pub fn with_list<'a, T>(op: Operation, values: &[T]) -> Result<Self, Error>
    where
        T: Into<Value<'a>> + Clone,
    {
        if values.len() < 256 {
            let mut v: [Value<'a>; 256] = std::array::from_fn(|_| Value::None);
            for (i, value) in values.iter().enumerate() {
                v[i] = value.clone().into();
            }
            Self::with_value_list(op, &v[..values.len()])
        } else {
            let mut v: Vec<Value<'a>> = Vec::with_capacity(values.len());
            for value in values {
                v.push(value.clone().into());
            }
            Self::with_value_list(op, &v)
        }
    }
    /// Creates a predicate from a **slice of [`Value`]s**.
    ///
    /// The value type is inferred from the **first element**; all elements must share
    /// the same [`ValueKind`]. This constructor is best suited for list-based
    /// operations such as [`Operation::IsOneOf`], [`Operation::IsNotOneOf`],
    /// [`Operation::ContainsOneOf`], etc.
    ///
    /// For slices of `T: Into<Value>` see the ergonomic wrapper
    /// [`with_list`](Predicate::with_list).
    ///
    /// # Errors
    ///
    /// - [`Error::EmptyListForOperation`] – `values` is empty.
    /// - [`Error::InvalidOperationForValue`] – `Bool` values cannot be used with
    ///   list operations.
    /// - Any other type-specific or operation-compatibility error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use whereexpr::{Predicate, Operation, Value};
    ///
    /// let values = [Value::String("foo"), Value::String("bar"), Value::String("baz")];
    /// let p = Predicate::with_value_list(Operation::IsOneOf, &values).unwrap();
    ///
    /// // Negated variant: "is not one of"
    /// let p_neg = Predicate::with_value_list(Operation::IsNotOneOf, &values).unwrap();
    /// ```
    pub fn with_value_list(op: Operation, values: &[Value<'_>]) -> Result<Self, Error> {
        if values.is_empty() {
            return Err(Error::EmptyListForOperation(op));
        }
        let kind = values[0].kind();
        let (op, negated) = op.operation_and_negated();
        let predicate = match kind {
            ValueKind::String => PredicateInner::StringPredicate(StringPredicate::with_value_list(op, values)?),
            ValueKind::Path => PredicateInner::PathPredicate(PathPredicate::with_value_list(op, values)?),
            ValueKind::Bytes => todo!(),
            ValueKind::U8 => PredicateInner::U8Predicate(U8Predicate::with_value_list(op, values)?),
            ValueKind::U16 => PredicateInner::U16Predicate(U16Predicate::with_value_list(op, values)?),
            ValueKind::U32 => PredicateInner::U32Predicate(U32Predicate::with_value_list(op, values)?),
            ValueKind::U64 => PredicateInner::U64Predicate(U64Predicate::with_value_list(op, values)?),
            ValueKind::I8 => PredicateInner::I8Predicate(I8Predicate::with_value_list(op, values)?),
            ValueKind::I16 => PredicateInner::I16Predicate(I16Predicate::with_value_list(op, values)?),
            ValueKind::I32 => PredicateInner::I32Predicate(I32Predicate::with_value_list(op, values)?),
            ValueKind::I64 => PredicateInner::I64Predicate(I64Predicate::with_value_list(op, values)?),
            ValueKind::F32 => PredicateInner::F32Predicate(F32Predicate::with_value_list(op, values)?),
            ValueKind::F64 => PredicateInner::F64Predicate(F64Predicate::with_value_list(op, values)?),
            ValueKind::Hash128 => PredicateInner::Hash128Predicate(Hash128Predicate::with_value_list(op, values)?),
            ValueKind::Hash160 => PredicateInner::Hash160Predicate(Hash160Predicate::with_value_list(op, values)?),
            ValueKind::Hash256 => PredicateInner::Hash256Predicate(Hash256Predicate::with_value_list(op, values)?),
            ValueKind::IpAddr => PredicateInner::IpAddrPredicate(IpAddrPredicate::with_value_list(op, values)?),
            ValueKind::DateTime => PredicateInner::DateTimePredicate(DateTimePredicate::with_value_list(op, values)?),
            ValueKind::Bool => return Err(Error::InvalidOperationForValue(op, ValueKind::Bool)),
            ValueKind::None => todo!(),
        };
        Ok(Predicate { predicate, negated })
    }
    /// Creates a predicate from a **string representation of a single value**.
    ///
    /// Because the string carries no inherent type information, the target type must
    /// be supplied explicitly via `value_kind`. The string is parsed into the
    /// corresponding internal representation at construction time, not at
    /// evaluation time.
    ///
    /// The `ignore_case` flag is only meaningful for string-like types
    /// (`ValueKind::String`, `ValueKind::Path`); it is silently ignored for all
    /// other kinds.
    ///
    /// This constructor is used internally by [`Condition::from_str`](crate::Condition::from_str)
    /// during [`ExpressionBuilder::build`](crate::ExpressionBuilder::build), but you
    /// can call it directly when you have a dynamic string value and know the target
    /// type at call site.
    ///
    /// # Errors
    ///
    /// - [`Error::FailToParseValue`] – the string cannot be parsed into `value_kind`.
    /// - [`Error::InvalidOperationForValue`] – the operation is incompatible with
    ///   `value_kind`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use whereexpr::{Predicate, Operation, ValueKind};
    ///
    /// // Parse "42" as a u32 and create a "greater than" predicate
    /// let p = Predicate::with_str(Operation::GreaterThan, "42", ValueKind::U32, false).unwrap();
    ///
    /// // Case-insensitive string equality
    /// let p = Predicate::with_str(Operation::Is, "Alice", ValueKind::String, true).unwrap();
    ///
    /// // Parse will fail if the string does not match the expected type
    /// let err = Predicate::with_str(Operation::Is, "not_a_number", ValueKind::U32, false);
    /// assert!(err.is_err());
    /// ```
    pub fn with_str(op: Operation, value: &str, value_kind: ValueKind, ignore_case: bool) -> Result<Self, Error> {
        let (op, negated) = op.operation_and_negated();
        let predicate = match value_kind {
            ValueKind::String => PredicateInner::StringPredicate(StringPredicate::with_value(op, value, ignore_case)?),
            ValueKind::Path => PredicateInner::PathPredicate(PathPredicate::with_str(op, value, ignore_case)?),
            ValueKind::Bytes => todo!(),
            ValueKind::U8 => PredicateInner::U8Predicate(U8Predicate::with_str(op, value)?),
            ValueKind::U16 => PredicateInner::U16Predicate(U16Predicate::with_str(op, value)?),
            ValueKind::U32 => PredicateInner::U32Predicate(U32Predicate::with_str(op, value)?),
            ValueKind::U64 => PredicateInner::U64Predicate(U64Predicate::with_str(op, value)?),
            ValueKind::I8 => PredicateInner::I8Predicate(I8Predicate::with_str(op, value)?),
            ValueKind::I16 => PredicateInner::I16Predicate(I16Predicate::with_str(op, value)?),
            ValueKind::I32 => PredicateInner::I32Predicate(I32Predicate::with_str(op, value)?),
            ValueKind::I64 => PredicateInner::I64Predicate(I64Predicate::with_str(op, value)?),
            ValueKind::F32 => PredicateInner::F32Predicate(F32Predicate::with_str(op, value)?),
            ValueKind::F64 => PredicateInner::F64Predicate(F64Predicate::with_str(op, value)?),
            ValueKind::Hash128 => PredicateInner::Hash128Predicate(Hash128Predicate::with_str(op, value)?),
            ValueKind::Hash160 => PredicateInner::Hash160Predicate(Hash160Predicate::with_str(op, value)?),
            ValueKind::Hash256 => PredicateInner::Hash256Predicate(Hash256Predicate::with_str(op, value)?),
            ValueKind::IpAddr => PredicateInner::IpAddrPredicate(IpAddrPredicate::with_str(op, value)?),
            ValueKind::DateTime => PredicateInner::DateTimePredicate(DateTimePredicate::with_str(op, value)?),
            ValueKind::Bool => PredicateInner::BoolPredicate(BoolPredicate::with_str(op, value)?),
            ValueKind::None => todo!(),
        };
        Ok(Predicate { predicate, negated })
    }
    /// Creates a predicate from a **slice of string representations of values**.
    ///
    /// Each element of `values` is parsed into `value_kind` at construction time.
    /// This is the multi-value counterpart of [`with_str`](Predicate::with_str) and
    /// is intended for list-based operations such as [`Operation::IsOneOf`],
    /// [`Operation::StartsWithOneOf`], [`Operation::ContainsOneOf`], etc.
    ///
    /// As with `with_str`, `ignore_case` applies only to string-like kinds and is
    /// silently ignored for numeric or other types.
    ///
    /// # Errors
    ///
    /// - [`Error::FailToParseValue`] – any element cannot be parsed into `value_kind`.
    /// - [`Error::InvalidOperationForValue`] – `Bool` values cannot be used with list
    ///   operations, or the chosen operation is otherwise incompatible with the type.
    /// - [`Error::EmptyListForIsOneOf`] / [`Error::EmptyListForGlobREMatch`] – the
    ///   slice is empty for operations that require at least one value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use whereexpr::{Predicate, Operation, ValueKind};
    ///
    /// // "is one of" check against a list of string values (case-insensitive)
    /// let p = Predicate::with_str_list(
    ///     Operation::IsOneOf,
    ///     &["alice", "bob", "carol"],
    ///     ValueKind::String,
    ///     true,
    /// ).unwrap();
    ///
    /// // "is not one of" for integers
    /// let p = Predicate::with_str_list(
    ///     Operation::IsNotOneOf,
    ///     &["0", "1", "2"],
    ///     ValueKind::U32,
    ///     false,
    /// ).unwrap();
    /// ```
    pub fn with_str_list(op: Operation, values: &[&str], value_kind: ValueKind, ignore_case: bool) -> Result<Self, Error> {
        let (op, negated) = op.operation_and_negated();
        let predicate = match value_kind {
            ValueKind::String => PredicateInner::StringPredicate(StringPredicate::with_str_list(op, values, ignore_case)?),
            ValueKind::Path => PredicateInner::PathPredicate(PathPredicate::with_str_list(op, values, ignore_case)?),
            ValueKind::Bytes => todo!(),
            ValueKind::U8 => PredicateInner::U8Predicate(U8Predicate::with_str_list(op, values)?),
            ValueKind::U16 => PredicateInner::U16Predicate(U16Predicate::with_str_list(op, values)?),
            ValueKind::U32 => PredicateInner::U32Predicate(U32Predicate::with_str_list(op, values)?),
            ValueKind::U64 => PredicateInner::U64Predicate(U64Predicate::with_str_list(op, values)?),
            ValueKind::I8 => PredicateInner::I8Predicate(I8Predicate::with_str_list(op, values)?),
            ValueKind::I16 => PredicateInner::I16Predicate(I16Predicate::with_str_list(op, values)?),
            ValueKind::I32 => PredicateInner::I32Predicate(I32Predicate::with_str_list(op, values)?),
            ValueKind::I64 => PredicateInner::I64Predicate(I64Predicate::with_str_list(op, values)?),
            ValueKind::F32 => PredicateInner::F32Predicate(F32Predicate::with_str_list(op, values)?),
            ValueKind::F64 => PredicateInner::F64Predicate(F64Predicate::with_str_list(op, values)?),
            ValueKind::Hash128 => PredicateInner::Hash128Predicate(Hash128Predicate::with_str_list(op, values)?),
            ValueKind::Hash160 => PredicateInner::Hash160Predicate(Hash160Predicate::with_str_list(op, values)?),
            ValueKind::Hash256 => PredicateInner::Hash256Predicate(Hash256Predicate::with_str_list(op, values)?),
            ValueKind::IpAddr => PredicateInner::IpAddrPredicate(IpAddrPredicate::with_str_list(op, values)?),
            ValueKind::DateTime => PredicateInner::DateTimePredicate(DateTimePredicate::with_str_list(op, values)?),
            ValueKind::Bool => return Err(Error::InvalidOperationForValue(op, ValueKind::Bool)),
            ValueKind::None => todo!(),
        };
        Ok(Predicate { predicate, negated })
    }
    pub(crate) fn evaluate(&self, field_value: &Value) -> Option<bool> {
        let result = match (&self.predicate, field_value) {
            // signed integer predicates
            (PredicateInner::I8Predicate(p), Value::I8(v)) => p.evaluate(*v),
            (PredicateInner::I16Predicate(p), Value::I16(v)) => p.evaluate(*v),
            (PredicateInner::I32Predicate(p), Value::I32(v)) => p.evaluate(*v),
            (PredicateInner::I64Predicate(p), Value::I64(v)) => p.evaluate(*v),
            // unsigned integer predicates
            (PredicateInner::U8Predicate(p), Value::U8(v)) => p.evaluate(*v),
            (PredicateInner::U16Predicate(p), Value::U16(v)) => p.evaluate(*v),
            (PredicateInner::U32Predicate(p), Value::U32(v)) => p.evaluate(*v),
            (PredicateInner::U64Predicate(p), Value::U64(v)) => p.evaluate(*v),
            // float predicates
            (PredicateInner::F32Predicate(p), Value::F32(v)) => p.evaluate(*v),
            (PredicateInner::F64Predicate(p), Value::F64(v)) => p.evaluate(*v),
            // string predicates
            (PredicateInner::StringPredicate(p), Value::String(v)) => p.evaluate(*v),
            // path predicates
            (PredicateInner::PathPredicate(p), Value::Path(v)) => p.evaluate(*v),
            // hash predicates
            (PredicateInner::Hash128Predicate(p), Value::Hash128(v)) => p.evaluate(Hash128::new(**v)),
            (PredicateInner::Hash160Predicate(p), Value::Hash160(v)) => p.evaluate(Hash160::new(**v)),
            (PredicateInner::Hash256Predicate(p), Value::Hash256(v)) => p.evaluate(Hash256::new(**v)),

            (PredicateInner::IpAddrPredicate(p), Value::IpAddr(v)) => p.evaluate(*v),
            (PredicateInner::DateTimePredicate(p), Value::DateTime(v)) => p.evaluate(*v),
            (PredicateInner::BoolPredicate(p), Value::Bool(v)) => p.evaluate(*v),
            _ => return None,
        };
        if self.negated {
            Some(!result)
        } else {
            Some(result)
        }
    }
}
