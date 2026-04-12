use crate::Operation;
use super::single_string::*;
use super::string_contains_one_of::ContainsOneOf;
use super::string_starts_with_one_of::StartsWithOneOf;
use super::string_ends_with_one_of::EndsWithOneOf;
use super::string_is_one_of::IsOneOf;


#[derive(Debug)]
pub(crate) enum StringPredicate {
    StartsWith(StartsWith),
    EndsWith(EndsWith),
    Contains(Contains),
    Equals(Equals),
    ContainsOneOf(ContainsOneOf),
    StartsWithOneOf(StartsWithOneOf),
    EndsWithOneOf(EndsWithOneOf),
    IsOneOf(IsOneOf),
}

impl StringPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: &str) -> bool {
        match self {
            StringPredicate::StartsWith(predicate) => predicate.evaluate(value),
            StringPredicate::EndsWith(predicate) => predicate.evaluate(value),
            StringPredicate::Contains(predicate) => predicate.evaluate(value),
            StringPredicate::Equals(predicate) => predicate.evaluate(value),
            StringPredicate::ContainsOneOf(predicate) => predicate.evaluate(value),
            StringPredicate::StartsWithOneOf(predicate) => predicate.evaluate(value),
            StringPredicate::EndsWithOneOf(predicate) => predicate.evaluate(value),
            StringPredicate::IsOneOf(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn new(operation: Operation, value: &str, ignore_case: bool) -> Option<Self> {
        match operation {
            Operation::StartsWith => Some(StringPredicate::StartsWith(StartsWith::new(value, ignore_case))),
            Operation::EndsWith => Some(StringPredicate::EndsWith(EndsWith::new(value, ignore_case))),
            Operation::Contains => Some(StringPredicate::Contains(Contains::new(value, ignore_case))),
            Operation::Is => Some(StringPredicate::Equals(Equals::new(value, ignore_case))),
            _ => None,
        }
    }
    pub(crate) fn new_with_values(operation: Operation, values: &[String], ignore_case: bool) -> Option<Self> {
        match operation {
            Operation::ContainsOneOf => Some(StringPredicate::ContainsOneOf(ContainsOneOf::new(values, ignore_case)?)),
            Operation::StartsWithOneOf => Some(StringPredicate::StartsWithOneOf(StartsWithOneOf::new(values, ignore_case)?)),
            Operation::EndsWithOneOf => Some(StringPredicate::EndsWithOneOf(EndsWithOneOf::new(values, ignore_case)?)),
            Operation::IsOneOf => Some(StringPredicate::IsOneOf(IsOneOf::new(values, ignore_case)?)),
            _ => None,
        }
    }
}
