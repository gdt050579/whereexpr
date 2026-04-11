use super::numeric::*;
use super::single_string::*;
use super::glob_re_match::GlobREMatch;
use super::string_contains_one_of::ContainsOneOf;
use super::string_is_one_of::IsOneOf;
use super::string_ends_with_one_of::EndsWithOneOf;
use super::string_starts_with_one_of::StartsWithOneOf;
use crate::Operation;
use super::{
    BoolPredicate, DateTimeInsideRange, DateTimePredicate, FloatNumberPredicate, Hash128Predicate,
    Hash160Predicate, IpAddrPredicate, PathPredicate, SignedNumberPredicate, StringPredicate,
    UnsignedNumberPredicate,
};
use crate::{DateTime, Hash128, Hash160};

#[test]
fn signed_smaller_than_or_equal() {
    let p = SignedSmallerThenOrEqualTo::new(5);
    assert!(p.evaluate(4));
    assert!(p.evaluate(5));
    assert!(!p.evaluate(6));
    let neg = SignedSmallerThenOrEqualTo::new(-2);
    assert!(neg.evaluate(-3));
    assert!(neg.evaluate(-2));
    assert!(!neg.evaluate(-1));
}

#[test]
fn signed_smaller_than() {
    let p = SignedSmallerThen::new(5);
    assert!(p.evaluate(4));
    assert!(!p.evaluate(5));
    assert!(!p.evaluate(6));
}

#[test]
fn signed_greater_than_or_equal() {
    let p = SignedGreaterThenOrEqualTo::new(5);
    assert!(!p.evaluate(4));
    assert!(p.evaluate(5));
    assert!(p.evaluate(6));
}

#[test]
fn signed_greater_than() {
    let p = SignedGreaterThen::new(5);
    assert!(!p.evaluate(4));
    assert!(!p.evaluate(5));
    assert!(p.evaluate(6));
}

#[test]
fn signed_equal_to() {
    let p = SignedEqualTo::new(7);
    assert!(!p.evaluate(6));
    assert!(p.evaluate(7));
    assert!(!p.evaluate(8));
    assert!(!p.evaluate(0));
}

#[test]
fn signed_different_than() {
    let p = SignedDifferentThen::new(7);
    assert!(p.evaluate(6));
    assert!(!p.evaluate(7));
    assert!(p.evaluate(8));
}

#[test]
fn unsigned_smaller_than_or_equal() {
    let p = UnsignedSmallerThenOrEqualTo::new(10);
    assert!(p.evaluate(9));
    assert!(p.evaluate(10));
    assert!(!p.evaluate(11));
    assert!(p.evaluate(0));
}

#[test]
fn unsigned_smaller_than() {
    let p = UnsignedSmallerThen::new(10);
    assert!(p.evaluate(9));
    assert!(!p.evaluate(10));
    assert!(!p.evaluate(11));
}

#[test]
fn unsigned_greater_than_or_equal() {
    let p = UnsignedGreaterThenOrEqualTo::new(10);
    assert!(!p.evaluate(9));
    assert!(p.evaluate(10));
    assert!(p.evaluate(11));
}

#[test]
fn unsigned_greater_than() {
    let p = UnsignedGreaterThen::new(10);
    assert!(!p.evaluate(9));
    assert!(!p.evaluate(10));
    assert!(p.evaluate(11));
}

#[test]
fn unsigned_equal_to() {
    let p = UnsignedEqualTo::new(42);
    assert!(!p.evaluate(41));
    assert!(p.evaluate(42));
    assert!(!p.evaluate(43));
}

#[test]
fn unsigned_different_than() {
    let p = UnsignedDifferentThen::new(42);
    assert!(p.evaluate(41));
    assert!(!p.evaluate(42));
    assert!(p.evaluate(43));
}

#[test]
fn signed_inside_range_inclusive() {
    let p = SignedInsideRange::new(&["2".to_string(), "8".to_string()]).unwrap();
    assert!(!p.evaluate(1));
    assert!(p.evaluate(2));
    assert!(p.evaluate(5));
    assert!(p.evaluate(8));
    assert!(!p.evaluate(9));
}

#[test]
fn signed_inside_range_single_point() {
    let p = SignedInsideRange::new(&["5".to_string(), "5".to_string()]).unwrap();
    assert!(!p.evaluate(4));
    assert!(p.evaluate(5));
    assert!(!p.evaluate(6));
}

#[test]
fn signed_inside_range_negative_bounds() {
    let p = SignedInsideRange::new(&["-10".to_string(), "-3".to_string()]).unwrap();
    assert!(!p.evaluate(-11));
    assert!(p.evaluate(-10));
    assert!(p.evaluate(-5));
    assert!(p.evaluate(-3));
    assert!(!p.evaluate(-2));
}

#[test]
fn unsigned_inside_range_inclusive() {
    let p = UnsignedInsideRange::new(&["100".to_string(), "200".to_string()]).unwrap();
    assert!(!p.evaluate(99));
    assert!(p.evaluate(100));
    assert!(p.evaluate(150));
    assert!(p.evaluate(200));
    assert!(!p.evaluate(201));
}

#[test]
fn unsigned_inside_range_single_point() {
    let p = UnsignedInsideRange::new(&["0".to_string(), "0".to_string()]).unwrap();
    assert!(p.evaluate(0));
    assert!(!p.evaluate(1));
}

#[test]
fn unsigned_inside_range_max_values() {
    let p = UnsignedInsideRange::new(&[format!("{}", u64::MAX - 1), format!("{}", u64::MAX)]).unwrap();
    assert!(!p.evaluate(u64::MAX - 2));
    assert!(p.evaluate(u64::MAX - 1));
    assert!(p.evaluate(u64::MAX));
}

#[test]
fn float_smaller_than_or_equal() {
    let p = FloatSmallerThenOrEqualTo::new(2.5);
    assert!(p.evaluate(1.0));
    assert!(p.evaluate(2.5));
    assert!(!p.evaluate(2.51));
    let neg = FloatSmallerThenOrEqualTo::new(-1.0);
    assert!(neg.evaluate(-2.0));
    assert!(neg.evaluate(-1.0));
    assert!(!neg.evaluate(-0.5));
}

#[test]
fn float_smaller_than() {
    let p = FloatSmallerThen::new(10.0);
    assert!(p.evaluate(9.999));
    assert!(!p.evaluate(10.0));
    assert!(!p.evaluate(10.001));
}

#[test]
fn float_greater_than_or_equal() {
    let p = FloatGreaterThenOrEqualTo::new(0.5);
    assert!(!p.evaluate(0.499));
    assert!(p.evaluate(0.5));
    assert!(p.evaluate(1.0));
}

#[test]
fn float_greater_than() {
    let p = FloatGreaterThen::new(0.5);
    assert!(!p.evaluate(0.5));
    assert!(p.evaluate(0.51));
}

#[test]
fn float_equal_to() {
    let p = FloatEqualTo::new(0.25);
    assert!(p.evaluate(0.25));
    assert!(!p.evaluate(0.5));
}

#[test]
fn float_different_than() {
    let p = FloatDifferentThen::new(1.0);
    assert!(p.evaluate(2.0));
    assert!(!p.evaluate(1.0));
}

#[test]
fn float_inside_range_inclusive() {
    let p = FloatInsideRange::new(&["-1.5".to_string(), "3.5".to_string()]).unwrap();
    assert!(!p.evaluate(-1.51));
    assert!(p.evaluate(-1.5));
    assert!(p.evaluate(0.0));
    assert!(p.evaluate(3.5));
    assert!(!p.evaluate(3.51));
}

#[test]
fn float_inside_range_single_point() {
    let pi = std::f64::consts::PI;
    let p = FloatInsideRange::new(&[format!("{}", pi), format!("{}", pi)]).unwrap();
    assert!(p.evaluate(pi));
    assert!(!p.evaluate(3.14));
}

// --- SignedNumberPredicate / UnsignedNumberPredicate (enum + new + evaluate) ---

#[test]
fn signed_number_predicate_new_evaluate_matches_numeric_predicates() {
    let t: i64 = 5;
    assert!(SignedNumberPredicate::new(Operation::LessThan, t)
        .unwrap()
        .evaluate(4));
    assert!(!SignedNumberPredicate::new(Operation::LessThan, t)
        .unwrap()
        .evaluate(5));
    assert!(SignedNumberPredicate::new(Operation::LessThanOrEqual, t)
        .unwrap()
        .evaluate(5));
    assert!(!SignedNumberPredicate::new(Operation::LessThanOrEqual, t)
        .unwrap()
        .evaluate(6));
    assert!(SignedNumberPredicate::new(Operation::GreaterThan, t)
        .unwrap()
        .evaluate(6));
    assert!(!SignedNumberPredicate::new(Operation::GreaterThan, t)
        .unwrap()
        .evaluate(5));
    assert!(SignedNumberPredicate::new(Operation::GreaterThanOrEqual, t)
        .unwrap()
        .evaluate(5));
    assert!(SignedNumberPredicate::new(Operation::Is, t).unwrap().evaluate(5));
    assert!(!SignedNumberPredicate::new(Operation::Is, t).unwrap().evaluate(4));
    assert!(SignedNumberPredicate::new(Operation::IsNot, t)
        .unwrap()
        .evaluate(4));
    assert!(!SignedNumberPredicate::new(Operation::IsNot, t)
        .unwrap()
        .evaluate(5));
    let neg: i64 = -3;
    assert!(SignedNumberPredicate::new(Operation::GreaterThanOrEqual, neg)
        .unwrap()
        .evaluate(-3));
    assert!(!SignedNumberPredicate::new(Operation::GreaterThan, neg)
        .unwrap()
        .evaluate(-3));
}

#[test]
fn signed_number_predicate_new_returns_none_for_non_scalar_numeric_ops() {
    for op in [
        Operation::InRange,
        Operation::IsOneOf,
        Operation::StartsWith,
        Operation::StartsWithOneOf,
        Operation::EndsWith,
        Operation::EndsWithOneOf,
        Operation::Contains,
        Operation::ContainsOneOf,
        Operation::GlobREMatch,
    ] {
        assert!(
            SignedNumberPredicate::new(op, 0).is_none(),
            "expected None for {op:?}"
        );
    }
}

#[test]
fn unsigned_number_predicate_new_evaluate_matches_numeric_predicates() {
    let t: u64 = 10;
    assert!(UnsignedNumberPredicate::new(Operation::LessThan, t)
        .unwrap()
        .evaluate(9));
    assert!(!UnsignedNumberPredicate::new(Operation::LessThan, t)
        .unwrap()
        .evaluate(10));
    assert!(UnsignedNumberPredicate::new(Operation::LessThanOrEqual, t)
        .unwrap()
        .evaluate(10));
    assert!(!UnsignedNumberPredicate::new(Operation::LessThanOrEqual, t)
        .unwrap()
        .evaluate(11));
    assert!(UnsignedNumberPredicate::new(Operation::GreaterThan, t)
        .unwrap()
        .evaluate(11));
    assert!(!UnsignedNumberPredicate::new(Operation::GreaterThan, t)
        .unwrap()
        .evaluate(10));
    assert!(UnsignedNumberPredicate::new(Operation::GreaterThanOrEqual, t)
        .unwrap()
        .evaluate(10));
    assert!(UnsignedNumberPredicate::new(Operation::Is, t)
        .unwrap()
        .evaluate(10));
    assert!(!UnsignedNumberPredicate::new(Operation::Is, t)
        .unwrap()
        .evaluate(9));
    assert!(UnsignedNumberPredicate::new(Operation::IsNot, t)
        .unwrap()
        .evaluate(9));
    assert!(!UnsignedNumberPredicate::new(Operation::IsNot, t)
        .unwrap()
        .evaluate(10));
    assert!(UnsignedNumberPredicate::new(Operation::LessThanOrEqual, 0)
        .unwrap()
        .evaluate(0));
}

#[test]
fn unsigned_number_predicate_new_returns_none_for_non_scalar_numeric_ops() {
    for op in [
        Operation::InRange,
        Operation::IsOneOf,
        Operation::StartsWith,
        Operation::StartsWithOneOf,
        Operation::EndsWith,
        Operation::EndsWithOneOf,
        Operation::Contains,
        Operation::ContainsOneOf,
        Operation::GlobREMatch,
    ] {
        assert!(
            UnsignedNumberPredicate::new(op, 0).is_none(),
            "expected None for {op:?}"
        );
    }
}

// --- FloatNumberPredicate (enum + new + evaluate) ---

#[test]
fn float_number_predicate_new_evaluate_matches_numeric_predicates() {
    let t: f64 = 1.25;
    assert!(FloatNumberPredicate::new(Operation::LessThan, t)
        .unwrap()
        .evaluate(1.24));
    assert!(!FloatNumberPredicate::new(Operation::LessThan, t)
        .unwrap()
        .evaluate(1.25));
    assert!(FloatNumberPredicate::new(Operation::LessThanOrEqual, t)
        .unwrap()
        .evaluate(1.25));
    assert!(!FloatNumberPredicate::new(Operation::LessThanOrEqual, t)
        .unwrap()
        .evaluate(1.26));
    assert!(FloatNumberPredicate::new(Operation::GreaterThan, t)
        .unwrap()
        .evaluate(1.26));
    assert!(!FloatNumberPredicate::new(Operation::GreaterThan, t)
        .unwrap()
        .evaluate(1.25));
    assert!(FloatNumberPredicate::new(Operation::GreaterThanOrEqual, t)
        .unwrap()
        .evaluate(1.25));
    assert!(FloatNumberPredicate::new(Operation::Is, t).unwrap().evaluate(1.25));
    assert!(!FloatNumberPredicate::new(Operation::Is, t).unwrap().evaluate(1.24));
    assert!(FloatNumberPredicate::new(Operation::IsNot, t)
        .unwrap()
        .evaluate(1.24));
    assert!(!FloatNumberPredicate::new(Operation::IsNot, t)
        .unwrap()
        .evaluate(1.25));
    let neg: f64 = -0.5;
    assert!(FloatNumberPredicate::new(Operation::GreaterThanOrEqual, neg)
        .unwrap()
        .evaluate(-0.5));
    assert!(!FloatNumberPredicate::new(Operation::GreaterThan, neg)
        .unwrap()
        .evaluate(-0.5));
}

#[test]
fn float_number_predicate_new_returns_none_for_non_scalar_numeric_ops() {
    for op in [
        Operation::InRange,
        Operation::IsOneOf,
        Operation::StartsWith,
        Operation::StartsWithOneOf,
        Operation::EndsWith,
        Operation::EndsWithOneOf,
        Operation::Contains,
        Operation::ContainsOneOf,
        Operation::GlobREMatch,
    ] {
        assert!(
            FloatNumberPredicate::new(op, 0.0).is_none(),
            "expected None for {op:?}"
        );
    }
}

// --- Operation::InRange via new_with_values (Signed / Unsigned / Float) ---

fn in_range_pair(a: &str, b: &str) -> Vec<String> {
    vec![a.to_string(), b.to_string()]
}

#[test]
fn signed_number_predicate_in_range_via_new_with_values() {
    let p = SignedNumberPredicate::new_with_values(Operation::InRange, &in_range_pair("2", "8"))
        .expect("predicate");
    assert!(!p.evaluate(1));
    assert!(p.evaluate(2));
    assert!(p.evaluate(5));
    assert!(p.evaluate(8));
    assert!(!p.evaluate(9));
}

#[test]
fn signed_number_predicate_new_with_values_in_range_negative_bounds() {
    let p = SignedNumberPredicate::new_with_values(Operation::InRange, &in_range_pair("-10", "-3"))
        .expect("predicate");
    assert!(!p.evaluate(-11));
    assert!(p.evaluate(-10));
    assert!(p.evaluate(-5));
    assert!(p.evaluate(-3));
    assert!(!p.evaluate(-2));
}

#[test]
fn signed_number_predicate_new_with_values_in_range_rejects_bad_values_slice() {
    assert!(SignedNumberPredicate::new_with_values(Operation::InRange, &[]).is_none());
    assert!(
        SignedNumberPredicate::new_with_values(Operation::InRange, &["1".to_string()]).is_none()
    );
    assert!(SignedNumberPredicate::new_with_values(
        Operation::InRange,
        &["1".to_string(), "2".to_string(), "3".to_string()],
    )
    .is_none());
    assert!(SignedNumberPredicate::new_with_values(
        Operation::InRange,
        &["nope".to_string(), "5".to_string()],
    )
    .is_none());
}

#[test]
fn signed_number_predicate_new_with_values_non_in_range_returns_none() {
    assert!(SignedNumberPredicate::new_with_values(
        Operation::GreaterThan,
        &in_range_pair("1", "2"),
    )
    .is_none());
}

#[test]
fn unsigned_number_predicate_in_range_via_new_with_values() {
    let p = UnsignedNumberPredicate::new_with_values(Operation::InRange, &in_range_pair("100", "200"))
        .expect("predicate");
    assert!(!p.evaluate(99));
    assert!(p.evaluate(100));
    assert!(p.evaluate(150));
    assert!(p.evaluate(200));
    assert!(!p.evaluate(201));
}

#[test]
fn unsigned_number_predicate_new_with_values_in_range_rejects_bad_values_slice() {
    assert!(UnsignedNumberPredicate::new_with_values(Operation::InRange, &[]).is_none());
    assert!(UnsignedNumberPredicate::new_with_values(
        Operation::InRange,
        &["-1".to_string(), "5".to_string()],
    )
    .is_none());
}

#[test]
fn unsigned_number_predicate_new_with_values_non_in_range_returns_none() {
    assert!(UnsignedNumberPredicate::new_with_values(
        Operation::Is,
        &in_range_pair("1", "1"),
    )
    .is_none());
}

#[test]
fn float_number_predicate_in_range_via_new_with_values() {
    let p = FloatNumberPredicate::new_with_values(Operation::InRange, &in_range_pair("-1.5", "3.5"))
        .expect("predicate");
    assert!(!p.evaluate(-1.51));
    assert!(p.evaluate(-1.5));
    assert!(p.evaluate(0.0));
    assert!(p.evaluate(3.5));
    assert!(!p.evaluate(3.51));
}

#[test]
fn float_number_predicate_new_with_values_in_range_rejects_bad_values_slice() {
    assert!(FloatNumberPredicate::new_with_values(Operation::InRange, &[]).is_none());
    assert!(FloatNumberPredicate::new_with_values(
        Operation::InRange,
        &["not-a-float".to_string(), "1".to_string()],
    )
    .is_none());
}

#[test]
fn float_number_predicate_new_with_values_non_in_range_returns_none() {
    assert!(FloatNumberPredicate::new_with_values(
        Operation::LessThan,
        &in_range_pair("0", "1"),
    )
    .is_none());
}

// --- Operation::IsOneOf via new_with_values (Signed / Unsigned) ---

#[test]
fn signed_number_predicate_is_one_of_via_new_with_values() {
    let values = ["-3".to_string(), "0".to_string(), "42".to_string()];
    let p = SignedNumberPredicate::new_with_values(Operation::IsOneOf, &values).expect("predicate");
    assert!(!p.evaluate(-4));
    assert!(p.evaluate(-3));
    assert!(!p.evaluate(-1));
    assert!(p.evaluate(0));
    assert!(!p.evaluate(7));
    assert!(p.evaluate(42));
    assert!(!p.evaluate(43));
}

#[test]
fn signed_number_predicate_is_one_of_dedupes_and_orders() {
    let values = ["10".to_string(), "5".to_string(), "5".to_string(), "10".to_string()];
    let p = SignedNumberPredicate::new_with_values(Operation::IsOneOf, &values).expect("predicate");
    assert!(!p.evaluate(4));
    assert!(p.evaluate(5));
    assert!(!p.evaluate(7));
    assert!(p.evaluate(10));
}

#[test]
fn signed_number_predicate_new_with_values_is_one_of_rejects_bad_values_slice() {
    assert!(SignedNumberPredicate::new_with_values(Operation::IsOneOf, &[]).is_none());
    assert!(SignedNumberPredicate::new_with_values(
        Operation::IsOneOf,
        &["1".to_string(), "not-int".to_string()],
    )
    .is_none());
}

#[test]
fn signed_number_predicate_new_with_values_non_is_one_of_returns_none() {
    assert!(SignedNumberPredicate::new_with_values(
        Operation::GreaterThan,
        &["1".to_string(), "2".to_string()],
    )
    .is_none());
}

#[test]
fn signed_number_predicate_is_one_of_many_values_binary_search_path() {
    let values: Vec<String> = (0i64..9).map(|n| n.to_string()).collect();
    let p = SignedNumberPredicate::new_with_values(Operation::IsOneOf, &values).expect("predicate");
    assert!(p.evaluate(0));
    assert!(p.evaluate(4));
    assert!(p.evaluate(8));
    assert!(!p.evaluate(-1));
    assert!(!p.evaluate(9));
}

#[test]
fn unsigned_number_predicate_is_one_of_via_new_with_values() {
    let values = ["0".to_string(), "100".to_string(), u64::MAX.to_string()];
    let p = UnsignedNumberPredicate::new_with_values(Operation::IsOneOf, &values).expect("predicate");
    assert!(p.evaluate(0));
    assert!(!p.evaluate(1));
    assert!(p.evaluate(100));
    assert!(p.evaluate(u64::MAX));
    assert!(!p.evaluate(u64::MAX - 1));
}

#[test]
fn unsigned_number_predicate_is_one_of_dedupes_and_orders() {
    let values = ["20".to_string(), "10".to_string(), "20".to_string()];
    let p = UnsignedNumberPredicate::new_with_values(Operation::IsOneOf, &values).expect("predicate");
    assert!(p.evaluate(10));
    assert!(p.evaluate(20));
    assert!(!p.evaluate(15));
}

#[test]
fn unsigned_number_predicate_new_with_values_is_one_of_rejects_bad_values_slice() {
    assert!(UnsignedNumberPredicate::new_with_values(Operation::IsOneOf, &[]).is_none());
    assert!(UnsignedNumberPredicate::new_with_values(
        Operation::IsOneOf,
        &["-1".to_string(), "5".to_string()],
    )
    .is_none());
    assert!(UnsignedNumberPredicate::new_with_values(
        Operation::IsOneOf,
        &["1".to_string(), "x".to_string()],
    )
    .is_none());
}

#[test]
fn unsigned_number_predicate_new_with_values_non_is_one_of_returns_none() {
    assert!(UnsignedNumberPredicate::new_with_values(
        Operation::InRange,
        &["1".to_string(), "2".to_string(), "3".to_string()],
    )
    .is_none());
}

#[test]
fn unsigned_number_predicate_is_one_of_many_values_binary_search_path() {
    let values: Vec<String> = (0u64..9).map(|n| n.to_string()).collect();
    let p = UnsignedNumberPredicate::new_with_values(Operation::IsOneOf, &values).expect("predicate");
    assert!(p.evaluate(0));
    assert!(p.evaluate(8));
    assert!(!p.evaluate(9));
}

// --- Hash128Predicate / Hash160Predicate (hash_type_predicates) ---

/// MD5 of the empty string (32 hex digits).
const SAMPLE_MD5_HEX: &str = "d41d8cd98f00b204e9800998ecf8427e";
/// SHA-1 of the empty string (40 hex digits).
const SAMPLE_SHA1_HEX: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";

#[test]
fn md5_predicate_is_evaluates_equal() {
    let p = Hash128Predicate::new(Operation::Is, SAMPLE_MD5_HEX).expect("predicate");
    let matching: Hash128 = SAMPLE_MD5_HEX.parse().expect("hash");
    let other: Hash128 = "00000000000000000000000000000000".parse().expect("hash");
    assert!(p.evaluate(matching));
    assert!(!p.evaluate(other));
}

#[test]
fn md5_predicate_is_not_evaluates_inequal() {
    let p = Hash128Predicate::new(Operation::IsNot, SAMPLE_MD5_HEX).expect("predicate");
    let matching: Hash128 = SAMPLE_MD5_HEX.parse().expect("hash");
    let other: Hash128 = "00000000000000000000000000000000".parse().expect("hash");
    assert!(!p.evaluate(matching));
    assert!(p.evaluate(other));
}

#[test]
fn md5_predicate_is_one_of_via_new_with_values() {
    let a = SAMPLE_MD5_HEX.to_string();
    let b = "00000000000000000000000000000000".to_string();
    let p = Hash128Predicate::new_with_values(Operation::IsOneOf, &[a.clone(), b.clone()]).expect("predicate");
    assert!(p.evaluate(a.parse().expect("hash")));
    assert!(p.evaluate(b.parse().expect("hash")));
    assert!(!p.evaluate("ffffffffffffffffffffffffffffffff".parse().expect("hash")));
}

#[test]
fn md5_predicate_new_rejects_invalid_hex_string() {
    assert!(Hash128Predicate::new(Operation::Is, "tooshort").is_none());
    assert!(Hash128Predicate::new(Operation::Is, "gggggggggggggggggggggggggggggggg").is_none());
}

#[test]
fn md5_predicate_new_returns_none_for_unsupported_ops() {
    for op in [
        Operation::IsOneOf,
        Operation::InRange,
        Operation::GreaterThan,
        Operation::GreaterThanOrEqual,
        Operation::LessThan,
        Operation::LessThanOrEqual,
        Operation::StartsWith,
        Operation::StartsWithOneOf,
        Operation::EndsWith,
        Operation::EndsWithOneOf,
        Operation::Contains,
        Operation::ContainsOneOf,
        Operation::GlobREMatch,
    ] {
        assert!(
            Hash128Predicate::new(op, SAMPLE_MD5_HEX).is_none(),
            "expected None for {op:?}"
        );
    }
}

#[test]
fn md5_predicate_new_with_values_is_one_of_rejects_bad_slice() {
    assert!(Hash128Predicate::new_with_values(Operation::IsOneOf, &[]).is_none());
    assert!(Hash128Predicate::new_with_values(
        Operation::IsOneOf,
        &[SAMPLE_MD5_HEX.to_string(), "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz".to_string()],
    )
    .is_none());
}

#[test]
fn md5_predicate_new_with_values_unsupported_op_returns_none() {
    assert!(Hash128Predicate::new_with_values(Operation::Is, &[SAMPLE_MD5_HEX.to_string()]).is_none());
}

#[test]
fn md5_predicate_is_accepts_uppercase_hex_in_pattern() {
    let upper = SAMPLE_MD5_HEX.to_ascii_uppercase();
    let p = Hash128Predicate::new(Operation::Is, &upper).expect("predicate");
    assert!(p.evaluate(SAMPLE_MD5_HEX.parse().expect("hash")));
}

#[test]
fn sha1_predicate_is_evaluates_equal() {
    let p = Hash160Predicate::new(Operation::Is, SAMPLE_SHA1_HEX).expect("predicate");
    let matching: Hash160 = SAMPLE_SHA1_HEX.parse().expect("hash");
    let other: Hash160 = "0000000000000000000000000000000000000000"
        .parse()
        .expect("hash");
    assert!(p.evaluate(matching));
    assert!(!p.evaluate(other));
}

#[test]
fn sha1_predicate_is_not_evaluates_inequal() {
    let p = Hash160Predicate::new(Operation::IsNot, SAMPLE_SHA1_HEX).expect("predicate");
    let matching: Hash160 = SAMPLE_SHA1_HEX.parse().expect("hash");
    let other: Hash160 = "0000000000000000000000000000000000000000"
        .parse()
        .expect("hash");
    assert!(!p.evaluate(matching));
    assert!(p.evaluate(other));
}

#[test]
fn sha1_predicate_is_one_of_via_new_with_values() {
    let a = SAMPLE_SHA1_HEX.to_string();
    let b = "0000000000000000000000000000000000000000".to_string();
    let p = Hash160Predicate::new_with_values(Operation::IsOneOf, &[a.clone(), b.clone()]).expect("predicate");
    assert!(p.evaluate(a.parse().expect("hash")));
    assert!(p.evaluate(b.parse().expect("hash")));
    assert!(!p.evaluate(
        "ffffffffffffffffffffffffffffffffffffffff"
            .parse()
            .expect("hash"),
    ));
}

#[test]
fn sha1_predicate_new_rejects_invalid_hex_string() {
    assert!(Hash160Predicate::new(Operation::Is, "deadbeef").is_none());
    assert!(Hash160Predicate::new(Operation::Is, &"g".repeat(40)).is_none());
}

#[test]
fn sha1_predicate_new_returns_none_for_unsupported_ops() {
    for op in [
        Operation::IsOneOf,
        Operation::InRange,
        Operation::GreaterThan,
        Operation::GreaterThanOrEqual,
        Operation::LessThan,
        Operation::LessThanOrEqual,
        Operation::StartsWith,
        Operation::StartsWithOneOf,
        Operation::EndsWith,
        Operation::EndsWithOneOf,
        Operation::Contains,
        Operation::ContainsOneOf,
        Operation::GlobREMatch,
    ] {
        assert!(
            Hash160Predicate::new(op, SAMPLE_SHA1_HEX).is_none(),
            "expected None for {op:?}"
        );
    }
}

#[test]
fn sha1_predicate_new_with_values_is_one_of_rejects_bad_slice() {
    assert!(Hash160Predicate::new_with_values(Operation::IsOneOf, &[]).is_none());
    assert!(Hash160Predicate::new_with_values(
        Operation::IsOneOf,
        &[
            SAMPLE_SHA1_HEX.to_string(),
            "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz".to_string(),
        ],
    )
    .is_none());
}

#[test]
fn sha1_predicate_new_with_values_unsupported_op_returns_none() {
    assert!(Hash160Predicate::new_with_values(Operation::IsNot, &[SAMPLE_SHA1_HEX.to_string()]).is_none());
}

#[test]
fn sha1_predicate_is_accepts_uppercase_hex_in_pattern() {
    let upper = SAMPLE_SHA1_HEX.to_ascii_uppercase();
    let p = Hash160Predicate::new(Operation::Is, &upper).expect("predicate");
    assert!(p.evaluate(SAMPLE_SHA1_HEX.parse().expect("hash")));
}

// --- IpAddrPredicate ---

#[test]
fn ip_addr_predicate_is_evaluates_equal() {
    let p = IpAddrPredicate::new(Operation::Is, "192.168.1.10").expect("predicate");
    assert!(p.evaluate("192.168.1.10".parse().expect("ip")));
    assert!(!p.evaluate("192.168.1.11".parse().expect("ip")));
}

#[test]
fn ip_addr_predicate_is_not_evaluates_inequal() {
    let p = IpAddrPredicate::new(Operation::IsNot, "2001:db8::1").expect("predicate");
    assert!(!p.evaluate("2001:db8::1".parse().expect("ip")));
    assert!(p.evaluate("2001:db8::2".parse().expect("ip")));
}

#[test]
fn ip_addr_predicate_in_range_ipv4_inclusive() {
    let p = IpAddrPredicate::new_with_values(
        Operation::InRange,
        &["10.0.0.1".to_string(), "10.0.0.10".to_string()],
    )
    .expect("predicate");
    assert!(!p.evaluate("10.0.0.0".parse().expect("ip")));
    assert!(p.evaluate("10.0.0.1".parse().expect("ip")));
    assert!(p.evaluate("10.0.0.5".parse().expect("ip")));
    assert!(p.evaluate("10.0.0.10".parse().expect("ip")));
    assert!(!p.evaluate("10.0.0.11".parse().expect("ip")));
}

#[test]
fn ip_addr_predicate_in_range_ipv6_inclusive() {
    let p = IpAddrPredicate::new_with_values(
        Operation::InRange,
        &["2001:db8::1".to_string(), "2001:db8::f".to_string()],
    )
    .expect("predicate");
    assert!(!p.evaluate("2001:db8::".parse().expect("ip")));
    assert!(p.evaluate("2001:db8::1".parse().expect("ip")));
    assert!(p.evaluate("2001:db8::a".parse().expect("ip")));
    assert!(p.evaluate("2001:db8::f".parse().expect("ip")));
    assert!(!p.evaluate("2001:db8::10".parse().expect("ip")));
}

#[test]
fn ip_addr_predicate_is_one_of_via_new_with_values() {
    let p = IpAddrPredicate::new_with_values(
        Operation::IsOneOf,
        &[
            "127.0.0.1".to_string(),
            "2001:db8::42".to_string(),
            "127.0.0.1".to_string(),
        ],
    )
    .expect("predicate");
    assert!(p.evaluate("127.0.0.1".parse().expect("ip")));
    assert!(p.evaluate("2001:db8::42".parse().expect("ip")));
    assert!(!p.evaluate("8.8.8.8".parse().expect("ip")));
}

#[test]
fn ip_addr_predicate_new_rejects_invalid_ip() {
    assert!(IpAddrPredicate::new(Operation::Is, "not-an-ip").is_none());
}

#[test]
fn ip_addr_predicate_new_with_values_rejects_bad_slices() {
    assert!(IpAddrPredicate::new_with_values(Operation::InRange, &[]).is_none());
    assert!(IpAddrPredicate::new_with_values(Operation::InRange, &["1.1.1.1".to_string()]).is_none());
    assert!(IpAddrPredicate::new_with_values(
        Operation::InRange,
        &["10.0.0.10".to_string(), "10.0.0.1".to_string()],
    )
    .is_none());
    assert!(IpAddrPredicate::new_with_values(
        Operation::InRange,
        &["bad".to_string(), "10.0.0.1".to_string()],
    )
    .is_none());
    assert!(IpAddrPredicate::new_with_values(Operation::IsOneOf, &[]).is_none());
    assert!(IpAddrPredicate::new_with_values(
        Operation::IsOneOf,
        &["192.168.0.1".to_string(), "bad".to_string()],
    )
    .is_none());
}

#[test]
fn ip_addr_predicate_new_and_new_with_values_unsupported_ops_return_none() {
    assert!(IpAddrPredicate::new(Operation::InRange, "10.0.0.1").is_none());
    assert!(IpAddrPredicate::new(Operation::IsOneOf, "10.0.0.1").is_none());
    assert!(IpAddrPredicate::new_with_values(Operation::Is, &["10.0.0.1".to_string()]).is_none());
    assert!(IpAddrPredicate::new_with_values(Operation::IsNot, &["10.0.0.1".to_string()]).is_none());
}

// --- single_string (StartsWith, EndsWith, Contains, Equals, Different) ---

#[test]
fn starts_with_case_sensitive() {
    let p = StartsWith::new("Hello", false);
    assert!(p.evaluate("Hello world"));
    assert!(!p.evaluate("hello world"));
    assert!(!p.evaluate("Hi Hello"));
}

#[test]
fn starts_with_ignore_case_ascii() {
    let p = StartsWith::new("Foo", true);
    assert!(p.evaluate("FOObar"));
    assert!(p.evaluate("foo"));
    assert!(!p.evaluate("barFoo"));
}

#[test]
fn starts_with_ignore_case_unicode() {
    let p = StartsWith::new("CAFÉ", true);
    assert!(p.evaluate("cafétail"));
    assert!(p.evaluate("Cafétail"));
    assert!(!p.evaluate("caf"));
}

#[test]
fn starts_with_empty_pattern() {
    let p = StartsWith::new("", false);
    assert!(p.evaluate("anything"));
    assert!(p.evaluate(""));
}

#[test]
fn ends_with_case_sensitive() {
    let p = EndsWith::new(".log", false);
    assert!(p.evaluate("app.log"));
    assert!(!p.evaluate("app.LOG"));
    assert!(!p.evaluate(".logapp"));
}

#[test]
fn ends_with_ignore_case_ascii() {
    let p = EndsWith::new("bar", true);
    assert!(p.evaluate("fooBAR"));
    assert!(p.evaluate("bar"));
    assert!(!p.evaluate("barfoo"));
}

#[test]
fn ends_with_ignore_case_unicode() {
    let p = EndsWith::new("été", true);
    assert!(p.evaluate("pluieÉTÉ"));
    assert!(!p.evaluate("pluieé"));
}

#[test]
fn contains_case_sensitive() {
    let p = Contains::new("needle", false);
    assert!(p.evaluate("hayneedlestack"));
    assert!(!p.evaluate("hayNEEDLEstack"));
    assert!(!p.evaluate("need"));
}

#[test]
fn contains_ignore_case_ascii() {
    let p = Contains::new("WoRlD", true);
    assert!(p.evaluate("hello world!"));
    assert!(p.evaluate("WORLD"));
    assert!(!p.evaluate("wod"));
}

#[test]
fn contains_ignore_case_unicode() {
    let p = Contains::new("NAÏVE", true);
    assert!(p.evaluate("xnaïvey"));
    assert!(!p.evaluate("naive"));
}

#[test]
fn equals_case_sensitive() {
    let p = Equals::new("abc", false);
    assert!(p.evaluate("abc"));
    assert!(!p.evaluate("ABC"));
    assert!(!p.evaluate("abcd"));
    assert!(!p.evaluate("ab"));
}

#[test]
fn equals_ignore_case_ascii() {
    let p = Equals::new("AbC", true);
    assert!(p.evaluate("abc"));
    assert!(p.evaluate("ABC"));
    assert!(!p.evaluate("abcd"));
}

#[test]
fn equals_ignore_case_unicode_full_string() {
    let p = Equals::new("é", true);
    assert!(p.evaluate("É"));
    assert!(p.evaluate("é"));
    assert!(!p.evaluate("éx"));
}

#[test]
fn different_case_sensitive() {
    let p = Different::new("x", false);
    assert!(p.evaluate("y"));
    assert!(!p.evaluate("x"));
    assert!(p.evaluate("X"));
}

#[test]
fn different_ignore_case() {
    let p = Different::new("Ab", true);
    assert!(!p.evaluate("ab"));
    assert!(!p.evaluate("AB"));
    assert!(p.evaluate("abc"));
    assert!(p.evaluate("a"));
}

// --- string_contains_one_of (ContainsOneOf) ---

fn patterns(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn contains_one_of_case_sensitive() {
    let p = ContainsOneOf::new(&patterns(&["foo", "bar"]), false).expect("automaton");
    assert!(p.evaluate("hellofoobar"));
    assert!(p.evaluate("prefixbarmid"));
    assert!(!p.evaluate("FOO"));
    assert!(!p.evaluate("baz"));
}

#[test]
fn contains_one_of_case_sensitive_exact_substring() {
    let p = ContainsOneOf::new(&patterns(&["Needle"]), false).expect("automaton");
    assert!(p.evaluate("xNeedley"));
    assert!(!p.evaluate("xneedley"));
}

#[test]
fn contains_one_of_ignore_case_ascii() {
    let p = ContainsOneOf::new(&patterns(&["HeLLo", "WoRlD"]), true).expect("automaton");
    assert!(p.evaluate("prefix hello suffix"));
    assert!(p.evaluate("WORLD"));
    assert!(!p.evaluate("hell"));
}

#[test]
fn contains_one_of_ignore_case_unicode() {
    let p = ContainsOneOf::new(&patterns(&["CAFÉ", "naïve"]), true).expect("automaton");
    assert!(p.evaluate("drink café now"));
    assert!(p.evaluate("NAÏVE case"));
    assert!(!p.evaluate("cafe"));
}

#[test]
fn contains_one_of_empty_string_pattern_matches_any_input() {
    // aho-corasick allows empty patterns; they match at every position.
    let p = ContainsOneOf::new(&patterns(&[""]), false).expect("automaton");
    assert!(p.evaluate(""));
    assert!(p.evaluate("abc"));
    let p2 = ContainsOneOf::new(&patterns(&["needle", ""]), true).expect("automaton");
    assert!(p2.evaluate("no needle here"));
}

#[test]
fn contains_one_of_empty_pattern_list() {
    let empty: Vec<String> = vec![];
    let p = ContainsOneOf::new(&empty, false).expect("empty list builds");
    assert!(!p.evaluate("anything"));
}

#[test]
fn contains_one_of_ignore_case_long_input_uses_heap_lowercase() {
    let haystack = format!("{}needle", "a".repeat(2500));
    let p = ContainsOneOf::new(&patterns(&["NEEDLE"]), true).expect("automaton");
    assert!(p.evaluate(&haystack));
    assert!(!p.evaluate(&"a".repeat(2500)));
}

// --- string_starts_with_one_of (StartsWithOneOf) ---

#[test]
fn starts_with_one_of_case_sensitive() {
    let p = StartsWithOneOf::new(&patterns(&["foo", "bar"]), false).expect("fst");
    assert!(p.evaluate("food"));
    assert!(p.evaluate("barrel"));
    assert!(p.evaluate("foo"));
    assert!(!p.evaluate("FOO"));
    assert!(!p.evaluate("hellofoobar"));
    assert!(!p.evaluate("baz"));
}

#[test]
fn starts_with_one_of_shared_prefix_still_matches_shorter_pattern() {
    let p = StartsWithOneOf::new(&patterns(&["foo", "foobar"]), false).expect("fst");
    assert!(p.evaluate("foonly"));
    assert!(p.evaluate("fooball"));
    assert!(p.evaluate("foobar"));
    assert!(!p.evaluate("fobar"));
    assert!(!p.evaluate("fomo"));
    assert!(!p.evaluate("fo"));
}

#[test]
fn starts_with_one_of_ignore_case_ascii() {
    let p = StartsWithOneOf::new(&patterns(&["HeLLo", "BAR"]), true).expect("fst");
    assert!(p.evaluate("helloKitty"));
    assert!(p.evaluate("Hello"));
    assert!(p.evaluate("BARx"));
    assert!(p.evaluate("bar"));
    assert!(!p.evaluate("nothello"));
    assert!(!p.evaluate("zbar"));
}

#[test]
fn starts_with_one_of_ignore_case_unicode() {
    let p = StartsWithOneOf::new(&patterns(&["CAFÉ"]), true).expect("fst");
    assert!(p.evaluate("cafétail"));
    assert!(p.evaluate("Café!"));
    assert!(!p.evaluate("cafe"));
}

#[test]
fn starts_with_one_of_pattern_longer_than_value() {
    let p = StartsWithOneOf::new(&patterns(&["toolong"]), false).expect("fst");
    assert!(!p.evaluate("too"));
    assert!(!p.evaluate("toolon"));
    assert!(p.evaluate("toolonger"));
}

#[test]
fn starts_with_one_of_empty_pattern_list() {
    let empty: Vec<String> = vec![];
    let p = StartsWithOneOf::new(&empty, false).expect("empty fst");
    assert!(!p.evaluate("anything"));
    assert!(!p.evaluate(""));
}

#[test]
fn starts_with_one_of_empty_string_pattern_matches_any_non_empty_value() {
    let p = StartsWithOneOf::new(&patterns(&[""]), false).expect("fst");
    assert!(p.evaluate("a"));
    assert!(p.evaluate(""));
}

#[test]
fn starts_with_one_of_ignore_case_long_input_uses_heap_lowercase() {
    let tail = "a".repeat(2500);
    let value = format!("HELLO{tail}");
    let p = StartsWithOneOf::new(&patterns(&["hello"]), true).expect("fst");
    assert!(p.evaluate(&value));
    assert!(!p.evaluate(&tail));
}

// --- string_ends_with_one_of (EndsWithOneOf) ---

#[test]
fn ends_with_one_of_case_sensitive() {
    let p = EndsWithOneOf::new(&patterns(&["foo", "bar"]), false).expect("fst");
    assert!(p.evaluate("xxfoo"));
    assert!(p.evaluate("bazbar"));
    assert!(p.evaluate("foo"));
    assert!(p.evaluate("bar"));
    assert!(p.evaluate("foobar"));
    assert!(!p.evaluate("FOO"));
    assert!(!p.evaluate("food"));
    assert!(!p.evaluate("barfood"));
}

#[test]
fn ends_with_one_of_shared_suffix_nested_patterns() {
    let p = EndsWithOneOf::new(&patterns(&["bar", "obar"]), false).expect("fst");
    assert!(p.evaluate("fooobar"));
    assert!(p.evaluate("xbar"));
    assert!(p.evaluate("obar"));
    assert!(!p.evaluate("barx"));
}

#[test]
fn ends_with_one_of_ignore_case_ascii() {
    let p = EndsWithOneOf::new(&patterns(&["HeLLo", "BAR"]), true).expect("fst");
    assert!(p.evaluate("kittyHELLO"));
    assert!(p.evaluate("xxbar"));
    assert!(p.evaluate("BAR"));
    assert!(!p.evaluate("helloPREFIX"));
    assert!(!p.evaluate("barfoo"));
}

#[test]
fn ends_with_one_of_ignore_case_ascii_suffix_after_unicode_prefix() {
    // Suffix bytes are ASCII; full-string lowercasing still walks non-ASCII prefix.
    let p = EndsWithOneOf::new(&patterns(&["BAR"]), true).expect("fst");
    assert!(p.evaluate("caféBAR"));
    assert!(p.evaluate("naïvebar"));
    assert!(!p.evaluate("barcafé"));
}

#[test]
fn ends_with_one_of_pattern_longer_than_value() {
    let p = EndsWithOneOf::new(&patterns(&["toolong"]), false).expect("fst");
    assert!(!p.evaluate("too"));
    assert!(!p.evaluate("toolon"));
    assert!(p.evaluate("xtoolong"));
}

#[test]
fn ends_with_one_of_empty_pattern_list_returns_none() {
    let empty: Vec<String> = vec![];
    assert!(EndsWithOneOf::new(&empty, false).is_none());
}

#[test]
fn ends_with_one_of_empty_string_pattern_matches_any_value() {
    let p = EndsWithOneOf::new(&patterns(&[""]), false).expect("fst");
    assert!(p.evaluate("a"));
    assert!(p.evaluate(""));
}

#[test]
fn ends_with_one_of_ignore_case_long_input_uses_heap_lowercase() {
    let prefix = "a".repeat(2500);
    let value = format!("{prefix}HELLO");
    let p = EndsWithOneOf::new(&patterns(&["hello"]), true).expect("fst");
    assert!(p.evaluate(&value));
    assert!(!p.evaluate(&prefix));
}

// --- string_is_one_of (IsOneOf) ---

#[test]
fn is_one_of_case_sensitive() {
    let p = IsOneOf::new(&patterns(&["foo", "bar"]), false).expect("set");
    assert!(p.evaluate("foo"));
    assert!(p.evaluate("bar"));
    assert!(!p.evaluate("FOO"));
    assert!(!p.evaluate("food"));
    assert!(!p.evaluate("fo"));
}

#[test]
fn is_one_of_ignore_case_ascii() {
    let p = IsOneOf::new(&patterns(&["AbC", "xY"]), true).expect("set");
    assert!(p.evaluate("abc"));
    assert!(p.evaluate("ABC"));
    assert!(p.evaluate("xy"));
    assert!(!p.evaluate("abcd"));
    assert!(!p.evaluate("ab"));
}

#[test]
fn is_one_of_ignore_case_unicode() {
    let p = IsOneOf::new(&patterns(&["CAFÉ", "naïve"]), true).expect("set");
    assert!(p.evaluate("café"));
    assert!(p.evaluate("NAÏVE"));
    assert!(!p.evaluate("cafe"));
}

#[test]
fn is_one_of_exact_match_not_substring() {
    let p = IsOneOf::new(&patterns(&["needle"]), false).expect("set");
    assert!(p.evaluate("needle"));
    assert!(!p.evaluate("need"));
    assert!(!p.evaluate("needlee"));
}

#[test]
fn is_one_of_deduplicates_normalized_entries() {
    let raw = patterns(&["Foo", "foo", "FOO"]);
    let p = IsOneOf::new(&raw, true).expect("set");
    assert!(p.evaluate("foo"));
    assert!(p.evaluate("FOO"));
}

#[test]
fn is_one_of_empty_list() {
    let empty: Vec<String> = vec![];
    let p = IsOneOf::new(&empty, false).expect("set");
    assert!(!p.evaluate(""));
    assert!(!p.evaluate("x"));
}

#[test]
fn is_one_of_empty_string_member() {
    let p = IsOneOf::new(&patterns(&["", "x"]), false).expect("set");
    assert!(p.evaluate(""));
    assert!(p.evaluate("x"));
    assert!(!p.evaluate(" "));
}

#[test]
fn is_one_of_linear_scan_fewer_than_sixteen_members() {
    let list = patterns(&[
        "k00", "k01", "k02", "k03", "k04", "k05", "k06", "k07", "k08", "k09", "k10", "k11", "k12", "k13", "k14",
    ]);
    let p = IsOneOf::new(&list, false).expect("set");
    assert!(p.evaluate("k07"));
    assert!(!p.evaluate("k15"));
}

#[test]
fn is_one_of_binary_search_sixteen_or_more_members() {
    let list = patterns(&[
        "s00", "s01", "s02", "s03", "s04", "s05", "s06", "s07", "s08", "s09", "s10", "s11", "s12", "s13", "s14",
        "s15",
    ]);
    let p = IsOneOf::new(&list, false).expect("set");
    assert!(p.evaluate("s00"));
    assert!(p.evaluate("s15"));
    assert!(!p.evaluate("s16"));
    assert!(!p.evaluate("s0"));
}

#[test]
fn is_one_of_ignore_case_long_input_uses_heap_lowercase() {
    let long = "a".repeat(600);
    let list = vec![long.clone()];
    let p = IsOneOf::new(&list, true).expect("set");
    assert!(p.evaluate(&long));
    assert!(!p.evaluate("b"));
}

// --- string_predicate (new / new_with_values dispatch) ---

#[test]
fn string_predicate_new_starts_with() {
    let p = StringPredicate::new(Operation::StartsWith, "Hi", false).expect("predicate");
    assert!(p.evaluate("Hi there"));
    assert!(!p.evaluate("there Hi"));
}

#[test]
fn string_predicate_new_ends_with_ignore_case() {
    let p = StringPredicate::new(Operation::EndsWith, "log", true).expect("predicate");
    assert!(p.evaluate("app.LOG"));
    assert!(!p.evaluate("logapp"));
}

#[test]
fn string_predicate_new_contains() {
    let p = StringPredicate::new(Operation::Contains, "mid", false).expect("predicate");
    assert!(p.evaluate("prefixmiddlesuffix"));
    assert!(!p.evaluate("prefixMIDsuffix"));
}

#[test]
fn string_predicate_new_is_and_is_not() {
    let eq = StringPredicate::new(Operation::Is, "x", false).expect("predicate");
    assert!(eq.evaluate("x"));
    assert!(!eq.evaluate("y"));
    let ne = StringPredicate::new(Operation::IsNot, "x", true).expect("predicate");
    assert!(!ne.evaluate("X"));
    assert!(ne.evaluate("y"));
}

#[test]
fn string_predicate_new_unsupported_operations_return_none() {
    assert!(StringPredicate::new(Operation::InRange, "a", false).is_none());
    assert!(StringPredicate::new(Operation::IsOneOf, "a", false).is_none());
    assert!(StringPredicate::new(Operation::ContainsOneOf, "a", false).is_none());
    assert!(StringPredicate::new(Operation::StartsWithOneOf, "a", false).is_none());
    assert!(StringPredicate::new(Operation::GreaterThan, "a", false).is_none());
    assert!(StringPredicate::new(Operation::GlobREMatch, "*.txt", false).is_none());
}

#[test]
fn string_predicate_new_with_values_delegates_to_list_predicates() {
    let p =
        StringPredicate::new_with_values(Operation::ContainsOneOf, &patterns(&["foo"]), false).expect("predicate");
    assert!(p.evaluate("afoob"));
    assert!(!p.evaluate("fOo"));

    let p2 =
        StringPredicate::new_with_values(Operation::StartsWithOneOf, &patterns(&["ab"]), false).expect("predicate");
    assert!(p2.evaluate("abc"));
    assert!(!p2.evaluate("zab"));

    let p3 =
        StringPredicate::new_with_values(Operation::EndsWithOneOf, &patterns(&["z"]), false).expect("predicate");
    assert!(p3.evaluate("yyz"));
    assert!(!p3.evaluate("zyy"));

    let p4 = StringPredicate::new_with_values(Operation::IsOneOf, &patterns(&["a", "b"]), false).expect("predicate");
    assert!(p4.evaluate("a"));
    assert!(p4.evaluate("b"));
    assert!(!p4.evaluate("ab"));
}

#[test]
fn string_predicate_new_with_values_propagates_inner_none() {
    let empty: Vec<String> = vec![];
    assert!(StringPredicate::new_with_values(Operation::EndsWithOneOf, &empty, false).is_none());
}

#[test]
fn string_predicate_new_with_values_unsupported_operations_return_none() {
    assert!(StringPredicate::new_with_values(Operation::Is, &patterns(&["a"]), false).is_none());
    assert!(StringPredicate::new_with_values(Operation::StartsWith, &patterns(&["a"]), false).is_none());
    assert!(StringPredicate::new_with_values(Operation::Contains, &patterns(&["a"]), false).is_none());
    assert!(
        StringPredicate::new_with_values(Operation::InRange, &in_range_pair("1", "2"), false).is_none()
    );
}

// --- path_predicate (new / new_with_values dispatch, &[u8] paths) ---

#[test]
fn path_predicate_new_starts_with() {
    let p = PathPredicate::new(Operation::StartsWith, "C:\\Win", false).expect("predicate");
    assert!(p.evaluate(br"C:\Windows\system32"));
    assert!(!p.evaluate(br"D:\Windows"));
}

#[test]
fn path_predicate_new_ends_with_ignore_case() {
    let p = PathPredicate::new(Operation::EndsWith, ".log", true).expect("predicate");
    assert!(p.evaluate(b"dir/app.LOG"));
    assert!(!p.evaluate(b"logapp"));
}

#[test]
fn path_predicate_new_contains() {
    let p = PathPredicate::new(Operation::Contains, "node_modules", false).expect("predicate");
    assert!(p.evaluate(b"proj/node_modules/pkg"));
    assert!(!p.evaluate(b"proj/NODE_MODULES/pkg"));
}

#[test]
fn path_predicate_new_is_and_is_not() {
    let eq = PathPredicate::new(Operation::Is, "/etc/hosts", false).expect("predicate");
    assert!(eq.evaluate(b"/etc/hosts"));
    assert!(!eq.evaluate(b"/etc/host"));
    let ne = PathPredicate::new(Operation::IsNot, "readme", true).expect("predicate");
    assert!(!ne.evaluate(b"README"));
    assert!(ne.evaluate(b"notes.txt"));
}

#[test]
fn path_predicate_new_glob_re_match() {
    let p = PathPredicate::new(Operation::GlobREMatch, "*.txt", false).expect("predicate");
    assert!(p.evaluate(b"notes.txt"));
    assert!(!p.evaluate(b"notes.md"));
}

#[test]
fn path_predicate_new_glob_re_match_invalid_pattern_returns_none() {
    assert!(PathPredicate::new(Operation::GlobREMatch, "[", false).is_none());
}

#[test]
fn path_predicate_new_unsupported_operations_return_none() {
    assert!(PathPredicate::new(Operation::InRange, "a", false).is_none());
    assert!(PathPredicate::new(Operation::IsOneOf, "a", false).is_none());
    assert!(PathPredicate::new(Operation::ContainsOneOf, "a", false).is_none());
    assert!(PathPredicate::new(Operation::StartsWithOneOf, "a", false).is_none());
    assert!(PathPredicate::new(Operation::GreaterThan, "a", false).is_none());
}

#[test]
fn path_predicate_new_with_values_delegates_to_list_predicates() {
    let p =
        PathPredicate::new_with_values(Operation::ContainsOneOf, &patterns(&["foo"]), false).expect("predicate");
    assert!(p.evaluate(b"afoob"));
    assert!(!p.evaluate(b"fOo"));

    let p2 =
        PathPredicate::new_with_values(Operation::StartsWithOneOf, &patterns(&["ab"]), false).expect("predicate");
    assert!(p2.evaluate(b"abc"));
    assert!(!p2.evaluate(b"zab"));

    let p3 =
        PathPredicate::new_with_values(Operation::EndsWithOneOf, &patterns(&["z"]), false).expect("predicate");
    assert!(p3.evaluate(b"yyz"));
    assert!(!p3.evaluate(b"zyy"));

    let p4 = PathPredicate::new_with_values(Operation::IsOneOf, &patterns(&["a", "b"]), false).expect("predicate");
    assert!(p4.evaluate(b"a"));
    assert!(p4.evaluate(b"b"));
    assert!(!p4.evaluate(b"ab"));
}

#[test]
fn path_predicate_new_with_values_glob_re_match() {
    let p = PathPredicate::new_with_values(Operation::GlobREMatch, &patterns(&["*.txt", "*.md"]), false)
        .expect("predicate");
    assert!(p.evaluate(b"x.txt"));
    assert!(p.evaluate(b"y.md"));
    assert!(!p.evaluate(b"z.log"));
}

#[test]
fn path_predicate_new_with_values_glob_re_match_empty_or_all_invalid_returns_none() {
    let empty: Vec<String> = vec![];
    assert!(PathPredicate::new_with_values(Operation::GlobREMatch, &empty, false).is_none());
    assert!(PathPredicate::new_with_values(Operation::GlobREMatch, &patterns(&["[", "("]), false).is_none());
}

#[test]
fn path_predicate_new_with_values_propagates_inner_none() {
    let empty: Vec<String> = vec![];
    assert!(PathPredicate::new_with_values(Operation::EndsWithOneOf, &empty, false).is_none());
}

#[test]
fn path_predicate_new_with_values_unsupported_operations_return_none() {
    assert!(PathPredicate::new_with_values(Operation::Is, &patterns(&["a"]), false).is_none());
    assert!(PathPredicate::new_with_values(Operation::StartsWith, &patterns(&["a"]), false).is_none());
    assert!(PathPredicate::new_with_values(Operation::Contains, &patterns(&["a"]), false).is_none());
    assert!(
        PathPredicate::new_with_values(Operation::InRange, &in_range_pair("1", "2"), false).is_none()
    );
}

// --- glob_re_match (GlobREMatch) ---

#[test]
fn glob_re_match_with_value_valid_wildcard() {
    let p = GlobREMatch::with_value("*.txt").expect("matcher");
    assert!(p.evaluate(b"file.txt"));
    assert!(p.evaluate(b"x.y.txt"));
    assert!(!p.evaluate(b"file.tx"));
}

#[test]
fn glob_re_match_with_value_invalid_pattern_returns_none() {
    assert!(GlobREMatch::with_value("[").is_none());
}

#[test]
fn glob_re_match_with_value_double_star_path() {
    let p = GlobREMatch::with_value("**/out.log").expect("matcher");
    assert!(p.evaluate(b"out.log"));
    assert!(p.evaluate(b"build/out.log"));
    assert!(!p.evaluate(b"out.log.old"));
}

#[test]
fn glob_re_match_with_value_equivalent_to_new_single_entry() {
    let pat = "**/*.{md,txt}";
    let via_with = GlobREMatch::with_value(pat).expect("with_value");
    let via_new = GlobREMatch::new(&patterns(&[pat])).expect("new");
    for path in [
        b"README.md".as_slice(),
        b"notes.txt".as_slice(),
        b"deep/x/y/z.txt".as_slice(),
        b"src/main.rs".as_slice(),
        b"pic.png".as_slice(),
    ] {
        assert_eq!(
            via_with.evaluate(path),
            via_new.evaluate(path),
            "{path:?}"
        );
    }
}

#[test]
fn glob_re_match_wildcard_suffix() {
    let p = GlobREMatch::new(&patterns(&["*.txt"])).expect("matcher");
    assert!(p.evaluate(b"file.txt"));
    assert!(p.evaluate(b"a.txt"));
    assert!(!p.evaluate(b"file.tx"));
    assert!(!p.evaluate(b"file.txt.gz"));
}

#[test]
fn glob_re_match_literal_and_alternation_via_any() {
    let p = GlobREMatch::new(&patterns(&["alpha", "beta"])).expect("matcher");
    assert!(p.evaluate(b"alpha"));
    assert!(p.evaluate(b"beta"));
    assert!(!p.evaluate(b"gamma"));
    assert!(!p.evaluate(b"alph"));
}

#[test]
fn glob_re_match_double_star_prefix() {
    let p = GlobREMatch::new(&patterns(&["**/out.log"])).expect("matcher");
    assert!(p.evaluate(b"out.log"));
    assert!(p.evaluate(b"build/out.log"));
    assert!(p.evaluate(b"a/b/c/out.log"));
    assert!(!p.evaluate(b"out.log.old"));
}

#[test]
fn glob_re_match_empty_list_returns_none() {
    let empty: Vec<String> = vec![];
    assert!(GlobREMatch::new(&empty).is_none());
}

#[test]
fn glob_re_match_all_invalid_patterns_returns_none() {
    assert!(GlobREMatch::new(&patterns(&["["])).is_none());
}

#[test]
fn glob_re_match_skips_invalid_keeps_valid() {
    let list = vec!["[".to_string(), "*.md".to_string()];
    let p = GlobREMatch::new(&list).expect("matcher");
    assert!(p.evaluate(b"README.md"));
    assert!(!p.evaluate(b"README.txt"));
}

#[test]
fn glob_re_match_evaluate_accepts_utf8_bytes() {
    let p = GlobREMatch::new(&patterns(&["café"])).expect("matcher");
    assert!(p.evaluate("café".as_bytes()));
    assert!(!p.evaluate(b"cafe"));
}

#[test]
fn glob_re_match_long_valid_utf8_input() {
    let body = "a".repeat(3000);
    let path = format!("{body}.log");
    let p = GlobREMatch::new(&patterns(&["*.log"])).expect("matcher");
    assert!(p.evaluate(path.as_bytes()));
    assert!(!p.evaluate(format!("{body}.txt").as_bytes()));
}

#[test]
fn glob_re_match_double_star_braced_go_rs_extensions() {
    let p = GlobREMatch::new(&patterns(&["**/{*.{go,rs}}"])).expect("matcher");
    assert!(p.evaluate(b"main.go"));
    assert!(p.evaluate(b"pkg/lib.rs"));
    assert!(p.evaluate(b"deep/nested/module.go"));
    assert!(p.evaluate(b"src/foo/bar.rs"));
    assert!(!p.evaluate(b"main.py"));
    assert!(!p.evaluate(b"README.md"));
}

#[test]
fn glob_re_match_double_star_md_or_txt() {
    let p = GlobREMatch::new(&patterns(&["**/*.{md,txt}"])).expect("matcher");
    assert!(p.evaluate(b"README.md"));
    assert!(p.evaluate(b"notes.txt"));
    assert!(p.evaluate(b"docs/guide.md"));
    assert!(p.evaluate(b"deep/path/LICENSE.txt"));
    assert!(!p.evaluate(b"src/main.rs"));
    assert!(!p.evaluate(b"image.png"));
}

#[test]
fn glob_re_match_double_star_many_text_like_extensions() {
    let p = GlobREMatch::new(&patterns(&["**/*.{md,rs,toml,txt,yaml,yml}"])).expect("matcher");
    assert!(p.evaluate(b"Cargo.toml"));
    assert!(p.evaluate(b"src/lib.rs"));
    assert!(p.evaluate(b"config.yaml"));
    assert!(p.evaluate(b"docker-compose.yml"));
    assert!(p.evaluate(b"readme.MD"));
    assert!(p.evaluate(b"plain.txt"));
    assert!(!p.evaluate(b"binary.exe"));
    assert!(!p.evaluate(b"photo.jpg"));
}

#[test]
fn glob_re_match_videos_nested_mp4_or_webm() {
    let p = GlobREMatch::new(&patterns(&["videos/**/{*.{mp4,webm}}"])).expect("matcher");
    assert!(p.evaluate(b"videos/clip.mp4"));
    assert!(p.evaluate(b"videos/raw/take.webm"));
    assert!(p.evaluate(b"videos/2024/jan/clip.mp4"));
    assert!(!p.evaluate(b"movies/clip.mp4"));
    assert!(!p.evaluate(b"videos/readme.txt"));
    assert!(!p.evaluate(b"videos/thumb.jpg"));
}

fn datetime_ts(s: &str) -> u64 {
    DateTime::from_str_representation(s).unwrap().into()
}

#[test]
fn datetime_inside_range_new_requires_two_values() {
    assert!(DateTimeInsideRange::new(&[]).is_none());
    assert!(DateTimeInsideRange::new(&["2023-01-01".to_string()]).is_none());
    assert!(
        DateTimeInsideRange::new(&[
            "2023-01-01".to_string(),
            "2023-06-01".to_string(),
            "2023-12-31".to_string(),
        ])
        .is_none()
    );
}

#[test]
fn datetime_inside_range_new_rejects_min_greater_than_max() {
    assert!(
        DateTimeInsideRange::new(&["2023-12-31".to_string(), "2023-01-01".to_string()]).is_none()
    );
}

#[test]
fn datetime_inside_range_new_rejects_unparseable_bound() {
    assert!(
        DateTimeInsideRange::new(&["bogus".to_string(), "2023-06-15".to_string()]).is_none()
    );
    assert!(
        DateTimeInsideRange::new(&["2023-06-15".to_string(), "not-a-date".to_string()]).is_none()
    );
}

#[test]
fn datetime_inside_range_evaluate_inclusive() {
    let p = DateTimeInsideRange::new(&["2023-01-01".to_string(), "2023-12-31".to_string()])
        .unwrap();
    let lo = datetime_ts("2023-01-01");
    let mid = datetime_ts("2023-06-15T12:00:00");
    let hi = datetime_ts("2023-12-31");
    assert!(!p.evaluate(lo - 1));
    assert!(p.evaluate(lo));
    assert!(p.evaluate(mid));
    assert!(p.evaluate(hi));
    assert!(!p.evaluate(hi + 1));
}

#[test]
fn datetime_inside_range_single_point() {
    let t = datetime_ts("2023-06-15T00:00:00");
    let p = DateTimeInsideRange::new(&["2023-06-15".to_string(), "2023-06-15".to_string()])
        .unwrap();
    assert!(!p.evaluate(t - 1));
    assert!(p.evaluate(t));
    assert!(!p.evaluate(t + 1));
}

#[test]
fn datetime_predicate_new_greater_than() {
    let thr = datetime_ts("2023-06-15T12:00:00");
    let p = DateTimePredicate::new(Operation::GreaterThan, "2023-06-15T12:00:00").unwrap();
    assert!(!p.evaluate(thr - 1));
    assert!(!p.evaluate(thr));
    assert!(p.evaluate(thr + 1));
}

#[test]
fn datetime_predicate_new_greater_than_or_equal() {
    let thr = datetime_ts("2023-06-15T12:00:00");
    let p = DateTimePredicate::new(Operation::GreaterThanOrEqual, "2023-06-15T12:00:00").unwrap();
    assert!(!p.evaluate(thr - 1));
    assert!(p.evaluate(thr));
    assert!(p.evaluate(thr + 1));
}

#[test]
fn datetime_predicate_new_less_than() {
    let thr = datetime_ts("2023-06-15T12:00:00");
    let p = DateTimePredicate::new(Operation::LessThan, "2023-06-15T12:00:00").unwrap();
    assert!(p.evaluate(thr - 1));
    assert!(!p.evaluate(thr));
    assert!(!p.evaluate(thr + 1));
}

#[test]
fn datetime_predicate_new_less_than_or_equal() {
    let thr = datetime_ts("2023-06-15T12:00:00");
    let p = DateTimePredicate::new(Operation::LessThanOrEqual, "2023-06-15T12:00:00").unwrap();
    assert!(p.evaluate(thr - 1));
    assert!(p.evaluate(thr));
    assert!(!p.evaluate(thr + 1));
}

#[test]
fn datetime_predicate_new_is() {
    let thr = datetime_ts("2023-06-15T12:00:00");
    let p = DateTimePredicate::new(Operation::Is, "2023-06-15T12:00:00").unwrap();
    assert!(!p.evaluate(thr - 1));
    assert!(p.evaluate(thr));
    assert!(!p.evaluate(thr + 1));
}

#[test]
fn datetime_predicate_new_is_not() {
    let thr = datetime_ts("2023-06-15T12:00:00");
    let p = DateTimePredicate::new(Operation::IsNot, "2023-06-15T12:00:00").unwrap();
    assert!(p.evaluate(thr - 1));
    assert!(!p.evaluate(thr));
    assert!(p.evaluate(thr + 1));
}

#[test]
fn datetime_predicate_new_rejects_invalid_datetime_string() {
    assert!(DateTimePredicate::new(Operation::Is, "not-a-datetime").is_none());
}

#[test]
fn datetime_predicate_new_rejects_unsupported_operations() {
    assert!(DateTimePredicate::new(Operation::InRange, "2023-01-01").is_none());
    assert!(DateTimePredicate::new(Operation::Contains, "2023-01-01").is_none());
    assert!(DateTimePredicate::new(Operation::IsOneOf, "2023-01-01").is_none());
    assert!(DateTimePredicate::new(Operation::GlobREMatch, "2023-01-01").is_none());
}

#[test]
fn datetime_predicate_new_with_values_in_range() {
    let p = DateTimePredicate::new_with_values(
        Operation::InRange,
        &["2023-01-01".to_string(), "2023-12-31".to_string()],
    )
    .unwrap();
    let lo = datetime_ts("2023-01-01");
    let hi = datetime_ts("2023-12-31");
    assert!(p.evaluate(lo));
    assert!(p.evaluate(datetime_ts("2023-06-15")));
    assert!(p.evaluate(hi));
}

#[test]
fn datetime_predicate_new_with_values_only_in_range_operation() {
    assert!(
        DateTimePredicate::new_with_values(Operation::Is, &["2023-01-01".to_string()]).is_none()
    );
    assert!(
        DateTimePredicate::new_with_values(
            Operation::GreaterThan,
            &["2023-01-01".to_string(), "2023-12-31".to_string()],
        )
        .is_none()
    );
}

#[test]
fn datetime_predicate_new_with_values_rejects_bad_bounds() {
    assert!(
        DateTimePredicate::new_with_values(
            Operation::InRange,
            &["2023-12-31".to_string(), "2023-01-01".to_string()],
        )
        .is_none()
    );
    assert!(DateTimePredicate::new_with_values(Operation::InRange, &[]).is_none());
    assert!(
        DateTimePredicate::new_with_values(
            Operation::InRange,
            &["2023-01-01".to_string()],
        )
        .is_none()
    );
}

#[test]
fn bool_predicate_is_true_matches_true() {
    let p = BoolPredicate::new(Operation::Is, "true").unwrap();
    assert!(p.evaluate(true));
    assert!(!p.evaluate(false));
}

#[test]
fn bool_predicate_is_false_matches_false() {
    let p = BoolPredicate::new(Operation::Is, "false").unwrap();
    assert!(!p.evaluate(true));
    assert!(p.evaluate(false));
}

#[test]
fn bool_predicate_new_rejects_unparseable_value() {
    assert!(BoolPredicate::new(Operation::Is, "").is_none());
    assert!(BoolPredicate::new(Operation::Is, "yes").is_none());
    assert!(BoolPredicate::new(Operation::Is, "0").is_none());
    assert!(BoolPredicate::new(Operation::Is, "maybe").is_none());
}

#[test]
fn bool_predicate_new_only_supports_is() {
    assert!(BoolPredicate::new(Operation::IsNot, "true").is_none());
    assert!(BoolPredicate::new(Operation::GreaterThan, "true").is_none());
    assert!(BoolPredicate::new(Operation::InRange, "true").is_none());
    assert!(BoolPredicate::new(Operation::Contains, "true").is_none());
}
