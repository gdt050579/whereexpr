#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    Is,
    IsNot,
    IsOneOf,
    IsNotOneOf,
    StartsWith,
    NotStartsWith,
    StartsWithOneOf,
    NotStartsWithOneOf,
    EndsWith,
    NotEndsWith,
    EndsWithOneOf,
    NotEndsWithOneOf,
    Contains,
    NotContains,
    ContainsOneOf,
    NotContainsOneOf,
    GlobREMatch,
    NotGlobREMatch,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    InRange,
    NotInRange,
}

impl Operation {
    pub fn is_negated(&self) -> bool {
        match self {
            Operation::IsNot
            | Operation::NotStartsWith
            | Operation::NotEndsWith
            | Operation::NotContains
            | Operation::NotContainsOneOf
            | Operation::NotGlobREMatch
            | Operation::NotInRange => true,
            _ => false,
        }
    }
}
