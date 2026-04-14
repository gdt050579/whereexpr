//! # whereexpr
//!
//! A library for building and evaluating **type-safe, compiled boolean filter
//! expressions** over arbitrary Rust structs.
//!
//! Expressions are written as human-readable strings — e.g.
//! `"age > 30 && name is-one-of [Alice, Bob]"` — parsed once at build time,
//! and then evaluated at zero-allocation cost against any number of objects.
//!
//! ---
//!
//! ## Quick start
//!
//! ### 1. Implement [`Attributes`] for your type
//!
//! Declare one [`AttributeIndex`] constant per field and implement the three
//! trait methods:
//!
//! ```rust
//! use whereexpr::{Attributes, AttributeIndex, Value, ValueKind};
//!
//! struct Person {
//!     name: String,
//!     age:  u32,
//! }
//!
//! impl Person {
//!     const NAME: AttributeIndex = AttributeIndex::new(0);
//!     const AGE:  AttributeIndex = AttributeIndex::new(1);
//! }
//!
//! impl Attributes for Person {
//!     fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
//!         match idx {
//!             Self::NAME => Some(Value::String(&self.name)),
//!             Self::AGE  => Some(Value::U32(self.age)),
//!             _          => None,
//!         }
//!     }
//!     fn kind(idx: AttributeIndex) -> Option<ValueKind> {
//!         match idx {
//!             Self::NAME => Some(ValueKind::String),
//!             Self::AGE  => Some(ValueKind::U32),
//!             _          => None,
//!         }
//!     }
//!     fn index(name: &str) -> Option<AttributeIndex> {
//!         match name {
//!             "name" => Some(Self::NAME),
//!             "age"  => Some(Self::AGE),
//!             _      => None,
//!         }
//!     }
//! }
//! ```
//!
//! ### 2. Build an expression
//!
//! Use [`ExpressionBuilder`] to register named conditions, then compile them
//! into a reusable [`Expression`] with a boolean expression string:
//!
//! ```rust
//! # use whereexpr::*;
//! # struct Person { name: String, age: u32 }
//! # impl Person {
//! #     const NAME: AttributeIndex = AttributeIndex::new(0);
//! #     const AGE: AttributeIndex = AttributeIndex::new(1);
//! # }
//! # impl Attributes for Person {
//! #     fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
//! #         match idx { Self::NAME => Some(Value::String(&self.name)), Self::AGE => Some(Value::U32(self.age)), _ => None }
//! #     }
//! #     fn kind(idx: AttributeIndex) -> Option<ValueKind> {
//! #         match idx { Self::NAME => Some(ValueKind::String), Self::AGE => Some(ValueKind::U32), _ => None }
//! #     }
//! #     fn index(name: &str) -> Option<AttributeIndex> {
//! #         match name { "name" => Some(Self::NAME), "age" => Some(Self::AGE), _ => None }
//! #     }
//! # }
//! let expr = ExpressionBuilder::<Person>::new()
//!     .add("has_name", Condition::from_str("name is-one-of [Alice, Bob] {ignore-case}"))
//!     .add("is_adult", Condition::from_str("age >= 18"))
//!     .build("has_name && is_adult")
//!     .unwrap();
//! ```
//!
//! ### 3. Evaluate
//!
//! Call [`Expression::matches`] to test any object:
//!
//! ```rust
//! # use whereexpr::*;
//! # struct Person { name: String, age: u32 }
//! # impl Person {
//! #     const NAME: AttributeIndex = AttributeIndex::new(0);
//! #     const AGE: AttributeIndex = AttributeIndex::new(1);
//! # }
//! # impl Attributes for Person {
//! #     fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
//! #         match idx { Self::NAME => Some(Value::String(&self.name)), Self::AGE => Some(Value::U32(self.age)), _ => None }
//! #     }
//! #     fn kind(idx: AttributeIndex) -> Option<ValueKind> {
//! #         match idx { Self::NAME => Some(ValueKind::String), Self::AGE => Some(ValueKind::U32), _ => None }
//! #     }
//! #     fn index(name: &str) -> Option<AttributeIndex> {
//! #         match name { "name" => Some(Self::NAME), "age" => Some(Self::AGE), _ => None }
//! #     }
//! # }
//! # let expr = ExpressionBuilder::<Person>::new()
//! #     .add("has_name", Condition::from_str("name is-one-of [Alice, Bob] {ignore-case}"))
//! #     .add("is_adult", Condition::from_str("age >= 18"))
//! #     .build("has_name && is_adult")
//! #     .unwrap();
//! let people = vec![
//!     Person { name: "Alice".into(), age: 30 },
//!     Person { name: "Charlie".into(), age: 25 },
//! ];
//!
//! let matches: Vec<_> = people.iter().filter(|p| expr.matches(p)).collect();
//! // → only Alice
//! ```
//!
//! ---
//!
//! ## Core concepts
//!
//! | Type | Role |
//! |---|---|
//! | [`Attributes`] | Trait your struct implements to expose its fields |
//! | [`AttributeIndex`] | Opaque index that identifies a field |
//! | [`Value`] | The runtime value of a field (tagged union) |
//! | [`ValueKind`] | The type tag of a field (no data) |
//! | [`Condition`] | Maps one attribute to a [`Predicate`] |
//! | [`Predicate`] | A compiled single-field test (operation + reference value) |
//! | [`Operation`] | The comparison operator (`is`, `>`, `contains`, `glob`, …) |
//! | [`ExpressionBuilder`] | Fluent builder: registers conditions and compiles the boolean expression |
//! | [`Expression`] | The compiled, reusable filter — call `.matches(&obj)` to evaluate |
//! | [`Error`] | All errors that can occur during building or predicate construction |
//!
//! ---
//!
//! ## Condition string syntax
//!
//! A condition string has the form:
//!
//! ```text
//! <attribute>  <operation>  <value>  [<modifiers>]
//! ```
//!
//! Single value:
//!
//! ```text
//! age > 30
//! name is Alice
//! status is-not deleted
//! filename ends-with .log
//! path glob /var/log/**/*.log
//! ```
//!
//! List value (one or more comma-separated entries in `[ ]`):
//!
//! ```text
//! status    is-one-of       [active, pending, paused]
//! extension ends-with-one-of [.jpg, .jpeg, .png]
//! message   contains-one-of  [error, fatal, critical]
//! ```
//!
//! Range (exactly two values in `[ ]`):
//!
//! ```text
//! age       in-range     [18, 65]
//! score     not-in-range [0, 10]
//! port      in-range     [1024, 65535]
//! ```
//!
//! Modifiers (appended in `{ }`):
//!
//! ```text
//! name is-one-of [alice, bob] {ignore-case}
//! path starts-with /Home       {ignore-case}
//! ```
//!
//! ---
//!
//! ## Boolean expression syntax
//!
//! The string passed to [`ExpressionBuilder::build`] combines named conditions
//! using standard boolean operators:
//!
//! | Operator | Aliases | Example |
//! |---|---|---|
//! | AND | `&&`, `AND` | `cond_a && cond_b` |
//! | OR  | `\|\|`, `OR` | `cond_a \|\| cond_b` |
//! | NOT | `!`, `NOT`  | `!cond_a` |
//! | Grouping | `( )` | `(cond_a \|\| cond_b) && cond_c` |
//!
//! `&&` and `||` **cannot be mixed** at the same nesting level without
//! parentheses — this is intentional to avoid precedence ambiguity:
//!
//! ```text
//! // ✗ error: mixed operators
//! cond_a && cond_b || cond_c
//!
//! // ✓ ok: grouped explicitly
//! (cond_a && cond_b) || cond_c
//! cond_a && (cond_b || cond_c)
//! ```
//!
//! ---
//!
//! ## Available operations
//!
//! See [`Operation`] for the full list with per-variant aliases and examples.
//!
//! | Family | Operations |
//! |---|---|
//! | Equality | `is`, `is-not` |
//! | Membership | `is-one-of`, `is-not-one-of` |
//! | String prefix | `starts-with`, `not-starts-with`, `starts-with-one-of`, `not-starts-with-one-of` |
//! | String suffix | `ends-with`, `not-ends-with`, `ends-with-one-of`, `not-ends-with-one-of` |
//! | Substring | `contains`, `not-contains`, `contains-one-of`, `not-contains-one-of` |
//! | Glob pattern | `glob`, `not-glob` |
//! | Numeric | `>`, `>=`, `<`, `<=`, `in-range`, `not-in-range` |
//!
//! ---
//!
//! ## Supported value types
//!
//! See [`Value`] and [`ValueKind`] for details. In brief:
//! `String`, `Path`, `u8`–`u64`, `i8`–`i64`, `f32`, `f64`,
//! `Hash128`, `Hash160`, `Hash256`, `IpAddr`, `DateTime` (Unix timestamp),
//! `Bool`.
//!
//! ---
//!
//! ## Feature flags
//!
//! | Flag | Effect |
//! |---|---|
//! | `error_description` | Implements [`std::fmt::Display`] for [`Error`], providing annotated error messages with `^` underlines pointing at the problematic token. |

mod predicate;
mod predicates;
mod operation;
mod value;
mod condition;
mod condition_list;
mod expression;
mod expr_parser;
mod cond_parser;
mod error;
mod types;

#[cfg(test)]
mod tests;

pub use value::Value;
pub use value::ValueKind;
pub use value::Attributes;
pub use value::AttributeIndex;
pub(crate) use condition::CompiledCondition;
pub(crate) use condition_list::ConditionList;
pub(crate) use condition::ConditionAttribute;
pub(crate) use condition::ConditionPredicate;
pub use expression::Expression;
pub use expression::ExpressionBuilder;
pub use operation::Operation;
pub use condition::Condition;
pub use predicate::Predicate;
pub use error::Error;

