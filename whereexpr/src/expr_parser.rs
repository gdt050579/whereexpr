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
use self::tokenizer::tokenize;
