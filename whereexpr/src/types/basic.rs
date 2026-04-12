use super::{IntoValueKind, FromRepr};
use crate::{ValueKind, Value};

macro_rules! IMPL_TRAITS {
    ($type:ty , $variant:ident) => {
        impl IntoValueKind for $type {
            const VALUE_KIND: ValueKind = ValueKind::$variant;
        }
        impl FromRepr<$type> for $type {
            fn from_repr(repr: &str) -> Result<$type, crate::Error> {
                Ok(repr.parse().map_err(|_| crate::Error::FailToParseValue(repr.to_string(), ValueKind::$variant))?)
            }
        }
        impl From<$type> for Value<'_> {
            fn from(value: $type) -> Self {
                Value::$variant(value)
            }
        }
        impl TryFrom<Value<'_>> for $type {
            type Error = crate::Error;
            fn try_from(value: Value<'_>) -> Result<Self, Self::Error> {
                match value {
                    Value::$variant(v) => Ok(v),
                    _ => Err(crate::Error::ExpectingADifferentValueKind(value.kind(), ValueKind::$variant)),
                }
            }
        }       
    };
}

IMPL_TRAITS!(i8 , I8);
IMPL_TRAITS!(i16 , I16);
IMPL_TRAITS!(i32 , I32);
IMPL_TRAITS!(i64 , I64);
IMPL_TRAITS!(u8 , U8);
IMPL_TRAITS!(u16 , U16);
IMPL_TRAITS!(u32 , U32);
IMPL_TRAITS!(u64 , U64);
IMPL_TRAITS!(f32 , F32);
IMPL_TRAITS!(f64 , F64);
