use crate::Operation;
use crate::Error;
use crate::ValueKind;

#[derive(Debug)]
struct Equal {
    value: bool,
}

impl Equal {
    pub(crate) fn new(value: bool) -> Self {
        Self { value }
    }
    pub(crate) fn evaluate(&self, value: bool) -> bool {
        self.value == value
    }
}

#[derive(Debug)]
pub(crate) enum BoolPredicate {
    Equal(Equal),
}

impl BoolPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: bool) -> bool {
        match self {
            BoolPredicate::Equal(predicate) => predicate.evaluate(value),
        }
    }
    pub(crate) fn with_value(operation: Operation, value: bool) -> Result<Self, Error> {
        match operation {
            Operation::Is => Ok(BoolPredicate::Equal(Equal::new(value))),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::Bool)),
        }
    }
    pub(crate) fn with_str(operation: Operation, value: &str) -> Result<Self, Error> {
        match operation {
            Operation::Is => Ok(BoolPredicate::Equal(Equal::new(value.parse().map_err(|_| Error::FailToParseValue(value.to_string(), ValueKind::Bool))?))),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::Bool)),
        }
    }
}
