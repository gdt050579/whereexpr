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

// signed number predicates
CREATE_PREDICATE!(SignedSmallerThenOrEqualTo, <= , i64);
CREATE_PREDICATE!(SignedSmallerThen, < , i64);
CREATE_PREDICATE!(SignedGreaterThenOrEqualTo, >= , i64);
CREATE_PREDICATE!(SignedGreaterThen, > , i64);
CREATE_PREDICATE!(SignedEqualTo, == , i64);
CREATE_PREDICATE!(SignedDifferentThen, != , i64);

// unsigned number predicates
CREATE_PREDICATE!(UnsignedSmallerThenOrEqualTo, <= , u64);
CREATE_PREDICATE!(UnsignedSmallerThen, < , u64);
CREATE_PREDICATE!(UnsignedGreaterThenOrEqualTo, >= , u64);
CREATE_PREDICATE!(UnsignedGreaterThen, > , u64);
CREATE_PREDICATE!(UnsignedEqualTo, == , u64);
CREATE_PREDICATE!(UnsignedDifferentThen, != , u64);

// Float number predicates
CREATE_PREDICATE!(FloatSmallerThenOrEqualTo, <= , f64);
CREATE_PREDICATE!(FloatSmallerThen, < , f64);
CREATE_PREDICATE!(FloatGreaterThenOrEqualTo, >= , f64);
CREATE_PREDICATE!(FloatGreaterThen, > , f64);
CREATE_PREDICATE!(FloatEqualTo, == , f64);
CREATE_PREDICATE!(FloatDifferentThen, != , f64);



macro_rules! CREATE_RANGE_PREDICATE {
    ($name:ident, $type:ty) => {
        #[derive(Debug)]
        pub(crate) struct $name {
            min: $type,
            max: $type,
        }

        impl $name {
            pub(crate) fn new(values: &[String]) -> Option<Self> {
                if values.len() != 2 {
                    return None;
                }
                let min = values[0].parse().ok()?;
                let max = values[1].parse().ok()?;
                if min > max {
                    return None;
                }
                Some(Self { min, max })
            }
            pub(crate) fn evaluate(&self, value: $type) -> bool {
                value >= self.min && value <= self.max
            }
        }
    };
}

CREATE_RANGE_PREDICATE!(SignedInsideRange, i64);
CREATE_RANGE_PREDICATE!(UnsignedInsideRange, u64);
CREATE_RANGE_PREDICATE!(FloatInsideRange, f64);
