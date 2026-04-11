use super::Operation;
use super::ValueKind;

pub enum Error {
    InvalidOperationForValue(Operation, ValueKind),
    FailToParseValue(String, ValueKind),
}