use crate::Error;
use crate::types::ValueKindConst;

macro_rules! CREATE_PREDICATE_ENUM {
    ($name:ident, $type:ty, $module:ident) => {
        #[derive(Debug)]
        pub(crate) enum $name {
            SmallerThanOrEqualTo(super::numeric::$module::SmallerThanOrEqualTo),
            SmallerThan(super::numeric::$module::SmallerThan),
            GreaterThanOrEqualTo(super::numeric::$module::GreaterThanOrEqualTo),
            GreaterThan(super::numeric::$module::GreaterThan),
            EqualTo(super::numeric::$module::EqualTo),
            DifferentThan(super::numeric::$module::DifferentThan),
            InsideRange(super::numeric::$module::InsideRange),
        }

        impl $name {
            #[inline(always)]
            pub(crate) fn evaluate(&self, value: $type) -> bool {
                match self {
                    Self::SmallerThanOrEqualTo(p) => p.evaluate(value),
                    Self::SmallerThan(p) => p.evaluate(value),
                    Self::GreaterThanOrEqualTo(p) => p.evaluate(value),
                    Self::GreaterThan(p) => p.evaluate(value),
                    Self::EqualTo(p) => p.evaluate(value),
                    Self::DifferentThan(p) => p.evaluate(value),
                    Self::InsideRange(p) => p.evaluate(value),
                }
            }

            pub(crate) fn with_value(operation: crate::Operation, value: $type) -> Result<Self, Error> {
                match operation {
                    crate::Operation::GreaterThan => Ok(Self::GreaterThan(super::numeric::$module::GreaterThan::new(value))),
                    crate::Operation::GreaterThanOrEqual => Ok(Self::GreaterThanOrEqualTo(super::numeric::$module::GreaterThanOrEqualTo::new(value))),
                    crate::Operation::LessThan => Ok(Self::SmallerThan(super::numeric::$module::SmallerThan::new(value))),
                    crate::Operation::LessThanOrEqual => Ok(Self::SmallerThanOrEqualTo(super::numeric::$module::SmallerThanOrEqualTo::new(value))),
                    crate::Operation::Is => Ok(Self::EqualTo(super::numeric::$module::EqualTo::new(value))),
                    crate::Operation::IsNot => Ok(Self::DifferentThan(super::numeric::$module::DifferentThan::new(value))),
                    _ => Err(Error::InvalidOperationForValue(operation, <$type>::VALUE_KIND)),
                }
            }
            pub(crate) fn with_str(operation: crate::Operation, value: &str) -> Result<Self, Error> {
                match value.parse::<$type>() {
                    Ok(value) => Self::with_value(operation, value),
                    Err(_) => Err(Error::FailToParseValue(value.to_string(), <$type>::VALUE_KIND)),
                }
            }

            pub(crate) fn with_str_list(operation: crate::Operation, values: &[&str]) -> Result<Self, Error> {
                match operation {
                    crate::Operation::InRange => Ok(Self::InsideRange(super::numeric::$module::InsideRange::with_str_list(values)?)),
                    _ => Err(Error::InvalidOperationForValue(operation, <$type>::VALUE_KIND)),
                }
            }
        }
    };
}

CREATE_PREDICATE_ENUM!(F32Predicate, f32, f32);
CREATE_PREDICATE_ENUM!(F64Predicate, f64, f64);