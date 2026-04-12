use super::Operation;
use super::ValueKind;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidOperationForValue(Operation, ValueKind),
    FailToParseValue(String, ValueKind),
    ExpectingTwoValuesForRange(ValueKind),
    ExpectingMinToBeLessThanMax(ValueKind),
    EmptyListForOperation(Operation),
    EmptyListForIsOneOf(ValueKind),
    EmptyListForGlobREMatch(ValueKind),
    ExpectingADifferentValueKind(ValueKind, ValueKind),
    FailToConvertValueIntoValueKind(String, ValueKind),
    FailToBuildInternalDataStructure(Operation,ValueKind),
    InvalidUTF8Value(Vec<u8>,ValueKind),
    InvalidConditionName(String),
    DuplicateConditionName(String),
    EmptyConditionList,

    ExpressioTooLong,                  // more than 0x7FFF characters
    EmptyExpression,
    UnexpectedChar(u16,u16,String),
    UnclosedParenthesis(u16,u16,String),   // ( without matching )
    UnexpectedCloseParen(u16,u16,String),  // ) without matching (
    MaxParenDepthExceeded(u16,u16,String), // nesting deeper than 8
    UnknownRuleName(u16,u16,String),       // rule name not found in resolve function

    // token pair errors
    DoubleNegation(u16,u16,String),         // NOT NOT
    NegationOfOperator(u16,u16,String),     // NOT AND / NOT OR
    NegationOfCloseParen(u16,u16,String),   // NOT )
    MissingOperator(u16,u16,String),        // rule1 rule2 or rule1 (
    MissingOperand(u16,u16,String),         // AND AND / OR OR / ( AND / ( OR
    OperatorAfterOpenParen(u16,u16,String), // ( AND / ( OR
    EmptyParenthesis(u16,u16,String),       // ()
    MixedOperators(u16,u16,String),         // rule1 AND rule2 OR rule3
    UnexpectedTokenAtStart(u16,u16,String), // starts with AND, OR, )
    UnexpectedTokenAtEnd(u16,u16,String),   // ends with AND, OR, NOT, (


}