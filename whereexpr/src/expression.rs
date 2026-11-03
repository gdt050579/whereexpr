use crate::rule_parser::RuleParser;

use super::Attributes;
use super::CompiledCondition;
use super::Condition;
use super::ConditionAttribute;
use super::ConditionList;
use super::ConditionPredicate;
use super::Error;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Composition {
    And,
    Or,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum EvaluationNode {
    Condition(u16),
    Group {
        composition: Composition,
        children: Vec<EvaluationNode>,
    },
    Not {
        child: Box<EvaluationNode>,
    },
}

impl EvaluationNode {
    pub(super) fn evaluate<T: Attributes>(&self, obj: &T, expression: &Expression) -> Option<bool> {
        match self {
            EvaluationNode::Condition(index) => {
                expression.conditions.get(*index).unwrap().evaluate(obj)
            }

            EvaluationNode::Not { child } => {
                child.evaluate(obj, expression).map(|v| !v)
            }

            EvaluationNode::Group { composition, children } => match composition {
                Composition::And => {
                    let mut unknown = false;
                    for child in children {
                        match child.evaluate(obj, expression) {
                            Some(false) => return Some(false),
                            Some(true) => {}
                            None => unknown = true,
                        }
                    }
                    if unknown { None } else { Some(true) }
                }
                Composition::Or => {
                    let mut unknown = false;
                    for child in children {
                        match child.evaluate(obj, expression) {
                            Some(true) => return Some(true),
                            Some(false) => {}
                            None => unknown = true,
                        }
                    }
                    if unknown { None } else { Some(false) }
                }
            },
        }
    }
}

/// A compiled, type-safe boolean expression that can be evaluated against objects
/// implementing [`Attributes`].
///
/// An `Expression` is built via [`ExpressionBuilder`] and holds a set of named conditions
/// combined by a boolean expression string (e.g. `"cond_a && cond_b || !cond_c"`).
/// It is tied at construction time to a specific type `T`. Evaluating it against a different
/// type is a programming error and panics (checked when the `enable_type_check` feature is enabled).
///
/// There are two ways to evaluate it:
/// - [`matches`](Expression::matches) returns a `bool` and treats an unknown result as no match.
/// - [`evaluate`](Expression::evaluate) returns the full three-valued result as an `Option<bool>`.
pub struct Expression {
    root: EvaluationNode,
    #[cfg(feature = "enable_type_check")]
    type_id: u64,
    #[cfg(feature = "enable_type_check")]
    type_name: &'static str,
    pub(super) conditions: ConditionList,
}

impl Expression {
    /// Evaluates the expression against the given object and returns `true` if it matches.
    ///
    /// Conditions are combined with three-valued (Kleene) logic: a condition whose attribute
    /// is missing (`Attributes::get` returns `None`) is *unknown*, and an unknown operand only
    /// affects the result when the other operands cannot decide it on their own
    /// (`false && unknown` is `false`, `true || unknown` is `true`).
    ///
    /// `matches` returns `true` only when the result is definitely true. An unknown result
    /// counts as no match and returns `false`, the same way a SQL `WHERE` clause drops rows
    /// whose condition evaluates to `NULL`. Use [`evaluate`](Expression::evaluate) if you
    /// need to tell "false" and "unknown" apart.
    ///
    /// # Panics
    ///
    /// Panics if `T` is not the type the expression was built for. This is a programming
    /// error, not a data problem. The check is only performed when the `enable_type_check`
    /// feature is enabled; without it, a mismatched type is not detected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use whereexpr::{Attributes, AttributeIndex, Value, ValueKind, Condition, ExpressionBuilder};
    ///
    /// struct Person { name: String, age: Option<u32> }
    ///
    /// impl Person {
    ///     const NAME: AttributeIndex = AttributeIndex::new(0);
    ///     const AGE:  AttributeIndex = AttributeIndex::new(1);
    /// }
    ///
    /// impl Attributes for Person {
    ///     const TYPE_ID: u64 = 0x517652f2; // unique ID for Person type (a hash or other unique identifier)
    ///     const TYPE_NAME: &'static str = "Person";
    ///     fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
    ///         match idx {
    ///             Self::NAME => Some(Value::String(&self.name)),
    ///             Self::AGE  => self.age.map(Value::U32), // `None` when the age is not known
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
    ///     .add("is_alice", Condition::from_str("name is Alice"))
    ///     .add("is_adult", Condition::from_str("age >= 18"))
    ///     .build("is_alice && is_adult")
    ///     .unwrap();
    ///
    /// let alice = Person { name: "Alice".into(), age: Some(30) };
    /// let bob   = Person { name: "Bob".into(),   age: Some(25) };
    /// assert!(expr.matches(&alice));
    /// assert!(!expr.matches(&bob));
    ///
    /// // Age unknown, but the name already makes the AND false.
    /// let bob_no_age = Person { name: "Bob".into(), age: None };
    /// assert!(!expr.matches(&bob_no_age));
    /// assert_eq!(expr.evaluate(&bob_no_age), Some(false));
    ///
    /// // Age unknown and the name is Alice, so the result is unknown: no match.
    /// let alice_no_age = Person { name: "Alice".into(), age: None };
    /// assert!(!expr.matches(&alice_no_age));
    /// assert_eq!(expr.evaluate(&alice_no_age), None);
    /// ```
    #[inline(always)]
    pub fn matches<T: Attributes>(&self, obj: &T) -> bool {
        #[cfg(feature = "enable_type_check")]
        if T::TYPE_ID != self.type_id {
            panic!(
                "object type mismatch (this expression is for type '{}', but the object you are trying to match is of type '{}')",
                self.type_name,
                T::TYPE_NAME
            );
        }
        self.root.evaluate(obj, self) == Some(true)
    }

    /// Evaluates the expression against the given object and returns the three-valued result.
    ///
    /// Conditions are combined with three-valued (Kleene) logic: a condition whose attribute
    /// is missing (`Attributes::get` returns `None`) is *unknown*, and an unknown operand only
    /// affects the result when the other operands cannot decide it on their own
    /// (`false && unknown` is `false`, `true || unknown` is `true`).
    ///
    /// Use this instead of [`matches`](Expression::matches) when you need to tell "does not
    /// match" apart from "cannot tell". `matches(obj)` is equivalent to
    /// `evaluate(obj) == Some(true)`.
    ///
    /// # Return value
    ///
    /// - `Some(true)`  – the object definitely satisfies the expression.
    /// - `Some(false)` – the object definitely does not satisfy the expression.
    /// - `None`        – the result is unknown: it depends on attributes the object does not have.
    ///
    /// # Panics
    ///
    /// Panics if `T` is not the type the expression was built for. This is a programming
    /// error, not a data problem. The check is only performed when the `enable_type_check`
    /// feature is enabled; without it, a mismatched type is not detected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use whereexpr::{Attributes, AttributeIndex, Value, ValueKind, Condition, ExpressionBuilder};
    ///
    /// struct Score { value: Option<u32> }
    ///
    /// impl Score {
    ///     const VALUE: AttributeIndex = AttributeIndex::new(0);
    /// }
    ///
    /// impl Attributes for Score {
    ///     const TYPE_ID: u64 = 0x517652f2; // unique ID for Score type (a hash or other unique identifier)
    ///     const TYPE_NAME: &'static str = "Score";
    ///     fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
    ///         match idx {
    ///             Self::VALUE => self.value.map(Value::U32), // `None` when there is no score
    ///             _           => None,
    ///         }
    ///     }
    ///     fn kind(idx: AttributeIndex) -> Option<ValueKind> {
    ///         match idx { Self::VALUE => Some(ValueKind::U32), _ => None }
    ///     }
    ///     fn index(name: &str) -> Option<AttributeIndex> {
    ///         match name { "value" => Some(Self::VALUE), _ => None }
    ///     }
    /// }
    ///
    /// let expr = ExpressionBuilder::<Score>::new()
    ///     .add("high", Condition::from_str("value > 90"))
    ///     .build("high")
    ///     .unwrap();
    ///
    /// assert_eq!(expr.evaluate(&Score { value: Some(95) }), Some(true));
    /// assert_eq!(expr.evaluate(&Score { value: Some(40) }), Some(false));
    ///
    /// // No score: the result is unknown, and `matches` reports it as no match.
    /// let missing = Score { value: None };
    /// assert_eq!(expr.evaluate(&missing), None);
    /// assert!(!expr.matches(&missing));
    /// ```
    #[inline(always)]
    pub fn evaluate<T: Attributes>(&self, obj: &T) -> Option<bool> {
        #[cfg(feature = "enable_type_check")]
        if T::TYPE_ID != self.type_id {
            panic!(
                "object type mismatch (this expression is for type '{}', but the object you are trying to match is of type '{}')",
                self.type_name,
                T::TYPE_NAME
            );
        }
        self.root.evaluate(obj, self)
    }
    pub fn from_rule_str<T: Attributes>(rule: &str) -> Result<Expression, Error> {
        let mut parser = RuleParser::new(rule);
        let res = parser.parse_next()?;
        if !res {
            return Err(Error::ExpectingRuleKeyword(0, 1, rule.to_string()));
        }
        #[cfg(feature = "enable_type_check")]
        if T::TYPE_NAME != parser.type_name().unwrap() {
            return Err(Error::TypeMismatch(parser.type_name().unwrap().to_string(), T::TYPE_NAME.to_string()));
        }
        let mut builder = ExpressionBuilder::<T>::new();
        for i in 0..parser.conditions_count() {
            let name = parser.condition_name(i).unwrap();
            let value = parser.condition_value(i).unwrap();
            let condition = Condition::from_str(value);
            builder = builder.add(name, condition);
        }
        builder.build(parser.expr().unwrap())
    }
}

/// A builder for constructing a type-safe [`Expression`].
///
/// Use [`ExpressionBuilder::new`] to create a builder, register one or more named
/// [`Condition`]s with [`add`](ExpressionBuilder::add), and then call
/// [`build`](ExpressionBuilder::build) with a boolean expression string that
/// combines those condition names.
///
/// # Boolean expression syntax
///
/// The expression string passed to `build` supports:
/// - `&&` / `AND` — logical AND
/// - `||` / `OR`  — logical OR
/// - `!`  / `NOT` — logical negation
/// - `(` `)` — grouping
///
/// Condition names referenced in the expression must exactly match the names
/// provided to [`add`](ExpressionBuilder::add).
///
/// # Example
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
///     const TYPE_ID: u64 = 0xffff1201; // unique ID for Person type (a hash or other unique identifier)
///     const TYPE_NAME: &'static str = "Person";
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
///     .add("is_alice", Condition::from_str("name is Alice"))
///     .add("is_adult", Condition::from_str("age >= 18"))
///     .build("is_alice && is_adult")
///     .unwrap();
/// ```
pub struct ExpressionBuilder<T: Attributes> {
    conditions: Vec<(String, Condition)>,
    _phantom: PhantomData<T>,
}

impl<T: Attributes> ExpressionBuilder<T> {
    /// Creates a new, empty `ExpressionBuilder` for the type `T`.
    ///
    /// At least one condition must be added with [`add`](ExpressionBuilder::add) before
    /// calling [`build`](ExpressionBuilder::build), otherwise `build` will return
    /// [`Error::EmptyConditionList`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use whereexpr::{Attributes, AttributeIndex, Value, ValueKind, ExpressionBuilder};
    ///
    /// struct Item { price: f64 }
    ///
    /// impl Attributes for Item {
    ///     const TYPE_ID: u64 = 0x1123F78; // unique ID for Item type (a hash or other unique identifier)
    ///     const TYPE_NAME: &'static str = "Item";
    ///     fn get(&self, _: AttributeIndex) -> Option<Value<'_>> { Some(Value::F64(self.price)) }
    ///     fn kind(_: AttributeIndex) -> Option<ValueKind> { Some(ValueKind::F64) }
    ///     fn index(_: &str) -> Option<AttributeIndex> { Some(AttributeIndex::new(0)) }
    /// }
    ///
    /// let builder = ExpressionBuilder::<Item>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            conditions: Vec::with_capacity(4),
            _phantom: PhantomData,
        }
    }

    /// Registers a named condition with this builder.
    ///
    /// The `name` is used to reference the condition in the boolean expression string
    /// passed to [`build`](ExpressionBuilder::build). Names must:
    /// - Be non-empty.
    /// - Start with an ASCII letter (`a-z`, `A-Z`).
    /// - Contain only ASCII letters, digits, `-`, or `_` after the first character.
    ///
    /// If these rules are violated, `build` will return [`Error::InvalidConditionName`].
    /// Registering the same name twice causes `build` to return
    /// [`Error::DuplicateConditionName`].
    ///
    /// This method consumes and returns `self`, enabling a fluent builder chain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use whereexpr::{Attributes, AttributeIndex, Value, ValueKind, Condition, ExpressionBuilder};
    ///
    /// struct Product { category: String, price: f64 }
    ///
    /// impl Product {
    ///     const CATEGORY: AttributeIndex = AttributeIndex::new(0);
    ///     const PRICE:    AttributeIndex = AttributeIndex::new(1);
    /// }
    ///
    /// impl Attributes for Product {
    ///     const TYPE_ID: u64 = 0x12345678; // unique ID for Product type (a hash or other unique identifier)
    ///     const TYPE_NAME: &'static str = "Product";
    ///     fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
    ///         match idx {
    ///             Self::CATEGORY => Some(Value::String(&self.category)),
    ///             Self::PRICE    => Some(Value::F64(self.price)),
    ///             _              => None,
    ///         }
    ///     }
    ///     fn kind(idx: AttributeIndex) -> Option<ValueKind> {
    ///         match idx {
    ///             Self::CATEGORY => Some(ValueKind::String),
    ///             Self::PRICE    => Some(ValueKind::F64),
    ///             _              => None,
    ///         }
    ///     }
    ///     fn index(name: &str) -> Option<AttributeIndex> {
    ///         match name {
    ///             "category" => Some(Self::CATEGORY),
    ///             "price"    => Some(Self::PRICE),
    ///             _          => None,
    ///         }
    ///     }
    /// }
    ///
    /// let expr = ExpressionBuilder::<Product>::new()
    ///     .add("is_book",      Condition::from_str("category is book"))
    ///     .add("is_expensive", Condition::from_str("price > 50"))
    ///     .build("is_book && is_expensive")
    ///     .unwrap();
    /// ```
    pub fn add(mut self, name: &str, condition: Condition) -> Self {
        self.conditions.push((name.to_string(), condition));
        self
    }

    /// Compiles all registered conditions and the boolean expression string into a
    /// reusable [`Expression`].
    ///
    /// # Parameters
    ///
    /// - `expr` – A boolean expression string combining the named conditions registered
    ///   via [`add`](ExpressionBuilder::add). Supported operators: `&&`/`AND`,
    ///   `||`/`OR`, `!`/`NOT`, and parentheses for grouping.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if any of the following occur:
    ///
    /// | Error variant | Cause |
    /// |---|---|
    /// | [`Error::EmptyConditionList`] | No conditions were added before calling `build`. |
    /// | [`Error::InvalidConditionName`] | A condition name violates naming rules. |
    /// | [`Error::DuplicateConditionName`] | The same name was registered more than once. |
    /// | [`Error::UnknownAttribute`] | A condition references an attribute name not exposed by `T`. |
    /// | [`Error::UnknownConditionName`] | The expression string references a name not registered via `add`. |
    /// | parse errors | The expression string or a condition string is malformed. |
    ///
    /// # Examples
    ///
    /// Basic usage with `Condition::from_str` (attribute resolved from the string):
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
    ///     const TYPE_ID: u64 = 0xC2DF0123; // unique ID for Person type (a hash or other unique identifier)
    ///     const TYPE_NAME: &'static str = "Person";
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
    /// // Matches people named "John" or "Jane" who are older than 25
    /// let expr = ExpressionBuilder::<Person>::new()
    ///     .add("named", Condition::from_str("name is-one-of [John, Jane]"))
    ///     .add("older", Condition::from_str("age > 25"))
    ///     .build("named && older")
    ///     .unwrap();
    ///
    /// let john = Person { name: "John".into(), age: 30 };
    /// assert!(expr.matches(&john));
    ///
    /// // With negation and grouping
    /// let expr2 = ExpressionBuilder::<Person>::new()
    ///     .add("is_john",  Condition::from_str("name is John"))
    ///     .add("is_young", Condition::from_str("age < 18"))
    ///     .build("is_john && !is_young")
    ///     .unwrap();
    ///
    /// assert!(expr2.matches(&john));
    /// ```
    ///
    /// Handling build errors:
    ///
    /// ```rust
    /// use whereexpr::{Attributes, AttributeIndex, Value, ValueKind, Condition, ExpressionBuilder, Error};
    ///
    /// struct Item { name: String }
    ///
    /// impl Item {
    ///     const NAME: AttributeIndex = AttributeIndex::new(0);
    /// }
    ///
    /// impl Attributes for Item {
    ///     const TYPE_ID: u64 = 0x517652f2; // unique ID for Item type (a hash or other unique identifier)
    ///     const TYPE_NAME: &'static str = "Item";
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
    /// // Referencing an unknown condition name in the expression string
    /// let result = ExpressionBuilder::<Item>::new()
    ///     .add("named", Condition::from_str("name is Widget"))
    ///     .build("named && typo_name");
    ///
    /// assert!(matches!(result, Err(Error::UnknownConditionName(..))));
    /// ```
    pub fn build(self, expr: &str) -> Result<Expression, Error> {
        // build the conditions list
        if self.conditions.is_empty() {
            return Err(Error::EmptyConditionList);
        }
        let mut clist = ConditionList::with_capacity(self.conditions.len());
        for (name, condition) in self.conditions {
            if !Self::is_valid_name(&name) {
                return Err(Error::InvalidConditionName(name));
            }
            let attr_index = match condition.attribute {
                ConditionAttribute::Name(attr_name) => T::index(&attr_name).ok_or(Error::UnknownAttribute(attr_name, name.clone()))?,
                ConditionAttribute::Index(index) => index,
            };
            let (attr_index, predicate) = match condition.predicate {
                ConditionPredicate::Resolved(p) => (attr_index, p),
                ConditionPredicate::Error(e) => return Err(e),
                ConditionPredicate::Raw(expr) => Condition::parse::<T>(&expr, &name)?,
            };
            let compiled_condition = CompiledCondition::new(attr_index, predicate);
            if !clist.add(&name, compiled_condition) {
                return Err(Error::DuplicateConditionName(name));
            }
        }
        let evaluation_node = crate::expr_parser::parse(expr, &clist)?;
        Ok(Expression {
            #[cfg(feature = "enable_type_check")]
            type_id: T::TYPE_ID,
            #[cfg(feature = "enable_type_check")]
            type_name: T::TYPE_NAME,
            root: evaluation_node,
            conditions: clist,
        })
    }

    fn is_valid_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let mut first = true;
        for c in name.chars() {
            if first {
                if !c.is_ascii_alphabetic() {
                    return false;
                }
                first = false;
            } else if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
                return false;
            }
        }
        true
    }
}
