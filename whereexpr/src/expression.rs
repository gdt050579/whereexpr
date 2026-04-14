use super::Attributes;
use super::CompiledCondition;
use super::Condition;
use super::ConditionAttribute;
use super::ConditionList;
use super::ConditionPredicate;
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
    pub(super) fn evaluate<T: Attributes>(&self, obj: &T, expression: &Expression) -> Option<bool> {
        match self {
            EvaluationNode::Condition(index) => expression.conditions.get(*index).unwrap().evaluate(obj),
            EvaluationNode::Group {
                composition,
                negated,
                children,
            } => {
                let result = match composition {
                    Composition::And => {
                        let mut result = true;
                        for child in children {
                            match child.evaluate(obj, expression) {
                                Some(v) => {
                                    result &= v;
                                    if !result {
                                        break;
                                    }
                                }
                                None => return None,
                            }
                        }
                        result
                    }
                    Composition::Or => {
                        let mut result = false;
                        for child in children {
                            match child.evaluate(obj, expression) {
                                Some(v) => {
                                    result |= v;
                                    if result {
                                        break;
                                    }
                                }
                                None => return None,
                            }
                        }
                        result
                    }
                };
                if *negated {
                    Some(!result)
                } else {
                    Some(result)
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
            panic!(
                "object type mismatch (this expression is for type '{}', but the object you are trying to match is of type '{}')",
                self.type_name,
                std::any::type_name::<T>()
            );
        }
        self.root.evaluate(obj, self).expect("evaluation failed !")
    }
    #[inline(always)]
    pub fn try_matches<T: Attributes + 'static>(&self, obj: &T) -> Option<bool> {
        if TypeId::of::<T>() != self.type_id {
            return None;
        }
        self.root.evaluate(obj, self)
    }
}

pub struct ExpressionBuilder<T: Attributes + 'static> {
    conditions: Vec<(String, Condition)>,
    _phantom: PhantomData<T>,
}

impl<T: Attributes + 'static> ExpressionBuilder<T> {
    pub fn new() -> Self {
        Self {
            conditions: Vec::with_capacity(4),
            _phantom: PhantomData,
        }
    }

    pub fn add(mut self, name: &str, condition: Condition) -> Self {
        self.conditions.push((name.to_string(), condition));
        self
    }

    pub fn build(self, expr: &str) -> Result<Expression, Error> {
        // build the conditions list
        if self.conditions.is_empty() {
            return Err(Error::EmptyConditionList);
        }
        let mut clist = ConditionList::with_capacity(self.conditions.len());
        for (name, condition) in self.conditions {
            if !Self::is_valid_name(&name) {
                return Err(Error::InvalidConditionName(name));
            }
            let attr_index = match condition.attribute {
                ConditionAttribute::Name(attr_name) => T::index(&attr_name).ok_or(Error::UnknownAttribute(attr_name, name.clone()))?,
                ConditionAttribute::Index(index) => index,
            };
            let (attr_index, predicate) = match condition.predicate {
                ConditionPredicate::Resolved(p) => (attr_index, p),
                ConditionPredicate::Error(e) => return Err(e),
                ConditionPredicate::Raw(expr) => Condition::parse::<T>(&expr, &name)?,
            };
            let compiled_condition = CompiledCondition::new(attr_index, predicate);
            if !clist.add(&name, compiled_condition) {
                return Err(Error::DuplicateConditionName(name));
            }
        }
        let evaluation_node = crate::expr_parser::parse(expr, &clist)?;
        Ok(Expression {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            root: evaluation_node,
            conditions: clist,
        })
    }

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
}
