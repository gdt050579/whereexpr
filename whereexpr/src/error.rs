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
    FailToBuildInternalDataStructure(Operation, ValueKind),
    InvalidUTF8Value(Vec<u8>, ValueKind),
    InvalidConditionName(String),
    DuplicateConditionName(String),
    UnknownAttribute(String, String), // attribute name, condition name
    EmptyConditionList,
    EmptyCondition,

    ExpressioTooLong, // more than 0x7FFF characters
    EmptyExpression,
    UnexpectedChar(u16, u16, String),
    UnclosedParenthesis(u16, u16, String),      // ( without matching )
    UnexpectedCloseParen(u16, u16, String),     // ) without matching (
    MaxParenDepthExceeded(u16, u16, String),    // nesting deeper than 8
    UnknownRuleName(u16, u16, String),          // rule name not found in resolve function
    InvalidAttributeName(u16, u16, String),     // invalid attribute name at position ${SPAN}
    UnmatchedModifierBracket(u16, u16, String), // } without matching {
    UnknownModifier(u16, u16, String),          // unknown modifier at position ${SPAN}
    EmptyModifierBlock(u16, u16, String),       // empty modifier block at position ${SPAN}

    // token pair errors
    DoubleNegation(u16, u16, String),         // NOT NOT
    NegationOfOperator(u16, u16, String),     // NOT AND / NOT OR
    NegationOfCloseParen(u16, u16, String),   // NOT )
    MissingOperator(u16, u16, String),        // rule1 rule2 or rule1 (
    MissingOperand(u16, u16, String),         // AND AND / OR OR / ( AND / ( OR
    OperatorAfterOpenParen(u16, u16, String), // ( AND / ( OR
    EmptyParenthesis(u16, u16, String),       // ()
    MixedOperators(u16, u16, String),         // rule1 AND rule2 OR rule3
    UnexpectedTokenAtStart(u16, u16, String), // starts with AND, OR, )
    UnexpectedTokenAtEnd(u16, u16, String),   // ends with AND, OR, NOT, (
}

impl Error {
    #[cfg(feature = "error_description")]
    fn parse_error(error: &str, start: u16, end: u16, expr: &str) -> String {
        let mut s = String::new();
        s.push_str("Description : ");
        if error.contains("${SPAN}") {
            s.push_str(error.replace("${SPAN}", &expr[start as usize..end as usize]).as_str());
        } else {
            s.push_str(error);
        }
        s.push('\n');
        s.push_str("Expression  : ");
        s.push_str(expr);
        s.push('\n');
        s.push_str("              ");
        for _ in 0..start {
            s.push(' ');
        }
        for _ in start..end {
            s.push('^');
        }
        s.push('\n');
        s
    }
    #[cfg(feature = "error_description")]
    pub(crate) fn description(&self) -> String {
        match self {
            Error::UnexpectedChar(start, end, expr) => {
                Self::parse_error("Unexpected/invalid character for expreession in condition definition", *start, *end, expr)
            }
            Error::UnclosedParenthesis(start, end, expr) => Self::parse_error("Unclosed parenthesis in condition definition", *start, *end, expr),
            Error::UnexpectedCloseParen(start, end, expr) => {
                Self::parse_error("Unexpected close parenthesis in condition definition", *start, *end, expr)
            }
            Error::MaxParenDepthExceeded(start, end, expr) => {
                Self::parse_error("Maximum parenthesis depth exceeded in condition definition", *start, *end, expr)
            }
            Error::UnknownRuleName(start, end, expr) => Self::parse_error("Unknown rule name '${SPAN}' in ccondition definition", *start, *end, expr),
            Error::DoubleNegation(start, end, expr) => Self::parse_error("Double negation in condition definition", *start, *end, expr),
            Error::NegationOfOperator(start, end, expr) => {
                Self::parse_error("Negation of operator '${SPAN}' in condition definition", *start, *end, expr)
            }
            Error::NegationOfCloseParen(start, end, expr) => {
                Self::parse_error("Negation of close parenthesis in condition definition", *start, *end, expr)
            }
            Error::MissingOperator(start, end, expr) => Self::parse_error("Missing operator '${SPAN}' in condition definition", *start, *end, expr),
            Error::MissingOperand(start, end, expr) => Self::parse_error("Missing operand in condition definition", *start, *end, expr),
            Error::OperatorAfterOpenParen(start, end, expr) => {
                Self::parse_error("Operator '${SPAN}' after open parenthesis in condition definition", *start, *end, expr)
            }
            Error::EmptyParenthesis(start, end, expr) => Self::parse_error("Empty parenthesis in condition definition", *start, *end, expr),
            Error::MixedOperators(start, end, expr) => Self::parse_error("Mixed operators in condition definition", *start, *end, expr),
            Error::UnexpectedTokenAtStart(start, end, expr) => {
                Self::parse_error("Unexpected token '${SPAN}' at start in condition definition", *start, *end, expr)
            }
            Error::UnexpectedTokenAtEnd(start, end, expr) => {
                Self::parse_error("Unexpected token '${SPAN}' at end in condition definition", *start, *end, expr)
            }
            Error::InvalidAttributeName(start, end, expr) => Self::parse_error(
                "Invalid attribute name '${SPAN}' in condition definition (an attribute name must start with a letter)",
                *start,
                *end,
                expr,
            ),
            Error::UnmatchedModifierBracket(start, end, expr) => Self::parse_error("Unmatched modifier bracket '${SPAN}' in condition definition", *start, *end, expr),
            Error::UnknownModifier(start, end, expr) => Self::parse_error("Unknown modifier '${SPAN}' in condition definition", *start, *end, expr),
            Error::EmptyModifierBlock(start, end, expr) => Self::parse_error("Empty modifier block '${SPAN}' in condition definition", *start, *end, expr),
            Error::InvalidOperationForValue(operation, value_kind) => {
                format!("Operation `{}` can not be applied on a value of type `{}`", value_kind, operation)
            }
            Error::FailToParseValue(v, value_kind) => format!("Fail to parse value `{}` into type `{}`", v, value_kind),
            Error::ExpectingTwoValuesForRange(value_kind) => format!("Expecting two values for range of type `{}`", value_kind),
            Error::ExpectingMinToBeLessThanMax(value_kind) => format!("Expecting min to be less than max for range of type `{}`", value_kind),
            Error::EmptyListForOperation(operation) => format!("Empty list of values are not allowed for operation `{}`", operation),
            Error::EmptyListForIsOneOf(value_kind) => {
                format!("Empty list of values are not allowed for type `{}` in `is one of` operation", value_kind)
            }
            Error::EmptyListForGlobREMatch(value_kind) => format!(
                "Empty list of values are not allowed for type `{}` in `glob re match` operation",
                value_kind
            ),
            Error::ExpectingADifferentValueKind(value_kind, value_kind1) => {
                format!("Expecting a value of type `{}` but got `{}`", value_kind1, value_kind)
            }
            Error::FailToConvertValueIntoValueKind(value, value_kind) => format!("Fail to convert value `{}` into type `{}`", value, value_kind),
            Error::FailToBuildInternalDataStructure(operation, value_kind) => format!(
                "Fail to build internal data structure for operation `{}` and value of type `{}`",
                operation, value_kind
            ),
            Error::InvalidUTF8Value(items, value_kind) => format!("Invalid UTF-8 value `{:?}` for type `{}`", items, value_kind),
            Error::InvalidConditionName(name) => format!(
                "Invalid condition name `{}` (must be a valid ASCII alphanumeric string with no spaces or special characters )",
                name
            ),
            Error::DuplicateConditionName(name) => format!(
                "Duplicate condition name `{}` (each condition name must be unique wthin the same expression)",
                name
            ),
            Error::UnknownAttribute(attr_name, condition_name) => format!("Unknown attribute `{}` for condition `{}`", attr_name, condition_name),
            Error::EmptyConditionList => "Empty condition list (at least one condition is required)".to_string(),
            Error::ExpressioTooLong => "Expression is too long (exceeded 0x7FFF characters)".to_string(),
            Error::EmptyExpression => "Empty expression".to_string(),
            Error::EmptyCondition => "Empty condition (expecting a format like this:  'attribute operation value'".to_string(),
        }
    }
}
