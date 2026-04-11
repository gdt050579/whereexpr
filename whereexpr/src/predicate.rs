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
        let value: Value<'a> = value.into();
        todo!()
    }
    pub fn with_values<'a, T>(op: Operation, values: &[T]) -> Result<Self, Error>
    where
        T: Into<Value<'a>>,
    {
        todo!()
    }
    pub fn with_str(op: Operation, value: &str, value_kind: ValueKind, ignore_case: bool) -> Result<Self, Error> {
        todo!()
    }
    pub fn with_strs(op: Operation, values: &[&str], value_kind: ValueKind, ignore_case: bool) -> Result<Self, Error> {
        todo!()
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
