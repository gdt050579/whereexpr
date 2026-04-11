use super::Predicate;
use super::Condition;
use super::AsValue;

pub(super) enum FilterMode {
    Include,
    Exclude,
}
pub(super) enum Composition {
    And,
    Or,
}

pub(super) enum FilterNode {
    Rule(u16),
    Group {
        composition: Composition,
        negated: bool,
        children: Vec<FilterNode>,
    },
}

impl FilterNode {
    pub(super) fn evaluate<T: AsValue>(&self, obj: &T, expression: &Expression) -> bool {
        match self {
            FilterNode::Rule(rule) => expression.conditions[*rule as usize].evaluate(obj, expression),
            FilterNode::Group { composition, negated, children } => {
                let result = match composition {
                    Composition::And => children.iter().all(|c| c.evaluate(obj, expression)),
                    Composition::Or  => children.iter().any(|c| c.evaluate(obj, expression)),
                };
                if *negated { !result } else { result }
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
    pub(crate) fn should_drop<T: AsValue>(&self, obj: &T) -> bool {
        let result = self.root.evaluate(obj, self);
        match self.mode {
            FilterMode::Include => !result,
            FilterMode::Exclude => result,
        }
    }
}  