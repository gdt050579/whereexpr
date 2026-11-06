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
/// An `Expression` is built via [`ExpressionBuilder`] or parsed from one rule with
/// [`from_rule_str`](Expression::from_rule_str). It holds a set of named conditions
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

    /// Compiles one rule string into an [`Expression`] for type `T`.
    ///
    /// This is the single-string form of [`ExpressionBuilder`]: the rule names each
    /// condition, gives its predicate, and combines those names in a `where` block.
    ///
    /// ```text
    /// rule adults: Person {
    ///     adult = "age >= 18"
    ///     named = "name is Alice"
    /// } where {
    ///     adult AND named
    /// }
    /// ```
    ///
    /// # Rule syntax
    ///
    /// ```text
    /// rule <name>: <TypeName> {
    ///     <condition-name> = "<condition>"
    /// } where {
    ///     <condition-name> AND <condition-name>
    /// }
    /// ```
    ///
    /// - `<name>` and each `<condition-name>` start with an ASCII letter. After that
    ///   they may contain letters, digits, `_`, and `-`.
    /// - `<TypeName>` is the type the rule applies to. When the `enable_type_check`
    ///   feature is enabled it must equal [`T::TYPE_NAME`](Attributes::TYPE_NAME).
    ///   Without that feature the name is not checked.
    /// - Each condition value is a [`Condition::from_str`](Condition::from_str) string
    ///   inside single quotes (`'...'`), double quotes (`"..."`), or a matching
    ///   triple quote (`'''...'''` or `"""..."""`). The quotes themselves are not
    ///   part of the condition.
    /// - The `where` block combines the condition names with the same operators as
    ///   [`ExpressionBuilder::build`]: `AND`/`&&`, `OR`/`||`, `NOT`/`!`/`~`, and
    ///   parentheses. Whitespace is allowed.
    ///
    /// At least one condition is required. A rule that is empty or only whitespace
    /// is rejected.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] when the rule text is malformed, a condition cannot be
    /// compiled, or the `where` block does not match the declared conditions.
    ///
    /// | Error variant | Cause |
    /// |---|---|
    /// | [`Error::ExpectingRuleKeyword`] | The text does not start with `rule`. |
    /// | [`Error::ExpectingRuleName`] | `rule` is not followed by a name. |
    /// | [`Error::ExpectingColon`] | The rule name is not followed by `:`. |
    /// | [`Error::ExpectingTypeName`] | `:` is not followed by a type name. |
    /// | [`Error::ExpectingOpeningBraceAfterTypeName`] | The type name is not followed by `{`. |
    /// | [`Error::ExpectingKeyName`] | A key-value pair has no key name, or the name does not start with a letter. |
    /// | [`Error::ExpectingEqualSignAfterKeyName`] | A key name is not followed by `=`. |
    /// | [`Error::ExpectingKeyValue`] | `=` is not followed by a value (`key = value`). |
    /// | [`Error::UnterminatedString`] | A condition value is missing its closing quote. |
    /// | [`Error::ExpectingWhereKeyword`] | The condition block is not followed by `where`. |
    /// | [`Error::ExpectingOpeningBraceAfterWhereKeyword`] | `where` is not followed by `{`. |
    /// | [`Error::InvalidCharacterInWhereExpression`] | The `where` block contains a character other than a letter, digit, `_`, `-`, `(`, `)`, `&`, `\|`, `!`, `~`, or whitespace. |
    /// | [`Error::ExpectingClosingBraceAfterWhereKeyword`] | The `where` block is missing its closing `}`. |
    /// | [`Error::TypeMismatch`] | `enable_type_check` is on and `<TypeName>` is not `T::TYPE_NAME`. |
    /// | [`Error::InvalidConditionName`] | A condition name does not start with an ASCII letter. |
    /// | [`Error::DuplicateConditionName`] | The same condition name is declared twice. |
    /// | [`Error::UnknownAttribute`] | A condition uses an attribute `T` does not expose. |
    /// | [`Error::UnknownConditionName`] | The `where` block names a condition that was not declared. |
    /// | [`Error::EmptyExpression`] | The `where` block is empty. |
    /// | condition parse errors | A condition string is malformed. These are the same errors as [`Condition::from_str`](Condition::from_str). |
    ///
    /// # Examples
    ///
    /// ```rust
    /// use whereexpr::{Attributes, AttributeIndex, Value, ValueKind, Expression};
    ///
    /// struct Person { name: String, age: u32 }
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
    /// let expr = Expression::from_rule_str::<Person>(r#"
    ///     rule adults: Person {
    ///         adult = "age >= 18"
    ///         named = "name is Alice"
    ///     } where {
    ///         adult AND named
    ///     }
    /// "#).unwrap();
    ///
    /// let alice = Person { name: "Alice".into(), age: 30 };
    /// let bob = Person { name: "Bob".into(), age: 40 };
    /// assert!(expr.matches(&alice));
    /// assert!(!expr.matches(&bob));
    /// ```
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
        Self::compile_parsed_rule::<T>(&parser)
    }

    /// Parses every rule in `text` and returns an iterator over the rules written for `T`.
    ///
    /// Each item is `Ok((rule_name, expression))`, or `Err` for a rule that could not be
    /// compiled. The rules use the same format as [`from_rule_str`](Expression::from_rule_str);
    /// any amount of whitespace may separate them.
    ///
    /// - **Rules for other types are skipped.** A rule whose type name is not
    ///   [`T::TYPE_NAME`](Attributes::TYPE_NAME) is not compiled and not returned, so one
    ///   text can hold rules for several types. This check is always done, with or without
    ///   the `enable_type_check` feature.
    /// - **A rule that fails to compile** (an unknown attribute, a malformed condition, an
    ///   unknown condition name in `where`, ...) is returned as `Err`, and iteration
    ///   continues with the next rule.
    /// - **A syntax error** (a missing `rule`, `{`, `}`, `where`, an unclosed string, ...)
    ///   is returned as `Err` once, and then the iterator ends: after a syntax error the
    ///   position of the next rule is not known.
    /// - **Duplicate rule names are allowed.** Every rule is returned; when collecting into
    ///   a map, a later rule replaces an earlier one with the same name.
    ///
    /// Empty or whitespace-only text yields no items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use whereexpr::{Attributes, AttributeIndex, Value, ValueKind, Expression};
    ///
    /// struct Person { name: String, age: u32 }
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
    /// let text = r#"
    ///     rule adults: Person { adult = "age >= 18" } where { adult }
    ///     rule alice:  Person { named = "name is Alice" } where { named }
    ///     rule cheap:  Book   { low = "price < 10" } where { low }
    /// "#;
    ///
    /// // all rules for Person, in order (the Book rule is skipped)
    /// let rules: Vec<(String, Expression)> =
    ///     Expression::parse_rules::<Person>(text).collect::<Result<_, _>>().unwrap();
    /// assert_eq!(rules.len(), 2);
    /// assert_eq!(rules[0].0, "adults");
    ///
    /// // or keyed by rule name
    /// let by_name: HashMap<String, Expression> =
    ///     Expression::parse_rules::<Person>(text).collect::<Result<_, _>>().unwrap();
    /// let bob = Person { name: "Bob".into(), age: 40 };
    /// assert!(by_name["adults"].matches(&bob));
    /// assert!(!by_name["alice"].matches(&bob));
    ///
    /// // or one by one, keeping the rules that compile
    /// for item in Expression::parse_rules::<Person>(text) {
    ///     match item {
    ///         Ok((name, expr)) => println!("{name}: {}", expr.matches(&bob)),
    ///         Err(e) => eprintln!("skipping a rule: {e:?}"),
    ///     }
    /// }
    /// ```
    pub fn parse_rules<T: Attributes>(text: &str) -> Rules<T> {
        Rules {
            parser: RuleParser::new(text),
            done: false,
            _type: PhantomData,
        }
    }

    /// Builds an [`Expression`] from the rule the parser has just read.
    fn compile_parsed_rule<T: Attributes>(parser: &RuleParser) -> Result<Expression, Error> {
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

/// Iterator over the rules in a text, returned by [`Expression::parse_rules`].
///
/// Yields `Ok((rule_name, expression))` for each rule written for `T`, or `Err` for a
/// rule that could not be parsed or compiled. See [`Expression::parse_rules`] for the
/// details.
pub struct Rules<T: Attributes> {
    parser: RuleParser,
    done: bool,
    _type: PhantomData<T>,
}

impl<T: Attributes> Iterator for Rules<T> {
    type Item = Result<(String, Expression), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.done {
            match self.parser.parse_next() {
                Ok(false) => self.done = true,
                Err(e) => {
                    // the position of the next rule is unknown after a syntax error
                    self.done = true;
                    return Some(Err(e));
                }
                Ok(true) => {
                    if self.parser.type_name() != Some(T::TYPE_NAME) {
                        continue; // a rule for another type
                    }
                    let name = self.parser.rule_name().unwrap_or_default().to_string();
                    return Some(Expression::compile_parsed_rule::<T>(&self.parser).map(|expr| (name, expr)));
                }
            }
        }
        None
    }
}

impl<T: Attributes> std::iter::FusedIterator for Rules<T> {}

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
