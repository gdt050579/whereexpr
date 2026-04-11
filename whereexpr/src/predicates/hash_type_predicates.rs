use crate::Operation;
use super::list_search::ListSearch;
use crate::Hash128;
use crate::Hash160;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug)]
pub(crate) struct Equals<T: Copy + Eq + FromStr + Debug + Ord> {
    value: T,
}
impl<T: Copy + Eq + FromStr + Debug + Ord> Equals<T> {
    pub(crate) fn new(value: &str) -> Option<Self> {
        Some(Self { value: value.parse().ok()? })
    }
    pub(crate) fn evaluate(&self, value: T) -> bool {
        self.value == value
    }
}

#[derive(Debug)]
pub(crate) struct Different<T: Copy + Eq + FromStr + Debug + Ord> {
    value: T,
}
impl<T: Copy + Eq + FromStr + Debug + Ord> Different<T> {
    pub(crate) fn new(value: &str) -> Option<Self> {
        Some(Self { value: value.parse().ok()? })
    }
}
impl<T: Copy + Eq + FromStr + Debug + Ord> Different<T> {
    pub(crate) fn evaluate(&self, value: T) -> bool {
        self.value != value
    }
}

#[derive(Debug)]
pub(crate) enum HashTypePredicate<T: Copy + Eq + FromStr + Debug + Ord> {
    Equals(Equals<T>),
    Different(Different<T>),
    IsOneOf(ListSearch<T>),
}

impl<T: Copy + Eq + FromStr + Debug + Ord> HashTypePredicate<T> {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: T) -> bool {
        match self {
            HashTypePredicate::Equals(predicate) => predicate.evaluate(value),
            HashTypePredicate::Different(predicate) => predicate.evaluate(value),
            HashTypePredicate::IsOneOf(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn new(operation: Operation, value: &str) -> Option<Self> {
        match operation {
            Operation::Is => Some(HashTypePredicate::Equals(Equals::new(value)?)),
            Operation::IsNot => Some(HashTypePredicate::Different(Different::new(value)?)),
            _ => None,
        }
    }
    pub(crate) fn new_with_values(operation: Operation, values: &[String]) -> Option<Self> {
        match operation {
            Operation::IsOneOf => Some(HashTypePredicate::IsOneOf(ListSearch::new(values)?)),
            _ => None,
        }
    }
}

pub(crate) type Hash128Predicate = HashTypePredicate<Hash128>;
pub(crate) type Hash160Predicate = HashTypePredicate<Hash160>;
