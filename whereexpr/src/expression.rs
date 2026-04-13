use super::Attributes;
use super::AttributeIndex;
use super::Condition;
use super::ConditionList;
use super::Predicate;
use super::Error;
use std::any::TypeId;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Composition {
    And,
    Or,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum EvaluationNode {
    Condition(u16),
    Group {
        composition: Composition,
        negated: bool,
        children: Vec<EvaluationNode>,
    },
}

impl EvaluationNode {
    pub(super) fn evaluate<T: Attributes>(&self, obj: &T, expression: &Expression) -> bool {
        match self {
            EvaluationNode::Condition(index) => {
                expression.conditions.get(*index).unwrap().evaluate(obj)
            }
            EvaluationNode::Group {
                composition,
                negated,
                children,
            } => {
                let result = match composition {
                    Composition::And => children.iter().all(|c| c.evaluate(obj, expression)),
                    Composition::Or => children.iter().any(|c| c.evaluate(obj, expression)),
                };
                if *negated {
                    !result
                } else {
                    result
                }
            }
        }
    }
}

pub struct Expression {
    root: EvaluationNode,
    type_id: TypeId,
    type_name: &'static str,    
    pub(super) conditions: ConditionList,
}

impl Expression {
    #[inline(always)]
    pub fn matches<T: Attributes + 'static>(&self, obj: &T) -> bool {
        if TypeId::of::<T>() != self.type_id {
            panic!("object type mismatch (this expression is for type '{}', but the object you are trying to match is of type '{}')", self.type_name, std::any::type_name::<T>());
        }
        self.root.evaluate(obj, self)
    }
}

pub struct ExpressionBuilder<T: Attributes + 'static> {
    conditions: ConditionList,
    error: Option<Error>,
    phantom: PhantomData<T>,
}

impl<T: Attributes + 'static> Default for ExpressionBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Attributes + 'static> ExpressionBuilder<T> {
    fn is_valid_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let mut first = true;
        for c in name.chars() {
            if first {
                if !c.is_ascii_alphabetic() {
                    return false;
                }
                first = false;
            } else if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
                return false;
            }
        }
        true
    }
    pub fn new() -> Self {
        Self {
            conditions: ConditionList::new(),
            error: None,
            phantom: PhantomData,
        }
    }
    pub fn add_condition(&mut self, name: &str, attribute_index: AttributeIndex, p: Predicate) {
        if self.error.is_some() {
            return;
        }
        if !Self::is_valid_name(name) {
            self.error = Some(Error::InvalidConditionName(name.to_string()));
            return;
        }
        if !self.conditions.add(name, Condition::new(attribute_index, p)) {
            self.error = Some(Error::DuplicateConditionName(name.to_string()));
        }
    }
    pub fn build(self, expr: &str) -> Result<Expression, Error> {
        if let Some(error) = self.error {
            return Err(error);
        }
        if self.conditions.is_empty() {
            return Err(Error::EmptyConditionList);
        }
        let evaluation_node = crate::expr_parser::parse(expr, &self.conditions)?;
        Ok(Expression {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            root: evaluation_node,
            conditions: self.conditions,
        })
    }
}
