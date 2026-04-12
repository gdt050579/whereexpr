use crate::Operation;
use crate::Error;
use crate::Value;
use super::list_search::ListSearch;
use crate::types::*;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug)]
pub(crate) struct Equals<T: Copy + Eq + FromStr + Debug + Ord> {
    value: T,
}
impl<T: Copy + Eq + FromStr + Debug + Ord> Equals<T> {
    pub(crate) fn new(value: T) -> Self {
        Self { value }
    }
    pub(crate) fn evaluate(&self, value: T) -> bool {
        self.value == value
    }
}

#[derive(Debug)]
pub(crate) enum HashTypePredicate<T: Copy + Eq + FromStr + Debug + Ord + IntoValueKind + FromRepr> {
    Equals(Equals<T>),
    IsOneOf(ListSearch<T>),
}

impl<T: Copy + Eq + FromStr + Debug + Ord + IntoValueKind + FromRepr> HashTypePredicate<T> {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: T) -> bool {
        match self {
            HashTypePredicate::Equals(predicate) => predicate.evaluate(value),
            HashTypePredicate::IsOneOf(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn with_value(operation: Operation, value: T) -> Result<Self, Error> {
        match operation {
            Operation::Is => Ok(HashTypePredicate::Equals(Equals::new(value))),
            _ => Err(Error::InvalidOperationForValue(operation, <T>::VALUE_KIND)),
        }
    }
    pub(crate) fn with_str(operation: crate::Operation, value: &str) -> Result<Self, Error> {   
        Self::with_value(operation, T::from_repr(value)?)
    }    
    pub(crate) fn with_str_list(operation: crate::Operation, values: &[&str]) -> Result<Self, Error> {
        match operation {
            crate::Operation::IsOneOf => Ok(HashTypePredicate::IsOneOf(super::list_search::ListSearch::with_str_list(values)?)),
            _ => Err(Error::InvalidOperationForValue(operation, T::VALUE_KIND)),
        }
    }   
    pub(crate) fn with_value_list<'a, V>(operation: crate::Operation, values: &[V]) ->  Result<Self, Error> 
    where 
        T: TryFrom<Value<'a>, Error=Error>,
        V: Into<Value<'a>> + Clone,
    {
        match operation {
            crate::Operation::IsOneOf => Ok(Self::IsOneOf(super::list_search::ListSearch::with_value_list(values)?)),
            _ => Err(Error::InvalidOperationForValue(operation, T::VALUE_KIND)),
        }
    }      

}

pub(crate) type Hash128Predicate = HashTypePredicate<Hash128>;
pub(crate) type Hash160Predicate = HashTypePredicate<Hash160>;
pub(crate) type Hash256Predicate = HashTypePredicate<Hash256>;


