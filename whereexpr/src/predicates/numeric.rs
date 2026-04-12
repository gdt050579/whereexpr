use crate::Error;
use crate::Value;
use crate::types::IntoValueKind;

macro_rules! CREATE_PREDICATE {
    ($name:ident, $op:tt, $type:ty) => {
        #[derive(Debug)]
        pub(crate) struct $name {
            value: $type,
        }

        impl $name {
            pub(crate) fn new(value: $type) -> Self {
                Self { value }
            }
            pub(crate) fn evaluate(&self, value: $type) -> bool {
                value $op self.value
            }
        }
    };
}

macro_rules! CREATE_RANGE_PREDICATE {
    ($name:ident, $type:ty) => {
        #[derive(Debug)]
        pub(crate) struct $name {
            min: $type,
            max: $type,
        }

        impl $name {
            pub(crate) fn with_str_list(values: &[&str]) -> Result<Self, Error> {
                if values.len() != 2 {
                    return Err(Error::ExpectingTwoValuesForRange(<$type>::VALUE_KIND));
                }
                let min = values[0].parse().map_err(|_| Error::FailToParseValue(values[0].to_string(), <$type>::VALUE_KIND))?;
                let max = values[1].parse().map_err(|_| Error::FailToParseValue(values[1].to_string(), <$type>::VALUE_KIND))?;
                if min > max {
                    return Err(Error::ExpectingMinToBeLessThanMax(<$type>::VALUE_KIND));
                }
                Ok(Self { min, max })
            }
            pub(crate) fn with_value_list<'a, T>(values: &[T]) -> Result<Self, Error>
            where
                $type: TryFrom<Value<'a>, Error=Error>,
                T: Into<Value<'a>> + Clone,
            {
                if values.len() != 2 {
                    return Err(Error::ExpectingTwoValuesForRange(<$type>::VALUE_KIND));
                }
                let min = <$type>::try_from(values[0].clone().into())?;
                let max = <$type>::try_from(values[1].clone().into())?;
                if min > max {
                    return Err(Error::ExpectingMinToBeLessThanMax(<$type>::VALUE_KIND));
                }
                Ok(Self { min, max })
            }
            pub(crate) fn evaluate(&self, value: $type) -> bool {
                value >= self.min && value <= self.max
            }
        }
    };
}

macro_rules! CREATE_NUMBER_PREDICATES {
    ($prefix:ident, $type:ty) => {
        pub(crate) mod $prefix {
            use super::*;
            CREATE_PREDICATE!(SmallerThanOrEqualTo, <=, $type);
            CREATE_PREDICATE!(SmallerThan, <, $type);
            CREATE_PREDICATE!(GreaterThanOrEqualTo, >=, $type);
            CREATE_PREDICATE!(GreaterThan, >, $type);
            CREATE_PREDICATE!(EqualTo, ==, $type);
            CREATE_PREDICATE!(DifferentThan, !=, $type);
            CREATE_RANGE_PREDICATE!(InsideRange, $type);
        }
    };
}

CREATE_NUMBER_PREDICATES!(i8, i8);
CREATE_NUMBER_PREDICATES!(i16, i16);
CREATE_NUMBER_PREDICATES!(i32, i32);
CREATE_NUMBER_PREDICATES!(i64, i64);
CREATE_NUMBER_PREDICATES!(u8, u8);
CREATE_NUMBER_PREDICATES!(u16, u16);
CREATE_NUMBER_PREDICATES!(u32, u32);
CREATE_NUMBER_PREDICATES!(u64, u64);
CREATE_NUMBER_PREDICATES!(f32, f32);
CREATE_NUMBER_PREDICATES!(f64, f64);
