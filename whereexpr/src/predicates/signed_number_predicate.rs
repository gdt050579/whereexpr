use super::numeric::*;
use crate::Operation;
use super::list_search::ListSearch;

#[derive(Debug)]
pub(crate) enum SignedNumberPredicate {
    SignedSmallerThenOrEqualTo(SignedSmallerThenOrEqualTo),
    SignedSmallerThen(SignedSmallerThen),
    SignedGreaterThenOrEqualTo(SignedGreaterThenOrEqualTo),
    SignedGreaterThen(SignedGreaterThen),
    SignedEqualTo(SignedEqualTo),
    SignedDifferentThen(SignedDifferentThen),
    SignedInsideRange(SignedInsideRange),
    SignedIsOneOf(ListSearch<i64>), 
}

impl SignedNumberPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: i64) -> bool {
        match self {
            SignedNumberPredicate::SignedSmallerThenOrEqualTo(predicate) => predicate.evaluate(value),
            SignedNumberPredicate::SignedSmallerThen(predicate) => predicate.evaluate(value),
            SignedNumberPredicate::SignedGreaterThenOrEqualTo(predicate) => predicate.evaluate(value),
            SignedNumberPredicate::SignedGreaterThen(predicate) => predicate.evaluate(value),
            SignedNumberPredicate::SignedEqualTo(predicate) => predicate.evaluate(value),
            SignedNumberPredicate::SignedDifferentThen(predicate) => predicate.evaluate(value),
            SignedNumberPredicate::SignedInsideRange(predicate) => predicate.evaluate(value),
            SignedNumberPredicate::SignedIsOneOf(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn new(operation: Operation, value: i64) -> Option<Self> {
        match operation {
            Operation::GreaterThan => Some(SignedNumberPredicate::SignedGreaterThen(SignedGreaterThen::new(value))),
            Operation::GreaterThanOrEqual => Some(SignedNumberPredicate::SignedGreaterThenOrEqualTo(SignedGreaterThenOrEqualTo::new(value))),
            Operation::LessThan => Some(SignedNumberPredicate::SignedSmallerThen(SignedSmallerThen::new(value))),
            Operation::LessThanOrEqual => Some(SignedNumberPredicate::SignedSmallerThenOrEqualTo(SignedSmallerThenOrEqualTo::new(value))),
            Operation::Is => Some(SignedNumberPredicate::SignedEqualTo(SignedEqualTo::new(value))),
            Operation::IsNot => Some(SignedNumberPredicate::SignedDifferentThen(SignedDifferentThen::new(value))),
            _ => None,
        }
    }
    pub(crate) fn new_with_values(operation: Operation, values: &[String]) -> Option<Self> {
        match operation {
            Operation::InRange => Some(SignedNumberPredicate::SignedInsideRange(SignedInsideRange::new(values)?)),
            Operation::IsOneOf => Some(SignedNumberPredicate::SignedIsOneOf(ListSearch::new(values)?)),
            _ => None,
        }
    }    
}