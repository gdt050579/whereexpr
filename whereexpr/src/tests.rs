use crate::AttributeIndex;
use crate::CompiledCondition;
use crate::ConditionList;
use crate::Error;
use crate::Operation;
use crate::Predicate;
use crate::expression::{Composition, EvaluationNode};

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
