use super::AttributeIndex;
use super::Attributes;
use super::Error;
use super::Predicate;

pub(super) struct CompiledCondition {
    attr_index: AttributeIndex,
    predicate: Predicate,
}

impl CompiledCondition {
    #[inline(always)]
    pub(super) fn new(attr_index: AttributeIndex, predicate: Predicate) -> Self {
        Self { attr_index, predicate }
    }
    pub(super) fn evaluate<T: Attributes>(&self, obj: &T) -> Option<bool>{
        if let Some(field_value) = obj.get(self.attr_index) {
            self.predicate.evaluate(&field_value)
        } else {
            None
        }
    }
}

/// A single condition that maps one attribute of a type `T` to a [`Predicate`].
///
/// A `Condition` is registered with [`ExpressionBuilder::add`] under a name and is
/// later referenced by that name in the boolean expression string passed to
/// [`ExpressionBuilder::build`].
///
/// There are five constructors, grouped by two axes:
///
/// | | Attribute identified by **name** | Attribute identified by **index** |
/// |---|---|---|
/// | **Predicate already built** | [`new`](Condition::new) | [`with_index`](Condition::with_index) |
/// | **Predicate is a `Result`** (error deferred to `build`) | [`try_new`](Condition::try_new) | [`try_with_index`](Condition::try_with_index) |
/// | **Full expression string** (parsed lazily at `build`) | [`from_str`](Condition::from_str) | — |
pub struct Condition {
    pub(crate) attribute: ConditionAttribute,
    pub(crate) predicate: ConditionPredicate,
}

pub(crate) enum ConditionAttribute {
    Name(String),
    Index(AttributeIndex),
}

pub(crate) enum ConditionPredicate {
    Resolved(Predicate),
    Raw(String),
    Error(Error),
}

impl Condition {
    /// Creates a condition from an **attribute name** and a pre-built [`Predicate`].
    ///
    /// The `attribute` string is resolved to an [`AttributeIndex`] when
    /// [`ExpressionBuilder::build`] is called. If the name is not recognised by
    /// the target type `T`, `build` returns [`Error::UnknownAttribute`].
    ///
    /// Prefer [`from_str`](Condition::from_str) when you want to write the entire
    /// condition as a human-readable string. Use `new` when you already have a
    /// [`Predicate`] built programmatically and know the attribute name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use whereexpr::{Condition, Predicate, Operation, Value};
    ///
    /// // Build a predicate that checks whether a string equals "Alice"
    /// let predicate = Predicate::with_value(Operation::Is, Value::String("Alice")).unwrap();
    /// let condition = Condition::new("name", predicate);
    /// ```
    pub fn new(attribute: &str, predicate: Predicate) -> Self {
        Self {
            attribute: ConditionAttribute::Name(attribute.to_string()),
            predicate: ConditionPredicate::Resolved(predicate),
        }
    }

    /// Creates a condition from an **[`AttributeIndex`]** and a pre-built [`Predicate`].
    ///
    /// This is the index-based counterpart of [`new`](Condition::new). Because the
    /// attribute is identified by its compile-time index rather than a string name,
    /// no name-to-index resolution is performed at build time, making it slightly
    /// more efficient and suitable for fully programmatic use cases.
    ///
    /// # Example
    ///
    /// ```rust
    /// use whereexpr::{AttributeIndex, Condition, Predicate, Operation, Value};
    ///
    /// // Suppose attribute index 2 corresponds to an "age" u32 field
    /// const AGE: AttributeIndex = AttributeIndex::new(2);
    ///
    /// let predicate = Predicate::with_value(Operation::GreaterThan, Value::U32(18)).unwrap();
    /// let condition = Condition::with_index(AGE, predicate);
    /// ```
    pub fn with_index(index: AttributeIndex, predicate: Predicate) -> Self {
        Self {
            attribute: ConditionAttribute::Index(index),
            predicate: ConditionPredicate::Resolved(predicate),
        }
    }

    /// Creates a condition from an **attribute name** and a `Result<Predicate, Error>`.
    ///
    /// This is a convenience wrapper around [`new`](Condition::new) for situations
    /// where predicate construction may fail (e.g. when calling fallible
    /// [`Predicate`] constructors). Any [`Error`] stored inside the `Result` is
    /// **not returned immediately**; instead it is carried along and surfaced when
    /// [`ExpressionBuilder::build`] is called. This allows the builder chain to
    /// remain fluent even when individual predicates are constructed fallibly.
    ///
    /// # Example
    ///
    /// ```rust
    /// use whereexpr::{Condition, Predicate, Operation, Value, ExpressionBuilder,
    ///                  Attributes, AttributeIndex, ValueKind};
    ///
    /// struct Item { name: String }
    ///
    /// impl Item {
    ///     const NAME: AttributeIndex = AttributeIndex::new(0);
    /// }
    ///
    /// impl Attributes for Item {
    ///     fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
    ///         match idx { Self::NAME => Some(Value::String(&self.name)), _ => None }
    ///     }
    ///     fn kind(idx: AttributeIndex) -> Option<ValueKind> {
    ///         match idx { Self::NAME => Some(ValueKind::String), _ => None }
    ///     }
    ///     fn index(name: &str) -> Option<AttributeIndex> {
    ///         match name { "name" => Some(Self::NAME), _ => None }
    ///     }
    /// }
    ///
    /// // The predicate result is Ok here, but the error would propagate to build() if Err
    /// let predicate_result = Predicate::with_value(Operation::Is, Value::String("Widget"));
    /// let condition = Condition::try_new("name", predicate_result);
    ///
    /// let result = ExpressionBuilder::<Item>::new()
    ///     .add("named", condition)
    ///     .build("named");
    ///
    /// assert!(result.is_ok());
    /// ```
    pub fn try_new(attribute: &str, predicate: Result<Predicate, Error>) -> Self {
        Self {
            attribute: ConditionAttribute::Name(attribute.to_string()),
            predicate: match predicate {
                Ok(p) => ConditionPredicate::Resolved(p),
                Err(e) => ConditionPredicate::Error(e),
            },
        }
    }

    /// Creates a condition from an **[`AttributeIndex`]** and a `Result<Predicate, Error>`.
    ///
    /// This is the index-based counterpart of [`try_new`](Condition::try_new). Any
    /// [`Error`] inside the `Result` is deferred and surfaced when
    /// [`ExpressionBuilder::build`] is called, keeping the builder chain fluent.
    ///
    /// # Example
    ///
    /// ```rust
    /// use whereexpr::{AttributeIndex, Condition, Predicate, Operation, Value,
    ///                  ExpressionBuilder, Attributes, ValueKind};
    ///
    /// struct Sensor { reading: f64 }
    ///
    /// impl Sensor {
    ///     const READING: AttributeIndex = AttributeIndex::new(0);
    /// }
    ///
    /// impl Attributes for Sensor {
    ///     fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
    ///         match idx { Self::READING => Some(Value::F64(self.reading)), _ => None }
    ///     }
    ///     fn kind(idx: AttributeIndex) -> Option<ValueKind> {
    ///         match idx { Self::READING => Some(ValueKind::F64), _ => None }
    ///     }
    ///     fn index(name: &str) -> Option<AttributeIndex> {
    ///         match name { "reading" => Some(Self::READING), _ => None }
    ///     }
    /// }
    ///
    /// let predicate_result = Predicate::with_value(Operation::LessThan, Value::F64(100.0));
    /// let condition = Condition::try_with_index(Sensor::READING, predicate_result);
    ///
    /// let expr = ExpressionBuilder::<Sensor>::new()
    ///     .add("below_limit", condition)
    ///     .build("below_limit")
    ///     .unwrap();
    ///
    /// assert!(expr.matches(&Sensor { reading: 42.5 }));
    /// ```
    pub fn try_with_index(index: AttributeIndex, predicate: Result<Predicate, Error>) -> Self {
        Self {
            attribute: ConditionAttribute::Index(index),
            predicate: match predicate {
                Ok(p) => ConditionPredicate::Resolved(p),
                Err(e) => ConditionPredicate::Error(e),
            },
        }
    }

    /// Creates a condition from a **human-readable condition expression string**.
    ///
    /// The string must follow the format `"<attribute> <operation> <value>"`, for
    /// example `"age > 30"` or `"name is-one-of [Alice, Bob] {ignore-case}"`.
    /// Parsing is **deferred** to [`ExpressionBuilder::build`], at which point both
    /// the attribute name and the value are resolved against the target type `T`.
    ///
    /// This is the most ergonomic constructor and is the one used in most examples
    /// throughout this crate.
    ///
    /// # Condition string syntax
    ///
    /// ```text
    /// <attribute> <operation> <value> [<modifiers>]
    /// ```
    ///
    /// - `<attribute>` – the attribute name as exposed by `T::index`.
    /// - `<operation>` – one of: `is`, `is-not`, `is-one-of`, `is-not-one-of`,
    ///   `starts-with`, `ends-with`, `contains`, `glob-re-match`, `>`, `>=`, `<`,
    ///   `<=`, `in-range`, `not-in-range`, and their negated counterparts.
    /// - `<value>` – a single value or a bracketed list `[val1, val2, ...]`.
    /// - `<modifiers>` – optional, e.g. `{ignore-case}` for case-insensitive string
    ///   matching.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use whereexpr::{Attributes, AttributeIndex, Value, ValueKind, Condition, ExpressionBuilder};
    ///
    /// struct Person { name: String, age: u32 }
    ///
    /// impl Person {
    ///     const NAME: AttributeIndex = AttributeIndex::new(0);
    ///     const AGE:  AttributeIndex = AttributeIndex::new(1);
    /// }
    ///
    /// impl Attributes for Person {
    ///     fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
    ///         match idx {
    ///             Self::NAME => Some(Value::String(&self.name)),
    ///             Self::AGE  => Some(Value::U32(self.age)),
    ///             _          => None,
    ///         }
    ///     }
    ///     fn kind(idx: AttributeIndex) -> Option<ValueKind> {
    ///         match idx {
    ///             Self::NAME => Some(ValueKind::String),
    ///             Self::AGE  => Some(ValueKind::U32),
    ///             _          => None,
    ///         }
    ///     }
    ///     fn index(name: &str) -> Option<AttributeIndex> {
    ///         match name {
    ///             "name" => Some(Self::NAME),
    ///             "age"  => Some(Self::AGE),
    ///             _      => None,
    ///         }
    ///     }
    /// }
    ///
    /// let expr = ExpressionBuilder::<Person>::new()
    ///     // exact match
    ///     .add("is_john",   Condition::from_str("name is John"))
    ///     // case-insensitive membership test
    ///     .add("known_surname", Condition::from_str("name is-one-of [Doe, Smith] {ignore-case}"))
    ///     // numeric comparison
    ///     .add("is_adult",  Condition::from_str("age >= 18"))
    ///     // numeric range (inclusive on both ends)
    ///     .add("middle_age", Condition::from_str("age in-range [30, 50]"))
    ///     .build("is_john && is_adult")
    ///     .unwrap();
    ///
    /// let john = Person { name: "John".into(), age: 30 };
    /// assert!(expr.matches(&john));
    /// ```
    pub fn from_str(expr: &str) -> Self {
        Self {
            attribute: ConditionAttribute::Index(AttributeIndex::new(0)),
            predicate: ConditionPredicate::Raw(expr.to_string()),
        }
    }

    pub(crate) fn parse<T: Attributes>(expr: &str, cond_name: &str) -> Result<(AttributeIndex, Predicate), Error> {
        let (attr_name, pos_operation) = crate::cond_parser::attribute::parse(expr)?;
        let attr_index = T::index(&attr_name).ok_or(Error::UnknownAttribute(attr_name.to_string(), cond_name.to_string()))?;
        let kind = T::kind(attr_index).ok_or(Error::UnknownAttribute(attr_name.to_string(), cond_name.to_string()))?;
        let (modifiers, pos_modifiers) = crate::cond_parser::modifiers::parse(expr)?;
        let (operation, pos_value) = crate::cond_parser::operation::parse(expr, pos_operation, pos_modifiers)?;
        let mut copy_buffer = String::new();
        let spans = crate::cond_parser::values::parse(expr, pos_value, pos_modifiers, &mut copy_buffer)?;
        match spans {
            crate::cond_parser::values::ParsedValue::Single(span) => {
                let value = span.as_slice(expr, &copy_buffer);
                let predicate = Predicate::with_str(operation, value, kind, modifiers.ignore_case)?;
                Ok((attr_index, predicate))
            }
            crate::cond_parser::values::ParsedValue::List(spans) => {
                let mut values = Vec::with_capacity(spans.len());
                for span in spans {
                    values.push(span.as_slice(expr, &copy_buffer));
                }
                let predicate = Predicate::with_str_list(operation, &values, kind, modifiers.ignore_case)?;
                Ok((attr_index, predicate))
            }
        }
    }
}
