use crate::Operation;
use crate::Error;
use crate::ValueKind;
use super::single_string::*;
use super::string_contains_one_of::ContainsOneOf;
use super::string_starts_with_one_of::StartsWithOneOf;
use super::string_ends_with_one_of::EndsWithOneOf;
use super::string_is_one_of::IsOneOf;


#[derive(Debug)]
struct StringListContains {
    p: Contains
}
impl StringListContains {
    fn new(value: &str, ignore_case: bool) -> Self {
        Self { p: Contains::new(value, ignore_case) }
    }
    fn evaluate(&self, value: &[&str]) -> bool {
        for v in value {
            if self.p.evaluate(v) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug)]
struct StringListContainsAnyOf {
    p: ContainsOneOf
}
impl StringListContainsAnyOf {
    fn new(values: &[&str], ignore_case: bool) -> Result<Self, Error> {
        Ok(Self { p: ContainsOneOf::with_str_list(values, ignore_case)? })
    }
    fn evaluate(&self, value: &[&str]) -> bool {
        for v in value {
            if self.p.evaluate(v) {
                return true;
            }
        }
        false
    }
}


#[derive(Debug)]
pub(crate) enum StringListPredicate {
    Contains(StringListContains),
    ContainsAnyOf(StringListContainsAnyOf),
}

impl StringListPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: &[&str]) -> bool {
        match self {
            StringListPredicate::Contains(predicate) => predicate.evaluate(value),
            StringListPredicate::ContainsAnyOf(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn with_str(operation: Operation, value: &str, ignore_case: bool) -> Result<Self, Error> {
        match operation {
            Operation::Contains => Ok(StringListPredicate::Contains(StringListContains::new(value, ignore_case))),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::StringList)),
        }
    }
    pub(crate) fn with_str_list(operation: Operation, values: &[&str], ignore_case: bool) -> Result<Self, Error> {
        match operation {
            Operation::ContainsAnyOf => Ok(StringListPredicate::ContainsAnyOf(StringListContainsAnyOf::new(values, ignore_case)?)),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::StringList)),
        }
    }  
}
