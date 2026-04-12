mod tokenizer;
mod tokens_validator;
mod redundancy_optimizations;
mod token;
mod parser;
#[cfg(test)]
mod tests;

use self::token::Token;
use self::token::TokenKind;
use self::token::TokenSpan;
use crate::condition_list::ConditionList;

pub(crate) fn parse(input: &str, conditions: &ConditionList) -> Result<crate::expression::EvaluationNode, crate::Error> {
    let mut tokens = tokenizer::tokenize(input)?;
    tokens_validator::validate(&mut tokens, input, conditions)?;
    redundancy_optimizations::reduce_parentheses(&mut tokens);
    let evaluation_node = parser::parse(&tokens);
    Ok(evaluation_node)
}