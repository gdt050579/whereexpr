use super::numeric::u64::*;
use crate::Value;
use crate::types::*;
use crate::Error;
use crate::Operation;
use crate::ValueKind;

#[derive(Debug)]
pub(crate) struct DateTimeInsideRange {
    min: u64,
    max: u64,
}

impl DateTimeInsideRange {
    pub(crate) fn with_str_list(values: &[&str]) -> Result<Self, Error> {
        if values.len() != 2 {
            return Err(Error::ExpectingTwoValuesForRange(ValueKind::DateTime));
        }
        let min: u64 = DateTime::from_repr(values[0])?.into();
        let max: u64 = DateTime::from_repr(values[1])?.into();
        if min > max {
            return Err(Error::ExpectingMinToBeLessThanMax(ValueKind::DateTime));
        }
        Ok(Self { min, max })
    }
    fn with_value_list<'a, V>(values: &[V]) -> Result<Self, Error>
    where
        V: TryFrom<Value<'a>, Error=Error>,
        V: Into<Value<'a>> + Clone,
    {
        if values.len() != 2 {
            return Err(Error::ExpectingTwoValuesForRange(ValueKind::DateTime));
        }
        let min = u64::try_from(values[0].clone().into())?;
        let max = u64::try_from(values[1].clone().into())?;
        if min > max {
            return Err(Error::ExpectingMinToBeLessThanMax(ValueKind::DateTime));
        }
        Ok(Self { min, max })
    }    
    pub(crate) fn evaluate(&self, value: u64) -> bool {
        value >= self.min && value <= self.max
    }
}

#[derive(Debug)]
pub(crate) enum DateTimePredicate {
    DateTimeSmallerThanOrEqualTo(SmallerThanOrEqualTo),
    DateTimeSmallerThan(SmallerThan),
    DateTimeGreaterThanOrEqualTo(GreaterThanOrEqualTo),
    DateTimeGreaterThan(GreaterThan),
    DateTimeEqualTo(EqualTo),
    DateTimeDifferentThan(DifferentThan),
    DateTimeInsideRange(DateTimeInsideRange),
}

impl DateTimePredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: u64) -> bool {
        match self {
            DateTimePredicate::DateTimeSmallerThanOrEqualTo(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeSmallerThan(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeGreaterThanOrEqualTo(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeGreaterThan(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeEqualTo(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeDifferentThan(predicate) => predicate.evaluate(value),
            DateTimePredicate::DateTimeInsideRange(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn with_value(operation: Operation, value: u64) -> Result<Self, Error> {
        match operation {
            Operation::GreaterThan => Ok(DateTimePredicate::DateTimeGreaterThan(GreaterThan::new(value))),
            Operation::GreaterThanOrEqual => Ok(DateTimePredicate::DateTimeGreaterThanOrEqualTo(GreaterThanOrEqualTo::new(value))),
            Operation::LessThan => Ok(DateTimePredicate::DateTimeSmallerThan(SmallerThan::new(value))),
            Operation::LessThanOrEqual => Ok(DateTimePredicate::DateTimeSmallerThanOrEqualTo(SmallerThanOrEqualTo::new(value))),
            Operation::Is => Ok(DateTimePredicate::DateTimeEqualTo(EqualTo::new(value))),
            Operation::IsNot => Ok(DateTimePredicate::DateTimeDifferentThan(DifferentThan::new(value))),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::DateTime)),
        }
    }

    pub(crate) fn with_str(operation: Operation, value: &str) -> Result<Self, Error> {
        Self::with_value(operation, DateTime::from_repr(value)?.into())
    }

    pub(crate) fn with_str_list(operation: crate::Operation, values: &[&str]) -> Result<Self, Error> {
        match operation {
            crate::Operation::InRange => Ok(Self::DateTimeInsideRange(DateTimeInsideRange::with_str_list(values)?)),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::DateTime)),
        }
    }
    pub(crate) fn with_value_list<'a, V>(op: Operation, values: &[V]) -> Result<Self, Error>
    where
        V: TryFrom<Value<'a>, Error=Error>,
        V: Into<Value<'a>> + Clone,
    {
        match op {
            Operation::InRange => Ok(Self::DateTimeInsideRange(DateTimeInsideRange::with_value_list(values)?)),
            _ => Err(Error::InvalidOperationForValue(op, ValueKind::DateTime)),
        }
    }    
}
