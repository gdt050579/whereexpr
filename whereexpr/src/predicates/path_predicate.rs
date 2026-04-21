use super::glob_re_match::GlobREMatch;
use super::single_string::*;
use super::string_contains_one_of::ContainsOneOf;
use super::string_ends_with_one_of::EndsWithOneOf;
use super::string_is_one_of::IsOneOf;
use super::string_starts_with_one_of::StartsWithOneOf;
use super::utf8_builder::Utf8Builder;
use crate::{Error, Operation, Value, ValueKind};

macro_rules! build_path_predicate {
    ($name:ident, $inner:ident) => {
        #[derive(Debug)]
        pub(crate) struct $name {
            inner: $inner,
        }

        impl $name {
            pub(crate) fn with_str(value: &str, ignore_case: bool) -> Self {
                Self {
                    inner: $inner::new(value, ignore_case),
                }
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
            pub(crate) fn with_str_list(values: &[&str], ignore_case: bool) -> Result<Self, Error> {
                let inner = $inner::with_str_list(values, ignore_case)?;
                Ok(Self { inner })
            }
            pub(crate) fn with_value_list(list: &[Value<'_>]) -> Result<Self, Error> {
                let mut input_list: Vec<&str> = Vec::with_capacity(list.len());
                for value in list {
                    match value {
                        Value::Path(bytes) => {
                            // if let Ok(s) = std::str::from_utf8(bytes) {
                                input_list.push(bytes);
                            // } else {
                            //     return Err(Error::InvalidUTF8Value(bytes.to_vec(), ValueKind::Path));
                            // }
                        }
                        _ => {
                            return Err(Error::ExpectingADifferentValueKind(value.kind(), ValueKind::Path));
                        }
                    }
                }
                let inner = $inner::with_str_list(&input_list, false)?;
                Ok(Self { inner })
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
    ContainsOneOf(PathContainsOneOf),
    StartsWithOneOf(PathStartsWithOneOf),
    EndsWithOneOf(PathEndsWithOneOf),
    IsOneOf(PathIsOneOf),
    GlobREMatch(GlobREMatch),
}

impl PathPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: &[u8]) -> bool {
        match self {
            PathPredicate::StartsWith(predicate) => predicate.evaluate(value),
            PathPredicate::EndsWith(predicate) => predicate.evaluate(value),
            PathPredicate::Contains(predicate) => predicate.evaluate(value),
            PathPredicate::Equals(predicate) => predicate.evaluate(value),
            PathPredicate::ContainsOneOf(predicate) => predicate.evaluate(value),
            PathPredicate::StartsWithOneOf(predicate) => predicate.evaluate(value),
            PathPredicate::EndsWithOneOf(predicate) => predicate.evaluate(value),
            PathPredicate::IsOneOf(predicate) => predicate.evaluate(value),
            PathPredicate::GlobREMatch(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn with_str(operation: Operation, value: &str, ignore_case: bool) -> Result<Self, Error> {
        let predicate = match operation {
            Operation::StartsWith => PathPredicate::StartsWith(PathStartsWith::with_str(value, ignore_case)),
            Operation::EndsWith => PathPredicate::EndsWith(PathEndsWith::with_str(value, ignore_case)),
            Operation::Contains => PathPredicate::Contains(PathContains::with_str(value, ignore_case)),
            Operation::Is => PathPredicate::Equals(PathEquals::with_str(value, ignore_case)),
            Operation::GlobREMatch => PathPredicate::GlobREMatch(GlobREMatch::with_str(value)?),
            _ => return Err(Error::InvalidOperationForValue(operation, ValueKind::Path)),
        };
        Ok(predicate)
    }
    pub(crate) fn with_value(operation: Operation, value: &[u8]) -> Result<Self, Error> {
        // convert value to &str
        if let Ok(s) = std::str::from_utf8(value) {
            Self::with_str(operation, s, false)
        } else {
            Err(Error::InvalidUTF8Value(value.to_vec(), ValueKind::Path))
        }
    }
    pub(crate) fn with_str_list(operation: Operation, values: &[&str], ignore_case: bool) -> Result<Self, Error> {
        match operation {
            Operation::ContainsOneOf => Ok(PathPredicate::ContainsOneOf(PathContainsOneOf::with_str_list(values, ignore_case)?)),
            Operation::StartsWithOneOf => Ok(PathPredicate::StartsWithOneOf(PathStartsWithOneOf::with_str_list(values, ignore_case)?)),
            Operation::EndsWithOneOf => Ok(PathPredicate::EndsWithOneOf(PathEndsWithOneOf::with_str_list(values, ignore_case)?)),
            Operation::IsOneOf => Ok(PathPredicate::IsOneOf(PathIsOneOf::with_str_list(values, ignore_case)?)),
            Operation::GlobREMatch => Ok(PathPredicate::GlobREMatch(GlobREMatch::with_str_list(values)?)),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::Path)),
        }
    }  
    pub(crate) fn with_value_list(operation: Operation, values: &[Value<'_>]) -> Result<Self, Error>
    {
        match operation {
            Operation::ContainsOneOf => Ok(PathPredicate::ContainsOneOf(PathContainsOneOf::with_value_list(values)?)),
            Operation::StartsWithOneOf => Ok(PathPredicate::StartsWithOneOf(PathStartsWithOneOf::with_value_list(values)?)),
            Operation::EndsWithOneOf => Ok(PathPredicate::EndsWithOneOf(PathEndsWithOneOf::with_value_list(values)?)),
            Operation::IsOneOf => Ok(PathPredicate::IsOneOf(PathIsOneOf::with_value_list(values)?)),
            Operation::GlobREMatch => Ok(PathPredicate::GlobREMatch(GlobREMatch::with_value_list(values)?)),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::Path)),
        }
    }        
}
