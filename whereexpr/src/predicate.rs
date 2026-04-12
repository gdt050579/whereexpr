use super::predicates::*;
use super::Error;
use super::Operation;
use super::{Value, ValueKind};

enum PredicateInner {
    I8Predicate(I8Predicate),
    I16Predicate(I16Predicate),
    I32Predicate(I32Predicate),
    I64Predicate(I64Predicate),
    U8Predicate(U8Predicate),
    U16Predicate(U16Predicate),
    U32Predicate(U32Predicate),
    U64Predicate(U64Predicate),
    F32Predicate(F32Predicate),
    F64Predicate(F64Predicate),
    StringPredicate(StringPredicate),
    PathPredicate(PathPredicate),
    Hash128Predicate(Hash128Predicate),
    Hash160Predicate(Hash160Predicate),
    IpAddrPredicate(IpAddrPredicate),
    DateTimePredicate(DateTimePredicate),
    BoolPredicate(BoolPredicate),
}

pub struct Predicate {
    predicate: PredicateInner,
    negated: bool,
}

impl Predicate {
    pub fn parse(expr: &str, kind: ValueKind) -> Result<Self, Error> {
        todo!()
    }
    pub fn with_value<'a, T>(op: Operation, value: T) -> Result<Self, Error>
    where
        T: Into<Value<'a>>,
    {
        let val: Value<'a> = value.into();
        let predicate = match val {
            Value::String(_) => todo!(),
            Value::Path(items) => todo!(),
            Value::Bytes(items) => todo!(),
            Value::U8(v) => PredicateInner::U8Predicate(U8Predicate::with_value(op, v)?),
            Value::U16(v) => PredicateInner::U16Predicate(U16Predicate::with_value(op, v)?),
            Value::U32(v) => PredicateInner::U32Predicate(U32Predicate::with_value(op, v)?),
            Value::U64(v) => PredicateInner::U64Predicate(U64Predicate::with_value(op, v)?),
            Value::I8(v) => PredicateInner::I8Predicate(I8Predicate::with_value(op, v)?),
            Value::I16(v) => PredicateInner::I16Predicate(I16Predicate::with_value(op, v)?),
            Value::I32(v) => PredicateInner::I32Predicate(I32Predicate::with_value(op, v)?),
            Value::I64(v) => PredicateInner::I64Predicate(I64Predicate::with_value(op, v)?),
            Value::F32(v) => PredicateInner::F32Predicate(F32Predicate::with_value(op, v)?),
            Value::F64(v) => PredicateInner::F64Predicate(F64Predicate::with_value(op, v)?),
            Value::Hash128(_) => todo!(),
            Value::Hash160(_) => todo!(),
            Value::Hash256(_) => todo!(),
            Value::IpAddr(ip_addr) => todo!(),
            Value::DateTime(_) => todo!(),
            Value::Bool(_) => todo!(),
            Value::None => todo!(),
        };
        Ok(Predicate { predicate, negated: op.is_negated() })
    }
    pub fn with_values<'a, T>(op: Operation, values: &[T]) -> Result<Self, Error>
    where
        T: Into<Value<'a>>,
    {
        todo!()
    }
    pub fn with_str(op: Operation, value: &str, value_kind: ValueKind, ignore_case: bool) -> Result<Self, Error> {
        let predicate = match value_kind {
            ValueKind::String => todo!(),
            ValueKind::Path => todo!(),
            ValueKind::Bytes => todo!(),
            ValueKind::U8 => PredicateInner::U8Predicate(U8Predicate::with_str(op, value)?),
            ValueKind::U16 => PredicateInner::U16Predicate(U16Predicate::with_str(op, value)?),
            ValueKind::U32 => PredicateInner::U32Predicate(U32Predicate::with_str(op, value)?),
            ValueKind::U64 => PredicateInner::U64Predicate(U64Predicate::with_str(op, value)?),
            ValueKind::I8 => PredicateInner::I8Predicate(I8Predicate::with_str(op, value)?),
            ValueKind::I16 => PredicateInner::I16Predicate(I16Predicate::with_str(op, value)?),
            ValueKind::I32 => PredicateInner::I32Predicate(I32Predicate::with_str(op, value)?),
            ValueKind::I64 => PredicateInner::I64Predicate(I64Predicate::with_str(op, value)?),
            ValueKind::F32 => PredicateInner::F32Predicate(F32Predicate::with_str(op, value)?),
            ValueKind::F64 => PredicateInner::F64Predicate(F64Predicate::with_str(op, value)?),
            ValueKind::Hash128 => todo!(),
            ValueKind::Hash160 => todo!(),
            ValueKind::Hash256 => todo!(),
            ValueKind::IpAddr => todo!(),
            ValueKind::DateTime => todo!(),
            ValueKind::Bool => todo!(),
            ValueKind::None => todo!(),
        };
        Ok(Predicate { predicate, negated: op.is_negated() })
    }
    pub fn with_str_list(op: Operation, values: &[&str], value_kind: ValueKind, ignore_case: bool) -> Result<Self, Error> {
        let predicate = match value_kind {
            ValueKind::String => todo!(),
            ValueKind::Path => todo!(),
            ValueKind::Bytes => todo!(),
            ValueKind::U8 => PredicateInner::U8Predicate(U8Predicate::with_str_list(op, values)?),
            ValueKind::U16 => PredicateInner::U16Predicate(U16Predicate::with_str_list(op, values)?),
            ValueKind::U32 => PredicateInner::U32Predicate(U32Predicate::with_str_list(op, values)?),
            ValueKind::U64 => PredicateInner::U64Predicate(U64Predicate::with_str_list(op, values)?),
            ValueKind::I8 => PredicateInner::I8Predicate(I8Predicate::with_str_list(op, values)?),
            ValueKind::I16 => PredicateInner::I16Predicate(I16Predicate::with_str_list(op, values)?),
            ValueKind::I32 => PredicateInner::I32Predicate(I32Predicate::with_str_list(op, values)?),
            ValueKind::I64 => PredicateInner::I64Predicate(I64Predicate::with_str_list(op, values)?),
            ValueKind::F32 => PredicateInner::F32Predicate(F32Predicate::with_str_list(op, values)?),
            ValueKind::F64 => PredicateInner::F64Predicate(F64Predicate::with_str_list(op, values)?),
            ValueKind::Hash128 => todo!(),
            ValueKind::Hash160 => todo!(),
            ValueKind::Hash256 => todo!(),
            ValueKind::IpAddr => todo!(),
            ValueKind::DateTime => todo!(),
            ValueKind::Bool => todo!(),
            ValueKind::None => todo!(),
        };
        Ok(Predicate { predicate, negated: op.is_negated() })
    }
    pub(crate) fn evaluate(&self, field_value: &Value) -> bool {
        let result = match (&self.predicate, field_value) {
            // signed integer predicates
            (PredicateInner::I8Predicate(p), Value::I8(v)) => p.evaluate(*v),
            (PredicateInner::I16Predicate(p), Value::I16(v)) => p.evaluate(*v),
            (PredicateInner::I32Predicate(p), Value::I32(v)) => p.evaluate(*v),
            (PredicateInner::I64Predicate(p), Value::I64(v)) => p.evaluate(*v),
            // unsigned integer predicates
            (PredicateInner::U8Predicate(p), Value::U8(v)) => p.evaluate(*v),
            (PredicateInner::U16Predicate(p), Value::U16(v)) => p.evaluate(*v),
            (PredicateInner::U32Predicate(p), Value::U32(v)) => p.evaluate(*v),
            (PredicateInner::U64Predicate(p), Value::U64(v)) => p.evaluate(*v),
            // float predicates
            (PredicateInner::F32Predicate(p), Value::F32(v)) => p.evaluate(*v),
            (PredicateInner::F64Predicate(p), Value::F64(v)) => p.evaluate(*v),
            // string predicates
            (PredicateInner::StringPredicate(p), Value::String(v)) => p.evaluate(*v),
            // path predicates
            (PredicateInner::PathPredicate(p), Value::Path(v)) => p.evaluate(*v),
            // hash predicates
            (PredicateInner::Hash128Predicate(p), Value::Hash128(v)) => p.evaluate(*v),
            (PredicateInner::Hash160Predicate(p), Value::Hash160(v)) => p.evaluate(*v),

            (PredicateInner::IpAddrPredicate(p), Value::IpAddr(v)) => p.evaluate(*v),
            (PredicateInner::DateTimePredicate(p), Value::DateTime(v)) => p.evaluate(*v),
            (PredicateInner::BoolPredicate(p), Value::Bool(v)) => p.evaluate(*v),
            _ => unreachable!(),
        };
        if self.negated {
            !result
        } else {
            result
        }
    }
}
