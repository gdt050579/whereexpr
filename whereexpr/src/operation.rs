#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    Is,
    IsNot,
    IsOneOf,
    StartsWith,
    StartsWithOneOf,
    EndsWith,
    EndsWithOneOf,
    Contains,
    ContainsOneOf,
    GlobREMatch,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    InRange,
}