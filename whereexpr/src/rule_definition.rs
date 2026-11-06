use crate::rule_parser::RuleParser;

/// The rule the parser has just read.
///
/// It borrows that parser, so it can be used until the next rule is read.
/// Build the expression from it before asking for another one.
pub struct RuleDefinition<'a> {
    parser: &'a RuleParser,
}

impl<'a> RuleDefinition<'a> {
    pub(crate) fn new(parser: &'a RuleParser) -> Self {
        Self { parser }
    }

    /// Name that follows the `rule` keyword.
    pub fn rule_name(&self) -> &str {
        self.parser.rule_name().unwrap()
    }

    /// Type name written after the colon.
    pub fn type_name(&self) -> &str {
        self.parser.type_name().unwrap()
    }

    /// Boolean expression inside the `where` block.
    ///
    /// Leading and trailing whitespace is not included. An empty block yields `""`.
    pub fn expr(&self) -> &str {
        self.parser.expr().unwrap()
    }

    pub fn conditions_count(&self) -> usize {
        self.parser.conditions_count()
    }

    pub fn condition_name(&self, index: usize) -> Option<&str> {
        self.parser.condition_name(index)
    }

    pub fn condition_value(&self, index: usize) -> Option<&str> {
        self.parser.condition_value(index)
    }

    /// Condition names and values, in source order.
    pub fn conditions(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        (0..self.conditions_count()).map(|index| {
            (
                self.condition_name(index).unwrap(),
                self.condition_value(index).unwrap(),
            )
        })
    }

    pub fn meta_count(&self) -> usize {
        self.parser.meta_count()
    }

    pub fn meta_name(&self, index: usize) -> Option<&str> {
        self.parser.meta_name(index)
    }

    pub fn meta_value(&self, index: usize) -> Option<&str> {
        self.parser.meta_value(index)
    }

    /// Meta names and values, in source order. Empty when the rule has no `with meta` block.
    pub fn meta(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        (0..self.meta_count()).map(|index| (self.meta_name(index).unwrap(), self.meta_value(index).unwrap()))
    }
}
