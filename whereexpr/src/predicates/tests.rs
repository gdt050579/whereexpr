use crate::operation::Operation;
use crate::predicates::*;
use crate::Error;
use crate::ValueKind;

/// `$alt`: a value `!= 0` used to assert `Is 0` / `IsNot 0` behavior (e.g. `-1i8` or `2u8`).
macro_rules! integer_predicate_tests {
    (
        $mod_name:ident,
        $ty:ty,
        $pred:ident,
        $kind:expr,
        $wrong_ty:ty,
        $wrong_kind:expr,
        $alt:expr
    ) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn greater_than_evaluates() {
                let p = $pred::with_value(Operation::GreaterThan, 10 as $ty).unwrap();
                assert!(!p.evaluate(9 as $ty));
                assert!(!p.evaluate(10 as $ty));
                assert!(p.evaluate(11 as $ty));
            }

            #[test]
            fn greater_than_or_equal_evaluates() {
                let p = $pred::with_value(Operation::GreaterThanOrEqual, 10 as $ty).unwrap();
                assert!(!p.evaluate(9 as $ty));
                assert!(p.evaluate(10 as $ty));
                assert!(p.evaluate(11 as $ty));
            }

            #[test]
            fn less_than_evaluates() {
                let p = $pred::with_value(Operation::LessThan, 10 as $ty).unwrap();
                assert!(p.evaluate(9 as $ty));
                assert!(!p.evaluate(10 as $ty));
                assert!(!p.evaluate(11 as $ty));
            }

            #[test]
            fn less_than_or_equal_evaluates() {
                let p = $pred::with_value(Operation::LessThanOrEqual, 10 as $ty).unwrap();
                assert!(p.evaluate(9 as $ty));
                assert!(p.evaluate(10 as $ty));
                assert!(!p.evaluate(11 as $ty));
            }

            #[test]
            fn equal_to_evaluates() {
                let p = $pred::with_value(Operation::Is, 0 as $ty).unwrap();
                assert!(p.evaluate(0 as $ty));
                assert!(!p.evaluate(1 as $ty));
                assert!(!p.evaluate($alt));
            }

            #[test]
            fn inside_range_inclusive_boundaries() {
                let p = $pred::with_str_list(Operation::InRange, &["2", "5"]).unwrap();
                assert!(!p.evaluate(1 as $ty));
                assert!(p.evaluate(2 as $ty));
                assert!(p.evaluate(3 as $ty));
                assert!(p.evaluate(5 as $ty));
                assert!(!p.evaluate(6 as $ty));
            }

            #[test]
            fn inside_range_min_equals_max() {
                let p = $pred::with_str_list(Operation::InRange, &["7", "7"]).unwrap();
                assert!(!p.evaluate(6 as $ty));
                assert!(p.evaluate(7 as $ty));
                assert!(!p.evaluate(8 as $ty));
            }

            #[test]
            fn is_one_of_linear_search_path() {
                let p = $pred::with_str_list(Operation::IsOneOf, &["1", "3", "5"]).unwrap();
                assert!(p.evaluate(1 as $ty));
                assert!(!p.evaluate(2 as $ty));
                assert!(p.evaluate(5 as $ty));
            }

            #[test]
            fn is_one_of_dedupes_and_finds_after_sort() {
                let p = $pred::with_str_list(Operation::IsOneOf, &["5", "1", "5", "3"]).unwrap();
                assert!(p.evaluate(3 as $ty));
                assert!(p.evaluate(5 as $ty));
            }

            #[test]
            fn is_one_of_binary_search_path() {
                let parts: &[&str] = &[
                    "1", "2", "3", "4", "5", "6", "7", "8", "9",
                ];
                let p = $pred::with_str_list(Operation::IsOneOf, parts).unwrap();
                assert!(p.evaluate(5 as $ty));
                assert!(!p.evaluate(0 as $ty));
                assert!(!p.evaluate(10 as $ty));
            }

            #[test]
            fn with_value_rejects_non_numeric_operations() {
                let err = $pred::with_value(Operation::StartsWith, 0 as $ty).unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::StartsWith, k) if k == $kind
                ));
            }

            #[test]
            fn with_str_parses_and_builds() {
                let p = $pred::with_str(Operation::Is, "42").unwrap();
                assert!(p.evaluate(42 as $ty));
            }

            #[test]
            fn with_str_parse_error() {
                let err = $pred::with_str(Operation::Is, "not-a-number").unwrap_err();
                assert!(matches!(
                    err,
                    Error::FailToParseValue(s, k) if s == "not-a-number" && k == $kind
                ));
            }

            #[test]
            fn with_str_list_in_range_wrong_length() {
                let err = $pred::with_str_list(Operation::InRange, &["1"]).unwrap_err();
                assert!(matches!(err, Error::ExpectingTwoValuesForRange(k) if k == $kind));

                let err = $pred::with_str_list(Operation::InRange, &["1", "2", "3"]).unwrap_err();
                assert!(matches!(err, Error::ExpectingTwoValuesForRange(k) if k == $kind));
            }

            #[test]
            fn with_str_list_in_range_min_greater_than_max() {
                let err = $pred::with_str_list(Operation::InRange, &["9", "1"]).unwrap_err();
                assert!(matches!(err, Error::ExpectingMinToBeLessThanMax(k) if k == $kind));
            }

            #[test]
            fn with_str_list_in_range_parse_error() {
                let err = $pred::with_str_list(Operation::InRange, &["x", "2"]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::FailToParseValue(s, k) if s == "x" && k == $kind
                ));
            }

            #[test]
            fn with_str_list_in_range_parse_error_on_second_bound() {
                let err = $pred::with_str_list(Operation::InRange, &["1", "not"]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::FailToParseValue(s, k) if s == "not" && k == $kind
                ));
            }

            #[test]
            fn with_str_list_is_one_of_parse_error() {
                let err = $pred::with_str_list(Operation::IsOneOf, &["1", "not-a-number"]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::FailToParseValue(s, k) if s == "not-a-number" && k == $kind
                ));
            }

            #[test]
            fn with_str_list_is_one_of_empty() {
                let err = $pred::with_str_list(Operation::IsOneOf, &[]).unwrap_err();
                assert!(matches!(err, Error::EmptyListForIsOneOf(k) if k == $kind));
            }

            #[test]
            fn with_str_list_rejects_invalid_operation() {
                let err = $pred::with_str_list(Operation::Is, &["1", "2"]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::Is, k) if k == $kind
                ));
            }

            #[test]
            fn with_value_list_in_range() {
                let p = $pred::with_value_list(Operation::InRange, &[2 as $ty, 5 as $ty]).unwrap();
                assert!(p.evaluate(4 as $ty));
            }

            #[test]
            fn with_value_list_in_range_wrong_len() {
                let err = $pred::with_value_list(Operation::InRange, &[1 as $ty]).unwrap_err();
                assert!(matches!(err, Error::ExpectingTwoValuesForRange(k) if k == $kind));
            }

            #[test]
            fn with_value_list_in_range_min_greater_than_max() {
                let err = $pred::with_value_list(Operation::InRange, &[9 as $ty, 1 as $ty]).unwrap_err();
                assert!(matches!(err, Error::ExpectingMinToBeLessThanMax(k) if k == $kind));
            }

            #[test]
            fn with_value_list_in_range_wrong_value_kind() {
                let err = $pred::with_value_list(
                    Operation::InRange,
                    &[1 as $wrong_ty, 2 as $wrong_ty],
                )
                .unwrap_err();
                assert!(matches!(
                    err,
                    Error::ExpectingADifferentValueKind(got, expected)
                        if got == $wrong_kind && expected == $kind
                ));
            }

            #[test]
            fn with_value_list_is_one_of() {
                let p = $pred::with_value_list(Operation::IsOneOf, &[1 as $ty, 3 as $ty]).unwrap();
                assert!(p.evaluate(3 as $ty));
                assert!(!p.evaluate(2 as $ty));
            }

            #[test]
            fn with_value_list_is_one_of_empty_matches_nothing() {
                let empty: Vec<$ty> = Vec::new();
                let p = $pred::with_value_list(Operation::IsOneOf, &empty).unwrap();
                assert!(!p.evaluate(0 as $ty));
            }

            #[test]
            fn with_value_list_is_one_of_wrong_value_kind() {
                let err = $pred::with_value_list(Operation::IsOneOf, &[1 as $wrong_ty]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::ExpectingADifferentValueKind(got, expected)
                        if got == $wrong_kind && expected == $kind
                ));
            }

            #[test]
            fn with_value_list_rejects_invalid_operation() {
                let err = $pred::with_value_list(
                    Operation::GreaterThan,
                    &[1 as $ty, 2 as $ty],
                )
                .unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::GreaterThan, k) if k == $kind
                ));
            }
        }
    };
}

/// Float predicates support scalar ops and `InRange` only (no `IsOneOf` lists).
macro_rules! float_predicate_tests {
    (
        $mod_name:ident,
        $ty:ty,
        $pred:ident,
        $kind:expr,
        $wrong_ty:ty,
        $wrong_kind:expr,
        $alt:expr
    ) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn greater_than_evaluates() {
                let p = $pred::with_value(Operation::GreaterThan, 10.0 as $ty).unwrap();
                assert!(!p.evaluate(9.0 as $ty));
                assert!(!p.evaluate(10.0 as $ty));
                assert!(p.evaluate(11.0 as $ty));
            }

            #[test]
            fn greater_than_or_equal_evaluates() {
                let p = $pred::with_value(Operation::GreaterThanOrEqual, 10.0 as $ty).unwrap();
                assert!(!p.evaluate(9.0 as $ty));
                assert!(p.evaluate(10.0 as $ty));
                assert!(p.evaluate(11.0 as $ty));
            }

            #[test]
            fn less_than_evaluates() {
                let p = $pred::with_value(Operation::LessThan, 10.0 as $ty).unwrap();
                assert!(p.evaluate(9.0 as $ty));
                assert!(!p.evaluate(10.0 as $ty));
                assert!(!p.evaluate(11.0 as $ty));
            }

            #[test]
            fn less_than_or_equal_evaluates() {
                let p = $pred::with_value(Operation::LessThanOrEqual, 10.0 as $ty).unwrap();
                assert!(p.evaluate(9.0 as $ty));
                assert!(p.evaluate(10.0 as $ty));
                assert!(!p.evaluate(11.0 as $ty));
            }

            #[test]
            fn equal_to_evaluates() {
                let p = $pred::with_value(Operation::Is, 0.0 as $ty).unwrap();
                assert!(p.evaluate(0.0 as $ty));
                assert!(!p.evaluate(1.0 as $ty));
                assert!(!p.evaluate($alt));
            }

            #[test]
            fn inside_range_inclusive_boundaries() {
                let p = $pred::with_str_list(Operation::InRange, &["2", "5"]).unwrap();
                assert!(!p.evaluate(1.0 as $ty));
                assert!(p.evaluate(2.0 as $ty));
                assert!(p.evaluate(3.5 as $ty));
                assert!(p.evaluate(5.0 as $ty));
                assert!(!p.evaluate(6.0 as $ty));
            }

            #[test]
            fn inside_range_min_equals_max() {
                let p = $pred::with_str_list(Operation::InRange, &["7", "7"]).unwrap();
                assert!(!p.evaluate(6.0 as $ty));
                assert!(p.evaluate(7.0 as $ty));
                assert!(!p.evaluate(8.0 as $ty));
            }

            #[test]
            fn with_value_rejects_non_numeric_operations() {
                let err = $pred::with_value(Operation::StartsWith, 0.0 as $ty).unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::StartsWith, k) if k == $kind
                ));
            }

            #[test]
            fn with_str_parses_and_builds() {
                let p = $pred::with_str(Operation::Is, "10.25").unwrap();
                assert!(p.evaluate(10.25 as $ty));
            }

            #[test]
            fn with_str_parse_error() {
                let err = $pred::with_str(Operation::Is, "not-a-number").unwrap_err();
                assert!(matches!(
                    err,
                    Error::FailToParseValue(s, k) if s == "not-a-number" && k == $kind
                ));
            }

            #[test]
            fn with_str_list_in_range_wrong_length() {
                let err = $pred::with_str_list(Operation::InRange, &["1"]).unwrap_err();
                assert!(matches!(err, Error::ExpectingTwoValuesForRange(k) if k == $kind));

                let err = $pred::with_str_list(Operation::InRange, &["1", "2", "3"]).unwrap_err();
                assert!(matches!(err, Error::ExpectingTwoValuesForRange(k) if k == $kind));
            }

            #[test]
            fn with_str_list_in_range_min_greater_than_max() {
                let err = $pred::with_str_list(Operation::InRange, &["9", "1"]).unwrap_err();
                assert!(matches!(err, Error::ExpectingMinToBeLessThanMax(k) if k == $kind));
            }

            #[test]
            fn with_str_list_in_range_parse_error() {
                let err = $pred::with_str_list(Operation::InRange, &["x", "2"]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::FailToParseValue(s, k) if s == "x" && k == $kind
                ));
            }

            #[test]
            fn with_str_list_in_range_parse_error_on_second_bound() {
                let err = $pred::with_str_list(Operation::InRange, &["1", "not"]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::FailToParseValue(s, k) if s == "not" && k == $kind
                ));
            }

            #[test]
            fn with_str_list_rejects_invalid_operation() {
                let err = $pred::with_str_list(Operation::Is, &["1", "2"]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::Is, k) if k == $kind
                ));
            }

            #[test]
            fn with_str_list_rejects_is_one_of() {
                let err = $pred::with_str_list(Operation::IsOneOf, &["1", "2"]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::IsOneOf, k) if k == $kind
                ));
            }

            #[test]
            fn with_value_list_in_range() {
                let p = $pred::with_value_list(Operation::InRange, &[2.0 as $ty, 5.0 as $ty]).unwrap();
                assert!(p.evaluate(4.0 as $ty));
            }

            #[test]
            fn with_value_list_in_range_wrong_len() {
                let err = $pred::with_value_list(Operation::InRange, &[1.0 as $ty]).unwrap_err();
                assert!(matches!(err, Error::ExpectingTwoValuesForRange(k) if k == $kind));
            }

            #[test]
            fn with_value_list_in_range_min_greater_than_max() {
                let err =
                    $pred::with_value_list(Operation::InRange, &[9.0 as $ty, 1.0 as $ty]).unwrap_err();
                assert!(matches!(err, Error::ExpectingMinToBeLessThanMax(k) if k == $kind));
            }

            #[test]
            fn with_value_list_in_range_wrong_value_kind() {
                let err = $pred::with_value_list(
                    Operation::InRange,
                    &[1 as $wrong_ty, 2 as $wrong_ty],
                )
                .unwrap_err();
                assert!(matches!(
                    err,
                    Error::ExpectingADifferentValueKind(got, expected)
                        if got == $wrong_kind && expected == $kind
                ));
            }

            #[test]
            fn with_value_list_rejects_invalid_operation() {
                let err = $pred::with_value_list(
                    Operation::GreaterThan,
                    &[1.0 as $ty, 2.0 as $ty],
                )
                .unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::GreaterThan, k) if k == $kind
                ));
            }

            #[test]
            fn with_value_list_rejects_is_one_of() {
                let err = $pred::with_value_list(
                    Operation::IsOneOf,
                    &[1.0 as $ty, 2.0 as $ty],
                )
                .unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::IsOneOf, k) if k == $kind
                ));
            }
        }
    };
}

integer_predicate_tests!(i8_predicate, i8, I8Predicate, ValueKind::I8, i32, ValueKind::I32, (-1i8));
integer_predicate_tests!(i16_predicate, i16, I16Predicate, ValueKind::I16, i32, ValueKind::I32, (-1i16));
integer_predicate_tests!(i32_predicate, i32, I32Predicate, ValueKind::I32, i8, ValueKind::I8, (-1i32));
integer_predicate_tests!(i64_predicate, i64, I64Predicate, ValueKind::I64, i32, ValueKind::I32, (-1i64));

integer_predicate_tests!(u8_predicate, u8, U8Predicate, ValueKind::U8, i32, ValueKind::I32, (2u8));
integer_predicate_tests!(u16_predicate, u16, U16Predicate, ValueKind::U16, i32, ValueKind::I32, (2u16));
integer_predicate_tests!(u32_predicate, u32, U32Predicate, ValueKind::U32, i8, ValueKind::I8, (2u32));
integer_predicate_tests!(u64_predicate, u64, U64Predicate, ValueKind::U64, i32, ValueKind::I32, (2u64));

float_predicate_tests!(f32_predicate, f32, F32Predicate, ValueKind::F32, i8, ValueKind::I8, -0.5f32);
float_predicate_tests!(f64_predicate, f64, F64Predicate, ValueKind::F64, i32, ValueKind::I32, -0.5f64);

mod bool_predicate_tests {
    use super::*;

    #[test]
    fn is_true_and_false_evaluate() {
        let pt = BoolPredicate::with_value(Operation::Is, true).unwrap();
        assert!(pt.evaluate(true));
        assert!(!pt.evaluate(false));

        let pf = BoolPredicate::with_value(Operation::Is, false).unwrap();
        assert!(!pf.evaluate(true));
        assert!(pf.evaluate(false));
    }

    #[test]
    fn with_value_rejects_is_not() {
        let err = BoolPredicate::with_value(Operation::IsNot, true).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::IsNot, ValueKind::Bool)
        ));
    }

    #[test]
    fn with_value_rejects_greater_than() {
        let err = BoolPredicate::with_value(Operation::GreaterThan, true).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::GreaterThan, ValueKind::Bool)
        ));
    }

    #[test]
    fn with_str_parses_false() {
        let p = BoolPredicate::with_str(Operation::Is, "false").unwrap();
        assert!(p.evaluate(false));
        assert!(!p.evaluate(true));
    }

    #[test]
    fn with_str_parses_true() {
        let p = BoolPredicate::with_str(Operation::Is, "true").unwrap();
        assert!(p.evaluate(true));
    }

    #[test]
    fn with_str_parse_error() {
        let err = BoolPredicate::with_str(Operation::Is, "maybe").unwrap_err();
        assert!(matches!(
            err,
            Error::FailToParseValue(s, k) if s == "maybe" && k == ValueKind::Bool
        ));
    }

    #[test]
    fn with_str_rejects_is_not() {
        let err = BoolPredicate::with_str(Operation::IsNot, "true").unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::IsNot, ValueKind::Bool)
        ));
    }

    #[test]
    fn with_str_case_sensitive() {
        let err = BoolPredicate::with_str(Operation::Is, "True").unwrap_err();
        assert!(matches!(
            err,
            Error::FailToParseValue(s, k) if s == "True" && k == ValueKind::Bool
        ));
    }
}

mod ip_addr_predicate_tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    fn ip(s: &str) -> IpAddr {
        IpAddr::from_str(s).unwrap()
    }

    #[test]
    fn equals_evaluates() {
        let p = IpAddrPredicate::with_value(Operation::Is, ip("192.168.1.10")).unwrap();
        assert!(p.evaluate(ip("192.168.1.10")));
        assert!(!p.evaluate(ip("192.168.1.11")));
    }

    #[test]
    fn equals_ipv6() {
        let p = IpAddrPredicate::with_str(Operation::Is, "2001:db8::1").unwrap();
        assert!(p.evaluate(ip("2001:db8::1")));
        assert!(!p.evaluate(ip("2001:db8::2")));
    }

    #[test]
    fn with_value_rejects_greater_than() {
        let err = IpAddrPredicate::with_value(Operation::GreaterThan, ip("0.0.0.0")).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::GreaterThan, ValueKind::IpAddr)
        ));
    }

    #[test]
    fn with_str_parse_error() {
        let err = IpAddrPredicate::with_str(Operation::Is, "not-an-ip").unwrap_err();
        assert!(matches!(
            err,
            Error::FailToParseValue(s, k) if s == "not-an-ip" && k == ValueKind::IpAddr
        ));
    }

    #[test]
    fn with_str_list_in_range_inclusive_and_trim() {
        let p = IpAddrPredicate::with_str_list(Operation::InRange, &["  10.0.0.2  ", "  10.0.0.5  "]).unwrap();
        assert!(!p.evaluate(ip("10.0.0.1")));
        assert!(p.evaluate(ip("10.0.0.2")));
        assert!(p.evaluate(ip("10.0.0.4")));
        assert!(p.evaluate(ip("10.0.0.5")));
        assert!(!p.evaluate(ip("10.0.0.6")));
    }

    #[test]
    fn with_str_list_in_range_min_equals_max() {
        let p = IpAddrPredicate::with_str_list(Operation::InRange, &["172.16.0.7", "172.16.0.7"]).unwrap();
        assert!(!p.evaluate(ip("172.16.0.6")));
        assert!(p.evaluate(ip("172.16.0.7")));
        assert!(!p.evaluate(ip("172.16.0.8")));
    }

    #[test]
    fn with_str_list_in_range_wrong_length() {
        let err = IpAddrPredicate::with_str_list(Operation::InRange, &["10.0.0.1"]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingTwoValuesForRange(ValueKind::IpAddr)
        ));

        let err = IpAddrPredicate::with_str_list(Operation::InRange, &["10.0.0.1", "10.0.0.2", "10.0.0.3"])
            .unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingTwoValuesForRange(ValueKind::IpAddr)
        ));
    }

    #[test]
    fn with_str_list_in_range_start_greater_than_end() {
        let err =
            IpAddrPredicate::with_str_list(Operation::InRange, &["10.0.0.9", "10.0.0.1"]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingMinToBeLessThanMax(ValueKind::IpAddr)
        ));
    }

    #[test]
    fn with_str_list_in_range_parse_error_first() {
        let err = IpAddrPredicate::with_str_list(Operation::InRange, &["nope", "10.0.0.1"]).unwrap_err();
        assert!(matches!(
            err,
            Error::FailToParseValue(s, k) if s == "nope" && k == ValueKind::IpAddr
        ));
    }

    #[test]
    fn with_str_list_in_range_parse_error_second() {
        let err = IpAddrPredicate::with_str_list(Operation::InRange, &["10.0.0.1", "bad"]).unwrap_err();
        assert!(matches!(
            err,
            Error::FailToParseValue(s, k) if s == "bad" && k == ValueKind::IpAddr
        ));
    }

    #[test]
    fn with_str_list_is_one_of_linear_search_path() {
        let p = IpAddrPredicate::with_str_list(
            Operation::IsOneOf,
            &["10.0.0.1", "10.0.0.3", "10.0.0.5"],
        )
        .unwrap();
        assert!(p.evaluate(ip("10.0.0.1")));
        assert!(!p.evaluate(ip("10.0.0.2")));
        assert!(p.evaluate(ip("10.0.0.5")));
    }

    #[test]
    fn with_str_list_is_one_of_dedupes_and_sorts() {
        let p = IpAddrPredicate::with_str_list(
            Operation::IsOneOf,
            &["10.0.0.5", "10.0.0.1", "10.0.0.5", "10.0.0.3"],
        )
        .unwrap();
        assert!(p.evaluate(ip("10.0.0.3")));
        assert!(p.evaluate(ip("10.0.0.5")));
    }

    #[test]
    fn with_str_list_is_one_of_binary_search_path() {
        let parts: &[&str] = &[
            "10.0.0.1",
            "10.0.0.2",
            "10.0.0.3",
            "10.0.0.4",
            "10.0.0.5",
            "10.0.0.6",
            "10.0.0.7",
            "10.0.0.8",
            "10.0.0.9",
        ];
        let p = IpAddrPredicate::with_str_list(Operation::IsOneOf, parts).unwrap();
        assert!(p.evaluate(ip("10.0.0.5")));
        assert!(!p.evaluate(ip("10.0.0.0")));
        assert!(!p.evaluate(ip("10.0.0.10")));
    }

    #[test]
    fn with_str_list_is_one_of_empty() {
        let err = IpAddrPredicate::with_str_list(Operation::IsOneOf, &[]).unwrap_err();
        assert!(matches!(
            err,
            Error::EmptyListForIsOneOf(ValueKind::IpAddr)
        ));
    }

    #[test]
    fn with_str_list_is_one_of_parse_error() {
        let err = IpAddrPredicate::with_str_list(Operation::IsOneOf, &["10.0.0.1", "not-an-ip"]).unwrap_err();
        assert!(matches!(
            err,
            Error::FailToParseValue(s, k) if s == "not-an-ip" && k == ValueKind::IpAddr
        ));
    }

    #[test]
    fn with_str_list_rejects_invalid_operation() {
        let err = IpAddrPredicate::with_str_list(Operation::Is, &["10.0.0.1", "10.0.0.2"]).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::Is, ValueKind::IpAddr)
        ));
    }

    #[test]
    fn with_value_list_in_range() {
        let p = IpAddrPredicate::with_value_list(
            Operation::InRange,
            &[ip("192.168.0.10"), ip("192.168.0.20")],
        )
        .unwrap();
        assert!(p.evaluate(ip("192.168.0.15")));
        assert!(!p.evaluate(ip("192.168.0.9")));
    }

    #[test]
    fn with_value_list_in_range_wrong_len() {
        let err = IpAddrPredicate::with_value_list(Operation::InRange, &[ip("10.0.0.1")]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingTwoValuesForRange(ValueKind::IpAddr)
        ));
    }

    #[test]
    fn with_value_list_in_range_start_greater_than_end() {
        let err = IpAddrPredicate::with_value_list(
            Operation::InRange,
            &[ip("10.0.0.5"), ip("10.0.0.1")],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingMinToBeLessThanMax(ValueKind::IpAddr)
        ));
    }

    #[test]
    fn with_value_list_in_range_wrong_value_kind() {
        let err = IpAddrPredicate::with_value_list(Operation::InRange, &[1_i32, 2_i32]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingADifferentValueKind(got, expected)
                if got == ValueKind::I32 && expected == ValueKind::IpAddr
        ));
    }

    #[test]
    fn with_value_list_is_one_of() {
        let p = IpAddrPredicate::with_value_list(
            Operation::IsOneOf,
            &[ip("10.0.0.1"), ip("10.0.0.3")],
        )
        .unwrap();
        assert!(p.evaluate(ip("10.0.0.3")));
        assert!(!p.evaluate(ip("10.0.0.2")));
    }

    #[test]
    fn with_value_list_is_one_of_empty_matches_nothing() {
        let empty: Vec<IpAddr> = Vec::new();
        let p = IpAddrPredicate::with_value_list(Operation::IsOneOf, &empty).unwrap();
        assert!(!p.evaluate(ip("127.0.0.1")));
    }

    #[test]
    fn with_value_list_is_one_of_wrong_value_kind() {
        let err = IpAddrPredicate::with_value_list(Operation::IsOneOf, &[1_i32]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingADifferentValueKind(got, expected)
                if got == ValueKind::I32 && expected == ValueKind::IpAddr
        ));
    }

    #[test]
    fn with_value_list_rejects_invalid_operation() {
        let err = IpAddrPredicate::with_value_list(
            Operation::GreaterThan,
            &[ip("10.0.0.1"), ip("10.0.0.2")],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::GreaterThan, ValueKind::IpAddr)
        ));
    }
}

/// `$zero_repeat`: count of `'0'` digits before `{:02x}` so `repeat + 2` equals the hash’s hex length (32 / 40 / 64).
macro_rules! hash_type_predicate_tests {
    (
        $mod_name:ident,
        $pred:ident,
        $hash_ty:ident,
        $kind:expr,
        $wrong_ty:ty,
        $wrong_kind:expr,
        $zero_repeat:literal,
        $zero_hex:literal,
        $alt_hex:literal
    ) => {
        mod $mod_name {
            use super::*;
            use crate::types::$hash_ty;

            fn zero() -> $hash_ty {
                $zero_hex.parse().unwrap()
            }

            fn alt() -> $hash_ty {
                $alt_hex.parse().unwrap()
            }

            #[test]
            fn equals_evaluates() {
                let p = $pred::with_value(Operation::Is, zero()).unwrap();
                assert!(p.evaluate(zero()));
                assert!(!p.evaluate(alt()));
            }

            #[test]
            fn with_value_rejects_greater_than() {
                let err = $pred::with_value(Operation::GreaterThan, zero()).unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::GreaterThan, k) if k == $kind
                ));
            }

            #[test]
            fn with_str_parses_hex_and_trims() {
                let s = format!("  {}  ", $zero_hex);
                let p = $pred::with_str(Operation::Is, s.as_str()).unwrap();
                assert!(p.evaluate(zero()));
            }

            #[test]
            fn with_str_accepts_uppercase_hex() {
                let upper: String = $zero_hex.to_ascii_uppercase();
                let p = $pred::with_str(Operation::Is, upper.as_str()).unwrap();
                assert!(p.evaluate(zero()));
            }

            #[test]
            fn with_str_parse_error_wrong_length() {
                let err = $pred::with_str(Operation::Is, "00").unwrap_err();
                assert!(matches!(
                    err,
                    Error::FailToParseValue(s, k) if s == "00" && k == $kind
                ));
            }

            #[test]
            fn with_str_parse_error_invalid_hex() {
                let mut bad = String::from($zero_hex);
                bad.pop();
                bad.pop();
                bad.push_str("gg");
                let err = $pred::with_str(Operation::Is, bad.as_str()).unwrap_err();
                assert!(matches!(err, Error::FailToParseValue(_, k) if k == $kind));
            }

            #[test]
            fn with_str_list_is_one_of_linear_search_path() {
                let a = format!("{}{:02x}", "0".repeat($zero_repeat), 1u8);
                let b = format!("{}{:02x}", "0".repeat($zero_repeat), 3u8);
                let c = format!("{}{:02x}", "0".repeat($zero_repeat), 5u8);
                let p = $pred::with_str_list(Operation::IsOneOf, &[a.as_str(), b.as_str(), c.as_str()]).unwrap();
                let h1: $hash_ty = a.parse().unwrap();
                let h_mid: $hash_ty = format!("{}{:02x}", "0".repeat($zero_repeat), 2u8)
                    .parse()
                    .unwrap();
                let h5: $hash_ty = c.parse().unwrap();
                assert!(p.evaluate(h1));
                assert!(!p.evaluate(h_mid));
                assert!(p.evaluate(h5));
            }

            #[test]
            fn with_str_list_is_one_of_dedupes_and_sorts() {
                let a = format!("{}{:02x}", "0".repeat($zero_repeat), 1u8);
                let b = format!("{}{:02x}", "0".repeat($zero_repeat), 3u8);
                let c = format!("{}{:02x}", "0".repeat($zero_repeat), 5u8);
                let p = $pred::with_str_list(Operation::IsOneOf, &[c.as_str(), a.as_str(), c.as_str(), b.as_str()])
                    .unwrap();
                let h3: $hash_ty = b.parse().unwrap();
                let h5: $hash_ty = c.parse().unwrap();
                assert!(p.evaluate(h3));
                assert!(p.evaluate(h5));
            }

            #[test]
            fn with_str_list_is_one_of_binary_search_path() {
                let strings: Vec<String> = (1u8..=9)
                    .map(|i| format!("{}{:02x}", "0".repeat($zero_repeat), i))
                    .collect();
                let refs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
                let p = $pred::with_str_list(Operation::IsOneOf, &refs).unwrap();
                let mid: $hash_ty = format!("{}{:02x}", "0".repeat($zero_repeat), 5u8)
                    .parse()
                    .unwrap();
                let not_in: $hash_ty = format!("{}{:02x}", "0".repeat($zero_repeat), 10u8)
                    .parse()
                    .unwrap();
                assert!(p.evaluate(mid));
                assert!(!p.evaluate(zero()));
                assert!(!p.evaluate(not_in));
            }

            #[test]
            fn with_str_list_is_one_of_empty() {
                let err = $pred::with_str_list(Operation::IsOneOf, &[]).unwrap_err();
                assert!(matches!(err, Error::EmptyListForIsOneOf(k) if k == $kind));
            }

            #[test]
            fn with_str_list_is_one_of_parse_error() {
                let good = format!("{}{:02x}", "0".repeat($zero_repeat), 1u8);
                let mut bad = String::from($zero_hex);
                bad.pop();
                bad.pop();
                bad.push_str("zz");
                let err = $pred::with_str_list(Operation::IsOneOf, &[good.as_str(), bad.as_str()]).unwrap_err();
                assert!(matches!(err, Error::FailToParseValue(_, k) if k == $kind));
            }

            #[test]
            fn with_str_list_rejects_in_range() {
                let err = $pred::with_str_list(Operation::InRange, &[$zero_hex, $alt_hex]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::InRange, k) if k == $kind
                ));
            }

            #[test]
            fn with_str_list_rejects_is() {
                let err = $pred::with_str_list(Operation::Is, &[$zero_hex]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::Is, k) if k == $kind
                ));
            }

            #[test]
            fn with_value_list_is_one_of() {
                let a: $hash_ty = format!("{}{:02x}", "0".repeat($zero_repeat), 1u8)
                    .parse()
                    .unwrap();
                let b: $hash_ty = format!("{}{:02x}", "0".repeat($zero_repeat), 3u8)
                    .parse()
                    .unwrap();
                let p = $pred::with_value_list(Operation::IsOneOf, &[&a, &b]).unwrap();
                assert!(p.evaluate(b));
                let mid: $hash_ty = format!("{}{:02x}", "0".repeat($zero_repeat), 2u8)
                    .parse()
                    .unwrap();
                assert!(!p.evaluate(mid));
            }

            #[test]
            fn with_value_list_is_one_of_empty_matches_nothing() {
                let empty: Vec<&$hash_ty> = Vec::new();
                let p = $pred::with_value_list(Operation::IsOneOf, &empty).unwrap();
                assert!(!p.evaluate(zero()));
            }

            #[test]
            fn with_value_list_is_one_of_wrong_value_kind() {
                let err = $pred::with_value_list(Operation::IsOneOf, &[1 as $wrong_ty]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::ExpectingADifferentValueKind(got, expected)
                        if got == $wrong_kind && expected == $kind
                ));
            }

            #[test]
            fn with_value_list_rejects_greater_than() {
                let a = zero();
                let b = alt();
                let err = $pred::with_value_list(Operation::GreaterThan, &[&a, &b]).unwrap_err();
                assert!(matches!(
                    err,
                    Error::InvalidOperationForValue(Operation::GreaterThan, k) if k == $kind
                ));
            }
        }
    };
}

hash_type_predicate_tests!(
    hash128_predicate_tests,
    Hash128Predicate,
    Hash128,
    ValueKind::Hash128,
    i32,
    ValueKind::I32,
    30usize,
    "00000000000000000000000000000000",
    "00000000000000000000000000000001"
);
hash_type_predicate_tests!(
    hash160_predicate_tests,
    Hash160Predicate,
    Hash160,
    ValueKind::Hash160,
    i32,
    ValueKind::I32,
    38usize,
    "0000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000001"
);
hash_type_predicate_tests!(
    hash256_predicate_tests,
    Hash256Predicate,
    Hash256,
    ValueKind::Hash256,
    i32,
    ValueKind::I32,
    62usize,
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000001"
);

mod datetime_predicate_tests {
    use super::*;
    use crate::types::{DateTime, FromRepr};

    fn ts(s: &str) -> u64 {
        DateTime::from_repr(s).unwrap().into()
    }

    #[test]
    fn greater_than_evaluates() {
        let t = ts("2020-06-15");
        let p = DateTimePredicate::with_value(Operation::GreaterThan, t).unwrap();
        assert!(!p.evaluate(ts("2020-06-14")));
        assert!(!p.evaluate(t));
        assert!(p.evaluate(ts("2020-06-16")));
    }

    #[test]
    fn greater_than_or_equal_evaluates() {
        let t = ts("2020-06-15");
        let p = DateTimePredicate::with_value(Operation::GreaterThanOrEqual, t).unwrap();
        assert!(!p.evaluate(ts("2020-06-14")));
        assert!(p.evaluate(t));
        assert!(p.evaluate(ts("2020-06-16")));
    }

    #[test]
    fn less_than_evaluates() {
        let t = ts("2020-06-15");
        let p = DateTimePredicate::with_value(Operation::LessThan, t).unwrap();
        assert!(p.evaluate(ts("2020-06-14")));
        assert!(!p.evaluate(t));
        assert!(!p.evaluate(ts("2020-06-16")));
    }

    #[test]
    fn less_than_or_equal_evaluates() {
        let t = ts("2020-06-15");
        let p = DateTimePredicate::with_value(Operation::LessThanOrEqual, t).unwrap();
        assert!(p.evaluate(ts("2020-06-14")));
        assert!(p.evaluate(t));
        assert!(!p.evaluate(ts("2020-06-16")));
    }

    #[test]
    fn equal_to_evaluates() {
        let t0 = ts("2020-03-01");
        let t1 = ts("2020-03-02");
        let p = DateTimePredicate::with_value(Operation::Is, t0).unwrap();
        assert!(p.evaluate(t0));
        assert!(!p.evaluate(t1));
    }

    #[test]
    fn with_value_rejects_starts_with() {
        let err = DateTimePredicate::with_value(Operation::StartsWith, 0).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::StartsWith, ValueKind::DateTime)
        ));
    }

    #[test]
    fn with_str_parses_date_only() {
        let t = ts("2019-12-25");
        let p = DateTimePredicate::with_str(Operation::Is, "2019-12-25").unwrap();
        assert!(p.evaluate(t));
    }

    #[test]
    fn with_str_parses_slash_separator() {
        let t = DateTime::from_repr("2021/07/04").unwrap().into();
        let p = DateTimePredicate::with_str(Operation::Is, "2021/07/04").unwrap();
        assert!(p.evaluate(t));
    }

    #[test]
    fn with_str_parses_datetime_with_t_and_z() {
        let t = DateTime::from_repr("2022-05-10T14:30:00Z").unwrap().into();
        let p = DateTimePredicate::with_str(Operation::Is, "2022-05-10T14:30:00Z").unwrap();
        assert!(p.evaluate(t));
    }

    #[test]
    fn with_str_parses_time_with_spaces_after_date() {
        let t = DateTime::from_repr("2022-05-10   08:15").unwrap().into();
        let p = DateTimePredicate::with_str(Operation::Is, "2022-05-10   08:15").unwrap();
        assert!(p.evaluate(t));
    }

    #[test]
    fn with_str_parse_error() {
        let err = DateTimePredicate::with_str(Operation::Is, "not-a-datetime").unwrap_err();
        assert!(matches!(
            err,
            Error::FailToParseValue(s, k) if s == "not-a-datetime" && k == ValueKind::DateTime
        ));
    }

    #[test]
    fn with_str_list_in_range_inclusive_boundaries() {
        let p = DateTimePredicate::with_str_list(Operation::InRange, &["2020-06-10", "2020-06-20"]).unwrap();
        assert!(!p.evaluate(ts("2020-06-09")));
        assert!(p.evaluate(ts("2020-06-10")));
        assert!(p.evaluate(ts("2020-06-15")));
        assert!(p.evaluate(ts("2020-06-20")));
        assert!(!p.evaluate(ts("2020-06-21")));
    }

    #[test]
    fn with_str_list_in_range_min_equals_max() {
        let day = ts("2020-08-08");
        let p = DateTimePredicate::with_str_list(Operation::InRange, &["2020-08-08", "2020-08-08"]).unwrap();
        assert!(!p.evaluate(ts("2020-08-07")));
        assert!(p.evaluate(day));
        assert!(!p.evaluate(ts("2020-08-09")));
    }

    #[test]
    fn with_str_list_in_range_wrong_length() {
        let err = DateTimePredicate::with_str_list(Operation::InRange, &["2020-01-01"]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingTwoValuesForRange(ValueKind::DateTime)
        ));

        let err = DateTimePredicate::with_str_list(
            Operation::InRange,
            &["2020-01-01", "2020-01-02", "2020-01-03"],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingTwoValuesForRange(ValueKind::DateTime)
        ));
    }

    #[test]
    fn with_str_list_in_range_min_greater_than_max() {
        let err = DateTimePredicate::with_str_list(Operation::InRange, &["2020-06-20", "2020-06-10"]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingMinToBeLessThanMax(ValueKind::DateTime)
        ));
    }

    #[test]
    fn with_str_list_in_range_parse_error_first() {
        let err = DateTimePredicate::with_str_list(Operation::InRange, &["bad-date", "2020-01-01"]).unwrap_err();
        assert!(matches!(
            err,
            Error::FailToParseValue(s, k) if s == "bad-date" && k == ValueKind::DateTime
        ));
    }

    #[test]
    fn with_str_list_in_range_parse_error_second() {
        let err = DateTimePredicate::with_str_list(Operation::InRange, &["2020-01-01", "oops"]).unwrap_err();
        assert!(matches!(
            err,
            Error::FailToParseValue(s, k) if s == "oops" && k == ValueKind::DateTime
        ));
    }

    #[test]
    fn with_str_list_rejects_is() {
        let err = DateTimePredicate::with_str_list(Operation::Is, &["2020-01-01"]).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::Is, ValueKind::DateTime)
        ));
    }

    #[test]
    fn with_str_list_rejects_is_one_of() {
        let err = DateTimePredicate::with_str_list(Operation::IsOneOf, &["2020-01-01", "2020-01-02"]).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::IsOneOf, ValueKind::DateTime)
        ));
    }

    #[test]
    fn with_value_list_in_range() {
        let a = ts("2020-04-01");
        let b = ts("2020-04-30");
        let p = DateTimePredicate::with_value_list(Operation::InRange, &[a, b]).unwrap();
        assert!(p.evaluate(ts("2020-04-15")));
        assert!(!p.evaluate(ts("2020-03-31")));
    }

    #[test]
    fn with_value_list_in_range_wrong_len() {
        let err = DateTimePredicate::with_value_list(Operation::InRange, &[ts("2020-01-01")]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingTwoValuesForRange(ValueKind::DateTime)
        ));
    }

    #[test]
    fn with_value_list_in_range_min_greater_than_max() {
        let err = DateTimePredicate::with_value_list(
            Operation::InRange,
            &[ts("2020-06-20"), ts("2020-06-10")],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingMinToBeLessThanMax(ValueKind::DateTime)
        ));
    }

    #[test]
    fn with_value_list_in_range_wrong_value_kind() {
        // Range bounds are parsed via `u64::try_from(Value)` (see `DateTimeInsideRange::with_value_list`).
        let err = DateTimePredicate::with_value_list(Operation::InRange, &[1_i32, 2_i32]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingADifferentValueKind(got, expected)
                if got == ValueKind::I32 && expected == ValueKind::U64
        ));
    }

    #[test]
    fn with_value_list_rejects_greater_than() {
        let err = DateTimePredicate::with_value_list(
            Operation::GreaterThan,
            &[ts("2020-01-01"), ts("2020-01-02")],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::GreaterThan, ValueKind::DateTime)
        ));
    }

    #[test]
    fn with_value_list_rejects_is_one_of() {
        let err = DateTimePredicate::with_value_list(
            Operation::IsOneOf,
            &[ts("2020-01-01"), ts("2020-01-02")],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::IsOneOf, ValueKind::DateTime)
        ));
    }
}

mod string_predicate_tests {
    use super::*;

    #[test]
    fn starts_with_case_sensitive() {
        let p = StringPredicate::with_value(Operation::StartsWith, "foo", false).unwrap();
        assert!(p.evaluate("foobar"));
        assert!(!p.evaluate("barfoo"));
        assert!(!p.evaluate("FooBar"));
    }

    #[test]
    fn starts_with_ignore_case_ascii() {
        let p = StringPredicate::with_value(Operation::StartsWith, "Foo", true).unwrap();
        assert!(p.evaluate("foobar"));
        assert!(!p.evaluate("barfoo"));
    }

    #[test]
    fn ends_with_case_sensitive() {
        let p = StringPredicate::with_value(Operation::EndsWith, "bar", false).unwrap();
        assert!(p.evaluate("foobar"));
        assert!(!p.evaluate("barfoo"));
    }

    #[test]
    fn ends_with_ignore_case_ascii() {
        let p = StringPredicate::with_value(Operation::EndsWith, "BAR", true).unwrap();
        assert!(p.evaluate("FooBar"));
    }

    #[test]
    fn contains_case_sensitive() {
        let p = StringPredicate::with_value(Operation::Contains, "oba", false).unwrap();
        assert!(p.evaluate("foobar"));
        assert!(!p.evaluate("foobor"));
    }

    #[test]
    fn contains_ignore_case_ascii() {
        let p = StringPredicate::with_value(Operation::Contains, "OBA", true).unwrap();
        assert!(p.evaluate("foobar"));
    }

    #[test]
    fn equals_case_sensitive() {
        let p = StringPredicate::with_value(Operation::Is, "hello", false).unwrap();
        assert!(p.evaluate("hello"));
        assert!(!p.evaluate("Hello"));
        assert!(!p.evaluate("hello!"));
    }

    #[test]
    fn equals_ignore_case_unicode() {
        let p = StringPredicate::with_value(Operation::Is, "café", true).unwrap();
        assert!(p.evaluate("CAFÉ"));
        assert!(p.evaluate("café"));
    }

    #[test]
    fn with_value_rejects_is_not() {
        let err = StringPredicate::with_value(Operation::IsNot, "x", false).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::IsNot, ValueKind::String)
        ));
    }

    #[test]
    fn with_value_rejects_greater_than() {
        let err = StringPredicate::with_value(Operation::GreaterThan, "x", false).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::GreaterThan, ValueKind::String)
        ));
    }

    #[test]
    fn with_str_list_contains_one_of() {
        let p = StringPredicate::with_str_list(Operation::ContainsOneOf, &["oo", "ar"], false).unwrap();
        assert!(p.evaluate("foobar"));
        assert!(p.evaluate("bart"));
        assert!(!p.evaluate("hello"));
    }

    #[test]
    fn with_str_list_contains_one_of_ignore_case() {
        let p = StringPredicate::with_str_list(Operation::ContainsOneOf, &["OO", "AR"], true).unwrap();
        assert!(p.evaluate("FooBar"));
    }

    #[test]
    fn with_str_list_starts_with_one_of() {
        let p = StringPredicate::with_str_list(Operation::StartsWithOneOf, &["foo", "bar"], false).unwrap();
        assert!(p.evaluate("food"));
        assert!(p.evaluate("bark"));
        assert!(!p.evaluate("xfoo"));
    }

    #[test]
    fn with_str_list_starts_with_one_of_ignore_case() {
        let p = StringPredicate::with_str_list(Operation::StartsWithOneOf, &["FOO", "BAR"], true).unwrap();
        assert!(p.evaluate("food"));
    }

    #[test]
    fn with_str_list_ends_with_one_of() {
        let p = StringPredicate::with_str_list(Operation::EndsWithOneOf, &["bar", "baz"], false).unwrap();
        assert!(p.evaluate("foobar"));
        assert!(p.evaluate("quxbaz"));
        assert!(!p.evaluate("barfoo"));
    }

    #[test]
    fn with_str_list_ends_with_one_of_ignore_case() {
        let p = StringPredicate::with_str_list(Operation::EndsWithOneOf, &["BAR", "BAZ"], true).unwrap();
        assert!(p.evaluate("FooBar"));
    }

    #[test]
    fn with_str_list_is_one_of_small_list_linear_scan() {
        let p = StringPredicate::with_str_list(Operation::IsOneOf, &["a", "b", "c"], false).unwrap();
        assert!(p.evaluate("b"));
        assert!(!p.evaluate("d"));
    }

    #[test]
    fn with_str_list_is_one_of_dedupes() {
        let p = StringPredicate::with_str_list(Operation::IsOneOf, &["z", "a", "z", "m"], false).unwrap();
        assert!(p.evaluate("m"));
        assert!(p.evaluate("z"));
    }

    #[test]
    fn with_str_list_is_one_of_ignore_case() {
        let p = StringPredicate::with_str_list(Operation::IsOneOf, &["Alpha", "Beta"], true).unwrap();
        assert!(p.evaluate("alpha"));
        assert!(p.evaluate("BETA"));
        assert!(!p.evaluate("gamma"));
    }

    #[test]
    fn with_str_list_is_one_of_binary_search_path() {
        let parts: Vec<String> = (0..16).map(|i| format!("item{i:02}")).collect();
        let refs: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
        let p = StringPredicate::with_str_list(Operation::IsOneOf, &refs, false).unwrap();
        assert!(p.evaluate("item07"));
        assert!(!p.evaluate("item99"));
    }

    #[test]
    fn with_str_list_rejects_in_range() {
        let err = StringPredicate::with_str_list(Operation::InRange, &["a", "b"], false).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::InRange, ValueKind::String)
        ));
    }

    #[test]
    fn with_str_list_rejects_is() {
        let err = StringPredicate::with_str_list(Operation::Is, &["only"], false).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::Is, ValueKind::String)
        ));
    }

    #[test]
    fn with_value_list_contains_one_of() {
        let p = StringPredicate::with_value_list(Operation::ContainsOneOf, &["x", "yz"]).unwrap();
        assert!(p.evaluate("ayz"));
        assert!(!p.evaluate("ab"));
    }

    #[test]
    fn with_value_list_starts_with_one_of() {
        let p = StringPredicate::with_value_list(Operation::StartsWithOneOf, &["pre", "post"]).unwrap();
        assert!(p.evaluate("prefix"));
        assert!(!p.evaluate("xpre"));
    }

    #[test]
    fn with_value_list_ends_with_one_of() {
        let p = StringPredicate::with_value_list(Operation::EndsWithOneOf, &["ing", "ed"]).unwrap();
        assert!(p.evaluate("running"));
        assert!(p.evaluate("walked"));
        assert!(!p.evaluate("ingx"));
    }

    #[test]
    fn with_value_list_is_one_of() {
        let p = StringPredicate::with_value_list(Operation::IsOneOf, &["one", "two"]).unwrap();
        assert!(p.evaluate("two"));
        assert!(!p.evaluate("three"));
    }

    #[test]
    fn with_value_list_wrong_value_kind() {
        let err = StringPredicate::with_value_list(Operation::ContainsOneOf, &[1_i32, 2_i32]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingADifferentValueKind(got, expected)
                if got == ValueKind::I32 && expected == ValueKind::String
        ));
    }

    #[test]
    fn with_value_list_rejects_starts_with() {
        let err = StringPredicate::with_value_list(Operation::StartsWith, &["a", "b"]).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::StartsWith, ValueKind::String)
        ));
    }
}

mod path_predicate_tests {
    use crate::Value;

    use super::*;

    #[test]
    fn starts_with_on_bytes() {
        let p = PathPredicate::with_str(Operation::StartsWith, "foo/", false).unwrap();
        assert!(p.evaluate(b"foo/bar"));
        assert!(!p.evaluate(b"bar/foo"));
    }

    #[test]
    fn starts_with_ignore_case() {
        let p = PathPredicate::with_str(Operation::StartsWith, "Foo/", true).unwrap();
        assert!(p.evaluate(b"foo/bar"));
    }

    #[test]
    fn ends_with_on_bytes() {
        let p = PathPredicate::with_str(Operation::EndsWith, ".rs", false).unwrap();
        assert!(p.evaluate(b"src/main.rs"));
        assert!(!p.evaluate(b"src/main.txt"));
    }

    #[test]
    fn contains_on_bytes() {
        let p = PathPredicate::with_str(Operation::Contains, "/src/", false).unwrap();
        assert!(p.evaluate(b"proj/src/lib.rs"));
        assert!(!p.evaluate(b"proj/lib.rs"));
    }

    #[test]
    fn equals_on_bytes() {
        let p = PathPredicate::with_str(Operation::Is, "exact/path", false).unwrap();
        assert!(p.evaluate(b"exact/path"));
        assert!(!p.evaluate(b"exact/path/more"));
    }

    #[test]
    fn with_str_rejects_is_not() {
        let err = PathPredicate::with_str(Operation::IsNot, "x", false).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::IsNot, ValueKind::Path)
        ));
    }

    #[test]
    fn with_str_rejects_in_range() {
        let err = PathPredicate::with_str(Operation::InRange, "a", false).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::InRange, ValueKind::Path)
        ));
    }

    #[test]
    fn with_value_valid_utf8_delegates_to_with_str() {
        let p = PathPredicate::with_value(Operation::Is, b"same").unwrap();
        assert!(p.evaluate(b"same"));
    }

    #[test]
    fn with_value_invalid_utf8() {
        let err = PathPredicate::with_value(Operation::Is, &[0xff, 0xfe, 0xff]).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidUTF8Value(bytes, ValueKind::Path) if bytes == [0xff, 0xfe, 0xff]
        ));
    }

    #[test]
    fn glob_re_match_single_pattern() {
        let p = PathPredicate::with_str(Operation::GlobREMatch, "*.txt", false).unwrap();
        assert!(p.evaluate(b"notes.txt"));
        assert!(!p.evaluate(b"notes.md"));
    }

    #[test]
    fn glob_re_match_unbuildable_pattern() {
        // Unclosed alternation — `Glob::new` returns `None`, so the matcher cannot be built.
        let err = PathPredicate::with_str(Operation::GlobREMatch, "{a,b", false).unwrap_err();
        assert!(matches!(
            err,
            Error::FailToBuildInternalDataStructure(Operation::GlobREMatch, ValueKind::Path)
        ));
    }

    #[test]
    fn with_str_list_contains_one_of() {
        let p = PathPredicate::with_str_list(Operation::ContainsOneOf, &["oo", "ar"], false).unwrap();
        assert!(p.evaluate(b"foobar"));
        assert!(!p.evaluate(b"hello"));
    }

    #[test]
    fn with_str_list_starts_with_one_of() {
        let p = PathPredicate::with_str_list(Operation::StartsWithOneOf, &["foo", "bar"], false).unwrap();
        assert!(p.evaluate(b"food"));
        assert!(!p.evaluate(b"xfoo"));
    }

    #[test]
    fn with_str_list_ends_with_one_of() {
        let p = PathPredicate::with_str_list(Operation::EndsWithOneOf, &[".rs", ".toml"], false).unwrap();
        assert!(p.evaluate(b"Cargo.toml"));
        assert!(!p.evaluate(b"README"));
    }

    #[test]
    fn with_str_list_is_one_of() {
        let p = PathPredicate::with_str_list(Operation::IsOneOf, &["a", "b", "c"], false).unwrap();
        assert!(p.evaluate(b"b"));
        assert!(!p.evaluate(b"z"));
    }

    #[test]
    fn with_str_list_glob_re_match_multiple() {
        let p = PathPredicate::with_str_list(Operation::GlobREMatch, &["*.txt", "*.md"], false).unwrap();
        assert!(p.evaluate(b"readme.md"));
        assert!(p.evaluate(b"log.txt"));
        assert!(!p.evaluate(b"image.png"));
    }

    #[test]
    fn with_str_list_glob_re_match_empty_list() {
        let err = PathPredicate::with_str_list(Operation::GlobREMatch, &[], false).unwrap_err();
        assert!(matches!(
            err,
            Error::EmptyListForGlobREMatch(ValueKind::Path)
        ));
    }

    #[test]
    fn with_str_list_glob_re_match_all_patterns_invalid_yields_empty() {
        let err = PathPredicate::with_str_list(Operation::GlobREMatch, &["{x,y", "[z"], false).unwrap_err();
        assert!(matches!(
            err,
            Error::EmptyListForGlobREMatch(ValueKind::Path)
        ));
    }

    #[test]
    fn with_str_list_rejects_is() {
        let err = PathPredicate::with_str_list(Operation::Is, &["x"], false).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::Is, ValueKind::Path)
        ));
    }

    #[test]
    fn with_value_list_contains_one_of() {
        let p = PathPredicate::with_value_list(Operation::ContainsOneOf, &["x", "yz"]).unwrap();
        assert!(p.evaluate(b"ayz"));
    }

    #[test]
    fn with_value_list_glob_re_match_from_strs() {
        let p = PathPredicate::with_value_list(Operation::GlobREMatch, &["*.log", "*.cfg"]).unwrap();
        assert!(p.evaluate(b"app.log"));
        assert!(p.evaluate(b"defaults.cfg"));
    }

    #[test]
    fn with_value_list_glob_re_match_from_path_bytes() {
        let a: &[u8] = b"*.dat";
        let b: &[u8] = b"*.bin";
        let p = PathPredicate::with_value_list(Operation::GlobREMatch, &[Value::Path(a), Value::Path(b)]).unwrap();
        assert!(p.evaluate(b"data.dat"));
        assert!(!p.evaluate(b"x.txt"));
    }

    #[test]
    fn with_value_list_glob_re_match_invalid_utf8_in_path_value() {
        let bad: &[u8] = &[0xff, 0xfe];
        let good: &[u8] = b"*.txt";
        let err = PathPredicate::with_value_list(Operation::GlobREMatch, &[Value::Path(good), Value::Path(bad)]).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidUTF8Value(bytes, ValueKind::Path) if bytes == [0xff, 0xfe]
        ));
    }

    #[test]
    fn with_value_list_glob_re_match_wrong_value_kind() {
        let err = PathPredicate::with_value_list(Operation::GlobREMatch, &[1_i32]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingADifferentValueKind(got, expected)
                if got == ValueKind::I32 && expected == ValueKind::String
        ));
    }

    #[test]
    fn with_value_list_contains_one_of_wrong_value_kind() {
        let err = PathPredicate::with_value_list(Operation::ContainsOneOf, &[1_i32]).unwrap_err();
        assert!(matches!(
            err,
            Error::ExpectingADifferentValueKind(got, expected)
                if got == ValueKind::I32 && expected == ValueKind::String
        ));
    }

    #[test]
    fn with_value_list_rejects_starts_with() {
        let err = PathPredicate::with_value_list(Operation::StartsWith, &["a", "b"]).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidOperationForValue(Operation::StartsWith, ValueKind::String)
        ));
    }
}

#[test]
fn i8_predicate_type_extremes() {
    let p = I8Predicate::with_value(Operation::Is, i8::MIN).unwrap();
    assert!(p.evaluate(i8::MIN));
    assert!(!p.evaluate(i8::MAX));

    let p = I8Predicate::with_str_list(Operation::InRange, &["-128", "127"]).unwrap();
    assert!(p.evaluate(-1));
    assert!(p.evaluate(127));
}

#[test]
fn i16_predicate_type_extremes() {
    let p = I16Predicate::with_value(Operation::Is, i16::MIN).unwrap();
    assert!(p.evaluate(i16::MIN));

    let p = I16Predicate::with_str_list(Operation::InRange, &["-1000", "1000"]).unwrap();
    assert!(p.evaluate(0));
}

#[test]
fn i32_predicate_type_extremes() {
    let p = I32Predicate::with_value(Operation::GreaterThan, i32::MAX - 1).unwrap();
    assert!(p.evaluate(i32::MAX));
    assert!(!p.evaluate(i32::MAX - 1));
}

#[test]
fn i64_predicate_type_extremes() {
    let p = I64Predicate::with_value(Operation::LessThan, i64::MIN + 1).unwrap();
    assert!(p.evaluate(i64::MIN));
    assert!(!p.evaluate(i64::MIN + 1));
}

#[test]
fn u8_predicate_type_extremes() {
    let p = U8Predicate::with_value(Operation::Is, u8::MAX).unwrap();
    assert!(p.evaluate(u8::MAX));
    assert!(!p.evaluate(0));

    let p = U8Predicate::with_str_list(Operation::InRange, &["0", "255"]).unwrap();
    assert!(p.evaluate(0));
    assert!(p.evaluate(255));
}

#[test]
fn u16_predicate_type_extremes() {
    let p = U16Predicate::with_value(Operation::Is, u16::MAX).unwrap();
    assert!(p.evaluate(u16::MAX));

    let p = U16Predicate::with_str_list(Operation::InRange, &["0", "1000"]).unwrap();
    assert!(p.evaluate(500));
}

#[test]
fn u32_predicate_type_extremes() {
    let p = U32Predicate::with_value(Operation::GreaterThan, u32::MAX - 1).unwrap();
    assert!(p.evaluate(u32::MAX));
    assert!(!p.evaluate(u32::MAX - 1));
}

#[test]
fn u64_predicate_type_extremes() {
    let p = U64Predicate::with_value(Operation::GreaterThan, u64::MAX - 1).unwrap();
    assert!(p.evaluate(u64::MAX));
    assert!(!p.evaluate(u64::MAX - 1));
}

#[test]
fn f32_predicate_equal_to_nan_never_matches() {
    let nan = f32::NAN;
    let p = F32Predicate::with_value(Operation::Is, nan).unwrap();
    assert!(!p.evaluate(nan));
}

#[test]
fn f64_predicate_equal_to_nan_never_matches() {
    let nan = f64::NAN;
    let p = F64Predicate::with_value(Operation::Is, nan).unwrap();
    assert!(!p.evaluate(nan));
}

#[test]
fn f32_predicate_negative_range() {
    let p = F32Predicate::with_str_list(Operation::InRange, &["-3.5", "-0.5"]).unwrap();
    assert!(p.evaluate(-1.0));
    assert!(!p.evaluate(0.0));
}

#[test]
fn f64_predicate_scientific_notation_str() {
    let p = F64Predicate::with_str(Operation::Is, "1e2").unwrap();
    assert!(p.evaluate(100.0));
}
