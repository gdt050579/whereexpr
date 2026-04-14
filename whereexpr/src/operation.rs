/// The comparison operation used in a condition.
///
/// An `Operation` is the middle part of a condition string — it sits between
/// the attribute name and the reference value:
///
/// ```text
/// <attribute>  <operation>  <value>
///    name          is         Alice
///    age           >          30
///    status    is-one-of   [active, pending]
/// ```
///
/// # String aliases
///
/// Operations are case-insensitive in condition strings, and hyphens (`-`) and
/// underscores (`_`) are ignored during parsing, so `is-one-of`, `is_one_of`,
/// and `isoneof` are all equivalent.
///
/// # Applicable types
///
/// - **Equality / membership** (`Is`, `IsNot`, `IsOneOf`, `IsNotOneOf`): all types.
/// - **String pattern** (`StartsWith`, `EndsWith`, `Contains`, and their variants):
///   `String` and `Path` only.
/// - **Glob** (`GlobREMatch`, `NotGlobREMatch`): `String` and `Path` only.
/// - **Numeric comparison** (`GreaterThan`, `GreaterThanOrEqual`, `LessThan`,
///   `LessThanOrEqual`, `InRange`, `NotInRange`): all numeric types, `DateTime`,
///   and `IpAddr`.
///
/// # Negated variants
///
/// Every "negative" operation (`IsNot`, `NotContains`, `NotInRange`, …) is stored
/// internally as its positive counterpart plus a `negated` flag. This means that
/// passing `IsNot` to a [`Predicate`](crate::Predicate) constructor is exactly
/// equivalent to passing `Is` and then flipping the result.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    /// True when the attribute value **exactly equals** the reference value.
    ///
    /// Aliases: `is`, `==`, `eq`, `equals`
    ///
    /// ```text
    /// name is Alice
    /// status == active
    /// role eq admin
    /// ```
    Is,

    /// True when the attribute value **does not equal** the reference value.
    ///
    /// Aliases: `is-not`, `!=`, `neq`, `not-equals`
    ///
    /// ```text
    /// status is-not deleted
    /// code != 0
    /// role neq guest
    /// ```
    IsNot,

    /// True when the attribute value **equals one of** the values in the list.
    ///
    /// Aliases: `is-one-of`, `in`
    ///
    /// ```text
    /// status is-one-of [active, pending, paused]
    /// role in [admin, moderator]
    /// ```
    IsOneOf,

    /// True when the attribute value **does not equal any** of the values in the list.
    ///
    /// Aliases: `is-not-one-of`, `not-in`
    ///
    /// ```text
    /// status is-not-one-of [deleted, banned]
    /// role not-in [guest, anonymous]
    /// ```
    IsNotOneOf,

    /// True when the string attribute **starts with** the reference value.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `starts-with`
    ///
    /// ```text
    /// filename starts-with report_
    /// path starts-with /home/user
    /// ```
    StartsWith,

    /// True when the string attribute **does not start with** the reference value.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `not-starts-with`
    ///
    /// ```text
    /// filename not-starts-with tmp_
    /// path not-starts-with /var/cache
    /// ```
    NotStartsWith,

    /// True when the string attribute **starts with one of** the values in the list.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `starts-with-one-of`
    ///
    /// ```text
    /// filename starts-with-one-of [report_, summary_, digest_]
    /// path starts-with-one-of [/home, /root]
    /// ```
    StartsWithOneOf,

    /// True when the string attribute **does not start with any** of the values in the list.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `not-starts-with-one-of`
    ///
    /// ```text
    /// filename not-starts-with-one-of [tmp_, cache_, ~]
    /// ```
    NotStartsWithOneOf,

    /// True when the string attribute **ends with** the reference value.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `ends-with`
    ///
    /// ```text
    /// filename ends-with .log
    /// path ends-with /index.html
    /// ```
    EndsWith,

    /// True when the string attribute **does not end with** the reference value.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `not-ends-with`
    ///
    /// ```text
    /// filename not-ends-with .tmp
    /// path not-ends-with .bak
    /// ```
    NotEndsWith,

    /// True when the string attribute **ends with one of** the values in the list.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `ends-with-one-of`
    ///
    /// ```text
    /// filename ends-with-one-of [.jpg, .jpeg, .png, .gif]
    /// path ends-with-one-of [.rs, .toml]
    /// ```
    EndsWithOneOf,

    /// True when the string attribute **does not end with any** of the values in the list.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `not-ends-with-one-of`
    ///
    /// ```text
    /// filename not-ends-with-one-of [.tmp, .bak, .swp]
    /// ```
    NotEndsWithOneOf,

    /// True when the string attribute **contains** the reference value as a substring.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `contains`
    ///
    /// ```text
    /// message contains error
    /// path contains /node_modules/
    /// ```
    Contains,

    /// True when the string attribute **does not contain** the reference value as a substring.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `not-contains`
    ///
    /// ```text
    /// message not-contains spam
    /// path not-contains /.git/
    /// ```
    NotContains,

    /// True when the string attribute **contains at least one** of the substrings in the list.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `contains-one-of`
    ///
    /// ```text
    /// message contains-one-of [error, fatal, critical]
    /// path contains-one-of [/tmp/, /var/cache/]
    /// ```
    ContainsOneOf,

    /// True when the string attribute **contains none** of the substrings in the list.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Alias: `not-contains-one-of`
    ///
    /// ```text
    /// message not-contains-one-of [spam, advertisement, unsubscribe]
    /// ```
    NotContainsOneOf,

    /// True when the string attribute **matches** the glob pattern.
    ///
    /// Supports standard glob wildcards: `*` (any sequence of characters),
    /// `?` (any single character), and `[…]` (character classes).
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Aliases: `glob`, `glob-match`
    ///
    /// ```text
    /// filename glob *.log
    /// path glob /home/**/*.rs
    /// mime-type glob-match image/*
    /// ```
    GlobREMatch,

    /// True when the string attribute **does not match** the glob pattern.
    ///
    /// Applicable to `String` and `Path` types.
    ///
    /// Aliases: `not-glob`, `not-glob-match`
    ///
    /// ```text
    /// filename not-glob *.tmp
    /// path not-glob-match /var/cache/**
    /// ```
    NotGlobREMatch,

    /// True when the numeric attribute is **strictly greater than** the reference value.
    ///
    /// Applicable to all numeric types, `DateTime`, and `IpAddr`.
    ///
    /// Aliases: `>`, `gt`, `greater-than`
    ///
    /// ```text
    /// age > 18
    /// price gt 99
    /// score greater-than 500
    /// ```
    GreaterThan,

    /// True when the numeric attribute is **greater than or equal to** the reference value.
    ///
    /// Applicable to all numeric types, `DateTime`, and `IpAddr`.
    ///
    /// Aliases: `>=`, `gte`, `greater-than-or-equal`
    ///
    /// ```text
    /// age >= 18
    /// score gte 1000
    /// temperature greater-than-or-equal 37
    /// ```
    GreaterThanOrEqual,

    /// True when the numeric attribute is **strictly less than** the reference value.
    ///
    /// Applicable to all numeric types, `DateTime`, and `IpAddr`.
    ///
    /// Aliases: `<`, `lt`, `less-than`
    ///
    /// ```text
    /// age < 65
    /// response-time lt 200
    /// priority less-than 3
    /// ```
    LessThan,

    /// True when the numeric attribute is **less than or equal to** the reference value.
    ///
    /// Applicable to all numeric types, `DateTime`, and `IpAddr`.
    ///
    /// Aliases: `<=`, `lte`, `less-than-or-equal`
    ///
    /// ```text
    /// age <= 17
    /// retries lte 3
    /// load less-than-or-equal 0.9
    /// ```
    LessThanOrEqual,

    /// True when the numeric attribute falls **within the inclusive range** `[min, max]`.
    ///
    /// The range is specified as a two-element list `[min, max]` where `min < max`.
    /// Both bounds are **inclusive**.
    ///
    /// Applicable to all numeric types, `DateTime`, and `IpAddr`.
    ///
    /// Alias: `in-range`
    ///
    /// ```text
    /// age in-range [18, 65]
    /// score in-range [0, 100]
    /// port in-range [1024, 65535]
    /// ```
    InRange,

    /// True when the numeric attribute falls **outside the inclusive range** `[min, max]`.
    ///
    /// The inverse of [`InRange`](Operation::InRange): true when the value is strictly
    /// less than `min` or strictly greater than `max`.
    ///
    /// Alias: `not-in-range`
    ///
    /// ```text
    /// age not-in-range [18, 65]
    /// temperature not-in-range [36, 37]
    /// port not-in-range [0, 1023]
    /// ```
    NotInRange,
}

impl Operation {
    /// Parses an operation from its string representation.
    ///
    /// Parsing is **case-insensitive** and ignores hyphens (`-`) and underscores
    /// (`_`), so all of the aliases listed on each variant are accepted
    /// interchangeably.
    ///
    /// Returns `None` if the string does not match any known operation token.
    /// For a `Result`-returning alternative see the [`FromStr`](std::str::FromStr)
    /// implementation on `Operation`.
    ///
    /// # Examples
    ///
    /// ```text
    /// "is"              → Operation::Is
    /// "=="              → Operation::Is
    /// "is-not"          → Operation::IsNot
    /// "!="              → Operation::IsNot
    /// "is-one-of"       → Operation::IsOneOf
    /// "in"              → Operation::IsOneOf
    /// "starts-with"     → Operation::StartsWith
    /// "ends-with"       → Operation::EndsWith
    /// "contains"        → Operation::Contains
    /// "glob"            → Operation::GlobREMatch
    /// ">"               → Operation::GreaterThan
    /// ">="              → Operation::GreaterThanOrEqual
    /// "<"               → Operation::LessThan
    /// "<="              → Operation::LessThanOrEqual
    /// "in-range"        → Operation::InRange
    /// "not-in-range"    → Operation::NotInRange
    /// "unknown-op"      → None
    /// ```
    pub fn parse_str(repr: &str) -> Option<Operation> {
        let (op, _) = crate::cond_parser::operation::parse(repr, 0, repr.len()).ok()?;
        Some(op)
    }
    pub(crate) fn operation_and_negated(&self) -> (Operation, bool) {
        match self {
            Operation::Is => (Operation::Is, false),
            Operation::IsNot => (Operation::Is, true),
            Operation::IsOneOf => (Operation::IsOneOf, false),
            Operation::IsNotOneOf => (Operation::IsOneOf, true),
            Operation::StartsWith => (Operation::StartsWith, false),
            Operation::NotStartsWith => (Operation::StartsWith, true),
            Operation::StartsWithOneOf => (Operation::StartsWithOneOf, false),
            Operation::NotStartsWithOneOf => (Operation::StartsWithOneOf, true),
            Operation::EndsWith => (Operation::EndsWith, false),
            Operation::NotEndsWith => (Operation::EndsWith, true),
            Operation::EndsWithOneOf => (Operation::EndsWithOneOf, false),
            Operation::NotEndsWithOneOf => (Operation::EndsWithOneOf, true),
            Operation::Contains => (Operation::Contains, false),
            Operation::NotContains => (Operation::Contains, true),
            Operation::ContainsOneOf => (Operation::ContainsOneOf, false),
            Operation::NotContainsOneOf => (Operation::ContainsOneOf, true),
            Operation::GlobREMatch => (Operation::GlobREMatch, false),
            Operation::NotGlobREMatch => (Operation::GlobREMatch, true),
            Operation::GreaterThan => (Operation::GreaterThan, false),
            Operation::GreaterThanOrEqual => (Operation::GreaterThanOrEqual, false),
            Operation::LessThan => (Operation::LessThan, false),
            Operation::LessThanOrEqual => (Operation::LessThanOrEqual, false),
            Operation::InRange => (Operation::InRange, false),
            Operation::NotInRange => (Operation::InRange, true),
        }
    }
}

impl std::str::FromStr for Operation {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (op, _) = crate::cond_parser::operation::parse(s, 0, s.len())?;
        Ok(op)
    }
}

#[cfg(feature = "error_description")]
impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Is => write!(f, "is"),
            Operation::IsNot => write!(f, "is not"),
            Operation::IsOneOf => write!(f, "is one of"),
            Operation::IsNotOneOf => write!(f, "is not one of"),
            Operation::StartsWith => write!(f, "starts with"),
            Operation::NotStartsWith => write!(f, "does not start with"),
            Operation::StartsWithOneOf => write!(f, "starts with one of"),
            Operation::NotStartsWithOneOf => write!(f, "does not start with one of"),
            Operation::EndsWith => write!(f, "ends with"),
            Operation::NotEndsWith => write!(f, "does not end with"),
            Operation::EndsWithOneOf => write!(f, "ends with one of"),
            Operation::NotEndsWithOneOf => write!(f, "does not end with one of"),
            Operation::Contains => write!(f, "contains"),
            Operation::NotContains => write!(f, "does not contain"),
            Operation::ContainsOneOf => write!(f, "contains one of"),
            Operation::NotContainsOneOf => write!(f, "does not contain one of"),
            Operation::GlobREMatch => write!(f, "glob re match"),
            Operation::NotGlobREMatch => write!(f, "does not glob re match"),
            Operation::GreaterThan => write!(f, "greater than"),
            Operation::GreaterThanOrEqual => write!(f, "greater than or equal"),
            Operation::LessThan => write!(f, "less than"),
            Operation::LessThanOrEqual => write!(f, "less than or equal"),
            Operation::InRange => write!(f, "in range"),
            Operation::NotInRange => write!(f, "not in range"),
        }
    }
}