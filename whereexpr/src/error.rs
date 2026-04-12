use super::Operation;
use super::ValueKind;

#[derive(Debug)]
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
}