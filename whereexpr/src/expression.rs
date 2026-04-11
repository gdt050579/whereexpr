use super::Attributes;
use super::Condition;
use super::Predicate;
use super::Error;

pub(super) enum FilterMode {
    Include,
    Exclude,
}
pub(super) enum Composition {
    And,
    Or,
}

pub(super) enum FilterNode {
    Condition(u16),
    Group {
        composition: Composition,
        negated: bool,
        children: Vec<FilterNode>,
    },
}

impl FilterNode {
    pub(super) fn evaluate<T: Attributes>(&self, obj: &T, expression: &Expression) -> bool {
        match self {
            FilterNode::Condition(rule) => {
                expression.conditions[*rule as usize].evaluate(obj, expression)
            }
            FilterNode::Group {
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
    root: FilterNode,
    mode: FilterMode,
    pub(super) conditions: Vec<Condition>,
    pub(super) predicates: Vec<Predicate>,
}

impl Expression {
    pub(crate) fn should_drop<T: Attributes>(&self, obj: &T) -> bool {
        let result = self.root.evaluate(obj, self);
        match self.mode {
            FilterMode::Include => !result,
            FilterMode::Exclude => result,
        }
    }
}

pub struct ExpressionBuilder {
    filter_mode: FilterMode,
    conditions: Vec<Condition>,
    predicates: Vec<Predicate>,
}

impl ExpressionBuilder {
    pub fn new() -> Self {
        Self {
            filter_mode: FilterMode::Include,
            conditions: Vec::new(),
            predicates: Vec::new(),
        }
    }
    pub fn filter_mode(&mut self, mode: FilterMode) -> &mut Self {
        self.filter_mode = mode;
        self
    }
    pub fn add_condition(&mut self, name: &str, attribute_index: u16, p: Predicate) -> &mut Self {
        self
    }
    pub fn build(mut self, expr: &str) -> Result<Expression, Error> {
        todo!()
    }
}
