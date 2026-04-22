use std::fmt::Display;

use super::Operation;
use super::ValueKind;

/// All errors that can be returned by [`ExpressionBuilder::build`](crate::ExpressionBuilder::build)
/// or by [`Predicate`](crate::Predicate) constructors.
///
/// Errors are grouped into four categories:
///
/// 1. **Predicate / value errors** — type mismatches and invalid values when
///    building a [`Predicate`](crate::Predicate).
/// 2. **Builder errors** — problems with condition names or the condition list
///    passed to [`ExpressionBuilder`](crate::ExpressionBuilder).
/// 3. **Condition string parse errors** — malformed `"attribute op value"` strings
///    (e.g. from [`Condition::from_str`](crate::Condition::from_str)).
/// 4. **Boolean expression parse errors** — structural problems in the boolean
///    expression string passed to
///    [`ExpressionBuilder::build`](crate::ExpressionBuilder::build).
///
/// # Span variants
///
/// Many variants carry `(u32, u32, String)` — these are **positional** errors
/// produced by the parser. The fields are `(start, end, expression)` where
/// `start` and `end` are byte offsets into `expression` that highlight exactly
/// which token caused the problem. When the `error_description` feature is
/// enabled, [`Display`] renders this as an annotated excerpt with a `^` underline.
#[cfg_attr(not(feature = "error_description"), derive(Debug))]
#[derive(PartialEq)]
pub enum Error {
    // -----------------------------------------------------------------------
    // 1. Predicate / value errors
    // -----------------------------------------------------------------------

    /// The [`Operation`] cannot be applied to the given [`ValueKind`].
    ///
    /// ```text
    /// // "contains" is only valid for String/Path, not numeric types
    /// age contains 3
    ///
    /// // list operations are not valid for Bool
    /// is-active is-one-of [true, false]
    /// ```
    InvalidOperationForValue(Operation, ValueKind),

    /// A string literal could not be parsed into the expected [`ValueKind`].
    ///
    /// ```text
    /// // "abc" cannot be parsed as u32
    /// age is abc
    ///
    /// // "1.5.6" is not a valid IP address
    /// client-ip is 1.5.6
    /// ```
    FailToParseValue(String, ValueKind),

    /// A range operation (`in-range` / `not-in-range`) was given anything other
    /// than exactly two values.
    ///
    /// ```text
    /// // range requires exactly [min, max]
    /// age in-range [18]
    /// age in-range [18, 30, 50]
    /// ```
    ExpectingTwoValuesForRange(ValueKind),

    /// The minimum of a range is not strictly less than the maximum.
    ///
    /// ```text
    /// // min must be < max
    /// age in-range [50, 18]
    /// score in-range [100, 100]
    /// ```
    ExpectingMinToBeLessThanMax(ValueKind),

    /// A list-based operation was given an empty list.
    ///
    /// ```text
    /// name is-one-of []
    /// filename ends-with-one-of []
    /// ```
    EmptyListForOperation(Operation),

    /// The `is-one-of` / `is-not-one-of` operation was given an empty list for
    /// the specified type.
    ///
    /// ```text
    /// status is-one-of []
    /// ```
    EmptyListForIsOneOf(ValueKind),

    /// The `glob` / `glob-match` operation was given an empty list.
    ///
    /// ```text
    /// filename glob []
    /// ```
    EmptyListForGlobREMatch(ValueKind),

    /// A [`Value`](crate::Value) of one kind was found where a different kind
    /// was expected. Fields are `(actual, expected)`.
    ///
    /// ```text
    /// // mixing types inside a list value
    /// status is-one-of [active, 42]
    /// ```
    ExpectingADifferentValueKind(ValueKind, ValueKind),

    /// A string could not be converted into the requested [`ValueKind`] during
    /// internal value coercion.
    ///
    /// ```text
    /// // "yes" cannot be coerced into a bool in a programmatic predicate
    /// ```
    FailToConvertValueIntoValueKind(String, ValueKind),

    /// An internal data structure (e.g. a hash set or trie) could not be built
    /// for the given operation and value type combination. This is an internal
    /// error that should not occur under normal usage.
    FailToBuildInternalDataStructure(Operation, ValueKind, String),

    /// A byte sequence that was expected to be valid UTF-8 is not.
    ///
    /// ```text
    /// // raw bytes for a path predicate are not valid UTF-8
    /// ```
    InvalidUTF8Value(Vec<u8>, ValueKind),

    // -----------------------------------------------------------------------
    // 2. Builder errors
    // -----------------------------------------------------------------------

    /// A condition name passed to [`ExpressionBuilder::add`](crate::ExpressionBuilder::add)
    /// violates the naming rules. Names must start with an ASCII letter and
    /// contain only ASCII letters, digits, `-`, or `_`.
    ///
    /// ```text
    /// // invalid: starts with a digit
    /// builder.add("1cond", ...)
    ///
    /// // invalid: contains a space
    /// builder.add("my cond", ...)
    ///
    /// // invalid: empty string
    /// builder.add("", ...)
    /// ```
    InvalidConditionName(String),

    /// The same condition name was registered more than once with
    /// [`ExpressionBuilder::add`](crate::ExpressionBuilder::add).
    ///
    /// ```text
    /// builder
    ///     .add("is_active", ...)
    ///     .add("is_active", ...)  // duplicate → error at build()
    /// ```
    DuplicateConditionName(String),

    /// A condition references an attribute name that is not exposed by the
    /// target type `T` (i.e. [`Attributes::index`](crate::Attributes::index)
    /// returned `None`). Fields are `(attribute_name, condition_name)`.
    ///
    /// ```text
    /// // "email" is not declared in T::index
    /// Condition::from_str("email is alice@example.com")
    /// ```
    UnknownAttribute(String, String),

    /// [`ExpressionBuilder::build`](crate::ExpressionBuilder::build) was called
    /// without adding any conditions first.
    ///
    /// ```text
    /// ExpressionBuilder::<Person>::new().build("cond_a")  // → EmptyConditionList
    /// ```
    EmptyConditionList,

    /// A condition string passed to [`Condition::from_str`](crate::Condition::from_str)
    /// is empty or contains only whitespace.
    ///
    /// ```text
    /// Condition::from_str("")
    /// Condition::from_str("   ")
    /// ```
    EmptyCondition,

    // -----------------------------------------------------------------------
    // 3. Boolean expression parse errors
    // -----------------------------------------------------------------------

    /// The boolean expression string passed to
    /// [`ExpressionBuilder::build`](crate::ExpressionBuilder::build) exceeds
    /// 32 767 characters (`0x7FFF`).
    ExpressioTooLong,

    /// The boolean expression string is empty or contains only whitespace.
    ///
    /// ```text
    /// builder.build("")
    /// ```
    EmptyExpression,

    /// An unexpected or illegal character was encountered while tokenising the
    /// boolean expression. `(start, end, expression)`
    ///
    /// ```text
    /// cond_a && cond_b @ cond_c
    ///                  ^
    /// ```
    UnexpectedChar(u32, u32, String),

    /// A `(` was opened but never closed. `(start, end, expression)`
    ///
    /// ```text
    /// cond_a && (cond_b || cond_c
    ///           ^
    /// ```
    UnclosedParenthesis(u32, u32, String),

    /// A `)` was found without a matching `(`. `(start, end, expression)`
    ///
    /// ```text
    /// cond_a && cond_b)
    ///                 ^
    /// ```
    UnexpectedCloseParen(u32, u32, String),

    /// Parentheses are nested deeper than the supported maximum (8 levels).
    /// `(start, end, expression)`
    ///
    /// ```text
    /// ((((((((( cond_a )))))))))
    /// ```
    MaxParenDepthExceeded(u32, u32, String),

    /// A condition name referenced in the boolean expression was never
    /// registered via [`ExpressionBuilder::add`](crate::ExpressionBuilder::add).
    /// `(start, end, expression)`
    ///
    /// ```text
    /// // "typo" was never added to the builder
    /// cond_a && typo
    ///           ^^^^
    /// ```
    UnknownConditionName(u32, u32, String),

    /// An attribute name inside a condition string starts with a non-letter or
    /// contains illegal characters. `(start, end, expression)`
    ///
    /// ```text
    /// // attribute names must start with a letter
    /// 9name is Alice
    /// ^
    /// ```
    InvalidAttributeName(u32, u32, String),

    /// A `}` modifier-block terminator was found without a matching `{`.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// name is Alice }
    ///               ^
    /// ```
    UnmatchedModifierBracket(u32, u32, String),

    /// A modifier inside `{ }` is not recognised. `(start, end, expression)`
    ///
    /// ```text
    /// name is Alice {case-sensitive}
    ///                ^^^^^^^^^^^^^^
    /// ```
    UnknownModifier(u32, u32, String),

    /// A `{ }` modifier block is present but contains nothing.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// name is Alice {}
    ///               ^^
    /// ```
    EmptyModifierBlock(u32, u32, String),

    /// The parser expected an operation token but found something else (or
    /// reached the end of input). `(start, end, expression)`
    ///
    /// ```text
    /// name
    ///     ^  (nothing after the attribute name)
    /// ```
    ExpectingOperation(u32, u32, String),

    /// An operation token was found but it does not match any known operation.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// name resembles Alice
    ///      ^^^^^^^^^
    /// ```
    UnknownOperation(u32, u32, String),

    /// A type-cast token (used to override value type inference) is not a
    /// recognised [`ValueKind`]. `(start, end, expression)`
    ///
    /// ```text
    /// age is:integer 30
    ///        ^^^^^^^
    /// ```
    UnknownValueKind(u32, u32, String),

    /// The parser expected a value after the operation but found nothing.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// name is
    ///        ^  (no value provided)
    /// ```
    ExpectingAValue(u32, u32, String),

    /// A list operation requires `[`, but the `[` was absent.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// name is-one-of Alice, Bob
    ///                ^  (expected '[')
    /// ```
    MissingStartingBracket(u32, u32, String),

    /// A list was opened with `[` but the closing `]` was never found.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// name is-one-of [Alice, Bob
    ///                           ^  (missing ']')
    /// ```
    MissingEndingBracket(u32, u32, String),

    /// An empty list `[]` was used, which is never valid.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// name is-one-of []
    ///                ^^
    /// ```
    EmptyArrayList(u32, u32, String),

    /// An unrecognised `\x` escape sequence was found inside a quoted string
    /// value. `(start, end, expression)`
    ///
    /// ```text
    /// name is "Alice\z"
    ///               ^^
    /// ```
    InvalidEscapeSequence(u32, u32, String),

    /// A quoted string was opened but never closed.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// name is "Alice
    ///         ^  (unterminated string)
    /// ```
    UnterminatedString(u32, u32, String),

    /// Multiple bare (unquoted, non-list) values were found where only one was
    /// expected. `(start, end, expression)`
    ///
    /// ```text
    /// name is Alice Bob
    ///              ^^^  (second value unexpected)
    /// ```
    ExpectingASingleValue(u32, u32, String),

    /// Inside a list `[…]`, a comma or the closing `]` was expected but
    /// something else was found. `(start, end, expression)`
    ///
    /// ```text
    /// name is-one-of [Alice Bob Carol]
    ///                      ^^^  (missing comma)
    /// ```
    ExpectedCommaOrEnd(u32, u32, String),

    // -----------------------------------------------------------------------
    // 4. Boolean expression token-pair errors
    // -----------------------------------------------------------------------

    /// Two consecutive `!`/`NOT` operators were found.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// ! !cond_a
    /// ^^^
    /// ```
    DoubleNegation(u32, u32, String),

    /// A `!`/`NOT` directly precedes `&&`/`AND` or `||`/`OR`.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// cond_a && !&& cond_b
    ///           ^^^
    /// ```
    NegationOfOperator(u32, u32, String),

    /// A `!`/`NOT` directly precedes a `)`.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// (cond_a && !)
    ///            ^^
    /// ```
    NegationOfCloseParen(u32, u32, String),

    /// Two condition names (or a name followed by `(`) appear consecutively
    /// without a `&&` or `||` between them. `(start, end, expression)`
    ///
    /// ```text
    /// cond_a cond_b
    ///        ^^^^^^  (missing operator)
    /// ```
    MissingOperator(u32, u32, String),

    /// A `&&`/`||` operator appears where an operand is expected — e.g. two
    /// operators in a row or an operator immediately after `(`.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// cond_a && && cond_b
    ///           ^^
    /// (|| cond_a)
    ///  ^^
    /// ```
    MissingOperand(u32, u32, String),

    /// A `&&`/`||` operator appears immediately after an opening `(`.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// (&& cond_a || cond_b)
    ///  ^^
    /// ```
    OperatorAfterOpenParen(u32, u32, String),

    /// A pair of parentheses contains nothing: `()`.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// cond_a && ()
    ///           ^^
    /// ```
    EmptyParenthesis(u32, u32, String),

    /// `&&` and `||` are mixed at the same parenthesis level without grouping.
    /// Use parentheses to make precedence explicit.
    /// `(start, end, expression)`
    ///
    /// ```text
    /// cond_a && cond_b || cond_c
    ///                  ^^  (ambiguous; wrap one group in parentheses)
    /// ```
    MixedOperators(u32, u32, String),

    /// The boolean expression starts with an operator or `)` instead of a
    /// condition name, `!`, or `(`. `(start, end, expression)`
    ///
    /// ```text
    /// && cond_a
    /// ^^
    /// ) cond_a
    /// ^
    /// ```
    UnexpectedTokenAtStart(u32, u32, String),

    /// The boolean expression ends with an operator, `!`, or `(` instead of a
    /// condition name or `)`. `(start, end, expression)`
    ///
    /// ```text
    /// cond_a &&
    ///        ^^
    /// cond_a || !
    ///           ^
    /// ```
    UnexpectedTokenAtEnd(u32, u32, String),
}

impl Error {
    #[cfg(feature = "error_description")]
    fn parse_error(error: &str, start: u32, end: u32, expr: &str) -> String {
        let mut s = String::new();
        s.push_str("Description : ");
        if error.contains("${SPAN}") {
            s.push_str(error.replace("${SPAN}", &expr[start as usize..end as usize]).as_str());
        } else {
            s.push_str(error);
        }
        s.push('\n');
        s.push_str("Expression  : ");
        s.push_str(expr);
        s.push('\n');
        s.push_str("              ");
        for _ in 0..start {
            s.push(' ');
        }
        for _ in start..end {
            s.push('^');
        }
        s.push('\n');
        s
    }
    #[cfg(feature = "error_description")]
    pub(crate) fn description(&self) -> String {
        match self {
            Error::UnexpectedChar(start, end, expr) => {
                Self::parse_error("Unexpected/invalid character for expreession in condition definition", *start, *end, expr)
            }
            Error::UnclosedParenthesis(start, end, expr) => Self::parse_error("Unclosed parenthesis in condition definition", *start, *end, expr),
            Error::UnexpectedCloseParen(start, end, expr) => {
                Self::parse_error("Unexpected close parenthesis in condition definition", *start, *end, expr)
            }
            Error::MaxParenDepthExceeded(start, end, expr) => {
                Self::parse_error("Maximum parenthesis depth exceeded in condition definition", *start, *end, expr)
            }
            Error::UnknownConditionName(start, end, expr) => Self::parse_error("Unknown condition name '${SPAN}' in ccondition definition", *start, *end, expr),
            Error::DoubleNegation(start, end, expr) => Self::parse_error("Double negation in condition definition", *start, *end, expr),
            Error::NegationOfOperator(start, end, expr) => {
                Self::parse_error("Negation of operator '${SPAN}' in condition definition", *start, *end, expr)
            }
            Error::NegationOfCloseParen(start, end, expr) => {
                Self::parse_error("Negation of close parenthesis in condition definition", *start, *end, expr)
            }
            Error::MissingOperator(start, end, expr) => Self::parse_error("Missing operator '${SPAN}' in condition definition", *start, *end, expr),
            Error::MissingOperand(start, end, expr) => Self::parse_error("Missing operand in condition definition", *start, *end, expr),
            Error::OperatorAfterOpenParen(start, end, expr) => {
                Self::parse_error("Operator '${SPAN}' after open parenthesis in condition definition", *start, *end, expr)
            }
            Error::EmptyParenthesis(start, end, expr) => Self::parse_error("Empty parenthesis in condition definition", *start, *end, expr),
            Error::MixedOperators(start, end, expr) => Self::parse_error("Mixed operators in condition definition", *start, *end, expr),
            Error::UnexpectedTokenAtStart(start, end, expr) => {
                Self::parse_error("Unexpected token '${SPAN}' at start in condition definition", *start, *end, expr)
            }
            Error::UnexpectedTokenAtEnd(start, end, expr) => {
                Self::parse_error("Unexpected token '${SPAN}' at end in condition definition", *start, *end, expr)
            }
            Error::InvalidAttributeName(start, end, expr) => Self::parse_error(
                "Invalid attribute name '${SPAN}' in condition definition (an attribute name must start with a letter)",
                *start,
                *end,
                expr,
            ),
            Error::UnmatchedModifierBracket(start, end, expr) => {
                Self::parse_error("Unmatched modifier bracket '${SPAN}' in condition definition", *start, *end, expr)
            }
            Error::UnknownModifier(start, end, expr) => Self::parse_error("Unknown modifier '${SPAN}' in condition definition", *start, *end, expr),
            Error::EmptyModifierBlock(start, end, expr) => {
                Self::parse_error("Empty modifier block '${SPAN}' in condition definition", *start, *end, expr)
            }
            Error::ExpectingOperation(start, end, expr) => {
                Self::parse_error("Expecting operation '${SPAN}' in condition definition", *start, *end, expr)
            }
            Error::UnknownOperation(start, end, expr) => Self::parse_error("Unknown operation '${SPAN}' in condition definition", *start, *end, expr),
            Error::UnknownValueKind(start, end, expr) => Self::parse_error("Unknown value kind '${SPAN}' in condition definition", *start, *end, expr),
            Error::ExpectingAValue(start, end, expr) => Self::parse_error(
                "Expecting a value in condition definition (Format should be 'attribute operation value')",
                *start,
                *end,
                expr,
            ),
            Error::ExpectedCommaOrEnd(start, end, expr) => Self::parse_error("Expecting comma or end of list in condition definition", *start, *end, expr),
            Error::MissingStartingBracket(start, end, expr) => Self::parse_error("Missing starting bracket '[' for array list in condition definition", *start, *end, expr),
            Error::MissingEndingBracket(start, end, expr) => Self::parse_error("Missing ending bracket ']' for array list in condition definition", *start, *end, expr),
            Error::EmptyArrayList(start, end, expr) => Self::parse_error("Empty array list [] in condition definition are not allowed", *start, *end, expr),
            Error::InvalidEscapeSequence(start, end, expr) => Self::parse_error("Invalid escape sequence '${SPAN}' in condition definition", *start, *end, expr),
            Error::ExpectingASingleValue(start, end, expr) => Self::parse_error("Expecting a single value or a list of values [value,value,...] in condition definition (Format should be 'attribute operation value') but found multiple values that are not part of a list", *start, *end, expr),
            Error::UnterminatedString(start, end, expr) => Self::parse_error("Unterminated string '${SPAN}' in condition definition", *start, *end, expr),
            Error::InvalidOperationForValue(operation, value_kind) => {
                format!("Operation `{}` can not be applied on a value of type `{}`", value_kind, operation)
            }
            Error::FailToParseValue(v, value_kind) => format!("Fail to parse value `{}` into type `{}`", v, value_kind),
            Error::ExpectingTwoValuesForRange(value_kind) => format!("Expecting two values for range of type `{}`", value_kind),
            Error::ExpectingMinToBeLessThanMax(value_kind) => format!("Expecting min to be less than max for range of type `{}`", value_kind),
            Error::EmptyListForOperation(operation) => format!("Empty list of values are not allowed for operation `{}`", operation),
            Error::EmptyListForIsOneOf(value_kind) => {
                format!("Empty list of values are not allowed for type `{}` in `is one of` operation", value_kind)
            }
            Error::EmptyListForGlobREMatch(value_kind) => format!(
                "Empty list of values are not allowed for type `{}` in `glob re match` operation",
                value_kind
            ),
            Error::ExpectingADifferentValueKind(value_kind, value_kind1) => {
                format!("Expecting a value of type `{}` but got `{}`", value_kind1, value_kind)
            }
            Error::FailToConvertValueIntoValueKind(value, value_kind) => format!("Fail to convert value `{}` into type `{}`", value, value_kind),
            Error::FailToBuildInternalDataStructure(operation, value_kind, error) => format!(
                "Fail to build internal data structure for operation `{}` and value of type `{}`: {}",
                operation, value_kind, error
            ),
            Error::InvalidUTF8Value(items, value_kind) => format!("Invalid UTF-8 value `{:?}` for type `{}`", items, value_kind),
            Error::InvalidConditionName(name) => format!(
                "Invalid condition name `{}` (must be a valid ASCII alphanumeric string with no spaces or special characters )",
                name
            ),
            Error::DuplicateConditionName(name) => format!(
                "Duplicate condition name `{}` (each condition name must be unique wthin the same expression)",
                name
            ),
            Error::UnknownAttribute(attr_name, condition_name) => format!("Unknown attribute `{}` for condition `{}`", attr_name, condition_name),
            Error::EmptyConditionList => "Empty condition list (at least one condition is required)".to_string(),
            Error::ExpressioTooLong => "Expression is too long (exceeded 0x7FFF characters)".to_string(),
            Error::EmptyExpression => "Empty expression".to_string(),
            Error::EmptyCondition => "Empty condition (expecting a format like this:  'attribute operation value'".to_string(),
        }
    }
}
#[cfg(feature = "error_description")]
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}
#[cfg(feature = "error_description")]
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}