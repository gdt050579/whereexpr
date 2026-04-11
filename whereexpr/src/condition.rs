use super::Expression;
use super::AsValue;

pub(super) struct Condition {
    field_index: u8,
    negated: bool,
    predicate_index: u16,
}

impl Condition {
    pub(super) fn evaluate<T: AsValue>(&self, obj: &T, expression: &Expression) -> bool {
        if let Some(field_value) = obj.as_value(self.field_index) {
            let result = expression.predicates[self.predicate_index as usize].evaluate(&field_value);
            if self.negated {
                !result
            } else {
                result
            }
        } else {
            false
        }
    }
}
