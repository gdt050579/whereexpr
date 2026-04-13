use std::convert::TryFrom;
use std::net::IpAddr;
use std::str::FromStr;

use crate::AttributeIndex;
use crate::CompiledCondition;
use crate::ConditionList;
use crate::Error;
use crate::Operation;
use crate::Predicate;
use crate::Value;
use crate::ValueKind;
use crate::expression::{Composition, EvaluationNode};
use crate::types::{DateTime, FromRepr, Hash128, Hash160, Hash256};

fn sample_condition() -> CompiledCondition {
    CompiledCondition::new(
        AttributeIndex::new(0),
        Predicate::with_value(Operation::Is, 0u8).unwrap(),
    )
}

fn condition_list_for_expr_parse(names: &[&str]) -> ConditionList {
    let mut list = ConditionList::new();
    for name in names {
        let p = Predicate::with_value(Operation::Is, "x").expect("predicate");
        assert!(
            list.add(name, CompiledCondition::new(AttributeIndex::new(0), p)),
            "duplicate rule name {name}"
        );
    }
    list
}

fn parse_expression(input: &str, names: &[&str]) -> Result<EvaluationNode, Error> {
    let list = condition_list_for_expr_parse(names);
    crate::expr_parser::parse(input, &list)
}

#[test]
fn new_is_empty() {
    let list = ConditionList::new();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}

#[test]
fn with_capacity_starts_empty() {
    let list = ConditionList::with_capacity(64);
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}

#[test]
fn add_inserts_and_returns_true() {
    let mut list = ConditionList::new();
    assert!(list.add("alpha", sample_condition()));
    assert_eq!(list.len(), 1);
    assert!(!list.is_empty());
    assert_eq!(list.from_name("alpha"), Some(0));
    assert!(list.get(0).is_some());
}

#[test]
fn add_duplicate_name_returns_false() {
    let mut list = ConditionList::new();
    assert!(list.add("x", sample_condition()));
    assert!(!list.add("x", sample_condition()));
    assert_eq!(list.len(), 1);
    assert_eq!(list.from_name("x"), Some(0));
}

#[test]
fn names_are_case_insensitive_for_identity() {
    let mut list = ConditionList::new();
    assert!(list.add("Status", sample_condition()));
    assert!(!list.add("status", sample_condition()));
    assert_eq!(list.len(), 1);
    assert_eq!(list.from_name("STATUS"), Some(0));
}

#[test]
fn from_name_unknown_returns_none() {
    let list = ConditionList::new();
    assert_eq!(list.from_name("missing"), None);
}

#[test]
fn get_on_empty_list_returns_none() {
    let list = ConditionList::with_capacity(8);
    assert!(list.get(0).is_none());
}

#[test]
fn indices_match_insertion_order() {
    let mut list = ConditionList::new();
    assert!(list.add("first", sample_condition()));
    assert!(list.add("second", sample_condition()));
    assert_eq!(list.from_name("first"), Some(0));
    assert_eq!(list.from_name("second"), Some(1));
    assert!(list.get(0).is_some());
    assert!(list.get(1).is_some());
}

#[test]
fn get_out_of_range_returns_none() {
    let mut list = ConditionList::new();
    assert!(list.add("only", sample_condition()));
    assert!(list.get(0).is_some());
    assert!(list.get(1).is_none());
}

#[test]
fn lookups_remain_valid_across_linear_to_sorted_transition() {
    let mut list = ConditionList::new();
    for i in 0..16 {
        let name = format!("key_{i:02}");
        assert!(list.add(&name, sample_condition()), "add {name}");
        for j in 0..=i {
            let prev = format!("key_{j:02}");
            assert_eq!(
                list.from_name(&prev),
                Some(j as u16),
                "lookup {prev} after inserting {name}"
            );
        }
    }
    assert_eq!(list.len(), 16);
}

#[test]
fn many_adds_linear_then_sorted_index() {
    let mut list = ConditionList::new();
    for i in 0..50 {
        let name = format!("n{i}");
        assert!(list.add(&name, sample_condition()), "add {name}");
    }
    assert_eq!(list.len(), 50);
    for i in 0..50 {
        let name = format!("n{i}");
        assert_eq!(list.from_name(&name), Some(i as u16), "lookup {name}");
        assert!(list.get(i as u16).is_some());
    }
}

#[test]
fn operation_and_negated_splits_positive_and_negated_forms() {
    let cases: &[(Operation, Operation, bool)] = &[
        (Operation::Is, Operation::Is, false),
        (Operation::IsNot, Operation::Is, true),
        (Operation::IsOneOf, Operation::IsOneOf, false),
        (Operation::IsNotOneOf, Operation::IsOneOf, true),
        (Operation::StartsWith, Operation::StartsWith, false),
        (Operation::NotStartsWith, Operation::StartsWith, true),
        (Operation::StartsWithOneOf, Operation::StartsWithOneOf, false),
        (Operation::NotStartsWithOneOf, Operation::StartsWithOneOf, true),
        (Operation::EndsWith, Operation::EndsWith, false),
        (Operation::NotEndsWith, Operation::EndsWith, true),
        (Operation::EndsWithOneOf, Operation::EndsWithOneOf, false),
        (Operation::NotEndsWithOneOf, Operation::EndsWithOneOf, true),
        (Operation::Contains, Operation::Contains, false),
        (Operation::NotContains, Operation::Contains, true),
        (Operation::ContainsOneOf, Operation::ContainsOneOf, false),
        (Operation::NotContainsOneOf, Operation::ContainsOneOf, true),
        (Operation::GlobREMatch, Operation::GlobREMatch, false),
        (Operation::NotGlobREMatch, Operation::GlobREMatch, true),
        (Operation::GreaterThan, Operation::GreaterThan, false),
        (Operation::GreaterThanOrEqual, Operation::GreaterThanOrEqual, false),
        (Operation::LessThan, Operation::LessThan, false),
        (Operation::LessThanOrEqual, Operation::LessThanOrEqual, false),
        (Operation::InRange, Operation::InRange, false),
        (Operation::NotInRange, Operation::InRange, true),
    ];

    for &(input, expected_base, expected_negated) in cases {
        let got = input.operation_and_negated();
        assert_eq!(
            got,
            (expected_base, expected_negated),
            "operation_and_negated({input:?})"
        );
    }
}

#[test]
fn expr_parse_single_rule() {
    let node = parse_expression("a", &["a"]).expect("parse");
    assert_eq!(node, EvaluationNode::Condition(0));
}

#[test]
fn expr_parse_single_rule_extra_parens_reduced() {
    let node = parse_expression("(((a)))", &["a"]).expect("parse");
    assert_eq!(node, EvaluationNode::Condition(0));
}

#[test]
fn expr_parse_and_group() {
    let node = parse_expression("a && b", &["a", "b"]).expect("parse");
    assert_eq!(
        node,
        EvaluationNode::Group {
            composition: Composition::And,
            negated: false,
            children: vec![EvaluationNode::Condition(0), EvaluationNode::Condition(1)],
        }
    );
}

#[test]
fn expr_parse_or_group() {
    let node = parse_expression("a || b", &["a", "b"]).expect("parse");
    assert_eq!(
        node,
        EvaluationNode::Group {
            composition: Composition::Or,
            negated: false,
            children: vec![EvaluationNode::Condition(0), EvaluationNode::Condition(1)],
        }
    );
}

#[test]
fn expr_parse_not_rule() {
    let node = parse_expression("NOT a", &["a"]).expect("parse");
    assert_eq!(
        node,
        EvaluationNode::Group {
            composition: Composition::And,
            negated: true,
            children: vec![EvaluationNode::Condition(0)],
        }
    );
}

#[test]
fn expr_parse_mixed_and_or_with_parens() {
    let node = parse_expression("(a OR b) AND c", &["a", "b", "c"]).expect("parse");
    assert_eq!(
        node,
        EvaluationNode::Group {
            composition: Composition::And,
            negated: false,
            children: vec![
                EvaluationNode::Group {
                    composition: Composition::Or,
                    negated: false,
                    children: vec![EvaluationNode::Condition(0), EvaluationNode::Condition(1)],
                },
                EvaluationNode::Condition(2),
            ],
        }
    );
}

#[test]
fn expr_parse_empty_input_errors() {
    let err = parse_expression("", &["a"]).expect_err("empty");
    assert_eq!(err, Error::EmptyExpression);
}

#[test]
fn expr_parse_whitespace_only_errors() {
    let err = parse_expression(" \t\n ", &["a"]).expect_err("whitespace");
    assert_eq!(err, Error::EmptyExpression);
}

#[test]
fn expr_parse_unknown_rule_errors() {
    let err = parse_expression("missing", &["known"]).expect_err("unknown rule");
    assert!(matches!(err, Error::UnknownRuleName(_, _, _)));
}

#[test]
fn predicate_with_value_is_matches_field() {
    let p = Predicate::with_value(Operation::Is, 7u8).expect("predicate");
    assert!(p.evaluate(&Value::U8(7)));
    assert!(!p.evaluate(&Value::U8(0)));
}

#[test]
fn predicate_with_value_is_not_negates_match() {
    let p = Predicate::with_value(Operation::IsNot, 7u8).expect("predicate");
    assert!(!p.evaluate(&Value::U8(7)));
    assert!(p.evaluate(&Value::U8(0)));
}

#[test]
fn predicate_with_list_is_one_of() {
    let p = Predicate::with_list(Operation::IsOneOf, &[1u8, 3u8, 5u8]).expect("predicate");
    assert!(p.evaluate(&Value::U8(3)));
    assert!(!p.evaluate(&Value::U8(2)));
}

#[test]
fn predicate_with_value_signed_integers_is() {
    let p = Predicate::with_value(Operation::Is, -99i8).expect("i8");
    assert!(p.evaluate(&Value::I8(-99)));
    assert!(!p.evaluate(&Value::I8(0)));

    let p = Predicate::with_value(Operation::Is, -10_000i16).expect("i16");
    assert!(p.evaluate(&Value::I16(-10_000)));
    assert!(!p.evaluate(&Value::I16(0)));

    let p = Predicate::with_value(Operation::Is, -2_000_000_000i32).expect("i32");
    assert!(p.evaluate(&Value::I32(-2_000_000_000)));
    assert!(!p.evaluate(&Value::I32(0)));

    let p = Predicate::with_value(Operation::Is, -9_000_000_000_000_000_000i64).expect("i64");
    assert!(p.evaluate(&Value::I64(-9_000_000_000_000_000_000)));
    assert!(!p.evaluate(&Value::I64(0)));
}

#[test]
fn predicate_with_value_unsigned_integers_is() {
    let p = Predicate::with_value(Operation::Is, 40_000u16).expect("u16");
    assert!(p.evaluate(&Value::U16(40_000)));
    assert!(!p.evaluate(&Value::U16(0)));

    let p = Predicate::with_value(Operation::Is, 3_000_000_000u32).expect("u32");
    assert!(p.evaluate(&Value::U32(3_000_000_000)));
    assert!(!p.evaluate(&Value::U32(0)));

    let p = Predicate::with_value(Operation::Is, 18_000_000_000_000_000_000u64).expect("u64");
    assert!(p.evaluate(&Value::U64(18_000_000_000_000_000_000)));
    assert!(!p.evaluate(&Value::U64(0)));
}

#[test]
fn predicate_with_value_floats_is() {
    let p = Predicate::with_value(Operation::Is, 1.25f32).expect("f32");
    assert!(p.evaluate(&Value::F32(1.25)));
    assert!(!p.evaluate(&Value::F32(0.0)));

    let p = Predicate::with_value(Operation::Is, std::f64::consts::PI).expect("f64");
    assert!(p.evaluate(&Value::F64(std::f64::consts::PI)));
    assert!(!p.evaluate(&Value::F64(0.0)));
}

#[test]
fn predicate_with_value_string_is() {
    let p = Predicate::with_value(Operation::Is, "needle").expect("string");
    assert!(p.evaluate(&Value::String("needle")));
    assert!(!p.evaluate(&Value::String("other")));
}

#[test]
fn predicate_with_value_path_is() {
    let p = Predicate::with_value(Operation::Is, Value::Path(b"/var/log/app.log")).expect("path");
    assert!(p.evaluate(&Value::Path(b"/var/log/app.log")));
    assert!(!p.evaluate(&Value::Path(b"/tmp/other.log")));
}

#[test]
fn predicate_with_value_hashes_is() {
    let b128 = [0x11u8; 16];
    let h128 = Hash128::new(b128);
    let p = Predicate::with_value(Operation::Is, &h128).expect("hash128");
    assert!(p.evaluate(&Value::Hash128(&b128)));
    assert!(!p.evaluate(&Value::Hash128(&[0u8; 16])));

    let b160 = [0x22u8; 20];
    let h160 = Hash160::new(b160);
    let p = Predicate::with_value(Operation::Is, &h160).expect("hash160");
    assert!(p.evaluate(&Value::Hash160(&b160)));
    assert!(!p.evaluate(&Value::Hash160(&[0u8; 20])));

    let b256 = [0x33u8; 32];
    let h256 = Hash256::new(b256);
    let p = Predicate::with_value(Operation::Is, &h256).expect("hash256");
    assert!(p.evaluate(&Value::Hash256(&b256)));
    assert!(!p.evaluate(&Value::Hash256(&[0u8; 32])));
}

#[test]
fn predicate_with_value_ip_addr_is() {
    let ip = IpAddr::from([192, 168, 1, 10]);
    let p = Predicate::with_value(Operation::Is, ip).expect("ip");
    assert!(p.evaluate(&Value::IpAddr(ip)));
    assert!(!p.evaluate(&Value::IpAddr(IpAddr::from([192, 168, 1, 11]))));
}

#[test]
fn predicate_with_value_datetime_is() {
    let t: u64 = DateTime::from_repr("2021-06-01").unwrap().into();
    let p = Predicate::with_value(Operation::Is, Value::DateTime(t)).expect("datetime");
    assert!(p.evaluate(&Value::DateTime(t)));
    let other: u64 = DateTime::from_repr("2021-06-02").unwrap().into();
    assert!(!p.evaluate(&Value::DateTime(other)));
}

#[test]
fn predicate_with_value_bool_is() {
    let p = Predicate::with_value(Operation::Is, true).expect("bool");
    assert!(p.evaluate(&Value::Bool(true)));
    assert!(!p.evaluate(&Value::Bool(false)));
}

#[test]
fn predicate_with_value_list_empty_errors() {
    match Predicate::with_value_list(Operation::IsOneOf, &[]) {
        Err(e) => assert_eq!(e, Error::EmptyListForOperation(Operation::IsOneOf)),
        Ok(_) => panic!("expected empty list error"),
    }
}

#[test]
fn predicate_with_value_list_bool_kind_rejected() {
    match Predicate::with_value_list(Operation::IsOneOf, &[Value::Bool(true)]) {
        Err(e) => assert_eq!(
            e,
            Error::InvalidOperationForValue(Operation::IsOneOf, ValueKind::Bool)
        ),
        Ok(_) => panic!("expected invalid operation for bool list"),
    }
}

#[test]
fn predicate_with_value_list_unsigned_integers_is_one_of() {
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::U8(1), Value::U8(3), Value::U8(5)],
    )
    .expect("u8");
    assert!(p.evaluate(&Value::U8(3)));
    assert!(!p.evaluate(&Value::U8(2)));

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::U16(1000), Value::U16(2000), Value::U16(3000)],
    )
    .expect("u16");
    assert!(p.evaluate(&Value::U16(2000)));
    assert!(!p.evaluate(&Value::U16(1500)));

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::U32(100), Value::U32(200), Value::U32(300)],
    )
    .expect("u32");
    assert!(p.evaluate(&Value::U32(200)));
    assert!(!p.evaluate(&Value::U32(150)));

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[
            Value::U64(10_000_000_000_000_000_000),
            Value::U64(11_000_000_000_000_000_000),
        ],
    )
    .expect("u64");
    assert!(p.evaluate(&Value::U64(11_000_000_000_000_000_000)));
    assert!(!p.evaluate(&Value::U64(10_500_000_000_000_000_000)));
}

#[test]
fn predicate_with_value_list_signed_integers_is_one_of() {
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::I8(-5), Value::I8(0), Value::I8(7)],
    )
    .expect("i8");
    assert!(p.evaluate(&Value::I8(-5)));
    assert!(!p.evaluate(&Value::I8(3)));

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::I16(-30_000), Value::I16(0), Value::I16(1000)],
    )
    .expect("i16");
    assert!(p.evaluate(&Value::I16(-30_000)));
    assert!(!p.evaluate(&Value::I16(-29_000)));

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::I32(-1), Value::I32(0), Value::I32(99)],
    )
    .expect("i32");
    assert!(p.evaluate(&Value::I32(99)));
    assert!(!p.evaluate(&Value::I32(50)));

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[
            Value::I64(-9_000_000_000_000_000_000),
            Value::I64(0),
            Value::I64(1),
        ],
    )
    .expect("i64");
    assert!(p.evaluate(&Value::I64(-9_000_000_000_000_000_000)));
    assert!(!p.evaluate(&Value::I64(-8_000_000_000_000_000_000)));
}

#[test]
fn predicate_with_value_list_unsigned_integers_in_range() {
    let p = Predicate::with_value_list(Operation::InRange, &[Value::U8(10), Value::U8(100)]).expect("u8");
    assert!(p.evaluate(&Value::U8(50)));
    assert!(!p.evaluate(&Value::U8(5)));

    let p = Predicate::with_value_list(Operation::InRange, &[Value::U16(1000), Value::U16(9000)]).expect("u16");
    assert!(p.evaluate(&Value::U16(5000)));
    assert!(!p.evaluate(&Value::U16(500)));

    let p = Predicate::with_value_list(
        Operation::InRange,
        &[Value::U32(100_000), Value::U32(200_000)],
    )
    .expect("u32");
    assert!(p.evaluate(&Value::U32(150_000)));
    assert!(!p.evaluate(&Value::U32(50_000)));

    let p = Predicate::with_value_list(
        Operation::InRange,
        &[
            Value::U64(1_000_000_000_000_000_000),
            Value::U64(15_000_000_000_000_000_000),
        ],
    )
    .expect("u64");
    assert!(p.evaluate(&Value::U64(10_000_000_000_000_000_000)));
    assert!(!p.evaluate(&Value::U64(500_000_000_000_000_000)));
}

#[test]
fn predicate_with_value_list_signed_integers_in_range() {
    let p = Predicate::with_value_list(Operation::InRange, &[Value::I8(-50), Value::I8(50)]).expect("i8");
    assert!(p.evaluate(&Value::I8(0)));
    assert!(!p.evaluate(&Value::I8(-100)));

    let p = Predicate::with_value_list(
        Operation::InRange,
        &[Value::I16(-20_000), Value::I16(20_000)],
    )
    .expect("i16");
    assert!(p.evaluate(&Value::I16(0)));
    assert!(!p.evaluate(&Value::I16(30_000)));

    let p = Predicate::with_value_list(
        Operation::InRange,
        &[Value::I32(-1_000_000), Value::I32(1_000_000)],
    )
    .expect("i32");
    assert!(p.evaluate(&Value::I32(500_000)));
    assert!(!p.evaluate(&Value::I32(2_000_000)));

    let p = Predicate::with_value_list(
        Operation::InRange,
        &[
            Value::I64(-5_000_000_000_000_000_000),
            Value::I64(5_000_000_000_000_000_000),
        ],
    )
    .expect("i64");
    assert!(p.evaluate(&Value::I64(0)));
    assert!(!p.evaluate(&Value::I64(9_000_000_000_000_000_000)));
}

#[test]
fn predicate_with_value_list_floats_in_range() {
    let p = Predicate::with_value_list(Operation::InRange, &[Value::F32(0.25), Value::F32(4.0)]).expect("f32");
    assert!(p.evaluate(&Value::F32(1.5)));
    assert!(!p.evaluate(&Value::F32(0.1)));

    let p = Predicate::with_value_list(Operation::InRange, &[Value::F64(1.5), Value::F64(10.5)]).expect("f64");
    assert!(p.evaluate(&Value::F64(5.0)));
    assert!(!p.evaluate(&Value::F64(0.0)));
}

#[test]
fn predicate_with_value_list_string_is_one_of() {
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::String("red"), Value::String("green")],
    )
    .expect("string");
    assert!(p.evaluate(&Value::String("green")));
    assert!(!p.evaluate(&Value::String("blue")));
}

#[test]
fn predicate_with_value_list_path_is_one_of() {
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::Path(b"/a"), Value::Path(b"/b/c")],
    )
    .expect("path");
    assert!(p.evaluate(&Value::Path(b"/b/c")));
    assert!(!p.evaluate(&Value::Path(b"/z")));
}

#[test]
fn predicate_with_value_list_hashes_is_one_of() {
    let a128 = [0xabu8; 16];
    let b128 = [0xcdu8; 16];
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::Hash128(&a128), Value::Hash128(&b128)],
    )
    .expect("hash128");
    assert!(p.evaluate(&Value::Hash128(&b128)));
    assert!(!p.evaluate(&Value::Hash128(&[0u8; 16])));

    let a160 = [0x11u8; 20];
    let b160 = [0x22u8; 20];
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::Hash160(&a160), Value::Hash160(&b160)],
    )
    .expect("hash160");
    assert!(p.evaluate(&Value::Hash160(&b160)));
    assert!(!p.evaluate(&Value::Hash160(&[0u8; 20])));

    let a256 = [0x33u8; 32];
    let b256 = [0x44u8; 32];
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::Hash256(&a256), Value::Hash256(&b256)],
    )
    .expect("hash256");
    assert!(p.evaluate(&Value::Hash256(&b256)));
    assert!(!p.evaluate(&Value::Hash256(&[0u8; 32])));
}

#[test]
fn predicate_with_value_list_ip_is_one_of() {
    let ip1: IpAddr = "10.0.0.1".parse().unwrap();
    let ip2: IpAddr = "192.168.0.1".parse().unwrap();
    let p = Predicate::with_value_list(Operation::IsOneOf, &[Value::IpAddr(ip1), Value::IpAddr(ip2)])
        .expect("ip");
    assert!(p.evaluate(&Value::IpAddr(ip2)));
    assert!(!p.evaluate(&Value::IpAddr("10.0.0.9".parse().unwrap())));
}

#[test]
fn predicate_with_value_list_ip_in_range() {
    let lo: IpAddr = "127.0.0.1".parse().unwrap();
    let hi: IpAddr = "127.0.0.10".parse().unwrap();
    let p = Predicate::with_value_list(Operation::InRange, &[Value::IpAddr(lo), Value::IpAddr(hi)]).expect("ip");
    assert!(p.evaluate(&Value::IpAddr("127.0.0.5".parse().unwrap())));
    assert!(!p.evaluate(&Value::IpAddr("127.0.0.0".parse().unwrap())));
}

#[test]
fn predicate_with_value_list_datetime_in_range() {
    let a: u64 = DateTime::from_repr("2020-06-10").unwrap().into();
    let b: u64 = DateTime::from_repr("2020-06-20").unwrap().into();
    let p = Predicate::with_value_list(Operation::InRange, &[Value::DateTime(a), Value::DateTime(b)])
        .expect("datetime");
    let mid: u64 = DateTime::from_repr("2020-06-15").unwrap().into();
    assert!(p.evaluate(&Value::DateTime(mid)));
    let before: u64 = DateTime::from_repr("2020-06-01").unwrap().into();
    assert!(!p.evaluate(&Value::DateTime(before)));
}

#[test]
fn predicate_with_str_parsed_u8_is() {
    let p = Predicate::with_str(Operation::Is, "200", ValueKind::U8, false).expect("predicate");
    assert!(p.evaluate(&Value::U8(200)));
    assert!(!p.evaluate(&Value::U8(199)));
}

#[test]
fn predicate_with_str_signed_integers_is() {
    let p = Predicate::with_str(Operation::Is, "-42", ValueKind::I8, false).expect("i8");
    assert!(p.evaluate(&Value::I8(-42)));

    let p = Predicate::with_str(Operation::Is, "-1000", ValueKind::I16, false).expect("i16");
    assert!(p.evaluate(&Value::I16(-1000)));

    let p = Predicate::with_str(Operation::Is, "-2000000000", ValueKind::I32, false).expect("i32");
    assert!(p.evaluate(&Value::I32(-2_000_000_000)));

    let p = Predicate::with_str(Operation::Is, "-5000000000000000000", ValueKind::I64, false).expect("i64");
    assert!(p.evaluate(&Value::I64(-5_000_000_000_000_000_000)));
}

#[test]
fn predicate_with_str_unsigned_integers_is() {
    let p = Predicate::with_str(Operation::Is, "65500", ValueKind::U16, false).expect("u16");
    assert!(p.evaluate(&Value::U16(65_500)));

    let p = Predicate::with_str(Operation::Is, "4000000000", ValueKind::U32, false).expect("u32");
    assert!(p.evaluate(&Value::U32(4_000_000_000)));

    let p = Predicate::with_str(Operation::Is, "17000000000000000000", ValueKind::U64, false).expect("u64");
    assert!(p.evaluate(&Value::U64(17_000_000_000_000_000_000)));
}

#[test]
fn predicate_with_str_floats_is() {
    let p = Predicate::with_str(Operation::Is, "-1.25", ValueKind::F32, false).expect("f32");
    assert!(p.evaluate(&Value::F32(-1.25)));

    let p = Predicate::with_str(Operation::Is, "2.718281828", ValueKind::F64, false).expect("f64");
    assert!(p.evaluate(&Value::F64(2.718281828)));
}

#[test]
fn predicate_with_str_string_respects_ignore_case() {
    let p = Predicate::with_str(Operation::Is, "CamelCase", ValueKind::String, true).expect("string");
    assert!(p.evaluate(&Value::String("camelcase")));
    assert!(!p.evaluate(&Value::String("other")));
}

#[test]
fn predicate_with_str_path_is() {
    let p = Predicate::with_str(Operation::Is, "/var/data", ValueKind::Path, false).expect("path");
    assert!(p.evaluate(&Value::Path(b"/var/data")));
    assert!(!p.evaluate(&Value::Path(b"/tmp")));
}

#[test]
fn predicate_with_str_hashes_is() {
    let s128 = "abababababababababababababababab";
    let p = Predicate::with_str(Operation::Is, s128, ValueKind::Hash128, false).expect("h128");
    let h = Hash128::from_str(s128).unwrap();
    assert!(p.evaluate(&Value::Hash128(h.as_bytes())));

    let s160 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let p = Predicate::with_str(Operation::Is, s160, ValueKind::Hash160, false).expect("h160");
    let h = Hash160::from_str(s160).unwrap();
    assert!(p.evaluate(&Value::Hash160(h.as_bytes())));

    let s256 = "0123456789abcdef".repeat(4);
    let p = Predicate::with_str(Operation::Is, &s256, ValueKind::Hash256, false).expect("h256");
    let h = Hash256::from_str(&s256).unwrap();
    assert!(p.evaluate(&Value::Hash256(h.as_bytes())));
}

#[test]
fn predicate_with_str_ip_addr_datetime_bool() {
    let p = Predicate::with_str(Operation::Is, "10.20.30.40", ValueKind::IpAddr, false).expect("ip");
    let ip: IpAddr = "10.20.30.40".parse().unwrap();
    assert!(p.evaluate(&Value::IpAddr(ip)));

    let p = Predicate::with_str(Operation::Is, "2019-12-25", ValueKind::DateTime, false).expect("dt");
    let t: u64 = DateTime::from_repr("2019-12-25").unwrap().into();
    assert!(p.evaluate(&Value::DateTime(t)));

    let p = Predicate::with_str(Operation::Is, "false", ValueKind::Bool, false).expect("bool");
    assert!(p.evaluate(&Value::Bool(false)));
    assert!(!p.evaluate(&Value::Bool(true)));
}

#[test]
fn predicate_with_str_is_not_string() {
    let p = Predicate::with_str(Operation::IsNot, "x", ValueKind::String, false).expect("predicate");
    assert!(!p.evaluate(&Value::String("x")));
    assert!(p.evaluate(&Value::String("y")));
}

#[test]
fn predicate_with_str_parse_failure() {
    match Predicate::with_str(Operation::Is, "not-a-number", ValueKind::I32, false) {
        Err(Error::FailToParseValue(s, k)) => {
            assert_eq!(s, "not-a-number");
            assert_eq!(k, ValueKind::I32);
        }
        Ok(_) => panic!("expected parse error"),
        Err(e) => panic!("unexpected error: {e:?}"),
    }
}

#[test]
fn predicate_with_str_list_string_is_one_of() {
    let p = Predicate::with_str_list(Operation::IsOneOf, &["red", "green"], ValueKind::String, false)
        .expect("predicate");
    assert!(p.evaluate(&Value::String("green")));
    assert!(!p.evaluate(&Value::String("blue")));
}

#[test]
fn predicate_with_str_list_string_ignore_case() {
    let p = Predicate::with_str_list(Operation::IsOneOf, &["Red", "BLUE"], ValueKind::String, true)
        .expect("predicate");
    assert!(p.evaluate(&Value::String("blue")));
}

#[test]
fn predicate_with_str_list_path_is_one_of() {
    let p = Predicate::with_str_list(Operation::IsOneOf, &["/a", "/b/c"], ValueKind::Path, false)
        .expect("predicate");
    assert!(p.evaluate(&Value::Path(b"/b/c")));
    assert!(!p.evaluate(&Value::Path(b"/z")));
}

#[test]
fn predicate_with_str_list_unsigned_integers_is_one_of() {
    let p = Predicate::with_str_list(Operation::IsOneOf, &["1", "3", "5"], ValueKind::U8, false).expect("u8");
    assert!(p.evaluate(&Value::U8(3)));
    assert!(!p.evaluate(&Value::U8(2)));

    let p = Predicate::with_str_list(Operation::IsOneOf, &["1000", "2000", "3000"], ValueKind::U16, false)
        .expect("u16");
    assert!(p.evaluate(&Value::U16(2000)));
    assert!(!p.evaluate(&Value::U16(1500)));

    let p = Predicate::with_str_list(Operation::IsOneOf, &["100", "200", "300"], ValueKind::U32, false)
        .expect("u32");
    assert!(p.evaluate(&Value::U32(200)));
    assert!(!p.evaluate(&Value::U32(150)));

    let p = Predicate::with_str_list(
        Operation::IsOneOf,
        &["10000000000000000000", "11000000000000000000"],
        ValueKind::U64,
        false,
    )
    .expect("u64");
    assert!(p.evaluate(&Value::U64(11_000_000_000_000_000_000)));
    assert!(!p.evaluate(&Value::U64(10_500_000_000_000_000_000)));
}

#[test]
fn predicate_with_str_list_signed_integers_is_one_of() {
    let p = Predicate::with_str_list(Operation::IsOneOf, &["-5", "0", "7"], ValueKind::I8, false).expect("i8");
    assert!(p.evaluate(&Value::I8(-5)));
    assert!(!p.evaluate(&Value::I8(3)));

    let p = Predicate::with_str_list(Operation::IsOneOf, &["-30000", "0", "1000"], ValueKind::I16, false)
        .expect("i16");
    assert!(p.evaluate(&Value::I16(-30_000)));
    assert!(!p.evaluate(&Value::I16(-29_000)));

    let p = Predicate::with_str_list(Operation::IsOneOf, &["-1", "0", "99"], ValueKind::I32, false)
        .expect("i32");
    assert!(p.evaluate(&Value::I32(99)));
    assert!(!p.evaluate(&Value::I32(50)));

    let p = Predicate::with_str_list(
        Operation::IsOneOf,
        &["-9000000000000000000", "0", "1"],
        ValueKind::I64,
        false,
    )
    .expect("i64");
    assert!(p.evaluate(&Value::I64(-9_000_000_000_000_000_000)));
    assert!(!p.evaluate(&Value::I64(-8_000_000_000_000_000_000)));
}

#[test]
fn predicate_with_str_list_unsigned_integers_in_range() {
    let p = Predicate::with_str_list(Operation::InRange, &["10", "100"], ValueKind::U8, false).expect("u8");
    assert!(p.evaluate(&Value::U8(50)));
    assert!(!p.evaluate(&Value::U8(5)));

    let p = Predicate::with_str_list(Operation::InRange, &["1000", "9000"], ValueKind::U16, false).expect("u16");
    assert!(p.evaluate(&Value::U16(5000)));
    assert!(!p.evaluate(&Value::U16(500)));

    let p = Predicate::with_str_list(Operation::InRange, &["100000", "200000"], ValueKind::U32, false)
        .expect("u32");
    assert!(p.evaluate(&Value::U32(150_000)));
    assert!(!p.evaluate(&Value::U32(50_000)));

    let p = Predicate::with_str_list(
        Operation::InRange,
        &["1000000000000000000", "15000000000000000000"],
        ValueKind::U64,
        false,
    )
    .expect("u64");
    assert!(p.evaluate(&Value::U64(10_000_000_000_000_000_000)));
    assert!(!p.evaluate(&Value::U64(500_000_000_000_000_000)));
}

#[test]
fn predicate_with_str_list_signed_integers_in_range() {
    let p = Predicate::with_str_list(Operation::InRange, &["-50", "50"], ValueKind::I8, false).expect("i8");
    assert!(p.evaluate(&Value::I8(0)));
    assert!(!p.evaluate(&Value::I8(-100)));

    let p = Predicate::with_str_list(Operation::InRange, &["-20000", "20000"], ValueKind::I16, false)
        .expect("i16");
    assert!(p.evaluate(&Value::I16(0)));
    assert!(!p.evaluate(&Value::I16(30_000)));

    let p = Predicate::with_str_list(Operation::InRange, &["-1000000", "1000000"], ValueKind::I32, false)
        .expect("i32");
    assert!(p.evaluate(&Value::I32(500_000)));
    assert!(!p.evaluate(&Value::I32(2_000_000)));

    let p = Predicate::with_str_list(
        Operation::InRange,
        &["-5000000000000000000", "5000000000000000000"],
        ValueKind::I64,
        false,
    )
    .expect("i64");
    assert!(p.evaluate(&Value::I64(0)));
    assert!(!p.evaluate(&Value::I64(9_000_000_000_000_000_000)));
}

#[test]
fn predicate_with_str_list_float_in_range() {
    let p = Predicate::with_str_list(Operation::InRange, &["0.25", "4.0"], ValueKind::F32, false).expect("f32");
    assert!(p.evaluate(&Value::F32(1.5)));
    assert!(!p.evaluate(&Value::F32(0.1)));

    let p = Predicate::with_str_list(Operation::InRange, &["1.5", "10.5"], ValueKind::F64, false)
        .expect("f64");
    assert!(p.evaluate(&Value::F64(5.0)));
    assert!(!p.evaluate(&Value::F64(0.0)));
}

#[test]
fn predicate_with_str_list_hash_is_one_of() {
    let a = "00000000000000000000000000000000";
    let b = "ffffffffffffffffffffffffffffffff";
    let p = Predicate::with_str_list(Operation::IsOneOf, &[a, b], ValueKind::Hash128, false).expect("predicate");
    let hb = Hash128::from_str(b).unwrap();
    assert!(p.evaluate(&Value::Hash128(hb.as_bytes())));
    let other = Hash128::new([1u8; 16]);
    assert!(!p.evaluate(&Value::Hash128(other.as_bytes())));
}

#[test]
fn predicate_with_str_list_hash160_and_hash256_is_one_of() {
    let z40 = "0".repeat(40);
    let f40 = "f".repeat(40);
    let p = Predicate::with_str_list(Operation::IsOneOf, &[z40.as_str(), f40.as_str()], ValueKind::Hash160, false)
        .expect("hash160");
    let hf = Hash160::from_str(&f40).unwrap();
    assert!(p.evaluate(&Value::Hash160(hf.as_bytes())));
    assert!(!p.evaluate(&Value::Hash160(Hash160::new([1u8; 20]).as_bytes())));

    let z64 = "0".repeat(64);
    let a64 = "a".repeat(64);
    let p = Predicate::with_str_list(Operation::IsOneOf, &[z64.as_str(), a64.as_str()], ValueKind::Hash256, false)
        .expect("hash256");
    let ha = Hash256::from_str(&a64).unwrap();
    assert!(p.evaluate(&Value::Hash256(ha.as_bytes())));
    assert!(!p.evaluate(&Value::Hash256(Hash256::new([2u8; 32]).as_bytes())));
}

#[test]
fn predicate_with_str_list_ip_is_one_of() {
    let p = Predicate::with_str_list(
        Operation::IsOneOf,
        &["10.0.0.1", "10.0.0.2", "192.168.0.1"],
        ValueKind::IpAddr,
        false,
    )
    .expect("predicate");
    assert!(p.evaluate(&Value::IpAddr("192.168.0.1".parse().unwrap())));
    assert!(!p.evaluate(&Value::IpAddr("10.0.0.9".parse().unwrap())));
}

#[test]
fn predicate_with_str_list_ip_in_range() {
    let p = Predicate::with_str_list(Operation::InRange, &["127.0.0.1", "127.0.0.10"], ValueKind::IpAddr, false)
        .expect("predicate");
    assert!(p.evaluate(&Value::IpAddr("127.0.0.5".parse().unwrap())));
    assert!(!p.evaluate(&Value::IpAddr("127.0.0.0".parse().unwrap())));
}

#[test]
fn predicate_with_str_list_datetime_in_range() {
    let p = Predicate::with_str_list(
        Operation::InRange,
        &["2020-06-10", "2020-06-20"],
        ValueKind::DateTime,
        false,
    )
    .expect("predicate");
    let mid: u64 = DateTime::from_repr("2020-06-15").unwrap().into();
    assert!(p.evaluate(&Value::DateTime(mid)));
    let before: u64 = DateTime::from_repr("2020-06-01").unwrap().into();
    assert!(!p.evaluate(&Value::DateTime(before)));
}

#[test]
fn predicate_with_str_list_bool_rejected() {
    match Predicate::with_str_list(Operation::IsOneOf, &["true"], ValueKind::Bool, false) {
        Err(e) => assert_eq!(
            e,
            Error::InvalidOperationForValue(Operation::IsOneOf, ValueKind::Bool)
        ),
        Ok(_) => panic!("expected invalid operation for bool str list"),
    }
}

#[test]
fn value_kind_matches_variant_for_each_value() {
    assert_eq!(Value::String("x").kind(), ValueKind::String);
    assert_eq!(Value::Path(b"p").kind(), ValueKind::Path);
    assert_eq!(Value::Bytes(b"b").kind(), ValueKind::Bytes);
    assert_eq!(Value::U8(1).kind(), ValueKind::U8);
    assert_eq!(Value::U16(2).kind(), ValueKind::U16);
    assert_eq!(Value::U32(3).kind(), ValueKind::U32);
    assert_eq!(Value::U64(4).kind(), ValueKind::U64);
    assert_eq!(Value::I8(-1).kind(), ValueKind::I8);
    assert_eq!(Value::I16(-2).kind(), ValueKind::I16);
    assert_eq!(Value::I32(-3).kind(), ValueKind::I32);
    assert_eq!(Value::I64(-4).kind(), ValueKind::I64);
    assert_eq!(Value::F32(1.0).kind(), ValueKind::F32);
    assert_eq!(Value::F64(2.0).kind(), ValueKind::F64);
    let h16 = [0u8; 16];
    assert_eq!(Value::Hash128(&h16).kind(), ValueKind::Hash128);
    let h20 = [0u8; 20];
    assert_eq!(Value::Hash160(&h20).kind(), ValueKind::Hash160);
    let h32 = [0u8; 32];
    assert_eq!(Value::Hash256(&h32).kind(), ValueKind::Hash256);
    let ip: IpAddr = [1, 2, 3, 4].into();
    assert_eq!(Value::IpAddr(ip).kind(), ValueKind::IpAddr);
    assert_eq!(Value::DateTime(99).kind(), ValueKind::DateTime);
    assert_eq!(Value::Bool(true).kind(), ValueKind::Bool);
    assert_eq!(Value::None.kind(), ValueKind::None);
}

#[test]
fn value_kind_default_value_has_matching_kind() {
    let kinds = [
        ValueKind::String,
        ValueKind::Path,
        ValueKind::Bytes,
        ValueKind::U8,
        ValueKind::U16,
        ValueKind::U32,
        ValueKind::U64,
        ValueKind::I8,
        ValueKind::I16,
        ValueKind::I32,
        ValueKind::I64,
        ValueKind::F32,
        ValueKind::F64,
        ValueKind::Hash128,
        ValueKind::Hash160,
        ValueKind::Hash256,
        ValueKind::IpAddr,
        ValueKind::DateTime,
        ValueKind::Bool,
        ValueKind::None,
    ];
    for k in kinds {
        assert_eq!(k._default_value().kind(), k, "{k:?}");
    }
}

#[test]
fn attribute_index_new_and_accessor() {
    let idx = AttributeIndex::new(4242);
    assert_eq!(idx.index(), 4242);
}

#[test]
fn value_from_borrowed_str() {
    let s = "hello";
    let v: Value = s.into();
    assert_eq!(v.kind(), ValueKind::String);
    assert_eq!(<&str>::try_from(v).unwrap(), "hello");
}

#[test]
fn try_from_value_to_str_only_string_succeeds() {
    assert_eq!(<&str>::try_from(Value::String("ok")).unwrap(), "ok");
    match <&str>::try_from(Value::Path(b"x")) {
        Err(Error::ExpectingADifferentValueKind(got, expected)) => {
            assert_eq!(got, ValueKind::Path);
            assert_eq!(expected, ValueKind::String);
        }
        Ok(_) => panic!("expected error"),
        Err(e) => panic!("unexpected error: {e:?}"),
    }
}

#[test]
fn try_from_value_to_bytes_slice_only_path_succeeds() {
    assert_eq!(<&[u8]>::try_from(Value::Path(b"abc")).unwrap(), b"abc" as &[u8]);
    match <&[u8]>::try_from(Value::Bytes(b"abc")) {
        Err(Error::ExpectingADifferentValueKind(got, expected)) => {
            assert_eq!(got, ValueKind::Bytes);
            assert_eq!(expected, ValueKind::Path);
        }
        Ok(_) => panic!("expected error"),
        Err(e) => panic!("unexpected error: {e:?}"),
    }
}

#[test]
fn value_clone_roundtrip_string() {
    let v = Value::String("clone-me");
    let v2 = v.clone();
    assert_eq!(<&str>::try_from(v2).unwrap(), "clone-me");
}
