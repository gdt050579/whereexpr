use super::AttributeIndex;
use super::Attributes;
use super::Error;
use super::Predicate;

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

pub struct Condition {
    pub(crate) attribute: ConditionAttribute,
    pub(crate) predicate: ConditionPredicate,
}

pub(crate) enum ConditionAttribute {
    Name(String),
    Index(AttributeIndex),
}

pub(crate) enum ConditionPredicate {
    Resolved(Predicate),
    Raw(String),
    Error(Error),
}

impl Condition {
    // attribute name + predicate
    pub fn new(attribute: &str, predicate: Predicate) -> Self {
        Self {
            attribute: ConditionAttribute::Name(attribute.to_string()),
            predicate: ConditionPredicate::Resolved(predicate),
        }
    }

    // attribute index + predicate (programmatic, no string resolution needed)
    pub fn with_index(index: AttributeIndex, predicate: Predicate) -> Self {
        Self {
            attribute: ConditionAttribute::Index(index),
            predicate: ConditionPredicate::Resolved(predicate),
        }
    }

    // Result<Predicate> - error propagated to build()
    pub fn try_new(attribute: &str, predicate: Result<Predicate, Error>) -> Self {
        Self {
            attribute: ConditionAttribute::Name(attribute.to_string()),
            predicate: match predicate {
                Ok(p) => ConditionPredicate::Resolved(p),
                Err(e) => ConditionPredicate::Error(e),
            },
        }
    }

    // Result<Predicate> with index - error propagated to build()
    pub fn try_with_index(index: AttributeIndex, predicate: Result<Predicate, Error>) -> Self {
        Self {
            attribute: ConditionAttribute::Index(index),
            predicate: match predicate {
                Ok(p) => ConditionPredicate::Resolved(p),
                Err(e) => ConditionPredicate::Error(e),
            },
        }
    }
    pub fn from_str(expr: &str) -> Self {
        Self {
            attribute: ConditionAttribute::Index(AttributeIndex::new(0)),
            predicate: ConditionPredicate::Raw(expr.to_string()),
        }
    }

    pub(crate) fn parse<T: Attributes + 'static>(expr: &str, cond_name: &str) -> Result<(AttributeIndex, Predicate), Error> {
        let (attr_name, pos_attr) = crate::cond_parser::attribute::parse(expr)?;
        let attr_index = T::index(&attr_name).ok_or(Error::UnknownAttribute(attr_name.to_string(), cond_name.to_string()))?;
        let kind = T::kind(attr_index).ok_or(Error::UnknownAttribute(attr_name.to_string(), cond_name.to_string()))?;
        let (modifiers, pos_modifiers) = crate::cond_parser::modifiers::parse(expr)?;
        let predicate = Predicate::parse(expr, pos_attr, kind)?;
        Ok((attr_index, predicate))
    }
}
