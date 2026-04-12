use crate::Operation;
use crate::Error;
use crate::ValueKind;
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
    pub(crate) fn with_value(operation: Operation, value: &str, ignore_case: bool) -> Result<Self, Error> {
        match operation {
            Operation::StartsWith => Ok(StringPredicate::StartsWith(StartsWith::new(value, ignore_case))),
            Operation::EndsWith => Ok(StringPredicate::EndsWith(EndsWith::new(value, ignore_case))),
            Operation::Contains => Ok(StringPredicate::Contains(Contains::new(value, ignore_case))),
            Operation::Is => Ok(StringPredicate::Equals(Equals::new(value, ignore_case))),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::String)),
        }
    }
    pub(crate) fn with_str_list(operation: Operation, values: &[&str], ignore_case: bool) -> Result<Self, Error> {
        match operation {
            Operation::ContainsOneOf => Ok(StringPredicate::ContainsOneOf(ContainsOneOf::with_str_list(values, ignore_case)?)),
            Operation::StartsWithOneOf => Ok(StringPredicate::StartsWithOneOf(StartsWithOneOf::with_str_list(values, ignore_case)?)),
            Operation::EndsWithOneOf => Ok(StringPredicate::EndsWithOneOf(EndsWithOneOf::with_str_list(values, ignore_case)?)),
            Operation::IsOneOf => Ok(StringPredicate::IsOneOf(IsOneOf::with_str_list(values, ignore_case)?)),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::String)),
        }
    }
}
