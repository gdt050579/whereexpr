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