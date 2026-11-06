use crate::Attributes;
use crate::RuleParser;
use crate::Expression;
use crate::Error;
use super::Meta;

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
    pub fn name(&self) -> &str {
        self.parser.rule_name().unwrap()
    }

    /// Type name written after the colon.
    pub fn type_name(&self) -> &str {
        self.parser.type_name().unwrap()
    }

    #[inline(always)]
    pub fn expression<T: Attributes>(&self) -> Result<Expression, Error> {
        Expression::from_rule_definition::<T>(self)
    }

    #[inline(always)]
    pub(crate) fn parser(&self) -> &RuleParser {
        self.parser
    }
    
    #[cfg(test)]
    pub(crate) fn conditions_count(&self) -> usize {
        self.parser.conditions_count()
    }

    #[cfg(test)]
    pub(crate) fn condition_name(&self, index: usize) -> Option<&str> {
        self.parser.condition_name(index)
    }

    #[cfg(test)]
    pub(crate) fn condition_value(&self, index: usize) -> Option<&str> {
        self.parser.condition_value(index)
    }

    #[cfg(test)]
    pub(crate) fn conditions(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        (0..self.conditions_count()).map(|index| {
            (
                self.condition_name(index).unwrap(),
                self.condition_value(index).unwrap(),
            )
        })
    }

    pub fn meta(&self) -> Meta<'a> {
        Meta::new(self.parser)
    }
}
