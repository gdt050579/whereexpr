use super::Attributes;
use super::Predicate;
use super::AttributeIndex;

pub(super) struct CompiledCondition {
    attr_index: AttributeIndex,
    predicate: Predicate,
}

impl CompiledCondition {
    #[inline(always)]
    pub(super) fn new(attr_index: AttributeIndex, predicate: Predicate) -> Self {
        Self { attr_index, predicate }
    }
    pub(super) fn evaluate<T: Attributes>(&self, obj: &T) -> bool {
        if let Some(field_value) = obj.get(self.attr_index) {
            self.predicate.evaluate(&field_value)
        } else {
            false
        }
    }
}
