use super::predicates::*;
use super::types::*;
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
    Hash256Predicate(Hash256Predicate),
    IpAddrPredicate(IpAddrPredicate),
    DateTimePredicate(DateTimePredicate),
    BoolPredicate(BoolPredicate),
}

pub struct Predicate {
    predicate: PredicateInner,
    negated: bool,
}

impl Predicate {
    pub fn with_value<'a, T>(op: Operation, value: T) -> Result<Self, Error>
    where
        T: Into<Value<'a>>,
    {
        let val: Value<'a> = value.into();
        let (op, negated) = op.operation_and_negated();
        let predicate = match val {
            Value::String(s) => PredicateInner::StringPredicate(StringPredicate::with_value(op, s, false)?),
            Value::Path(v) => PredicateInner::PathPredicate(PathPredicate::with_value(op, v)?),
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
            Value::Hash128(v) => PredicateInner::Hash128Predicate(Hash128Predicate::with_value(op, Hash128::new(*v))?),
            Value::Hash160(v) => PredicateInner::Hash160Predicate(Hash160Predicate::with_value(op, Hash160::new(*v))?),
            Value::Hash256(v) => PredicateInner::Hash256Predicate(Hash256Predicate::with_value(op, Hash256::new(*v))?),
            Value::IpAddr(ip_addr) => PredicateInner::IpAddrPredicate(IpAddrPredicate::with_value(op, ip_addr)?),
            Value::DateTime(v) => PredicateInner::DateTimePredicate(DateTimePredicate::with_value(op, v)?),
            Value::Bool(v) => PredicateInner::BoolPredicate(BoolPredicate::with_value(op, v)?),
            Value::None => todo!(),
        };
        Ok(Predicate { predicate, negated })
    }
    pub fn with_list<'a, T>(op: Operation, values: &[T]) -> Result<Self, Error>
    where
        T: Into<Value<'a>> + Clone,
    {
        if values.len() < 256 {
            let mut v: [Value<'a>; 256] = std::array::from_fn(|_| Value::None);
            for (i, value) in values.iter().enumerate() {
                v[i] = value.clone().into();
            }
            Self::with_value_list(op, &v[..values.len()])
        } else {
            let mut v: Vec<Value<'a>> = Vec::with_capacity(values.len());
            for value in values {
                v.push(value.clone().into());
            }
            Self::with_value_list(op, &v)
        }
    }
    pub fn with_value_list(op: Operation, values: &[Value<'_>]) -> Result<Self, Error> {
        if values.is_empty() {
            return Err(Error::EmptyListForOperation(op));
        }
        let kind = values[0].kind();
        let (op, negated) = op.operation_and_negated();
        let predicate = match kind {
            ValueKind::String => PredicateInner::StringPredicate(StringPredicate::with_value_list(op, values)?),
            ValueKind::Path => PredicateInner::PathPredicate(PathPredicate::with_value_list(op, values)?),
            ValueKind::Bytes => todo!(),
            ValueKind::U8 => PredicateInner::U8Predicate(U8Predicate::with_value_list(op, values)?),
            ValueKind::U16 => PredicateInner::U16Predicate(U16Predicate::with_value_list(op, values)?),
            ValueKind::U32 => PredicateInner::U32Predicate(U32Predicate::with_value_list(op, values)?),
            ValueKind::U64 => PredicateInner::U64Predicate(U64Predicate::with_value_list(op, values)?),
            ValueKind::I8 => PredicateInner::I8Predicate(I8Predicate::with_value_list(op, values)?),
            ValueKind::I16 => PredicateInner::I16Predicate(I16Predicate::with_value_list(op, values)?),
            ValueKind::I32 => PredicateInner::I32Predicate(I32Predicate::with_value_list(op, values)?),
            ValueKind::I64 => PredicateInner::I64Predicate(I64Predicate::with_value_list(op, values)?),
            ValueKind::F32 => PredicateInner::F32Predicate(F32Predicate::with_value_list(op, values)?),
            ValueKind::F64 => PredicateInner::F64Predicate(F64Predicate::with_value_list(op, values)?),
            ValueKind::Hash128 => PredicateInner::Hash128Predicate(Hash128Predicate::with_value_list(op, values)?),
            ValueKind::Hash160 => PredicateInner::Hash160Predicate(Hash160Predicate::with_value_list(op, values)?),
            ValueKind::Hash256 => PredicateInner::Hash256Predicate(Hash256Predicate::with_value_list(op, values)?),
            ValueKind::IpAddr => PredicateInner::IpAddrPredicate(IpAddrPredicate::with_value_list(op, values)?),
            ValueKind::DateTime => PredicateInner::DateTimePredicate(DateTimePredicate::with_value_list(op, values)?),
            ValueKind::Bool => return Err(Error::InvalidOperationForValue(op, ValueKind::Bool)),
            ValueKind::None => todo!(),
        };
        Ok(Predicate { predicate, negated })
    }
    pub fn with_str(op: Operation, value: &str, value_kind: ValueKind, ignore_case: bool) -> Result<Self, Error> {
        let (op, negated) = op.operation_and_negated();
        let predicate = match value_kind {
            ValueKind::String => PredicateInner::StringPredicate(StringPredicate::with_value(op, value, ignore_case)?),
            ValueKind::Path => PredicateInner::PathPredicate(PathPredicate::with_str(op, value, ignore_case)?),
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
            ValueKind::Hash128 => PredicateInner::Hash128Predicate(Hash128Predicate::with_str(op, value)?),
            ValueKind::Hash160 => PredicateInner::Hash160Predicate(Hash160Predicate::with_str(op, value)?),
            ValueKind::Hash256 => PredicateInner::Hash256Predicate(Hash256Predicate::with_str(op, value)?),
            ValueKind::IpAddr => PredicateInner::IpAddrPredicate(IpAddrPredicate::with_str(op, value)?),
            ValueKind::DateTime => PredicateInner::DateTimePredicate(DateTimePredicate::with_str(op, value)?),
            ValueKind::Bool => PredicateInner::BoolPredicate(BoolPredicate::with_str(op, value)?),
            ValueKind::None => todo!(),
        };
        Ok(Predicate { predicate, negated })
    }
    pub fn with_str_list(op: Operation, values: &[&str], value_kind: ValueKind, ignore_case: bool) -> Result<Self, Error> {
        let (op, negated) = op.operation_and_negated();
        let predicate = match value_kind {
            ValueKind::String => PredicateInner::StringPredicate(StringPredicate::with_str_list(op, values, ignore_case)?),
            ValueKind::Path => PredicateInner::PathPredicate(PathPredicate::with_str_list(op, values, ignore_case)?),
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
            ValueKind::Hash128 => PredicateInner::Hash128Predicate(Hash128Predicate::with_str_list(op, values)?),
            ValueKind::Hash160 => PredicateInner::Hash160Predicate(Hash160Predicate::with_str_list(op, values)?),
            ValueKind::Hash256 => PredicateInner::Hash256Predicate(Hash256Predicate::with_str_list(op, values)?),
            ValueKind::IpAddr => PredicateInner::IpAddrPredicate(IpAddrPredicate::with_str_list(op, values)?),
            ValueKind::DateTime => PredicateInner::DateTimePredicate(DateTimePredicate::with_str_list(op, values)?),
            ValueKind::Bool => return Err(Error::InvalidOperationForValue(op, ValueKind::Bool)),
            ValueKind::None => todo!(),
        };
        Ok(Predicate { predicate, negated })
    }
    pub(crate) fn evaluate(&self, field_value: &Value) -> Option<bool> {
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
            (PredicateInner::Hash128Predicate(p), Value::Hash128(v)) => p.evaluate(Hash128::new(**v)),
            (PredicateInner::Hash160Predicate(p), Value::Hash160(v)) => p.evaluate(Hash160::new(**v)),
            (PredicateInner::Hash256Predicate(p), Value::Hash256(v)) => p.evaluate(Hash256::new(**v)),

            (PredicateInner::IpAddrPredicate(p), Value::IpAddr(v)) => p.evaluate(*v),
            (PredicateInner::DateTimePredicate(p), Value::DateTime(v)) => p.evaluate(*v),
            (PredicateInner::BoolPredicate(p), Value::Bool(v)) => p.evaluate(*v),
            _ => return None,
        };
        if self.negated {
            Some(!result)
        } else {
            Some(result)
        }
    }
}
