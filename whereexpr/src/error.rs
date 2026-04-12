use super::Operation;
use super::ValueKind;

#[derive(Debug)]
pub enum Error {
    InvalidOperationForValue(Operation, ValueKind),
    FailToParseValue(String, ValueKind),
    ExpectingTwoValuesForRange(ValueKind),
    ExpectingMinToBeLessThanMax(ValueKind),
    EmptyListForIsOneOf(ValueKind),
    ExpectingADifferentValueKind(ValueKind, ValueKind),
    FailToConvertValueIntoValueKind(String, ValueKind),
}