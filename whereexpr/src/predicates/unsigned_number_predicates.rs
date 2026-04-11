use super::numeric::*;
use crate::Operation;
use super::list_search::ListSearch;

#[derive(Debug)]
pub(crate) enum UnsignedNumberPredicate {
    UnsignedSmallerThenOrEqualTo(UnsignedSmallerThenOrEqualTo),
    UnsignedSmallerThen(UnsignedSmallerThen),
    UnsignedGreaterThenOrEqualTo(UnsignedGreaterThenOrEqualTo),
    UnsignedGreaterThen(UnsignedGreaterThen),
    UnsignedEqualTo(UnsignedEqualTo),
    UnsignedDifferentThen(UnsignedDifferentThen),
    UnsignedInsideRange(UnsignedInsideRange),
    UnsignedIsOneOf(ListSearch<u64>),
}

impl UnsignedNumberPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: u64) -> bool {
        match self {
            UnsignedNumberPredicate::UnsignedSmallerThenOrEqualTo(predicate) => predicate.evaluate(value),
            UnsignedNumberPredicate::UnsignedSmallerThen(predicate) => predicate.evaluate(value),
            UnsignedNumberPredicate::UnsignedGreaterThenOrEqualTo(predicate) => predicate.evaluate(value),
            UnsignedNumberPredicate::UnsignedGreaterThen(predicate) => predicate.evaluate(value),
            UnsignedNumberPredicate::UnsignedEqualTo(predicate) => predicate.evaluate(value),
            UnsignedNumberPredicate::UnsignedDifferentThen(predicate) => predicate.evaluate(value),
            UnsignedNumberPredicate::UnsignedInsideRange(predicate) => predicate.evaluate(value),
            UnsignedNumberPredicate::UnsignedIsOneOf(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn new(operation: Operation, value: u64) -> Option<Self> {
        match operation {
            Operation::GreaterThan => Some(UnsignedNumberPredicate::UnsignedGreaterThen(UnsignedGreaterThen::new(value))),
            Operation::GreaterThanOrEqual => Some(UnsignedNumberPredicate::UnsignedGreaterThenOrEqualTo(UnsignedGreaterThenOrEqualTo::new(value))),
            Operation::LessThan => Some(UnsignedNumberPredicate::UnsignedSmallerThen(UnsignedSmallerThen::new(value))),
            Operation::LessThanOrEqual => Some(UnsignedNumberPredicate::UnsignedSmallerThenOrEqualTo(UnsignedSmallerThenOrEqualTo::new(value))),
            Operation::Is => Some(UnsignedNumberPredicate::UnsignedEqualTo(UnsignedEqualTo::new(value))),
            Operation::IsNot => Some(UnsignedNumberPredicate::UnsignedDifferentThen(UnsignedDifferentThen::new(value))),
            _ => None,
        }
    }
    pub(crate) fn new_with_values(operation: Operation, values: &[String]) -> Option<Self> {
        match operation {
            Operation::InRange => Some(UnsignedNumberPredicate::UnsignedInsideRange(UnsignedInsideRange::new(values)?)),
            Operation::IsOneOf => Some(UnsignedNumberPredicate::UnsignedIsOneOf(ListSearch::new(values)?)),
            _ => None,
        }
    }
}