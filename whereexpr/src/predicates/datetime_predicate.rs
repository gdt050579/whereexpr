use crate::Operation;
use super::numeric::*;

#[derive(Debug)]
pub(crate) struct DateTimeInsideRange {
    min: u64,
    max: u64,
}

impl DateTimeInsideRange {
    pub(crate) fn new(values: &[String]) -> Option<Self> {
        if values.len() != 2 {
            return None;
        }
        let min: u64 = crate::DateTime::from_str_representation(values[0].as_str())?.into();
        let max: u64 = crate::DateTime::from_str_representation(values[1].as_str())?.into();
        if min > max {
            return None;
        }
        Some(Self { min, max })
    }
    pub(crate) fn evaluate(&self, value: u64) -> bool {
        value >= self.min && value <= self.max
    }
}

#[derive(Debug)]
pub(crate) enum DateTimePredicate {
    DateTimeSmallerThenOrEqualTo(UnsignedSmallerThenOrEqualTo),
    DateTimeSmallerThen(UnsignedSmallerThen),
    DateTimeGreaterThenOrEqualTo(UnsignedGreaterThenOrEqualTo),
    DateTimeGreaterThen(UnsignedGreaterThen),
    DateTimeEqualTo(UnsignedEqualTo),
    DateTimeDifferentThen(UnsignedDifferentThen),
    DateTimeInsideRange(DateTimeInsideRange),
}

impl DateTimePredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: u64) -> bool {
        match self {
            DateTimePredicate::DateTimeSmallerThenOrEqualTo(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeSmallerThen(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeGreaterThenOrEqualTo(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeGreaterThen(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeEqualTo(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeDifferentThen(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeInsideRange(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn new(operation: Operation, value: &str) -> Option<Self> {
        let u64value:u64 = crate::DateTime::from_str_representation(value)?.into();
        match operation {
            Operation::GreaterThan => Some(DateTimePredicate::DateTimeGreaterThen(UnsignedGreaterThen::new(u64value))),
            Operation::GreaterThanOrEqual => Some(DateTimePredicate::DateTimeGreaterThenOrEqualTo(UnsignedGreaterThenOrEqualTo::new(
                u64value,
            ))),
            Operation::LessThan => Some(DateTimePredicate::DateTimeSmallerThen(UnsignedSmallerThen::new(u64value))),
            Operation::LessThanOrEqual => Some(DateTimePredicate::DateTimeSmallerThenOrEqualTo(UnsignedSmallerThenOrEqualTo::new(
                u64value,
            ))),
            Operation::Is => Some(DateTimePredicate::DateTimeEqualTo(UnsignedEqualTo::new(u64value))),
            Operation::IsNot => Some(DateTimePredicate::DateTimeDifferentThen(UnsignedDifferentThen::new(u64value))),
            _ => None,
        }
    }
    pub(crate) fn new_with_values(operation: Operation, values: &[String]) -> Option<Self> {
        match operation {
            Operation::InRange => Some(DateTimePredicate::DateTimeInsideRange(DateTimeInsideRange::new(values)?)),
            _ => None,
        }
    }
}
