use super::Attributes;
use super::Predicate;

pub(super) struct Condition {
    attr_index: u16,
    predicate: Predicate,
}

impl Condition {
    #[inline(always)]
    pub(super) fn new(attr_index: u16, predicate: Predicate) -> Self {
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
