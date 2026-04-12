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
            EvaluationNode::Condition(rule) => {
                expression.conditions[*rule as usize].evaluate(obj)
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
    mode: FilterMode,
    pub(super) conditions: Vec<Condition>,
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
    error: Option<Error>,
}

impl ExpressionBuilder {
    fn is_valid_name(name: &str) -> bool {
        let mut first = true;
        for c in name.chars() {
            if first {
                if !c.is_ascii_alphabetic() {
                    return false;
                }
                first = false;
            } else if (!c.is_ascii_alphanumeric() && c != '_') {
                return false;
            }
        }
        true
    }
    pub fn new() -> Self {
        Self {
            filter_mode: FilterMode::Include,
            conditions: Vec::new(),
            error: None,
        }
    }
    pub fn filter_mode(&mut self, mode: FilterMode) -> &mut Self {
        self.filter_mode = mode;
        self
    }
    pub fn add_condition(&mut self, name: &str, attribute_index: u16, p: Predicate) -> &mut Self {
        if self.error.is_some() {
            return self;
        }
        // check if the name is [A-Za_z][A-Za-z0_9_]+
        if !Self::is_valid_name(name) {
            self.error = Some(Error::InvalidConditionName(name.to_string()));
            return self;
        }
        // check if the name is unique
        self.conditions.push(Condition::new(attribute_index, p));
        self
    }
    pub fn build(mut self, expr: &str) -> Result<Expression, Error> {
        if let Some(error) = self.error {
            return Err(error);
        }
        todo!()
    }
}
