use crate::Operation;

#[derive(Debug)]
struct Equal {
    value: bool,
}

impl Equal {
    pub(crate) fn new(value: &str) -> Option<Self> {
        Some(Self { value: value.parse().ok()? })
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
    pub(crate) fn new(operation: Operation, value: &str) -> Option<Self> {
        match operation {
            Operation::Is => Some(BoolPredicate::Equal(Equal::new(value)?)),
            _ => None,
        }
    }
}
