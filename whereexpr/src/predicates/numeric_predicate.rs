macro_rules! CREATE_NUMBER_PREDICATE_ENUM {
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
            IsOneOf(super::list_search::ListSearch<$type>),
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
                    Self::IsOneOf(p) => p.evaluate(value),
                }
            }

            pub(crate) fn new(operation: crate::Operation, value: $type) -> Option<Self> {
                match operation {
                    crate::Operation::GreaterThan => Some(Self::GreaterThan(super::numeric::$module::GreaterThan::new(value))),
                    crate::Operation::GreaterThanOrEqual => Some(Self::GreaterThanOrEqualTo(super::numeric::$module::GreaterThanOrEqualTo::new(value))),
                    crate::Operation::LessThan => Some(Self::SmallerThan(super::numeric::$module::SmallerThan::new(value))),
                    crate::Operation::LessThanOrEqual => Some(Self::SmallerThanOrEqualTo(super::numeric::$module::SmallerThanOrEqualTo::new(value))),
                    crate::Operation::Is => Some(Self::EqualTo(super::numeric::$module::EqualTo::new(value))),
                    crate::Operation::IsNot => Some(Self::DifferentThan(super::numeric::$module::DifferentThan::new(value))),
                    _ => None,
                }
            }

            pub(crate) fn new_with_values(operation: crate::Operation, values: &[String]) -> Option<Self> {
                match operation {
                    crate::Operation::InRange => Some(Self::InsideRange(super::numeric::$module::InsideRange::new(values)?)),
                    crate::Operation::IsOneOf => Some(Self::IsOneOf(super::list_search::ListSearch::new(values)?)),
                    _ => None,
                }
            }
        }
    };
}

CREATE_NUMBER_PREDICATE_ENUM!(I8Predicate,  i8,  i8);
CREATE_NUMBER_PREDICATE_ENUM!(I16Predicate, i16, i16);
CREATE_NUMBER_PREDICATE_ENUM!(I32Predicate, i32, i32);
CREATE_NUMBER_PREDICATE_ENUM!(I64Predicate, i64, i64);
CREATE_NUMBER_PREDICATE_ENUM!(U8Predicate,  u8,  u8);
CREATE_NUMBER_PREDICATE_ENUM!(U16Predicate, u16, u16);
CREATE_NUMBER_PREDICATE_ENUM!(U32Predicate, u32, u32);
CREATE_NUMBER_PREDICATE_ENUM!(U64Predicate, u64, u64);
CREATE_NUMBER_PREDICATE_ENUM!(F32Predicate, f32, f32);
CREATE_NUMBER_PREDICATE_ENUM!(F64Predicate, f64, f64);