use super::predicates::*;
use super::{Value, ValueKind};
use super::Operation;
use crate::Hash128;
use crate::Hash160;

#[derive(Debug)]
pub(crate) enum Predicate {
    SignedNumberPredicate(SignedNumberPredicate),
    UnsignedNumberPredicate(UnsignedNumberPredicate),
    FloatNumberPredicate(FloatNumberPredicate),
    StringPredicate(StringPredicate),
    PathPredicate(PathPredicate),
    Hash128Predicate(Hash128Predicate),
    Hash160Predicate(Hash160Predicate),
    IpAddrPredicate(IpAddrPredicate),
    DateTimePredicate(DateTimePredicate),
    BoolPredicate(BoolPredicate),
}

impl Predicate {
    pub(crate) fn with_value(operation: Operation, value: &str, kind: ValueKind, ignore_case: bool) -> Option<Self> {
        match kind {
            ValueKind::String => Some(Predicate::StringPredicate(StringPredicate::new(operation, value, ignore_case)?)),
            ValueKind::Signed => Some(Predicate::SignedNumberPredicate(SignedNumberPredicate::new(operation, value.parse().unwrap())?)),
            ValueKind::Unsigned => Some(Predicate::UnsignedNumberPredicate(UnsignedNumberPredicate::new(operation, value.parse().unwrap())?)),
            ValueKind::Float => Some(Predicate::FloatNumberPredicate(FloatNumberPredicate::new(operation, value.parse().unwrap())?)),
            ValueKind::Path => Some(Predicate::PathPredicate(PathPredicate::new(operation, value, ignore_case)?)),
            ValueKind::Hash128 => Some(Predicate::Hash128Predicate(Hash128Predicate::new(operation, value)?)),
            ValueKind::Hash160 => Some(Predicate::Hash160Predicate(Hash160Predicate::new(operation, value)?)),
            ValueKind::IpAddr => Some(Predicate::IpAddrPredicate(IpAddrPredicate::new(operation, value)?)),
            ValueKind::DateTime => Some(Predicate::DateTimePredicate(DateTimePredicate::new(operation, value)?)),
            ValueKind::Bool => Some(Predicate::BoolPredicate(BoolPredicate::new(operation, value)?)),
        }
    }
    pub(crate) fn with_values(operation: Operation, values: &[String], kind: ValueKind, ignore_case: bool) -> Option<Self> {
        match kind {
            ValueKind::Unsigned => Some(Predicate::UnsignedNumberPredicate(UnsignedNumberPredicate::new_with_values(operation, values)?)),
            ValueKind::Signed => Some(Predicate::SignedNumberPredicate(SignedNumberPredicate::new_with_values(operation, values)?)),
            ValueKind::Float => Some(Predicate::FloatNumberPredicate(FloatNumberPredicate::new_with_values(operation, values)?)),
            ValueKind::String => Some(Predicate::StringPredicate(StringPredicate::new_with_values(operation, values, ignore_case)?)),
            ValueKind::Path => Some(Predicate::PathPredicate(PathPredicate::new_with_values(operation, values, ignore_case)?)),
            ValueKind::Hash128 => Some(Predicate::Hash128Predicate(Hash128Predicate::new_with_values(operation, values)?)),
            ValueKind::Hash160 => Some(Predicate::Hash160Predicate(Hash160Predicate::new_with_values(operation, values)?)),
            ValueKind::IpAddr => Some(Predicate::IpAddrPredicate(IpAddrPredicate::new_with_values(operation, values)?)),
            ValueKind::DateTime => Some(Predicate::DateTimePredicate(DateTimePredicate::new_with_values(operation, values)?)),
            ValueKind::Bool => None,
        }
    }
    pub(crate) fn evaluate(&self, field_value: &Value) -> bool {
        match (self, field_value) {
            (Predicate::SignedNumberPredicate(predicate), Value::Signed(value)) => predicate.evaluate(*value),
            (Predicate::UnsignedNumberPredicate(predicate), Value::Unsigned(value)) => predicate.evaluate(*value),
            (Predicate::StringPredicate(predicate), Value::String(value)) => predicate.evaluate(*value),
            (Predicate::PathPredicate(predicate), Value::Path(value)) => predicate.evaluate(*value),
            (Predicate::FloatNumberPredicate(predicate), Value::Float(value)) => predicate.evaluate(*value),
            (Predicate::Hash128Predicate(predicate), Value::Hash128(value)) => predicate.evaluate(Hash128::new(*value)),
            (Predicate::Hash160Predicate(predicate), Value::Hash160(value)) => predicate.evaluate(Hash160::new(*value)),
            (Predicate::IpAddrPredicate(predicate), Value::IpAddr(value)) => predicate.evaluate(*value),
            (Predicate::DateTimePredicate(predicate), Value::DateTime(value)) => predicate.evaluate(*value),
            (Predicate::BoolPredicate(predicate), Value::Bool(value)) => predicate.evaluate(*value),
            _ => unreachable!(),
        }
    }
}
