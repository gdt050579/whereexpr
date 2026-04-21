use super::numeric::u64::*;
use crate::Value;
use crate::types::*;
use crate::Error;
use crate::Operation;
use crate::ValueKind;
use super::numeric::*;

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
    pub(crate) fn with_value_list(values: &[Value<'_>]) -> Result<Self, Error>
    {
        if values.len() != 2 {
            return Err(Error::ExpectingTwoValuesForRange(ValueKind::DateTime));
        }
        let start = match values[0] {
            Value::DateTime(v) => v,
            _ => return Err(Error::ExpectingADifferentValueKind(values[0].kind(), ValueKind::DateTime)),
        };
        let end = match values[1] {
            Value::DateTime(v) => v,
            _ => return Err(Error::ExpectingADifferentValueKind(values[1].kind(), ValueKind::DateTime)),
        };
        if start > end {
            return Err(Error::ExpectingMinToBeLessThanMax(ValueKind::DateTime));
        }
        Ok(Self { min: start, max: end })
    }  
    pub(crate) fn evaluate(&self, value: u64) -> bool {
        value >= self.min && value <= self.max
    }
}

#[derive(Debug)]
pub(crate) enum DateTimePredicate {
    SmallerThanOrEqualTo(SmallerThanOrEqualTo),
    SmallerThan(SmallerThan),
    GreaterThanOrEqualTo(GreaterThanOrEqualTo),
    GreaterThan(GreaterThan),
    EqualTo(EqualTo),
    InsideRange(DateTimeInsideRange),
}

impl DateTimePredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: u64) -> bool {
        match self {
            DateTimePredicate::SmallerThanOrEqualTo(predicate) => predicate.evaluate(value),
            DateTimePredicate::SmallerThan(predicate) => predicate.evaluate(value),
            DateTimePredicate::GreaterThanOrEqualTo(predicate) => predicate.evaluate(value),
            DateTimePredicate::GreaterThan(predicate) => predicate.evaluate(value),
            DateTimePredicate::EqualTo(predicate) => predicate.evaluate(value),
            DateTimePredicate::InsideRange(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn with_value(operation: Operation, value: u64) -> Result<Self, Error> {
        match operation {
            Operation::GreaterThan => Ok(DateTimePredicate::GreaterThan(GreaterThan::new(value))),
            Operation::GreaterThanOrEqual => Ok(DateTimePredicate::GreaterThanOrEqualTo(GreaterThanOrEqualTo::new(value))),
            Operation::LessThan => Ok(DateTimePredicate::SmallerThan(SmallerThan::new(value))),
            Operation::LessThanOrEqual => Ok(DateTimePredicate::SmallerThanOrEqualTo(SmallerThanOrEqualTo::new(value))),
            Operation::Is => Ok(DateTimePredicate::EqualTo(EqualTo::new(value))),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::DateTime)),
        }
    }

    pub(crate) fn with_str(operation: Operation, value: &str) -> Result<Self, Error> {
        Self::with_value(operation, DateTime::from_repr(value)?.into())
    }

    pub(crate) fn with_str_list(operation: crate::Operation, values: &[&str]) -> Result<Self, Error> {
        match operation {
            crate::Operation::InRange => Ok(Self::InsideRange(DateTimeInsideRange::with_str_list(values)?)),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::DateTime)),
        }
    }

    pub(crate) fn with_value_list(operation: crate::Operation, values: &[Value<'_>]) ->  Result<Self, Error> 
    {
        match operation {
            Operation::InRange => Ok(Self::InsideRange(DateTimeInsideRange::with_value_list(values)?)),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::DateTime)),
        }
    }  
}
