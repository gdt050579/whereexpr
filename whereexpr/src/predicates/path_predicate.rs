use crate::Operation;
use super::single_string::*;
use super::string_contains_one_of::ContainsOneOf;
use super::string_starts_with_one_of::StartsWithOneOf;
use super::string_ends_with_one_of::EndsWithOneOf;
use super::glob_re_match::GlobREMatch;
use super::string_is_one_of::IsOneOf;
use super::utf8_builder::Utf8Builder;


macro_rules! build_path_predicate {
    ($name:ident, $inner:ident) => {
        #[derive(Debug)]
        pub(crate) struct $name {
            inner: $inner,
        }

        impl $name {
            pub(crate) fn new(value: &str, ignore_case: bool) -> Self {
                Self { inner: $inner::new(value, ignore_case) }
            }

            pub(crate) fn evaluate(&self, value: &[u8]) -> bool {
                let s = Utf8Builder::<2048>::new(value);
                self.inner.evaluate(s.as_str())
            }
        }
    };
}

macro_rules! build_path_predicate_with_values {
    ($name:ident, $inner:ident) => {
        #[derive(Debug)]
        pub(crate) struct $name {
            inner: $inner,
        }

        impl $name {
            pub(crate) fn new(values: &[String], ignore_case: bool) -> Option<Self> {
                if let Some(inner) = $inner::new(values, ignore_case) {
                    Some(Self { inner })
                } else {
                    None
                }
            }

            pub(crate) fn evaluate(&self, value: &[u8]) -> bool {
                let s = Utf8Builder::<2048>::new(value);
                self.inner.evaluate(s.as_str())
            }
        }
    };
}

build_path_predicate!(PathStartsWith, StartsWith);
build_path_predicate!(PathEndsWith, EndsWith);
build_path_predicate!(PathContains, Contains);
build_path_predicate!(PathEquals, Equals);
build_path_predicate!(PathDifferent, Different);
build_path_predicate_with_values!(PathContainsOneOf, ContainsOneOf);
build_path_predicate_with_values!(PathStartsWithOneOf, StartsWithOneOf);
build_path_predicate_with_values!(PathEndsWithOneOf, EndsWithOneOf);
build_path_predicate_with_values!(PathIsOneOf, IsOneOf);

#[derive(Debug)]
pub(crate) enum PathPredicate {
    StartsWith(PathStartsWith),
    EndsWith(PathEndsWith),
    Contains(PathContains),
    Equals(PathEquals),
    Different(PathDifferent),
    ContainsOneOf(PathContainsOneOf),
    StartsWithOneOf(PathStartsWithOneOf),
    EndsWithOneOf(PathEndsWithOneOf),
    IsOneOf(PathIsOneOf),
    GlobREMatch(GlobREMatch)
}

impl PathPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: &[u8]) -> bool {
        match self {
            PathPredicate::StartsWith(predicate) => predicate.evaluate(value),
            PathPredicate::EndsWith(predicate) => predicate.evaluate(value),
            PathPredicate::Contains(predicate) => predicate.evaluate(value),
            PathPredicate::Equals(predicate) => predicate.evaluate(value),
            PathPredicate::Different(predicate) => predicate.evaluate(value),
            PathPredicate::ContainsOneOf(predicate) => predicate.evaluate(value),
            PathPredicate::StartsWithOneOf(predicate) => predicate.evaluate(value),
            PathPredicate::EndsWithOneOf(predicate) => predicate.evaluate(value),
            PathPredicate::IsOneOf(predicate) => predicate.evaluate(value),
            PathPredicate::GlobREMatch(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn new(operation: Operation, value: &str, ignore_case: bool) -> Option<Self> {
        match operation {
            Operation::StartsWith => Some(PathPredicate::StartsWith(PathStartsWith::new(value, ignore_case))),
            Operation::EndsWith => Some(PathPredicate::EndsWith(PathEndsWith::new(value, ignore_case))),
            Operation::Contains => Some(PathPredicate::Contains(PathContains::new(value, ignore_case))),
            Operation::Is => Some(PathPredicate::Equals(PathEquals::new(value, ignore_case))),
            Operation::IsNot => Some(PathPredicate::Different(PathDifferent::new(value, ignore_case))),
            Operation::GlobREMatch => Some(PathPredicate::GlobREMatch(GlobREMatch::with_value(value)?)),
            _ => None,
        }
    }
    pub(crate) fn new_with_values(operation: Operation, values: &[String], ignore_case: bool) -> Option<Self> {
        match operation {
            Operation::ContainsOneOf => Some(PathPredicate::ContainsOneOf(PathContainsOneOf::new(values, ignore_case)?)),
            Operation::StartsWithOneOf => Some(PathPredicate::StartsWithOneOf(PathStartsWithOneOf::new(values, ignore_case)?)),
            Operation::EndsWithOneOf => Some(PathPredicate::EndsWithOneOf(PathEndsWithOneOf::new(values, ignore_case)?)),
            Operation::IsOneOf => Some(PathPredicate::IsOneOf(PathIsOneOf::new(values, ignore_case)?)),
            Operation::GlobREMatch => Some(PathPredicate::GlobREMatch(GlobREMatch::new(values)?)),
            _ => None,
        }
    }
}
