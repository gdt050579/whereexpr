use super::numeric::*;
use crate::Operation;

#[derive(Debug)]
pub(crate) enum FloatNumberPredicate {
    FloatSmallerThenOrEqualTo(FloatSmallerThenOrEqualTo),
    FloatSmallerThen(FloatSmallerThen),
    FloatGreaterThenOrEqualTo(FloatGreaterThenOrEqualTo),
    FloatGreaterThen(FloatGreaterThen),
    FloatEqualTo(FloatEqualTo),
    FloatDifferentThen(FloatDifferentThen),
    FloatInsideRange(FloatInsideRange),
}

impl FloatNumberPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: f64) -> bool {
        match self {
            FloatNumberPredicate::FloatSmallerThenOrEqualTo(predicate) => predicate.evaluate(value),
            FloatNumberPredicate::FloatSmallerThen(predicate) => predicate.evaluate(value),
            FloatNumberPredicate::FloatGreaterThenOrEqualTo(predicate) => predicate.evaluate(value),
            FloatNumberPredicate::FloatGreaterThen(predicate) => predicate.evaluate(value),
            FloatNumberPredicate::FloatEqualTo(predicate) => predicate.evaluate(value),
            FloatNumberPredicate::FloatDifferentThen(predicate) => predicate.evaluate(value),
            FloatNumberPredicate::FloatInsideRange(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn new(operation: Operation, value: f64) -> Option<Self> {
        match operation {
            Operation::GreaterThan => Some(FloatNumberPredicate::FloatGreaterThen(FloatGreaterThen::new(value))),
            Operation::GreaterThanOrEqual => Some(FloatNumberPredicate::FloatGreaterThenOrEqualTo(FloatGreaterThenOrEqualTo::new(value))),
            Operation::LessThan => Some(FloatNumberPredicate::FloatSmallerThen(FloatSmallerThen::new(value))),
            Operation::LessThanOrEqual => Some(FloatNumberPredicate::FloatSmallerThenOrEqualTo(FloatSmallerThenOrEqualTo::new(value))),
            Operation::Is => Some(FloatNumberPredicate::FloatEqualTo(FloatEqualTo::new(value))),
            Operation::IsNot => Some(FloatNumberPredicate::FloatDifferentThen(FloatDifferentThen::new(value))),
            _ => None,
        }
    } 
    pub(crate) fn new_with_values(operation: Operation, values: &[String]) -> Option<Self> {
        match operation {
            Operation::InRange => Some(FloatNumberPredicate::FloatInsideRange(FloatInsideRange::new(values)?)),
            _ => None,
        }
    }
}