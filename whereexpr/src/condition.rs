use super::Expression;
use super::Attributes;

pub(super) struct Condition {
    attr_index: u16,
    predicate_index: u16,
}

impl Condition {
    const NEGATED_MASK: u16 = 0x8000;
    const ATTR_INDEX_MASK: u16 = 0x7FFF;
    pub(super) fn evaluate<T: Attributes>(&self, obj: &T, expression: &Expression) -> bool {
        if let Some(field_value) = obj.get(self.attr_index & Self::ATTR_INDEX_MASK) {
            let result = expression.predicates[self.predicate_index as usize].evaluate(&field_value);
            if self.attr_index & Self::NEGATED_MASK != 0 {
                !result
            } else {
                result
            }
        } else {
            false
        }
    }
}
