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
