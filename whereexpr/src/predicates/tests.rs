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
            fn different_than_evaluates() {
                let p = $pred::with_value(Operation::IsNot, 0 as $ty).unwrap();
                assert!(!p.evaluate(0 as $ty));
                assert!(p.evaluate(1 as $ty));
                assert!(p.evaluate($alt));
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

integer_predicate_tests!(i8_predicate, i8, I8Predicate, ValueKind::I8, i32, ValueKind::I32, (-1i8));
integer_predicate_tests!(i16_predicate, i16, I16Predicate, ValueKind::I16, i32, ValueKind::I32, (-1i16));
integer_predicate_tests!(i32_predicate, i32, I32Predicate, ValueKind::I32, i8, ValueKind::I8, (-1i32));
integer_predicate_tests!(i64_predicate, i64, I64Predicate, ValueKind::I64, i32, ValueKind::I32, (-1i64));

integer_predicate_tests!(u8_predicate, u8, U8Predicate, ValueKind::U8, i32, ValueKind::I32, (2u8));
integer_predicate_tests!(u16_predicate, u16, U16Predicate, ValueKind::U16, i32, ValueKind::I32, (2u16));
integer_predicate_tests!(u32_predicate, u32, U32Predicate, ValueKind::U32, i8, ValueKind::I8, (2u32));
integer_predicate_tests!(u64_predicate, u64, U64Predicate, ValueKind::U64, i32, ValueKind::I32, (2u64));

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
