use std::convert::TryFrom;
use std::net::IpAddr;
use std::str::FromStr;

use crate::AttributeIndex;
use crate::Attributes;
use crate::CompiledCondition;
use crate::Condition;
use crate::ConditionList;
use crate::Error;
use crate::Expression;
use crate::ExpressionBuilder;
use crate::Operation;
use crate::Predicate;
use crate::Value;
use crate::ValueKind;
use crate::RuleParser;
use crate::expression::{Composition, EvaluationNode};
use crate::types::{DateTime, FromRepr, Hash128, Hash160, Hash256};
use std::cell::Cell;

/// Minimal `Attributes` for `Condition::parse` / `ExpressionBuilder` tests.
#[derive(Debug)]
struct TestPerson {
    age: u32,
    label: String,
}

impl TestPerson {
    const AGE: AttributeIndex = AttributeIndex::new(0);
    const LABEL: AttributeIndex = AttributeIndex::new(1);
}

impl Attributes for TestPerson {
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx.index() {
            0 => Some(Value::U32(self.age)),
            1 => Some(Value::String(self.label.as_str())),
            _ => None,
        }
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx.index() {
            0 => Some(ValueKind::U32),
            1 => Some(ValueKind::String),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "age" => Some(Self::AGE),
            "label" => Some(Self::LABEL),
            _ => None,
        }
    }
    const TYPE_ID: u64 = 1;
    const TYPE_NAME: &'static str = "TestPerson";
}

/// Second `Attributes` type for `Expression::matches` / `evaluate` type-id checks.
#[derive(Debug)]
struct OtherPerson;

impl Attributes for OtherPerson {
    const TYPE_ID: u64 = 2;
    const TYPE_NAME: &'static str = "OtherPerson";
    fn get(&self, _idx: AttributeIndex) -> Option<Value<'_>> {
        None
    }

    fn kind(_idx: AttributeIndex) -> Option<ValueKind> {
        None
    }

    fn index(_name: &str) -> Option<AttributeIndex> {
        None
    }
}

/// `Attributes` type exposing both a `String` and a `Path` attribute, used by the
/// `re-match` tests to check that the operation is wired into both predicate families.
#[derive(Debug)]
struct TestFile {
    name: String,
    path: String,
}

impl TestFile {
    const NAME: AttributeIndex = AttributeIndex::new(0);
    const PATH: AttributeIndex = AttributeIndex::new(1);
}

impl Attributes for TestFile {
    const TYPE_ID: u64 = 4;
    const TYPE_NAME: &'static str = "TestFile";

    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::NAME => Some(Value::String(self.name.as_str())),
            Self::PATH => Some(Value::Path(self.path.as_str())),
            _ => None,
        }
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::NAME => Some(ValueKind::String),
            Self::PATH => Some(ValueKind::Path),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "name" => Some(Self::NAME),
            "path" => Some(Self::PATH),
            _ => None,
        }
    }
}

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

#[derive(Debug)]
struct ShortcircuitTest {
    op1: bool,
    op2: bool,
    op3: bool, 
    visited1: Cell<bool>,
    visited2: Cell<bool>,
    visited3: Cell<bool>
}

impl ShortcircuitTest {
    const OP1: AttributeIndex = AttributeIndex::new(0);
    const OP2: AttributeIndex = AttributeIndex::new(1);
    const OP3: AttributeIndex = AttributeIndex::new(2);
}

impl Attributes for ShortcircuitTest {
    const TYPE_ID: u64 = 3;
    const TYPE_NAME: &'static str = "ShortcircuitTest";

    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::OP1 => {
                self.visited1.set(true);
                Some(Value::Bool(self.op1))
            },
            Self::OP2 => {
                self.visited2.set(true);
                Some(Value::Bool(self.op2))
            },
            Self::OP3 => {
                self.visited3.set(true);
                Some(Value::Bool(self.op3))
            },
            _ => None,
        }
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::OP1 => Some(ValueKind::Bool),
            Self::OP2 => Some(ValueKind::Bool),
            Self::OP3 => Some(ValueKind::Bool),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "op1" => Some(Self::OP1),
            "op2" => Some(Self::OP2),
            "op3" => Some(Self::OP3),
            _ => None,
        }
    }
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
    assert_eq!(list.index_of("alpha"), Some(0));
    assert!(list.get(0).is_some());
}

#[test]
fn add_duplicate_name_returns_false() {
    let mut list = ConditionList::new();
    assert!(list.add("x", sample_condition()));
    assert!(!list.add("x", sample_condition()));
    assert_eq!(list.len(), 1);
    assert_eq!(list.index_of("x"), Some(0));
}

#[test]
fn names_are_case_insensitive_for_identity() {
    let mut list = ConditionList::new();
    assert!(list.add("Status", sample_condition()));
    assert!(!list.add("status", sample_condition()));
    assert_eq!(list.len(), 1);
    assert_eq!(list.index_of("STATUS"), Some(0));
}

#[test]
fn from_name_unknown_returns_none() {
    let list = ConditionList::new();
    assert_eq!(list.index_of("missing"), None);
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
    assert_eq!(list.index_of("first"), Some(0));
    assert_eq!(list.index_of("second"), Some(1));
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
                list.index_of(&prev),
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
        assert_eq!(list.index_of(&name), Some(i as u16), "lookup {name}");
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

/// Keep in sync with `cond_parser::tests::OPERATION_ALIAS_CASES` and `cond_parser::operation::OPERATIONS`.
const OPERATION_PARSE_STR_CASES: &[(&str, Operation)] = &[
    ("is", Operation::Is),
    ("==", Operation::Is),
    ("eq", Operation::Is),
    ("equals", Operation::Is),
    ("isnot", Operation::IsNot),
    ("is-not", Operation::IsNot),
    ("!=", Operation::IsNot),
    ("neq", Operation::IsNot),
    ("notequals", Operation::IsNot),
    ("isoneof", Operation::IsOneOf),
    ("is-one-of", Operation::IsOneOf),
    ("in", Operation::IsOneOf),
    ("isnotoneof", Operation::IsNotOneOf),
    ("is-not-one-of", Operation::IsNotOneOf),
    ("notin", Operation::IsNotOneOf),
    ("not-in", Operation::IsNotOneOf),
    ("startswith", Operation::StartsWith),
    ("starts-with", Operation::StartsWith),
    ("not-starts-with", Operation::NotStartsWith),
    ("notstartswith", Operation::NotStartsWith),
    ("startswithoneof", Operation::StartsWithOneOf),
    ("notstartswithoneof", Operation::NotStartsWithOneOf),
    ("not-starts-with-one-of", Operation::NotStartsWithOneOf),
    ("endswith", Operation::EndsWith),
    ("ends-with", Operation::EndsWith),
    ("notendswith", Operation::NotEndsWith),
    ("not-ends-with", Operation::NotEndsWith),
    ("endswithoneof", Operation::EndsWithOneOf),
    ("ends-with-one-of", Operation::EndsWithOneOf),
    ("notendswithoneof", Operation::NotEndsWithOneOf),
    ("contains", Operation::Contains),
    ("notcontains", Operation::NotContains),
    ("not-contains", Operation::NotContains),
    ("containsoneof", Operation::ContainsOneOf),
    ("contains-one-of", Operation::ContainsOneOf),
    ("notcontainsoneof", Operation::NotContainsOneOf),
    ("not-contains-one-of", Operation::NotContainsOneOf),
    ("glob", Operation::GlobREMatch),
    ("globmatch", Operation::GlobREMatch),
    ("notglob", Operation::NotGlobREMatch),
    ("notglobmatch", Operation::NotGlobREMatch),
    ("rematch", Operation::ReMatch),
    ("re-match", Operation::ReMatch),
    ("re_match", Operation::ReMatch),
    ("notrematch", Operation::NotReMatch),
    ("not-re-match", Operation::NotReMatch),
    (">", Operation::GreaterThan),
    ("gt", Operation::GreaterThan),
    ("greaterthan", Operation::GreaterThan),
    ("greater-than", Operation::GreaterThan),
    (">=", Operation::GreaterThanOrEqual),
    ("gte", Operation::GreaterThanOrEqual),
    ("greaterthanorequal", Operation::GreaterThanOrEqual),
    ("greater-than-or-equal", Operation::GreaterThanOrEqual),
    ("<", Operation::LessThan),
    ("lt", Operation::LessThan),
    ("lessthan", Operation::LessThan),
    ("<=", Operation::LessThanOrEqual),
    ("lte", Operation::LessThanOrEqual),
    ("lessthanorequal", Operation::LessThanOrEqual),
    ("less-than-or-equal", Operation::LessThanOrEqual),
    ("inrange", Operation::InRange),
    ("in-range", Operation::InRange),
    ("notinrange", Operation::NotInRange),
    ("not-in-range", Operation::NotInRange),
];

#[test]
fn operation_parse_str_matches_from_str_for_each_alias() {
    for &(alias, expected) in OPERATION_PARSE_STR_CASES {
        let via_parse_str = Operation::parse_str(alias)
            .unwrap_or_else(|| panic!("parse_str({alias:?}) should be Some"));
        assert_eq!(via_parse_str, expected, "parse_str({alias:?})");

        let via_from_str = Operation::from_str(alias)
            .unwrap_or_else(|e| panic!("from_str({alias:?}) should be Ok: {e:?}"));
        assert_eq!(via_from_str, expected, "from_str({alias:?})");

        let via_parse: Operation = alias
            .parse()
            .unwrap_or_else(|e| panic!("str::parse({alias:?}) should be Ok: {e:?}"));
        assert_eq!(via_parse, expected, "parse({alias:?})");
    }
}

#[test]
fn operation_parse_str_returns_none_for_unknown_token() {
    assert_eq!(Operation::parse_str("totally-unknown-op"), None);
    assert_eq!(Operation::parse_str(""), None);
}

#[test]
fn operation_from_str_returns_unknown_operation_error() {
    match Operation::from_str("not-an-operation") {
        Err(Error::UnknownOperation(start, end, expr)) => {
            assert_eq!(expr, "not-an-operation");
            assert!(start < end);
            assert_eq!(&expr[start as usize..end as usize], "not-an-operation");
        }
        Ok(op) => panic!("expected error, got {op:?}"),
        Err(e) => panic!("unexpected error: {e:?}"),
    }
}

#[test]
fn operation_parse_str_trims_leading_whitespace() {
    assert_eq!(Operation::parse_str("  \tin\n"), Some(Operation::IsOneOf));
    assert_eq!(
        Operation::from_str("  \tgreaterthan").unwrap(),
        Operation::GreaterThan
    );
}

#[test]
fn operation_parse_str_hyphens_and_underscores_normalized_like_cond_parser() {
    assert_eq!(Operation::parse_str("not-in"), Some(Operation::IsNotOneOf));
    assert_eq!(
        Operation::parse_str("greater_than").unwrap(),
        Operation::GreaterThan
    );
    assert_eq!(
        Operation::parse_str("LessThan-Or-Equal").unwrap(),
        Operation::LessThanOrEqual
    );
}

/// Canonical spellings accepted by `ValueKind::parse_str` (see `value.rs`).
const VALUE_KIND_PARSE_STR_CASES: &[(&str, ValueKind)] = &[
    ("u8", ValueKind::U8),
    ("i8", ValueKind::I8),
    ("u16", ValueKind::U16),
    ("u32", ValueKind::U32),
    ("u64", ValueKind::U64),
    ("i16", ValueKind::I16),
    ("i32", ValueKind::I32),
    ("i64", ValueKind::I64),
    ("f32", ValueKind::F32),
    ("f64", ValueKind::F64),
    ("bool", ValueKind::Bool),
    ("none", ValueKind::Unknown),
    ("path", ValueKind::Path),
    ("string", ValueKind::String),
    ("bytes", ValueKind::Bytes),
    ("hash128", ValueKind::Hash128),
    ("hash160", ValueKind::Hash160),
    ("hash256", ValueKind::Hash256),
    ("ip", ValueKind::IpAddr),
    ("ipaddr", ValueKind::IpAddr),
    ("datetim", ValueKind::DateTime),
    ("datetime", ValueKind::DateTime),
];

#[test]
fn value_kind_parse_str_matches_from_str_for_each_canonical_name() {
    for &(name, expected) in VALUE_KIND_PARSE_STR_CASES {
        assert_eq!(
            ValueKind::parse_str(name),
            Some(expected),
            "parse_str({name:?})"
        );

        let via_from_str = ValueKind::from_str(name)
            .unwrap_or_else(|e| panic!("from_str({name:?}) should be Ok: {e:?}"));
        assert_eq!(via_from_str, expected, "from_str({name:?})");

        let via_parse: ValueKind = name
            .parse()
            .unwrap_or_else(|e| panic!("str::parse({name:?}) should be Ok: {e:?}"));
        assert_eq!(via_parse, expected, "parse({name:?})");
    }
}

#[test]
fn value_kind_parse_str_is_ascii_case_insensitive_for_letters() {
    assert_eq!(ValueKind::parse_str("U8"), Some(ValueKind::U8));
    assert_eq!(ValueKind::parse_str("Hash128"), Some(ValueKind::Hash128));
    assert_eq!(ValueKind::parse_str("STRING"), Some(ValueKind::String));
    assert_eq!(ValueKind::parse_str("DateTime"), Some(ValueKind::DateTime));
}

#[test]
fn value_kind_parse_str_two_char_ip_requires_lowercase_p() {
    assert_eq!(ValueKind::parse_str("ip"), Some(ValueKind::IpAddr));
    assert_eq!(ValueKind::parse_str("Ip"), Some(ValueKind::IpAddr));
    assert_eq!(ValueKind::parse_str("IP"), None);
}

#[test]
fn value_kind_parse_str_returns_none_for_unknown_or_whitespace_padded() {
    assert_eq!(ValueKind::parse_str("not-a-kind"), None);
    assert_eq!(ValueKind::parse_str("u16 "), None);
    assert_eq!(ValueKind::parse_str(" u8"), None);
}

#[test]
fn value_kind_from_str_returns_unknown_value_kind_error() {
    match ValueKind::from_str("unknownKind") {
        Err(Error::UnknownValueKind(start, end, expr)) => {
            assert_eq!((start, end), (0, expr.len() as u32));
            assert_eq!(expr, "unknownKind");
        }
        Ok(k) => panic!("expected error, got {k:?}"),
        Err(e) => panic!("unexpected error: {e:?}"),
    }
}

#[test]
fn condition_new_builds_with_named_attribute() {
    let pred = Predicate::with_value(Operation::Is, 40u32).expect("predicate");
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "rule1",
            Condition::new("age", pred),
        )
        .build("rule1")
        .expect("build");
    let person = TestPerson {
        age: 40,
        label: "x".into(),
    };
    assert!(ex.matches(&person));
    let person_no = TestPerson {
        age: 41,
        label: "x".into(),
    };
    assert!(!ex.matches(&person_no));
}

#[test]
fn condition_with_index_skips_name_resolution() {
    let pred = Predicate::with_value(Operation::GreaterThan, 10u32).expect("predicate");
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add("rule1", Condition::with_index(TestPerson::AGE, pred))
        .build("rule1")
        .expect("build");
    assert!(ex.matches(&TestPerson {
        age: 15,
        label: "".into(),
    }));
}

#[test]
fn condition_from_str_defers_parse_until_build() {
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add("rule1", Condition::from_str("age is 7"))
        .build("rule1")
        .expect("build");
    assert!(ex.matches(&TestPerson {
        age: 7,
        label: "".into(),
    }));
}

#[test]
fn condition_try_new_propagates_predicate_error_on_build() {
    let res = ExpressionBuilder::<TestPerson>::new()
        .add(
            "rule1",
            Condition::try_new("age", Err(Error::EmptyExpression)),
        )
        .build("rule1");
    match res {
        Err(e) => assert_eq!(e, Error::EmptyExpression),
        Ok(_) => panic!("expected build to fail"),
    }
}

#[test]
fn condition_try_with_index_propagates_predicate_error_on_build() {
    let res = ExpressionBuilder::<TestPerson>::new()
        .add(
            "rule1",
            Condition::try_with_index(TestPerson::AGE, Err(Error::EmptyCondition)),
        )
        .build("rule1");
    match res {
        Err(e) => assert_eq!(e, Error::EmptyCondition),
        Ok(_) => panic!("expected build to fail"),
    }
}

#[test]
fn condition_try_new_ok_builds_like_new() {
    let pred = Predicate::with_value(Operation::Is, 1u32).expect("predicate");
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add("rule1", Condition::try_new("age", Ok(pred)))
        .build("rule1")
        .expect("build");
    assert!(ex.matches(&TestPerson {
        age: 1,
        label: "".into(),
    }));
}

#[test]
fn condition_try_with_index_ok_builds() {
    let pred = Predicate::with_value(Operation::Is, 2u32).expect("predicate");
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "rule1",
            Condition::try_with_index(TestPerson::AGE, Ok(pred)),
        )
        .build("rule1")
        .expect("build");
    assert!(ex.matches(&TestPerson {
        age: 2,
        label: "".into(),
    }));
}

#[test]
fn condition_parse_list_is_one_of_string_with_ignore_case_modifier() {
    let (_idx, pred) = Condition::parse::<TestPerson>(
        "label is-one-of [X, y] {ignore-case}",
        "mycond",
    )
    .expect("parse");
    assert!(pred.evaluate(&Value::String("x")).unwrap());
    assert!(pred.evaluate(&Value::String("Y")).unwrap());
    assert!(!pred.evaluate(&Value::String("z")).unwrap());
}

#[test]
fn condition_parse_expecting_value_when_missing() {
    match Condition::parse::<TestPerson>("age is", "mycond") {
        Err(e) => assert!(matches!(e, Error::ExpectingAValue(_, _, _))),
        Ok(_) => panic!("expected parse error"),
    }
}

#[test]
fn condition_parse_single_value_u32() {
    let (idx, pred) =
        Condition::parse::<TestPerson>("age is 42", "mycond").expect("parse");
    assert_eq!(idx, TestPerson::AGE);
    assert!(pred.evaluate(&Value::U32(42)).unwrap());
    assert!(!pred.evaluate(&Value::U32(0)).unwrap());
}

#[test]
fn condition_parse_list_is_one_of_string() {
    let (_idx, pred) = Condition::parse::<TestPerson>(
        "label is-one-of [a, b, c]",
        "mycond",
    )
    .expect("parse");
    assert!(pred.evaluate(&Value::String("b")).unwrap());
    assert!(!pred.evaluate(&Value::String("z")).unwrap());
}

#[test]
fn condition_parse_u32_in_range() {
    let (_idx, pred) =
        Condition::parse::<TestPerson>("age inrange [10, 100]", "mycond").expect("parse");
    assert!(pred.evaluate(&Value::U32(50)).unwrap());
    assert!(!pred.evaluate(&Value::U32(5)).unwrap());
}

#[test]
fn condition_parse_unknown_attribute() {
    match Condition::parse::<TestPerson>("unknown is 1", "ruleA") {
        Err(e) => assert_eq!(
            e,
            Error::UnknownAttribute("unknown".into(), "ruleA".into())
        ),
        Ok(_) => panic!("expected parse error"),
    }
}

#[test]
fn condition_parse_empty_expression() {
    match Condition::parse::<TestPerson>("", "ruleA") {
        Err(e) => assert_eq!(e, Error::EmptyCondition),
        Ok(_) => panic!("expected parse error"),
    }
}

#[test]
fn condition_parse_invalid_attribute_name() {
    let input = "1bad is 1";
    match Condition::parse::<TestPerson>(input, "ruleA") {
        Err(e) => assert!(matches!(e, Error::InvalidAttributeName(_, _, _))),
        Ok(_) => panic!("expected parse error"),
    }
}

#[test]
fn condition_parse_unknown_operation() {
    match Condition::parse::<TestPerson>("age totallyUnknownOp 1", "ruleA") {
        Err(e) => assert!(matches!(e, Error::UnknownOperation(_, _, _))),
        Ok(_) => panic!("expected parse error"),
    }
}

#[test]
fn compiled_condition_evaluate_uses_object_field() {
    let pred = Predicate::with_value(Operation::Is, 99u32).expect("predicate");
    let cc = CompiledCondition::new(TestPerson::AGE, pred);
    let person = TestPerson {
        age: 99,
        label: "hi".into(),
    };
    assert!(cc.evaluate(&person).unwrap());
    let other = TestPerson {
        age: 1,
        label: "hi".into(),
    };
    assert!(!cc.evaluate(&other).unwrap());
}

#[test]
fn compiled_condition_evaluate_returns_none_when_attribute_missing() {
    let pred = Predicate::with_value(Operation::Is, 0u32).expect("predicate");
    let cc = CompiledCondition::new(AttributeIndex::new(99), pred);
    let person = TestPerson {
        age: 1,
        label: "x".into(),
    };
    assert!(cc.evaluate(&person).is_none());
}

#[test]
fn expression_builder_empty_returns_empty_condition_list_error() {
    match ExpressionBuilder::<TestPerson>::new().build("a") {
        Err(e) => assert_eq!(e, Error::EmptyConditionList),
        Ok(_) => panic!("expected build to fail"),
    }
}

#[test]
fn expression_builder_rejects_invalid_condition_name() {
    let pred = Predicate::with_value(Operation::Is, 1u32).expect("predicate");
    let res = ExpressionBuilder::<TestPerson>::new()
        .add("1bad", Condition::new("age", pred))
        .build("1bad");
    match res {
        Err(Error::InvalidConditionName(name)) => assert_eq!(name, "1bad"),
        Ok(_) => panic!("expected error"),
        Err(e) => panic!("unexpected error: {e:?}"),
    }
}

#[test]
fn expression_builder_rejects_duplicate_condition_name() {
    let p1 = Predicate::with_value(Operation::Is, 1u32).expect("predicate");
    let p2 = Predicate::with_value(Operation::Is, 2u32).expect("predicate");
    let res = ExpressionBuilder::<TestPerson>::new()
        .add("dup", Condition::new("age", p1))
        .add("dup", Condition::new("age", p2))
        .build("dup");
    match res {
        Err(Error::DuplicateConditionName(name)) => assert_eq!(name, "dup"),
        Ok(_) => panic!("expected error"),
        Err(e) => panic!("unexpected error: {e:?}"),
    }
}

#[cfg(feature = "enable_type_check")]
#[test]
#[should_panic(expected = "object type mismatch")]
fn expression_matches_panics_on_type_mismatch() {
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "rule1",
            Condition::new("age", Predicate::with_value(Operation::Is, 1u32).expect("predicate")),
        )
        .build("rule1")
        .expect("build");
    let _ = ex.matches(&OtherPerson);
}

#[test]
fn expression_evaluate_returns_none_when_rule_evaluates_to_none() {
    let pred = Predicate::with_value(Operation::Is, 1u32).expect("predicate");
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "rule1",
            Condition::with_index(AttributeIndex::new(99), pred),
        )
        .build("rule1")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 5,
            label: "x".into(),
        }),
        None
    );
}

#[test]
fn expression_evaluate_some_when_all_rules_evaluate() {
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "rule1",
            Condition::new("age", Predicate::with_value(Operation::Is, 10u32).expect("predicate")),
        )
        .build("rule1")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 10,
            label: "".into(),
        }),
        Some(true)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 11,
            label: "".into(),
        }),
        Some(false)
    );
}

#[test]
fn expression_and_group_evaluate_none_if_operand_evaluates_none() {
    let ok = Predicate::with_value(Operation::Is, 1u32).expect("predicate");
    let bad_idx = Predicate::with_value(Operation::Is, 0u32).expect("predicate");
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add("a", Condition::new("age", ok))
        .add("b", Condition::with_index(AttributeIndex::new(99), bad_idx))
        .build("a && b")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 1,
            label: "".into(),
        }),
        None
    );
}

#[test]
fn expression_or_group_evaluate_true_when_left_branch_matches() {
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "a",
            Condition::new("age", Predicate::with_value(Operation::Is, 5u32).expect("predicate")),
        )
        .add(
            "b",
            Condition::new("age", Predicate::with_value(Operation::Is, 9u32).expect("predicate")),
        )
        .build("a || b")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 5,
            label: "".into(),
        }),
        Some(true)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 9,
            label: "".into(),
        }),
        Some(true)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 0,
            label: "".into(),
        }),
        Some(false)
    );
}

#[test]
fn expression_or_group_evaluate_false_when_both_branches_false() {
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "a",
            Condition::new("age", Predicate::with_value(Operation::Is, 100u32).expect("predicate")),
        )
        .add(
            "b",
            Condition::new("age", Predicate::with_value(Operation::Is, 200u32).expect("predicate")),
        )
        .build("a || b")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 50,
            label: "".into(),
        }),
        Some(false)
    );
}

#[test]
fn expression_or_short_circuits_true_without_evaluating_later_operand() {
    let ok = Predicate::with_value(Operation::Is, 1u32).expect("predicate");
    let bad_idx = Predicate::with_value(Operation::Is, 0u32).expect("predicate");
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add("a", Condition::new("age", ok))
        .add("b", Condition::with_index(AttributeIndex::new(99), bad_idx))
        .build("a || b")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 1,
            label: "".into(),
        }),
        Some(true)
    );
}

#[test]
fn expression_or_group_evaluate_none_when_left_false_and_right_evaluates_none() {
    let left = Predicate::with_value(Operation::Is, 99u32).expect("predicate");
    let bad_idx = Predicate::with_value(Operation::Is, 0u32).expect("predicate");
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add("a", Condition::new("age", left))
        .add("b", Condition::with_index(AttributeIndex::new(99), bad_idx))
        .build("a || b")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 1,
            label: "".into(),
        }),
        None
    );
}

#[test]
fn expression_or_and_nested_evaluate() {
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "a",
            Condition::new("age", Predicate::with_value(Operation::Is, 1u32).expect("predicate")),
        )
        .add(
            "b",
            Condition::new("age", Predicate::with_value(Operation::Is, 2u32).expect("predicate")),
        )
        .add(
            "c",
            Condition::new(
                "label",
                Predicate::with_value(Operation::Is, "ok").expect("predicate"),
            ),
        )
        .build("(a || b) && c")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 2,
            label: "ok".into(),
        }),
        Some(true)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 2,
            label: "no".into(),
        }),
        Some(false)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 0,
            label: "ok".into(),
        }),
        Some(false)
    );
}

#[test]
fn expression_not_parenthesized_and_group_evaluate() {
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "a",
            Condition::new("age", Predicate::with_value(Operation::Is, 10u32).expect("predicate")),
        )
        .add(
            "b",
            Condition::new(
                "label",
                Predicate::with_value(Operation::Is, "x").expect("predicate"),
            ),
        )
        .build("NOT (a && b)")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 10,
            label: "x".into(),
        }),
        Some(false)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 10,
            label: "y".into(),
        }),
        Some(true)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 0,
            label: "x".into(),
        }),
        Some(true)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 0,
            label: "y".into(),
        }),
        Some(true)
    );
}

#[test]
fn expression_not_parenthesized_or_group_evaluate() {
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "a",
            Condition::new("age", Predicate::with_value(Operation::Is, 10u32).expect("predicate")),
        )
        .add(
            "b",
            Condition::new("age", Predicate::with_value(Operation::Is, 20u32).expect("predicate")),
        )
        .build("NOT (a || b)")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 10,
            label: "".into(),
        }),
        Some(false)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 20,
            label: "".into(),
        }),
        Some(false)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 15,
            label: "".into(),
        }),
        Some(true)
    );
}

#[test]
fn expression_and_with_negated_or_subgroup_evaluate() {
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add(
            "gate",
            Condition::new(
                "label",
                Predicate::with_value(Operation::Is, "ok").expect("predicate"),
            ),
        )
        .add(
            "low",
            Condition::new("age", Predicate::with_value(Operation::Is, 1u32).expect("predicate")),
        )
        .add(
            "high",
            Condition::new("age", Predicate::with_value(Operation::Is, 2u32).expect("predicate")),
        )
        .build("gate && NOT (low || high)")
        .expect("build");
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 0,
            label: "ok".into(),
        }),
        Some(true)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 1,
            label: "ok".into(),
        }),
        Some(false)
    );
    assert_eq!(
        ex.evaluate(&TestPerson {
            age: 0,
            label: "no".into(),
        }),
        Some(false)
    );
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
            children: vec![EvaluationNode::Condition(0), EvaluationNode::Condition(1)],
        }
    );
}

#[test]
fn expr_parse_not_rule() {
    let node = parse_expression("NOT a", &["a"]).expect("parse");
    assert_eq!(
        node,
        EvaluationNode::Not {
            child: Box::new(EvaluationNode::Condition(0)),
        }
    );
}

#[test]
fn expr_parse_not_of_parenthesized_and_group() {
    let node = parse_expression("NOT (a && b)", &["a", "b"]).expect("parse");
    assert_eq!(
        node,
        EvaluationNode::Not {
            child: Box::new(EvaluationNode::Group {
                composition: Composition::And,
                children: vec![EvaluationNode::Condition(0), EvaluationNode::Condition(1)],
            }),
        }
    );
}

#[test]
fn expr_parse_not_of_parenthesized_or_group() {
    let node = parse_expression("NOT (a || b)", &["a", "b"]).expect("parse");
    assert_eq!(
        node,
        EvaluationNode::Not {
            child: Box::new(EvaluationNode::Group {
                composition: Composition::Or,
                children: vec![EvaluationNode::Condition(0), EvaluationNode::Condition(1)],
            }),
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
            children: vec![
                EvaluationNode::Group {
                    composition: Composition::Or,
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
    assert!(matches!(err, Error::UnknownConditionName(_, _, _)));
}

#[test]
fn predicate_with_value_is_matches_field() {
    let p = Predicate::with_value(Operation::Is, 7u8).expect("predicate");
    assert!(p.evaluate(&Value::U8(7)).unwrap());
    assert!(!p.evaluate(&Value::U8(0)).unwrap());
}

#[test]
fn predicate_with_value_is_not_negates_match() {
    let p = Predicate::with_value(Operation::IsNot, 7u8).expect("predicate");
    assert!(!p.evaluate(&Value::U8(7)).unwrap());
    assert!(p.evaluate(&Value::U8(0)).unwrap());
}

#[test]
fn predicate_with_list_is_one_of() {
    let p = Predicate::with_list(Operation::IsOneOf, &[1u8, 3u8, 5u8]).expect("predicate");
    assert!(p.evaluate(&Value::U8(3)).unwrap());
    assert!(!p.evaluate(&Value::U8(2)).unwrap());
}

#[test]
fn predicate_with_value_signed_integers_is() {
    let p = Predicate::with_value(Operation::Is, -99i8).expect("i8");
    assert!(p.evaluate(&Value::I8(-99)).unwrap());
    assert!(!p.evaluate(&Value::I8(0)).unwrap());

    let p = Predicate::with_value(Operation::Is, -10_000i16).expect("i16");
    assert!(p.evaluate(&Value::I16(-10_000)).unwrap());
    assert!(!p.evaluate(&Value::I16(0)).unwrap());

    let p = Predicate::with_value(Operation::Is, -2_000_000_000i32).expect("i32");
    assert!(p.evaluate(&Value::I32(-2_000_000_000)).unwrap());
    assert!(!p.evaluate(&Value::I32(0)).unwrap());

    let p = Predicate::with_value(Operation::Is, -9_000_000_000_000_000_000i64).expect("i64");
    assert!(p.evaluate(&Value::I64(-9_000_000_000_000_000_000)).unwrap());
    assert!(!p.evaluate(&Value::I64(0)).unwrap());
}

#[test]
fn predicate_with_value_unsigned_integers_is() {
    let p = Predicate::with_value(Operation::Is, 40_000u16).expect("u16");
    assert!(p.evaluate(&Value::U16(40_000)).unwrap());
    assert!(!p.evaluate(&Value::U16(0)).unwrap());

    let p = Predicate::with_value(Operation::Is, 3_000_000_000u32).expect("u32");
    assert!(p.evaluate(&Value::U32(3_000_000_000)).unwrap());
    assert!(!p.evaluate(&Value::U32(0)).unwrap());

    let p = Predicate::with_value(Operation::Is, 18_000_000_000_000_000_000u64).expect("u64");
    assert!(p.evaluate(&Value::U64(18_000_000_000_000_000_000)).unwrap());
    assert!(!p.evaluate(&Value::U64(0)).unwrap());
}

#[test]
fn predicate_with_value_floats_is() {
    let p = Predicate::with_value(Operation::Is, 1.25f32).expect("f32");
    assert!(p.evaluate(&Value::F32(1.25)).unwrap());
    assert!(!p.evaluate(&Value::F32(0.0)).unwrap());

    let p = Predicate::with_value(Operation::Is, std::f64::consts::PI).expect("f64");
    assert!(p.evaluate(&Value::F64(std::f64::consts::PI)).unwrap());
    assert!(!p.evaluate(&Value::F64(0.0)).unwrap());
}

#[test]
fn predicate_with_value_string_is() {
    let p = Predicate::with_value(Operation::Is, "needle").expect("string");
    assert!(p.evaluate(&Value::String("needle")).unwrap());
    assert!(!p.evaluate(&Value::String("other")).unwrap());
}

#[test]
fn predicate_with_value_path_is() {
    let p = Predicate::with_value(Operation::Is, Value::Path("/var/log/app.log")).expect("path");
    assert!(p.evaluate(&Value::Path("/var/log/app.log")).unwrap());
    assert!(!p.evaluate(&Value::Path("/tmp/other.log")).unwrap());
}

#[test]
fn predicate_with_value_hashes_is() {
    let b128 = [0x11u8; 16];
    let h128 = Hash128::new(b128);
    let p = Predicate::with_value(Operation::Is, &h128).expect("hash128");
    assert!(p.evaluate(&Value::Hash128(&b128)).unwrap());
    assert!(!p.evaluate(&Value::Hash128(&[0u8; 16])).unwrap());

    let b160 = [0x22u8; 20];
    let h160 = Hash160::new(b160);
    let p = Predicate::with_value(Operation::Is, &h160).expect("hash160");
    assert!(p.evaluate(&Value::Hash160(&b160)).unwrap());
    assert!(!p.evaluate(&Value::Hash160(&[0u8; 20])).unwrap());

    let b256 = [0x33u8; 32];
    let h256 = Hash256::new(b256);
    let p = Predicate::with_value(Operation::Is, &h256).expect("hash256");
    assert!(p.evaluate(&Value::Hash256(&b256)).unwrap());
    assert!(!p.evaluate(&Value::Hash256(&[0u8; 32])).unwrap());
}

#[test]
fn predicate_with_value_ip_addr_is() {
    let ip = IpAddr::from([192, 168, 1, 10]);
    let p = Predicate::with_value(Operation::Is, ip).expect("ip");
    assert!(p.evaluate(&Value::IpAddr(ip)).unwrap());
    assert!(!p.evaluate(&Value::IpAddr(IpAddr::from([192, 168, 1, 11]))).unwrap());
}

#[test]
fn predicate_with_value_datetime_is() {
    let t: u64 = DateTime::from_repr("2021-06-01").unwrap().into();
    let p = Predicate::with_value(Operation::Is, Value::DateTime(t)).expect("datetime");
    assert!(p.evaluate(&Value::DateTime(t)).unwrap());
    let other: u64 = DateTime::from_repr("2021-06-02").unwrap().into();
    assert!(!p.evaluate(&Value::DateTime(other)).unwrap());
}

#[test]
fn predicate_with_value_bool_is() {
    let p = Predicate::with_value(Operation::Is, true).expect("bool");
    assert!(p.evaluate(&Value::Bool(true)).unwrap());
    assert!(!p.evaluate(&Value::Bool(false)).unwrap());
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
    assert!(p.evaluate(&Value::U8(3)).unwrap());
    assert!(!p.evaluate(&Value::U8(2)).unwrap());

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::U16(1000), Value::U16(2000), Value::U16(3000)],
    )
    .expect("u16");
    assert!(p.evaluate(&Value::U16(2000)).unwrap());
    assert!(!p.evaluate(&Value::U16(1500)).unwrap());

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::U32(100), Value::U32(200), Value::U32(300)],
    )
    .expect("u32");
    assert!(p.evaluate(&Value::U32(200)).unwrap());
    assert!(!p.evaluate(&Value::U32(150)).unwrap());

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[
            Value::U64(10_000_000_000_000_000_000),
            Value::U64(11_000_000_000_000_000_000),
        ],
    )
    .expect("u64");
    assert!(p.evaluate(&Value::U64(11_000_000_000_000_000_000)).unwrap());
    assert!(!p.evaluate(&Value::U64(10_500_000_000_000_000_000)).unwrap());
}

#[test]
fn predicate_with_value_list_signed_integers_is_one_of() {
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::I8(-5), Value::I8(0), Value::I8(7)],
    )
    .expect("i8");
    assert!(p.evaluate(&Value::I8(-5)).unwrap());
    assert!(!p.evaluate(&Value::I8(3)).unwrap());

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::I16(-30_000), Value::I16(0), Value::I16(1000)],
    )
    .expect("i16");
    assert!(p.evaluate(&Value::I16(-30_000)).unwrap());
    assert!(!p.evaluate(&Value::I16(-29_000)).unwrap());

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::I32(-1), Value::I32(0), Value::I32(99)],
    )
    .expect("i32");
    assert!(p.evaluate(&Value::I32(99)).unwrap());
    assert!(!p.evaluate(&Value::I32(50)).unwrap());

    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[
            Value::I64(-9_000_000_000_000_000_000),
            Value::I64(0),
            Value::I64(1),
        ],
    )
    .expect("i64");
    assert!(p.evaluate(&Value::I64(-9_000_000_000_000_000_000)).unwrap());
    assert!(!p.evaluate(&Value::I64(-8_000_000_000_000_000_000)).unwrap());
}

#[test]
fn predicate_with_value_list_unsigned_integers_in_range() {
    let p = Predicate::with_value_list(Operation::InRange, &[Value::U8(10), Value::U8(100)]).expect("u8");
    assert!(p.evaluate(&Value::U8(50)).unwrap());
    assert!(!p.evaluate(&Value::U8(5)).unwrap());

    let p = Predicate::with_value_list(Operation::InRange, &[Value::U16(1000), Value::U16(9000)]).expect("u16");
    assert!(p.evaluate(&Value::U16(5000)).unwrap());
    assert!(!p.evaluate(&Value::U16(500)).unwrap());

    let p = Predicate::with_value_list(
        Operation::InRange,
        &[Value::U32(100_000), Value::U32(200_000)],
    )
    .expect("u32");
    assert!(p.evaluate(&Value::U32(150_000)).unwrap());
    assert!(!p.evaluate(&Value::U32(50_000)).unwrap());

    let p = Predicate::with_value_list(
        Operation::InRange,
        &[
            Value::U64(1_000_000_000_000_000_000),
            Value::U64(15_000_000_000_000_000_000),
        ],
    )
    .expect("u64");
    assert!(p.evaluate(&Value::U64(10_000_000_000_000_000_000)).unwrap());
    assert!(!p.evaluate(&Value::U64(500_000_000_000_000_000)).unwrap());
}

#[test]
fn predicate_with_value_list_signed_integers_in_range() {
    let p = Predicate::with_value_list(Operation::InRange, &[Value::I8(-50), Value::I8(50)]).expect("i8");
    assert!(p.evaluate(&Value::I8(0)).unwrap());
    assert!(!p.evaluate(&Value::I8(-100)).unwrap());

    let p = Predicate::with_value_list(
        Operation::InRange,
        &[Value::I16(-20_000), Value::I16(20_000)],
    )
    .expect("i16");
    assert!(p.evaluate(&Value::I16(0)).unwrap());
    assert!(!p.evaluate(&Value::I16(30_000)).unwrap());

    let p = Predicate::with_value_list(
        Operation::InRange,
        &[Value::I32(-1_000_000), Value::I32(1_000_000)],
    )
    .expect("i32");
    assert!(p.evaluate(&Value::I32(500_000)).unwrap());
    assert!(!p.evaluate(&Value::I32(2_000_000)).unwrap());

    let p = Predicate::with_value_list(
        Operation::InRange,
        &[
            Value::I64(-5_000_000_000_000_000_000),
            Value::I64(5_000_000_000_000_000_000),
        ],
    )
    .expect("i64");
    assert!(p.evaluate(&Value::I64(0)).unwrap());
    assert!(!p.evaluate(&Value::I64(9_000_000_000_000_000_000)).unwrap());
}

#[test]
fn predicate_with_value_list_floats_in_range() {
    let p = Predicate::with_value_list(Operation::InRange, &[Value::F32(0.25), Value::F32(4.0)]).expect("f32");
    assert!(p.evaluate(&Value::F32(1.5)).unwrap());
    assert!(!p.evaluate(&Value::F32(0.1)).unwrap());

    let p = Predicate::with_value_list(Operation::InRange, &[Value::F64(1.5), Value::F64(10.5)]).expect("f64");
    assert!(p.evaluate(&Value::F64(5.0)).unwrap());
    assert!(!p.evaluate(&Value::F64(0.0)).unwrap());
}

#[test]
fn predicate_with_value_list_string_is_one_of() {
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::String("red"), Value::String("green")],
    )
    .expect("string");
    assert!(p.evaluate(&Value::String("green")).unwrap());
    assert!(!p.evaluate(&Value::String("blue")).unwrap());
}

#[test]
fn predicate_with_value_list_path_is_one_of() {
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::Path("/a"), Value::Path("/b/c")],
    )
    .expect("path");
    assert!(p.evaluate(&Value::Path("/b/c")).unwrap());
    assert!(!p.evaluate(&Value::Path("/z")).unwrap());
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
    assert!(p.evaluate(&Value::Hash128(&b128)).unwrap());
    assert!(!p.evaluate(&Value::Hash128(&[0u8; 16])).unwrap());

    let a160 = [0x11u8; 20];
    let b160 = [0x22u8; 20];
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::Hash160(&a160), Value::Hash160(&b160)],
    )
    .expect("hash160");
    assert!(p.evaluate(&Value::Hash160(&b160)).unwrap());
    assert!(!p.evaluate(&Value::Hash160(&[0u8; 20])).unwrap());

    let a256 = [0x33u8; 32];
    let b256 = [0x44u8; 32];
    let p = Predicate::with_value_list(
        Operation::IsOneOf,
        &[Value::Hash256(&a256), Value::Hash256(&b256)],
    )
    .expect("hash256");
    assert!(p.evaluate(&Value::Hash256(&b256)).unwrap());
    assert!(!p.evaluate(&Value::Hash256(&[0u8; 32])).unwrap());
}

#[test]
fn predicate_with_value_list_ip_is_one_of() {
    let ip1: IpAddr = "10.0.0.1".parse().unwrap();
    let ip2: IpAddr = "192.168.0.1".parse().unwrap();
    let p = Predicate::with_value_list(Operation::IsOneOf, &[Value::IpAddr(ip1), Value::IpAddr(ip2)])
        .expect("ip");
    assert!(p.evaluate(&Value::IpAddr(ip2)).unwrap());
    assert!(!p.evaluate(&Value::IpAddr("10.0.0.9".parse().unwrap())).unwrap());
}

#[test]
fn predicate_with_value_list_ip_in_range() {
    let lo: IpAddr = "127.0.0.1".parse().unwrap();
    let hi: IpAddr = "127.0.0.10".parse().unwrap();
    let p = Predicate::with_value_list(Operation::InRange, &[Value::IpAddr(lo), Value::IpAddr(hi)]).expect("ip");
    assert!(p.evaluate(&Value::IpAddr("127.0.0.5".parse().unwrap())).unwrap());
    assert!(!p.evaluate(&Value::IpAddr("127.0.0.0".parse().unwrap())).unwrap());
}

#[test]
fn predicate_with_value_list_datetime_in_range() {
    let a: u64 = DateTime::from_repr("2020-06-10").unwrap().into();
    let b: u64 = DateTime::from_repr("2020-06-20").unwrap().into();
    let p = Predicate::with_value_list(Operation::InRange, &[Value::DateTime(a), Value::DateTime(b)])
        .expect("datetime");
    let mid: u64 = DateTime::from_repr("2020-06-15").unwrap().into();
    assert!(p.evaluate(&Value::DateTime(mid)).unwrap());
    let before: u64 = DateTime::from_repr("2020-06-01").unwrap().into();
    assert!(!p.evaluate(&Value::DateTime(before)).unwrap());
}

#[test]
fn predicate_with_str_parsed_u8_is() {
    let p = Predicate::with_str(Operation::Is, "200", ValueKind::U8, false).expect("predicate");
    assert!(p.evaluate(&Value::U8(200)).unwrap());
    assert!(!p.evaluate(&Value::U8(199)).unwrap());
}

#[test]
fn predicate_with_str_signed_integers_is() {
    let p = Predicate::with_str(Operation::Is, "-42", ValueKind::I8, false).expect("i8");
    assert!(p.evaluate(&Value::I8(-42)).unwrap());

    let p = Predicate::with_str(Operation::Is, "-1000", ValueKind::I16, false).expect("i16");
    assert!(p.evaluate(&Value::I16(-1000)).unwrap());

    let p = Predicate::with_str(Operation::Is, "-2000000000", ValueKind::I32, false).expect("i32");
    assert!(p.evaluate(&Value::I32(-2_000_000_000)).unwrap());

    let p = Predicate::with_str(Operation::Is, "-5000000000000000000", ValueKind::I64, false).expect("i64");
    assert!(p.evaluate(&Value::I64(-5_000_000_000_000_000_000)).unwrap());
}

#[test]
fn predicate_with_str_unsigned_integers_is() {
    let p = Predicate::with_str(Operation::Is, "65500", ValueKind::U16, false).expect("u16");
    assert!(p.evaluate(&Value::U16(65_500)).unwrap());

    let p = Predicate::with_str(Operation::Is, "4000000000", ValueKind::U32, false).expect("u32");
    assert!(p.evaluate(&Value::U32(4_000_000_000)).unwrap());

    let p = Predicate::with_str(Operation::Is, "17000000000000000000", ValueKind::U64, false).expect("u64");
    assert!(p.evaluate(&Value::U64(17_000_000_000_000_000_000)).unwrap());
}

#[test]
fn predicate_with_str_floats_is() {
    let p = Predicate::with_str(Operation::Is, "-1.25", ValueKind::F32, false).expect("f32");
    assert!(p.evaluate(&Value::F32(-1.25)).unwrap());

    let p = Predicate::with_str(Operation::Is, "2.5", ValueKind::F64, false).expect("f64");
    assert!(p.evaluate(&Value::F64(2.5)).unwrap());
}

#[test]
fn predicate_with_str_string_respects_ignore_case() {
    let p = Predicate::with_str(Operation::Is, "CamelCase", ValueKind::String, true).expect("string");
    assert!(p.evaluate(&Value::String("camelcase")).unwrap());
    assert!(!p.evaluate(&Value::String("other")).unwrap());
}

#[test]
fn predicate_with_str_path_is() {
    let p = Predicate::with_str(Operation::Is, "/var/data", ValueKind::Path, false).expect("path");
    assert!(p.evaluate(&Value::Path("/var/data")).unwrap());
    assert!(!p.evaluate(&Value::Path("/tmp")).unwrap());
}

#[test]
fn predicate_with_str_hashes_is() {
    let s128 = "abababababababababababababababab";
    let p = Predicate::with_str(Operation::Is, s128, ValueKind::Hash128, false).expect("h128");
    let h = Hash128::from_str(s128).unwrap();
    assert!(p.evaluate(&Value::Hash128(h.as_bytes())).unwrap());

    let s160 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let p = Predicate::with_str(Operation::Is, s160, ValueKind::Hash160, false).expect("h160");
    let h = Hash160::from_str(s160).unwrap();
    assert!(p.evaluate(&Value::Hash160(h.as_bytes())).unwrap());

    let s256 = "0123456789abcdef".repeat(4);
    let p = Predicate::with_str(Operation::Is, &s256, ValueKind::Hash256, false).expect("h256");
    let h = Hash256::from_str(&s256).unwrap();
    assert!(p.evaluate(&Value::Hash256(h.as_bytes())).unwrap());
}

#[test]
fn predicate_with_str_ip_addr_datetime_bool() {
    let p = Predicate::with_str(Operation::Is, "10.20.30.40", ValueKind::IpAddr, false).expect("ip");
    let ip: IpAddr = "10.20.30.40".parse().unwrap();
    assert!(p.evaluate(&Value::IpAddr(ip)).unwrap());

    let p = Predicate::with_str(Operation::Is, "2019-12-25", ValueKind::DateTime, false).expect("dt");
    let t: u64 = DateTime::from_repr("2019-12-25").unwrap().into();
    assert!(p.evaluate(&Value::DateTime(t)).unwrap());

    let p = Predicate::with_str(Operation::Is, "false", ValueKind::Bool, false).expect("bool");
    assert!(p.evaluate(&Value::Bool(false)).unwrap());
    assert!(!p.evaluate(&Value::Bool(true)).unwrap());
}

#[test]
fn predicate_with_str_is_not_string() {
    let p = Predicate::with_str(Operation::IsNot, "x", ValueKind::String, false).expect("predicate");
    assert!(!p.evaluate(&Value::String("x")).unwrap());
    assert!(p.evaluate(&Value::String("y")).unwrap());
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
    assert!(p.evaluate(&Value::String("green")).unwrap());
    assert!(!p.evaluate(&Value::String("blue")).unwrap());
}

#[test]
fn predicate_with_str_list_string_ignore_case() {
    let p = Predicate::with_str_list(Operation::IsOneOf, &["Red", "BLUE"], ValueKind::String, true)
        .expect("predicate");
    assert!(p.evaluate(&Value::String("blue")).unwrap());
}

#[test]
fn predicate_with_str_list_path_is_one_of() {
    let p = Predicate::with_str_list(Operation::IsOneOf, &["/a", "/b/c"], ValueKind::Path, false)
        .expect("predicate");
    assert!(p.evaluate(&Value::Path("/b/c")).unwrap());
    assert!(!p.evaluate(&Value::Path("/z")).unwrap());
}

#[test]
fn predicate_with_str_list_unsigned_integers_is_one_of() {
    let p = Predicate::with_str_list(Operation::IsOneOf, &["1", "3", "5"], ValueKind::U8, false).expect("u8");
    assert!(p.evaluate(&Value::U8(3)).unwrap());
    assert!(!p.evaluate(&Value::U8(2)).unwrap());

    let p = Predicate::with_str_list(Operation::IsOneOf, &["1000", "2000", "3000"], ValueKind::U16, false)
        .expect("u16");
    assert!(p.evaluate(&Value::U16(2000)).unwrap());
    assert!(!p.evaluate(&Value::U16(1500)).unwrap());

    let p = Predicate::with_str_list(Operation::IsOneOf, &["100", "200", "300"], ValueKind::U32, false)
        .expect("u32");
    assert!(p.evaluate(&Value::U32(200)).unwrap());
    assert!(!p.evaluate(&Value::U32(150)).unwrap());

    let p = Predicate::with_str_list(
        Operation::IsOneOf,
        &["10000000000000000000", "11000000000000000000"],
        ValueKind::U64,
        false,
    )
    .expect("u64");
    assert!(p.evaluate(&Value::U64(11_000_000_000_000_000_000)).unwrap());
    assert!(!p.evaluate(&Value::U64(10_500_000_000_000_000_000)).unwrap());
}

#[test]
fn predicate_with_str_list_signed_integers_is_one_of() {
    let p = Predicate::with_str_list(Operation::IsOneOf, &["-5", "0", "7"], ValueKind::I8, false).expect("i8");
    assert!(p.evaluate(&Value::I8(-5)).unwrap());
    assert!(!p.evaluate(&Value::I8(3)).unwrap());

    let p = Predicate::with_str_list(Operation::IsOneOf, &["-30000", "0", "1000"], ValueKind::I16, false)
        .expect("i16");
    assert!(p.evaluate(&Value::I16(-30_000)).unwrap());
    assert!(!p.evaluate(&Value::I16(-29_000)).unwrap());

    let p = Predicate::with_str_list(Operation::IsOneOf, &["-1", "0", "99"], ValueKind::I32, false)
        .expect("i32");
    assert!(p.evaluate(&Value::I32(99)).unwrap());
    assert!(!p.evaluate(&Value::I32(50)).unwrap());

    let p = Predicate::with_str_list(
        Operation::IsOneOf,
        &["-9000000000000000000", "0", "1"],
        ValueKind::I64,
        false,
    )
    .expect("i64");
    assert!(p.evaluate(&Value::I64(-9_000_000_000_000_000_000)).unwrap());
    assert!(!p.evaluate(&Value::I64(-8_000_000_000_000_000_000)).unwrap());
}

#[test]
fn predicate_with_str_list_unsigned_integers_in_range() {
    let p = Predicate::with_str_list(Operation::InRange, &["10", "100"], ValueKind::U8, false).expect("u8");
    assert!(p.evaluate(&Value::U8(50)).unwrap());
    assert!(!p.evaluate(&Value::U8(5)).unwrap());

    let p = Predicate::with_str_list(Operation::InRange, &["1000", "9000"], ValueKind::U16, false).expect("u16");
    assert!(p.evaluate(&Value::U16(5000)).unwrap());
    assert!(!p.evaluate(&Value::U16(500)).unwrap());

    let p = Predicate::with_str_list(Operation::InRange, &["100000", "200000"], ValueKind::U32, false)
        .expect("u32");
    assert!(p.evaluate(&Value::U32(150_000)).unwrap());
    assert!(!p.evaluate(&Value::U32(50_000)).unwrap());

    let p = Predicate::with_str_list(
        Operation::InRange,
        &["1000000000000000000", "15000000000000000000"],
        ValueKind::U64,
        false,
    )
    .expect("u64");
    assert!(p.evaluate(&Value::U64(10_000_000_000_000_000_000)).unwrap());
    assert!(!p.evaluate(&Value::U64(500_000_000_000_000_000)).unwrap());
}

#[test]
fn predicate_with_str_list_signed_integers_in_range() {
    let p = Predicate::with_str_list(Operation::InRange, &["-50", "50"], ValueKind::I8, false).expect("i8");
    assert!(p.evaluate(&Value::I8(0)).unwrap());
    assert!(!p.evaluate(&Value::I8(-100)).unwrap());

    let p = Predicate::with_str_list(Operation::InRange, &["-20000", "20000"], ValueKind::I16, false)
        .expect("i16");
    assert!(p.evaluate(&Value::I16(0)).unwrap());
    assert!(!p.evaluate(&Value::I16(30_000)).unwrap());

    let p = Predicate::with_str_list(Operation::InRange, &["-1000000", "1000000"], ValueKind::I32, false)
        .expect("i32");
    assert!(p.evaluate(&Value::I32(500_000)).unwrap());
    assert!(!p.evaluate(&Value::I32(2_000_000)).unwrap());

    let p = Predicate::with_str_list(
        Operation::InRange,
        &["-5000000000000000000", "5000000000000000000"],
        ValueKind::I64,
        false,
    )
    .expect("i64");
    assert!(p.evaluate(&Value::I64(0)).unwrap());
    assert!(!p.evaluate(&Value::I64(9_000_000_000_000_000_000)).unwrap());
}

#[test]
fn predicate_with_str_list_float_in_range() {
    let p = Predicate::with_str_list(Operation::InRange, &["0.25", "4.0"], ValueKind::F32, false).expect("f32");
    assert!(p.evaluate(&Value::F32(1.5)).unwrap());
    assert!(!p.evaluate(&Value::F32(0.1)).unwrap());

    let p = Predicate::with_str_list(Operation::InRange, &["1.5", "10.5"], ValueKind::F64, false)
        .expect("f64");
    assert!(p.evaluate(&Value::F64(5.0)).unwrap());
    assert!(!p.evaluate(&Value::F64(0.0)).unwrap());
}

#[test]
fn predicate_with_str_list_hash_is_one_of() {
    let a = "00000000000000000000000000000000";
    let b = "ffffffffffffffffffffffffffffffff";
    let p = Predicate::with_str_list(Operation::IsOneOf, &[a, b], ValueKind::Hash128, false).expect("predicate");
    let hb = Hash128::from_str(b).unwrap();
    assert!(p.evaluate(&Value::Hash128(hb.as_bytes())).unwrap());
    let other = Hash128::new([1u8; 16]);
    assert!(!p.evaluate(&Value::Hash128(other.as_bytes())).unwrap());
}

#[test]
fn predicate_with_str_list_hash160_and_hash256_is_one_of() {
    let z40 = "0".repeat(40);
    let f40 = "f".repeat(40);
    let p = Predicate::with_str_list(Operation::IsOneOf, &[z40.as_str(), f40.as_str()], ValueKind::Hash160, false)
        .expect("hash160");
    let hf = Hash160::from_str(&f40).unwrap();
    assert!(p.evaluate(&Value::Hash160(hf.as_bytes())).unwrap());
    assert!(!p.evaluate(&Value::Hash160(Hash160::new([1u8; 20]).as_bytes())).unwrap());

    let z64 = "0".repeat(64);
    let a64 = "a".repeat(64);
    let p = Predicate::with_str_list(Operation::IsOneOf, &[z64.as_str(), a64.as_str()], ValueKind::Hash256, false)
        .expect("hash256");
    let ha = Hash256::from_str(&a64).unwrap();
    assert!(p.evaluate(&Value::Hash256(ha.as_bytes())).unwrap());
    assert!(!p.evaluate(&Value::Hash256(Hash256::new([2u8; 32]).as_bytes())).unwrap());
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
    assert!(p.evaluate(&Value::IpAddr("192.168.0.1".parse().unwrap())).unwrap());
    assert!(!p.evaluate(&Value::IpAddr("10.0.0.9".parse().unwrap())).unwrap());
}

#[test]
fn predicate_with_str_list_ip_in_range() {
    let p = Predicate::with_str_list(Operation::InRange, &["127.0.0.1", "127.0.0.10"], ValueKind::IpAddr, false)
        .expect("predicate");
    assert!(p.evaluate(&Value::IpAddr("127.0.0.5".parse().unwrap())).unwrap());
    assert!(!p.evaluate(&Value::IpAddr("127.0.0.0".parse().unwrap())).unwrap());
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
    assert!(p.evaluate(&Value::DateTime(mid)).unwrap());
    let before: u64 = DateTime::from_repr("2020-06-01").unwrap().into();
    assert!(!p.evaluate(&Value::DateTime(before)).unwrap());
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
    assert_eq!(Value::Path("p").kind(), ValueKind::Path);
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
    assert_eq!(Value::Unknown.kind(), ValueKind::Unknown);
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
        ValueKind::Unknown,
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
    match <&str>::try_from(Value::Path("x")) {
        Err(Error::ExpectingADifferentValueKind(got, expected)) => {
            assert_eq!(got, ValueKind::Path);
            assert_eq!(expected, ValueKind::String);
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

#[test]
fn shortcirtuit_and_on_first_false() {
    let expr = ExpressionBuilder::<ShortcircuitTest>::new()
        .add("first", Condition::from_str("op1 is true"))
        .add("second", Condition::from_str("op2 is true"))
        .add("third", Condition::from_str("op3 is true"))
        .build("first && second && third")
        .unwrap();

    let test1 = ShortcircuitTest {
        op1: false,
        op2: true,
        op3: true,
        visited1: Cell::new(false),
        visited2: Cell::new(false),
        visited3: Cell::new(false),
    };

    let shortmatch = expr.matches(&test1);
    assert_eq!(shortmatch, false);
    assert_eq!(test1.visited1.get(), true);
    assert_eq!(test1.visited2.get(), false);
    assert_eq!(test1.visited3.get(), false);

    let test2 = ShortcircuitTest {
        op1: true,
        op2: false,
        op3: true,
        visited1: Cell::new(false),
        visited2: Cell::new(false),
        visited3: Cell::new(false),
    };

    let shortmatch = expr.matches(&test2);
    assert_eq!(shortmatch, false);
    assert_eq!(test2.visited1.get(), true);
    assert_eq!(test2.visited2.get(), true);
    assert_eq!(test2.visited3.get(), false);

    let test3 = ShortcircuitTest {
        op1: true,
        op2: true,
        op3: false,
        visited1: Cell::new(false),
        visited2: Cell::new(false),
        visited3: Cell::new(false),
    };

    let shortmatch = expr.matches(&test3);
    assert_eq!(shortmatch, false);
    assert_eq!(test3.visited1.get(), true);
    assert_eq!(test3.visited2.get(), true);
    assert_eq!(test3.visited3.get(), true);
}

#[test]
fn shortcirtuit_or_on_first_true() {
    let expr = ExpressionBuilder::<ShortcircuitTest>::new()
        .add("first", Condition::from_str("op1 is true"))
        .add("second", Condition::from_str("op2 is true"))
        .add("third", Condition::from_str("op3 is true"))
        .build("first || second || third")
        .unwrap();

    let test1 = ShortcircuitTest {
        op1: true,
        op2: false,
        op3: false,
        visited1: Cell::new(false),
        visited2: Cell::new(false),
        visited3: Cell::new(false),
    };

    let shortmatch = expr.matches(&test1);
    assert_eq!(shortmatch, true);
    assert_eq!(test1.visited1.get(), true);
    assert_eq!(test1.visited2.get(), false);
    assert_eq!(test1.visited3.get(), false);

    let test2 = ShortcircuitTest {
        op1: false,
        op2: true,
        op3: false,
        visited1: Cell::new(false),
        visited2: Cell::new(false),
        visited3: Cell::new(false),
    };

    let shortmatch = expr.matches(&test2);
    assert_eq!(shortmatch, true);
    assert_eq!(test2.visited1.get(), true);
    assert_eq!(test2.visited2.get(), true);
    assert_eq!(test2.visited3.get(), false);

    let test3 = ShortcircuitTest {
        op1: false,
        op2: false,
        op3: true,
        visited1: Cell::new(false),
        visited2: Cell::new(false),
        visited3: Cell::new(false),
    };

    let shortmatch = expr.matches(&test3);
    assert_eq!(shortmatch, true);
    assert_eq!(test3.visited1.get(), true);
    assert_eq!(test3.visited2.get(), true);
    assert_eq!(test3.visited3.get(), true);
}

// ---------------------------------------------------------------------------
// `re-match` — end-to-end through `Condition::parse` / `ExpressionBuilder`
// ---------------------------------------------------------------------------

fn sample_file() -> TestFile {
    TestFile {
        name: "report_2024-05-10".into(),
        path: "/var/log/app.log".into(),
    }
}

fn build_re(cond: &str) -> Result<crate::Expression, Error> {
    ExpressionBuilder::<TestFile>::new()
        .add("rule", Condition::from_str(cond))
        .build("rule")
}

/// `Predicate` has no `Debug` impl, so `unwrap_err` is unavailable on `Condition::parse`.
fn parse_err(cond: &str) -> Error {
    match Condition::parse::<TestFile>(cond, "mycond") {
        Err(e) => e,
        Ok(_) => panic!("expected a parse error for: {cond}"),
    }
}

#[test]
fn re_match_on_string_attribute() {
    let ex = build_re("name re-match '^report_'").expect("build");
    assert!(ex.matches(&sample_file()));
    assert!(!ex.matches(&TestFile {
        name: "draft".into(),
        path: "/tmp/x".into(),
    }));
}

#[test]
fn re_match_on_path_attribute() {
    let ex = build_re(r"path re-match '\.log$'").expect("build");
    assert!(ex.matches(&sample_file()));
    assert!(!ex.matches(&TestFile {
        name: "n".into(),
        path: "/var/log/app.txt".into(),
    }));
}

#[test]
fn not_re_match_negates_the_result() {
    let ex = build_re("name not-re-match '^draft'").expect("build");
    assert!(ex.matches(&sample_file()));
}

#[test]
fn re_match_quantifier_pattern_works_when_quoted() {
    // `{4}` / `{2}` would be eaten by the modifier parser if the value were unquoted.
    let ex = build_re(r"name re-match '^report_\d{4}-\d{2}-\d{2}$'").expect("build");
    assert!(ex.matches(&sample_file()));
}

#[test]
fn re_match_inline_case_insensitive_flag_works() {
    let ex = build_re("name re-match '(?i)^REPORT_'").expect("build");
    assert!(ex.matches(&sample_file()));
}

#[test]
fn re_match_requires_single_quotes() {
    let err = parse_err("name re-match ^report_");
    assert!(matches!(err, Error::UnquotedRegexPattern(_, _, _)));
}

#[test]
fn not_re_match_requires_single_quotes() {
    let err = parse_err("name not-re-match ^report_");
    assert!(matches!(err, Error::UnquotedRegexPattern(_, _, _)));
}

#[test]
fn re_match_rejects_double_quotes() {
    // Double-quoted values run through `unescape`, which would reject `\d` and friends.
    let err = parse_err("name re-match \"^report_\"");
    assert!(matches!(err, Error::UnquotedRegexPattern(_, _, _)));
}

#[test]
fn re_match_bracketed_pattern_is_not_parsed_as_a_list() {
    // Without the quoting rule `[a-z]` would parse as a one-element list.
    let err = parse_err("name re-match [a-z]");
    assert!(matches!(err, Error::UnquotedRegexPattern(_, _, _)));
}

#[test]
fn re_match_missing_value_reports_missing_value() {
    // The quoting check must not shadow the plain "no value given" diagnostic.
    let err = parse_err("name re-match ");
    assert!(matches!(err, Error::ExpectingAValue(_, _, _)));
}

#[test]
fn re_match_with_only_modifiers_reports_missing_value() {
    let err = parse_err("name re-match {ignore-case}");
    assert!(matches!(err, Error::ExpectingAValue(_, _, _)));
}

#[test]
fn re_match_rejects_ignore_case_modifier() {
    let err = parse_err("name re-match '^report_' {ignore-case}");
    assert!(matches!(err, Error::IgnoreCaseNotSupported(Operation::ReMatch)));
}

#[test]
fn not_re_match_rejects_ignore_case_modifier() {
    // Negation is collapsed before dispatch, so the error names the positive operation.
    let err = parse_err("name not-re-match '^report_' {ignore-case}");
    assert!(matches!(err, Error::IgnoreCaseNotSupported(Operation::ReMatch)));
}

#[test]
fn re_match_on_path_rejects_ignore_case_modifier() {
    let err = parse_err("path re-match '^/var' {ignore-case}");
    assert!(matches!(err, Error::IgnoreCaseNotSupported(Operation::ReMatch)));
}

#[test]
fn re_match_invalid_pattern_is_a_build_error_not_a_panic() {
    let err = parse_err("name re-match '['");
    assert!(matches!(
        err,
        Error::FailToBuildInternalDataStructure(Operation::ReMatch, _, _)
    ));
}

#[test]
fn re_match_list_form_is_rejected() {
    // A list starts with `[`, so the single-quote check rejects it before
    // `values::parse` ever splits it. Revisit this if a list form is ever added.
    let err = parse_err("name re-match ['^a', '^b']");
    assert!(matches!(err, Error::UnquotedRegexPattern(_, _, _)));
}

#[test]
fn re_match_list_form_is_rejected_at_the_predicate_level_too() {
    // Bypassing the condition parser, the list form is still unsupported.
    match Predicate::with_str_list(Operation::ReMatch, &["^a", "^b"], ValueKind::String, false) {
        Err(e) => assert!(matches!(
            e,
            Error::InvalidOperationForValue(Operation::ReMatch, ValueKind::String)
        )),
        Ok(_) => panic!("expected the list form to be rejected"),
    }
}

#[test]
fn re_match_combines_with_other_operations_in_one_expression() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("named", Condition::from_str("name re-match '^report_'"))
        .add("logfile", Condition::from_str(r"path re-match '\.log$'"))
        .add("under_var", Condition::from_str("path starts-with /var"))
        .build("named && (logfile || under_var)")
        .expect("build");
    assert!(ex.matches(&sample_file()));
}


#[test]
fn test_diacritice_endswith_ignore_case() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path endswith 'a' {ignore-case}"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(!ex.matches(&f1));
}
#[test]
fn test_diacritice_startwith_ignore_case() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path startswith 'a' {ignore-case}"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(!ex.matches(&f1));
}

#[test]
fn test_diacritice_contains_ignore_case() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path contains 'a' {ignore-case}"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(!ex.matches(&f1));
}

#[test]
fn test_diacritice_contains_empty_string_ignore_case() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path contains '' {ignore-case}"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(ex.matches(&f1));
}
#[test]
fn test_diacritice_contains_empty_string() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path contains ''"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(ex.matches(&f1));
}
#[test]
fn test_diacritice_startswith_empty_string_ignore_case() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path startswith '' {ignore-case}"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(ex.matches(&f1));
}
#[test]
fn test_diacritice_startswith_empty_string() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path startswith ''"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(ex.matches(&f1));
}
#[test]
fn test_diacritice_endswith_empty_string_ignore_case() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path endswith '' {ignore-case}"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(ex.matches(&f1));
}
#[test]
fn test_diacritice_endswith_empty_string() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path endswith ''"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(ex.matches(&f1));
}

#[test]
fn test_diacritice_eq_ignore_case() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path eq 'a' {ignore-case}"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(!ex.matches(&f1));
}
#[test]
fn test_diacritice_eq() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path eq 'a'"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(!ex.matches(&f1));
}
#[test]
fn test_diacritice_eq_empty_string_ignore_case() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path eq '' {ignore-case}"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(!ex.matches(&f1));
}
#[test]
fn test_diacritice_eq_empty_string() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path eq '' "))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"ș".into(),
    };
    assert!(!ex.matches(&f1));
}
#[test]
fn test_diacritice_eq_empty_string_ignore_case_empty_string() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path eq '' {ignore-case}"))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"".into(),
    };
    assert!(ex.matches(&f1));
}
#[test]
fn test_diacritice_eq_empty_string_empty_string() {
    let ex = ExpressionBuilder::<TestFile>::new()
        .add("test", Condition::from_str("path eq '' "))
        .build("test")
        .unwrap();
    let f1 = TestFile {
        name: "".into(),
        path: r"".into(),
    };
    assert!(ex.matches(&f1));
}

/// Three optional bools. `None` is an unknown attribute (`get` returns `None`).
struct OptionalFlags {
    a: Option<bool>,
    b: Option<bool>,
    c: Option<bool>,
}

impl OptionalFlags {
    const A: AttributeIndex = AttributeIndex::new(0);
    const B: AttributeIndex = AttributeIndex::new(1);
    const C: AttributeIndex = AttributeIndex::new(2);
}

impl Attributes for OptionalFlags {
    const TYPE_ID: u64 = 0x0E0E_0001;
    const TYPE_NAME: &'static str = "OptionalFlags";

    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        let value = match idx {
            Self::A => self.a,
            Self::B => self.b,
            Self::C => self.c,
            _ => return None,
        };
        value.map(Value::Bool)
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::A | Self::B | Self::C => Some(ValueKind::Bool),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "a" => Some(Self::A),
            "b" => Some(Self::B),
            "c" => Some(Self::C),
            _ => None,
        }
    }
}

fn optional_flags_expr(expr: &str) -> Expression {
    ExpressionBuilder::<OptionalFlags>::new()
        .add("a", Condition::from_str("a is true"))
        .add("b", Condition::from_str("b is true"))
        .add("c", Condition::from_str("c is true"))
        .build(expr)
        .unwrap()
}

/// False absorbs AND. Otherwise unknown absorbs AND. Otherwise AND is true.
fn kleene_and(values: [Option<bool>; 3]) -> Option<bool> {
    if values.contains(&Some(false)) {
        Some(false)
    } else if values.contains(&None) {
        None
    } else {
        Some(true)
    }
}

/// True absorbs OR. Otherwise unknown absorbs OR. Otherwise OR is false.
fn kleene_or(values: [Option<bool>; 3]) -> Option<bool> {
    if values.contains(&Some(true)) {
        Some(true)
    } else if values.contains(&None) {
        None
    } else {
        Some(false)
    }
}

#[test]
fn and_or_follow_kleene_logic_for_optional_bools() {
    let and_expr = optional_flags_expr("a && b && c");
    let or_expr = optional_flags_expr("a || b || c");
    let states = [Some(true), Some(false), None];

    for &a in &states {
        for &b in &states {
            for &c in &states {
                let obj = OptionalFlags { a, b, c };
                let values = [a, b, c];
                assert_eq!(and_expr.evaluate(&obj), kleene_and(values), "AND a={a:?} b={b:?} c={c:?}");
                assert_eq!(or_expr.evaluate(&obj), kleene_or(values), "OR a={a:?} b={b:?} c={c:?}");
            }
        }
    }
}

#[test]
fn not_and_nested_follow_kleene_logic_for_optional_bools() {
    let cases: &[(&str, OptionalFlags, Option<bool>)] = &[
        ("!a", OptionalFlags { a: Some(true), b: None, c: None }, Some(false)),
        ("!a", OptionalFlags { a: Some(false), b: None, c: None }, Some(true)),
        ("!a", OptionalFlags { a: None, b: None, c: None }, None),
        ("(a || b) && c", OptionalFlags { a: None, b: Some(true), c: Some(true) }, Some(true)),
        ("(a && b) || c", OptionalFlags { a: None, b: Some(false), c: Some(false) }, Some(false)),
        ("!(a && b)", OptionalFlags { a: None, b: Some(true), c: None }, None),
        ("!(a || b)", OptionalFlags { a: None, b: Some(true), c: None }, Some(false)),
    ];

    for &(expr, ref obj, expected) in cases {
        let got = optional_flags_expr(expr).evaluate(obj);
        assert_eq!(got, expected, "{expr} with a={:?} b={:?} c={:?}", obj.a, obj.b, obj.c);
    }
}
// ---------------------------------------------------------------------------
// Three-valued (Kleene) logic: direct `EvaluationNode` tests.
// T = Some(true), F = Some(false), U = None (unknown).
// ---------------------------------------------------------------------------

const T: Option<bool> = Some(true);
const F: Option<bool> = Some(false);
const U: Option<bool> = None;

/// Up to six optional bool attributes (`v0`..`v5`). `None` makes `get` return `None`,
/// so the matching condition evaluates to unknown.
struct TriFlags([Option<bool>; 6]);

impl Attributes for TriFlags {
    const TYPE_ID: u64 = 0x0E0E_0002;
    const TYPE_NAME: &'static str = "TriFlags";

    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        self.0.get(idx.index() as usize).copied().flatten().map(Value::Bool)
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        ((idx.index() as usize) < 6).then_some(ValueKind::Bool)
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        let i: u16 = name.strip_prefix('v')?.parse().ok()?;
        (i < 6).then(|| AttributeIndex::new(i))
    }
}

/// Expression with conditions `c0`..`c5` (`ci` is "vi is true", so condition index i == attribute i).
fn tri_expression() -> Expression {
    let mut b = ExpressionBuilder::<TriFlags>::new();
    for i in 0..6 {
        b = b.add(&format!("c{i}"), Condition::from_str(&format!("v{i} is true")));
    }
    b.build("c0").unwrap()
}

fn tri_obj(values: &[Option<bool>]) -> TriFlags {
    assert!(values.len() <= 6);
    let mut arr = [U; 6];
    arr[..values.len()].copy_from_slice(values);
    TriFlags(arr)
}

fn leaves(n: usize) -> Vec<EvaluationNode> {
    (0..n as u16).map(EvaluationNode::Condition).collect()
}

fn and_of(children: Vec<EvaluationNode>) -> EvaluationNode {
    EvaluationNode::Group { composition: Composition::And, children }
}

fn or_of(children: Vec<EvaluationNode>) -> EvaluationNode {
    EvaluationNode::Group { composition: Composition::Or, children }
}

fn not_of(child: EvaluationNode) -> EvaluationNode {
    EvaluationNode::Not { child: Box::new(child) }
}

/// Evaluates AND / OR over the leaves `c0..cn` with leaf i taking `values[i]`.
fn eval_flat(composition: Composition, values: &[Option<bool>]) -> Option<bool> {
    let expr = tri_expression();
    let node = EvaluationNode::Group { composition, children: leaves(values.len()) };
    node.evaluate(&tri_obj(values), &expr)
}

#[test]
fn kleene_and_cases() {
    let cases: &[(&[Option<bool>], Option<bool>)] = &[
        (&[T, T, T], T),
        (&[T, F, T], F),
        (&[T, U, T], U),
        (&[F, F, U, F], F),
        (&[F, F, U, T], F),
        (&[U, F], F),
        (&[F, U], F),
        (&[U, U], U),
        (&[U, T, T, F], F),
        (&[], T),
    ];
    for &(values, expected) in cases {
        assert_eq!(eval_flat(Composition::And, values), expected, "AND{values:?}");
    }
}

#[test]
fn kleene_or_cases() {
    let cases: &[(&[Option<bool>], Option<bool>)] = &[
        (&[F, F, F], F),
        (&[F, T, F], T),
        (&[F, U, F], U),
        (&[F, F, U, F], U),
        (&[F, F, U, T], T),
        (&[U, T], T),
        (&[T, U], T),
        (&[U, U], U),
        (&[U, F, F, T], T),
        (&[], F),
    ];
    for &(values, expected) in cases {
        assert_eq!(eval_flat(Composition::Or, values), expected, "OR{values:?}");
    }
}

#[test]
fn kleene_not_cases() {
    let expr = tri_expression();
    for (value, expected) in [(T, F), (F, T), (U, U)] {
        let node = not_of(EvaluationNode::Condition(0));
        assert_eq!(node.evaluate(&tri_obj(&[value]), &expr), expected, "NOT {value:?}");
    }
}

#[test]
fn kleene_nested_cases() {
    let expr = tri_expression();
    let c = EvaluationNode::Condition;

    // AND(OR(U, T), T) = T
    let node = and_of(vec![or_of(vec![c(0), c(1)]), c(2)]);
    assert_eq!(node.evaluate(&tri_obj(&[U, T, T]), &expr), T);

    // OR(AND(U, F), F) = F
    let node = or_of(vec![and_of(vec![c(0), c(1)]), c(2)]);
    assert_eq!(node.evaluate(&tri_obj(&[U, F, F]), &expr), F);

    // NOT(AND(U, T)) = U
    let node = not_of(and_of(vec![c(0), c(1)]));
    assert_eq!(node.evaluate(&tri_obj(&[U, T]), &expr), U);

    // NOT(OR(U, T)) = F
    let node = not_of(or_of(vec![c(0), c(1)]));
    assert_eq!(node.evaluate(&tri_obj(&[U, T]), &expr), F);
}

fn permutations(items: &[Option<bool>]) -> Vec<Vec<Option<bool>>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let first = rest.remove(i);
        for mut p in permutations(&rest) {
            p.insert(0, first);
            out.push(p);
        }
    }
    out
}

#[test]
fn kleene_and_or_are_order_independent() {
    let inputs: &[&[Option<bool>]] = &[
        &[T, F, U],
        &[T, T, U],
        &[F, F, U],
        &[F, F, U, F],
        &[F, F, U, T],
        &[T, U, U, F],
    ];
    for &input in inputs {
        let and_expected = eval_flat(Composition::And, input);
        let or_expected = eval_flat(Composition::Or, input);
        for p in permutations(input) {
            assert_eq!(eval_flat(Composition::And, &p), and_expected, "AND{p:?} vs AND{input:?}");
            assert_eq!(eval_flat(Composition::Or, &p), or_expected, "OR{p:?} vs OR{input:?}");
        }
    }
}

#[test]
fn kleene_four_operand_cases_through_parser() {
    let mut b = ExpressionBuilder::<TriFlags>::new();
    for i in 0..4 {
        b = b.add(&format!("c{i}"), Condition::from_str(&format!("v{i} is true")));
    }
    let and_expr = b.build("c0 && c1 && c2 && c3").unwrap();

    let mut b = ExpressionBuilder::<TriFlags>::new();
    for i in 0..4 {
        b = b.add(&format!("c{i}"), Condition::from_str(&format!("v{i} is true")));
    }
    let or_expr = b.build("c0 || c1 || c2 || c3").unwrap();

    assert_eq!(and_expr.evaluate(&tri_obj(&[F, F, U, F])), F);
    assert_eq!(and_expr.evaluate(&tri_obj(&[F, F, U, T])), F);
    assert_eq!(or_expr.evaluate(&tri_obj(&[F, F, U, F])), U);
    assert_eq!(or_expr.evaluate(&tri_obj(&[F, F, U, T])), T);
}

// ---------------------------------------------------------------------------
// `Expression::matches`: true only when the three-valued result is `Some(true)`.
// An unknown result (`None`) counts as "no match" and never panics.
// ---------------------------------------------------------------------------

#[test]
fn matches_returns_false_instead_of_panicking_when_attribute_is_missing() {
    let pred = Predicate::with_value(Operation::Is, 1u32).expect("predicate");
    let ex = ExpressionBuilder::<TestPerson>::new()
        .add("rule1", Condition::with_index(AttributeIndex::new(99), pred))
        .build("rule1")
        .expect("build");
    let person = TestPerson { age: 5, label: "x".into() };
    assert_eq!(ex.evaluate(&person), None);
    assert!(!ex.matches(&person));
}

#[test]
fn matches_treats_unknown_as_no_match() {
    // (expression, a, b, c, d, expected `matches`)
    let cases: &[(&str, [Option<bool>; 4], bool)] = &[
        // a single unknown condition
        ("c0", [U, U, U, U], false),
        // NOT unknown is still unknown, so it does not match either
        ("!c0", [U, U, U, U], false),
        ("!c0", [F, U, U, U], true),
        // AND: unknown only matters if nothing is false
        ("c0 && c1", [T, U, U, U], false),
        ("c0 && c1", [U, F, U, U], false),
        ("c0 && c1", [T, T, U, U], true),
        ("c0 && c1 && c2 && c3", [F, F, U, F], false),
        ("c0 && c1 && c2 && c3", [F, F, U, T], false),
        ("c0 && c1 && c2 && c3", [T, T, U, T], false),
        ("c0 && c1 && c2 && c3", [T, T, T, T], true),
        // OR: a true anywhere wins over unknown
        ("c0 || c1", [U, T, U, U], true),
        ("c0 || c1", [T, U, U, U], true),
        ("c0 || c1", [F, U, U, U], false),
        ("c0 || c1 || c2 || c3", [F, F, U, F], false),
        ("c0 || c1 || c2 || c3", [F, F, U, T], true),
        // nested
        ("(c0 || c1) && c2", [U, T, T, U], true),
        ("(c0 || c1) && c2", [U, F, T, U], false),
        ("!(c0 || c1)", [U, T, U, U], false),
        ("!(c0 && c1)", [U, F, U, U], true),
        ("!(c0 && c1)", [U, T, U, U], false),
    ];
    for &(expr, values, expected) in cases {
        let ex = tri_build(expr);
        let obj = tri_obj(&values);
        assert_eq!(ex.matches(&obj), expected, "matches({expr}) with {values:?}");
    }
}

#[test]
fn matches_agrees_with_evaluate_for_every_combination() {
    let states = [T, F, U];
    let exprs = [
        "c0 && c1 && c2",
        "c0 || c1 || c2",
        "!c0",
        "(c0 || c1) && c2",
        "(c0 && c1) || c2",
        "!(c0 && c1) || c2",
        "!(c0 || c1) && !c2",
    ];
    for expr in exprs {
        let ex = tri_build(expr);
        for &a in &states {
            for &b in &states {
                for &c in &states {
                    let obj = tri_obj(&[a, b, c]);
                    let three_valued = ex.evaluate(&obj);
                    assert_eq!(
                        ex.matches(&obj),
                        three_valued == Some(true),
                        "{expr} with a={a:?} b={b:?} c={c:?} (evaluate = {three_valued:?})"
                    );
                }
            }
        }
    }
}

#[test]
fn matches_filters_out_records_with_unknown_result() {
    let ex = tri_build("c0 || c1");
    let records = [
        tri_obj(&[T, U]), // true  -> kept
        tri_obj(&[U, U]), // unknown -> dropped
        tri_obj(&[F, U]), // unknown -> dropped
        tri_obj(&[F, F]), // false -> dropped
        tri_obj(&[U, T]), // true  -> kept
    ];
    let kept: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| ex.matches(*r))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(kept, vec![0, 4]);
}

fn parse_rule(input: &str) -> crate::rule_parser::RuleParser {
    let mut parser = crate::rule_parser::RuleParser::new(input);
    assert_eq!(parser.parse_next(), Ok(true), "input: {input:?}");
    parser
}

fn assert_rule_error(input: &str, expected: Error) {
    let mut parser = crate::rule_parser::RuleParser::new(input);
    assert_eq!(parser.parse_next(), Err(expected), "input: {input:?}");
}

#[test]
fn rule_parser_accessors_are_empty_before_parse() {
    let parser = crate::rule_parser::RuleParser::new("rule r: T { a = b } where {a}");
    assert_eq!(parser.rule_name(), None);
    assert_eq!(parser.type_name(), None);
    assert_eq!(parser.expr(), None);
    assert_eq!(parser.conditions_count(), 0);
    assert_eq!(parser.condition_name(0), None);
    assert_eq!(parser.condition_value(0), None);
    assert_eq!(parser.meta_count(), 0);
    assert_eq!(parser.meta_name(0), None);
    assert_eq!(parser.meta_value(0), None);
}

#[test]
fn rule_parser_blank_input_reports_no_rule() {
    for input in ["", "   ", "\n\t\r\n"] {
        let mut parser = crate::rule_parser::RuleParser::new(input);
        assert_eq!(parser.parse_next(), Ok(false), "{input:?}");
        assert_eq!(parser.rule_name(), None);
        assert_eq!(parser.type_name(), None);
        assert_eq!(parser.expr(), None);
        assert_eq!(parser.conditions_count(), 0);
        assert_eq!(parser.condition_name(0), None);
        assert_eq!(parser.condition_value(0), None);
    }
}

#[test]
fn rule_parser_parses_named_conditions_and_where_expression() {
    let input = r#"
rule student-review: Student {
    age-is-prime = "age is-one-of [19, 23, 29, 31]"
    common       = "name.family ends-with 'escu' {ignore-case}"
    old-enroll   = "enroll.year in-range [2010, 2020]"
    good-at-math = "grade.math >= 8"
} where {
    age-is-prime OR common OR (old-enroll AND good-at-math)
}
"#;
    let parser = parse_rule(input);
    assert_eq!(parser.rule_name(), Some("student-review"));
    assert_eq!(parser.type_name(), Some("Student"));
    assert_eq!(parser.conditions_count(), 4);
    assert_eq!(parser.condition_name(0), Some("age-is-prime"));
    assert_eq!(parser.condition_value(0), Some("age is-one-of [19, 23, 29, 31]"));
    assert_eq!(parser.condition_name(1), Some("common"));
    assert_eq!(
        parser.condition_value(1),
        Some("name.family ends-with 'escu' {ignore-case}")
    );
    assert_eq!(parser.condition_name(2), Some("old-enroll"));
    assert_eq!(parser.condition_value(2), Some("enroll.year in-range [2010, 2020]"));
    assert_eq!(parser.condition_name(3), Some("good-at-math"));
    assert_eq!(parser.condition_value(3), Some("grade.math >= 8"));
    assert_eq!(parser.condition_name(4), None);
    assert_eq!(parser.condition_value(4), None);
    assert_eq!(
        parser.expr(),
        Some("age-is-prime OR common OR (old-enroll AND good-at-math)")
    );
}

#[test]
fn rule_parser_keywords_are_case_insensitive_and_names_keep_their_case() {
    let parser = parse_rule("RuLe Adults: Person { Adult = \"age >= 18\" } WhErE {Adult}");
    assert_eq!(parser.rule_name(), Some("Adults"));
    assert_eq!(parser.type_name(), Some("Person"));
    assert_eq!(parser.condition_name(0), Some("Adult"));
    assert_eq!(parser.condition_value(0), Some("age >= 18"));
    assert_eq!(parser.expr(), Some("Adult"));
}

#[test]
fn rule_parser_accepts_hyphen_and_underscore_in_words() {
    // Rule and condition names start with a letter; `_` and `-` are allowed after it.
    // Type names and bare-word values may still start with `_`.
    let parser = parse_rule("rule student_review-1: _My_Type-1 { a_b-c = _x-1 } where {(a_b-c)}");
    assert_eq!(parser.rule_name(), Some("student_review-1"));
    assert_eq!(parser.type_name(), Some("_My_Type-1"));
    assert_eq!(parser.condition_name(0), Some("a_b-c"));
    assert_eq!(parser.condition_value(0), Some("_x-1"));
    assert_eq!(parser.expr(), Some("(a_b-c)"));
}

#[test]
fn rule_parser_accepts_every_condition_value_form() {
    let input = "\
rule labels: Item {
    quoted = \"plain\"
    single = 'one'
    word = bare-word
    triple = \"\"\"two words\"\"\"
    ticks = '''two words'''
    empty = \"\"
}
where {(quoted OR single) AND (word OR triple)}";
    let parser = parse_rule(input);
    assert_eq!(parser.conditions_count(), 6);
    assert_eq!(parser.condition_name(0), Some("quoted"));
    assert_eq!(parser.condition_value(0), Some("plain"));
    assert_eq!(parser.condition_name(1), Some("single"));
    assert_eq!(parser.condition_value(1), Some("one"));
    assert_eq!(parser.condition_name(2), Some("word"));
    assert_eq!(parser.condition_value(2), Some("bare-word"));
    assert_eq!(parser.condition_name(3), Some("triple"));
    assert_eq!(parser.condition_value(3), Some("two words"));
    assert_eq!(parser.condition_name(4), Some("ticks"));
    assert_eq!(parser.condition_value(4), Some("two words"));
    assert_eq!(parser.condition_name(5), Some("empty"));
    assert_eq!(parser.condition_value(5), Some(""));
    assert_eq!(parser.expr(), Some("(quoted OR single) AND (word OR triple)"));
}

#[test]
fn rule_parser_quoted_values_may_contain_braces_and_the_where_keyword() {
    let parser = parse_rule("rule r: Type { a = \"see } where {\" } where {a}");
    assert_eq!(parser.condition_value(0), Some("see } where {"));
    assert_eq!(parser.expr(), Some("a"));
}

#[test]
fn rule_parser_where_body_may_be_empty_and_leading_whitespace_is_skipped() {
    let empty = parse_rule("rule r: T { a = b } where {}");
    assert_eq!(empty.expr(), Some(""));

    let blank = parse_rule("rule r: T { a = b } where {\n\t  }");
    assert_eq!(blank.expr(), Some(""));

    let indented = parse_rule("rule r: T { a = b } where {\n  (a - 1) AND _b0}");
    assert_eq!(indented.expr(), Some("(a - 1) AND _b0"));
}

#[test]
fn rule_parser_where_expression_keeps_internal_whitespace() {
    let spaced = parse_rule("rule r: T { a = b } where {a OR b}");
    assert_eq!(spaced.expr(), Some("a OR b"));

    let padded = parse_rule("rule r: T { a = b } where {  a OR b }");
    assert_eq!(padded.expr(), Some("a OR b"));

    let broken = parse_rule("rule r: T { a = b } where {a\nOR\tb\r\n}");
    assert_eq!(broken.expr(), Some("a\nOR\tb"));
}

#[test]
fn rule_parser_stops_after_one_rule() {
    let parser = parse_rule("rule first: T { a = b } where {a} rule second: T { c = d } where {c}");
    assert_eq!(parser.rule_name(), Some("first"));
    assert_eq!(parser.conditions_count(), 1);
    assert_eq!(parser.condition_name(0), Some("a"));
    assert_eq!(parser.expr(), Some("a"));
}

#[test]
fn rule_parser_reports_syntax_errors() {
    let input = "not a rule";
    assert_rule_error(input, Error::ExpectingRuleKeyword(0, 1, input.to_string()));

    let input = "  not";
    assert_rule_error(input, Error::ExpectingRuleKeyword(2, 3, input.to_string()));

    let input = "rule";
    assert_rule_error(
        input,
        Error::ExpectingRuleName(input.len() as u32, input.len() as u32 + 1, input.to_string()),
    );

    let input = "rule : T { a = b } where {a}";
    let pos = input.find(':').unwrap() as u32;
    assert_rule_error(input, Error::ExpectingRuleName(pos, pos + 1, input.to_string()));

    let input = "rule 1name: T { a = b } where {a}";
    let pos = input.find('1').unwrap() as u32;
    assert_rule_error(input, Error::ExpectingRuleName(pos, pos + 1, input.to_string()));

    let input = "rule name T { a = b } where {a}";
    let pos = input.find(" T").unwrap() as u32 + 1;
    assert_rule_error(input, Error::ExpectingColon(pos, pos + 1, input.to_string()));

    let input = "rule name:";
    assert_rule_error(
        input,
        Error::ExpectingTypeName(input.len() as u32, input.len() as u32 + 1, input.to_string()),
    );

    let input = "rule name: { a = b } where {a}";
    let pos = input.find('{').unwrap() as u32;
    assert_rule_error(input, Error::ExpectingTypeName(pos, pos + 1, input.to_string()));

    let input = "rule name: 2Type { a = b } where {a}";
    let pos = input.find('2').unwrap() as u32;
    assert_rule_error(input, Error::ExpectingTypeName(pos, pos + 1, input.to_string()));

    let input = "rule name: Type a = b } where {a}";
    let pos = input.find(" a =").unwrap() as u32 + 1;
    assert_rule_error(
        input,
        Error::ExpectingOpeningBraceAfterTypeName(pos, pos + 1, input.to_string()),
    );

    let input = "rule name: Type {";
    assert_rule_error(
        input,
        Error::ExpectingClosingBrace(input.len() as u32, input.len() as u32 + 1, input.to_string()),
    );

    let input = "rule name: Type { } where {a}";
    let parser = parse_rule(input);
    assert_eq!(parser.conditions_count(), 0);
    assert_eq!(parser.expr(), Some("a"));

    let input = "rule name: Type { 9bad = x } where {a}";
    let pos = input.find('9').unwrap() as u32;
    assert_rule_error(input, Error::ExpectingKeyName(pos, pos + 1, input.to_string()));

    let input = "rule name: Type { good bad } where {a}";
    let pos = input.find("bad").unwrap() as u32;
    assert_rule_error(
        input,
        Error::ExpectingEqualSignAfterKeyName(pos, pos + 1, input.to_string()),
    );

    let input = "rule name: Type { good = } where {a}";
    let pos = input.find('}').unwrap() as u32;
    assert_rule_error(input, Error::ExpectingKeyValue(pos, pos + 1, input.to_string()));

    let input = "rule name: Type { good = >18 } where {a}";
    let pos = input.find('>').unwrap() as u32;
    assert_rule_error(input, Error::ExpectingKeyValue(pos, pos + 1, input.to_string()));

    let input = "rule name: Type { good = \"unterminated } where {a}";
    let pos = input.find('"').unwrap() as u32;
    assert_rule_error(input, Error::UnterminatedString(pos, pos + 1, input.to_string()));

    let input = "rule name: Type { good = 'unterminated } where {a}";
    let pos = input.find('\'').unwrap() as u32;
    assert_rule_error(input, Error::UnterminatedString(pos, pos + 1, input.to_string()));

    let input = "rule name: Type { good = \"\"\"";
    let pos = input.find('"').unwrap() as u32;
    assert_rule_error(input, Error::UnterminatedString(pos, pos + 1, input.to_string()));

    let input = "rule name: Type { good = bad } {a}";
    let pos = input.rfind('{').unwrap() as u32;
    assert_rule_error(input, Error::ExpectingWhereKeyword(pos, pos + 1, input.to_string()));

    let input = "rule name: Type { good = bad } where a}";
    let pos = input.find("where a").unwrap() as u32 + "where ".len() as u32;
    assert_rule_error(
        input,
        Error::ExpectingOpeningBraceAfterWhereKeyword(pos, pos + 1, input.to_string()),
    );

    for (input, bad) in [
        ("rule name: Type { good = bad } where {a[0]}", '['),
        ("rule name: Type { good = bad } where {a.b}", '.'),
    ] {
        let pos = input.find(bad).unwrap() as u32;
        assert_rule_error(
            input,
            Error::InvalidCharacterInWhereExpression(pos, pos + 1, input.to_string()),
        );
    }

    let input = "rule name: Type { good = bad } where {good";
    assert_rule_error(
        input,
        Error::ExpectingClosingBraceAfterWhereKeyword(
            input.len() as u32,
            input.len() as u32 + 1,
            input.to_string(),
        ),
    );

    let input = "rule name: Type { good = bad } where {";
    assert_rule_error(
        input,
        Error::ExpectingClosingBraceAfterWhereKeyword(
            input.len() as u32,
            input.len() as u32 + 1,
            input.to_string(),
        ),
    );
}

#[test]
fn rule_parser_reads_meta_arrays() {
    let input = r#"rule a: T { k = v } where {k} with meta { tags = [1,2,3] label = hello }"#;
    let parser = parse_rule(input);
    assert_eq!(parser.meta_count(), 2);
    assert_eq!(parser.meta_name(0), Some("tags"));
    assert_eq!(parser.meta_value(0), Some("[1,2,3]"));
    assert_eq!(parser.meta_name(1), Some("label"));
    assert_eq!(parser.meta_value(1), Some("hello"));

    // `]` and `}` inside quotes belong to the string, not the array.
    let input = r#"rule a: T { k = v } where {k} with meta { tags = [1, "]" , "}" , 2] }"#;
    let parser = parse_rule(input);
    assert_eq!(parser.meta_value(0), Some(r#"[1, "]" , "}" , 2]"#));

    let input = r#"rule a: T { k = v } where {k} with meta { tags = [1, ']', 2] }"#;
    let parser = parse_rule(input);
    assert_eq!(parser.meta_value(0), Some("[1, ']', 2]"));

    // An array is only valid in the meta block.
    let input = "rule a: T { k = [1, 2] } where {k}";
    let pos = input.find('[').unwrap() as u32;
    assert_rule_error(input, Error::ExpectingKeyValue(pos, pos + 1, input.to_string()));
}

#[test]
fn rule_parser_reports_meta_and_array_errors() {
    let input = "rule a: T { k = v } where {k} with extra";
    let pos = input.find("extra").unwrap() as u32;
    assert_rule_error(input, Error::ExpectingMetaKeyword(pos, pos + 1, input.to_string()));

    let input = "rule a: T { k = v } where {k} with meta extra";
    let pos = input.find("extra").unwrap() as u32;
    assert_rule_error(
        input,
        Error::ExpectingOpeningBraceAfterMetaKeyword(pos, pos + 1, input.to_string()),
    );

    let input = "rule a: T { k = v } where {k} with meta { tags = [1, 2";
    let pos = input.find('[').unwrap() as u32;
    assert_rule_error(input, Error::UnterminatedArray(pos, pos + 1, input.to_string()));

    let input = "rule a: T { k = v } where {k} with meta { tags = [1, 2}";
    let pos = input.rfind('}').unwrap() as u32;
    assert_rule_error(input, Error::UnexpectedClosingBraceInArray(pos, pos + 1, input.to_string()));

    let input = r#"rule a: T { k = v } where {k} with meta { tags = [1, "no end] }"#;
    let pos = input.find('"').unwrap() as u32;
    assert_rule_error(input, Error::UnterminatedString(pos, pos + 1, input.to_string()));

    let input = "rule a: T { k = v } where {k} with meta { tags = hello";
    assert_rule_error(
        input,
        Error::ExpectingClosingBrace(input.len() as u32, input.len() as u32 + 1, input.to_string()),
    );
}

#[test]
fn rule_parser_builds_each_meta_value() {
    let input = r#"
        rule review: Person {
            adult = "age >= 18"
        } where {adult} with meta {
            tags = [1, 2, 3]
            note = "see ] and }"
            flag = yes
            count = 18
            offset = -3
            ratio = 1.5
            fraction = .5
            plus = +3
            when = 12:30:00
            day = 2024-01-01
            path = name.family
            quoted = [1, "]" , "}" , 2]
            ticks = [1, ']', 2]
        }
    "#;
    let parser = parse_rule(input);
    assert_eq!(parser.meta_count(), 13);
    assert_eq!(
        (parser.meta_name(0), parser.meta_value(0)),
        (Some("tags"), Some("[1, 2, 3]"))
    );
    assert_eq!(
        (parser.meta_name(1), parser.meta_value(1)),
        (Some("note"), Some("see ] and }"))
    );
    assert_eq!(
        (parser.meta_name(2), parser.meta_value(2)),
        (Some("flag"), Some("yes"))
    );
    assert_eq!(
        (parser.meta_name(3), parser.meta_value(3)),
        (Some("count"), Some("18"))
    );
    assert_eq!(
        (parser.meta_name(4), parser.meta_value(4)),
        (Some("offset"), Some("-3"))
    );
    assert_eq!(
        (parser.meta_name(5), parser.meta_value(5)),
        (Some("ratio"), Some("1.5"))
    );
    assert_eq!(
        (parser.meta_name(6), parser.meta_value(6)),
        (Some("fraction"), Some(".5"))
    );
    assert_eq!(
        (parser.meta_name(7), parser.meta_value(7)),
        (Some("plus"), Some("+3"))
    );
    assert_eq!(
        (parser.meta_name(8), parser.meta_value(8)),
        (Some("when"), Some("12:30:00"))
    );
    assert_eq!(
        (parser.meta_name(9), parser.meta_value(9)),
        (Some("day"), Some("2024-01-01"))
    );
    assert_eq!(
        (parser.meta_name(10), parser.meta_value(10)),
        (Some("path"), Some("name.family"))
    );
    assert_eq!(
        (parser.meta_name(11), parser.meta_value(11)),
        (Some("quoted"), Some(r#"[1, "]" , "}" , 2]"#))
    );
    assert_eq!(
        (parser.meta_name(12), parser.meta_value(12)),
        (Some("ticks"), Some("[1, ']', 2]"))
    );
    assert_eq!(parser.condition_name(0), Some("adult"));
    assert_eq!(parser.condition_value(0), Some("age >= 18"));
    assert_eq!(parser.expr(), Some("adult"));
}

fn person(age: u32, label: &str) -> TestPerson {
    TestPerson { age, label: label.to_string() }
}

fn rule_err(rule: &str) -> Error {
    match Expression::from_rule_str::<TestPerson>(rule) {
        Ok(_) => panic!("expected an error for {rule:?}"),
        Err(err) => err,
    }
}

#[test]
fn from_rule_str_evaluates_and_or_and_not() {
    let rule = r#"
rule review: TestPerson {
    is-adult = "age >= 18"
    is-senior = "age >= 65"
    is-ada = "label is ada {ignore-case}"
} where {
    is-ada OR (is-adult AND NOT is-senior)
}
"#;
    let expr = Expression::from_rule_str::<TestPerson>(rule).expect("rule");

    let cases = [
        (person(10, "Ada"), true),
        (person(70, "ADA"), true),
        (person(30, "Bob"), true),
        (person(18, "bob"), true),
        (person(70, "Bob"), false),
        (person(10, "Bob"), false),
    ];
    for (obj, expected) in cases {
        assert_eq!(expr.evaluate(&obj), Some(expected), "age={} label={}", obj.age, obj.label);
        assert_eq!(expr.matches(&obj), expected, "age={} label={}", obj.age, obj.label);
    }
}

#[test]
fn from_rule_str_accepts_single_and_triple_quoted_conditions() {
    let single = "rule r: TestPerson { adult = 'age >= 18' } where { adult }";
    let expr = Expression::from_rule_str::<TestPerson>(single).expect("single quotes");
    assert_eq!(expr.evaluate(&person(18, "x")), Some(true));
    assert_eq!(expr.evaluate(&person(17, "x")), Some(false));

    let triple = "rule r: TestPerson { adult = \"\"\"age >= 18\"\"\" } where { NOT adult }";
    let expr = Expression::from_rule_str::<TestPerson>(triple).expect("triple quotes");
    assert!(!expr.matches(&person(40, "x")));
    assert!(expr.matches(&person(16, "x")));
}

#[test]
fn from_rule_str_reports_an_empty_rule_and_build_errors() {
    assert_eq!(rule_err(""), Error::ExpectingRuleKeyword(0, 1, String::new()));
    assert_eq!(rule_err(" \n\t"), Error::ExpectingRuleKeyword(0, 1, " \n\t".to_string()));

    let input = "rule r TestPerson { adult = \"age >= 18\" } where { adult }";
    let pos = input.find("TestPerson").unwrap() as u32;
    assert_eq!(rule_err(input), Error::ExpectingColon(pos, pos + 1, input.to_string()));

    let input = "rule r: TestPerson { adult = \"age >= 18\" } where { missing }";
    assert_eq!(
        rule_err(input),
        Error::UnknownConditionName(0, "missing".len() as u32, "missing".to_string())
    );

    assert_eq!(
        rule_err("rule r: TestPerson { adult = \"years >= 18\" } where {adult}"),
        Error::UnknownAttribute("years".to_string(), "adult".to_string())
    );

    assert_eq!(rule_err("rule r: TestPerson { adult = \"\" } where {adult}"), Error::EmptyCondition);

    assert_eq!(
        rule_err("rule r: TestPerson { adult = \"age >= 18\" } where {}"),
        Error::EmptyExpression
    );

    let input = "rule r: TestPerson { adult = \"age >= 18\" } where {adult @ senior}";
    let pos = input.find('@').unwrap() as u32;
    assert_eq!(
        rule_err(input),
        Error::InvalidCharacterInWhereExpression(pos, pos + 1, input.to_string())
    );
}

#[cfg(feature = "enable_type_check")]
#[test]
fn from_rule_str_rejects_a_different_type_name() {
    let rule = "rule r: Person { adult = \"age >= 18\" } where { adult }";
    assert_eq!(rule_err(rule), Error::TypeMismatch("Person".to_string(), "TestPerson".to_string()));
}

#[cfg(not(feature = "enable_type_check"))]
#[test]
fn from_rule_str_ignores_the_type_name_without_type_check() {
    let rule = "rule r: Person { adult = \"age >= 18\" } where { adult }";
    let expr = Expression::from_rule_str::<TestPerson>(rule).expect("rule");
    assert!(expr.matches(&person(21, "x")));
    assert!(!expr.matches(&person(15, "x")));
}

/// Builds an expression over `TriFlags` with conditions `c0`..`c5` (`ci` is "vi is true").
fn tri_build(expr: &str) -> Expression {
    let mut b = ExpressionBuilder::<TriFlags>::new();
    for i in 0..6 {
        b = b.add(&format!("c{i}"), Condition::from_str(&format!("v{i} is true")));
    }
    b.build(expr).unwrap()
}


fn unterminated_at(rule: &str, opening: &str) -> u32 {
    rule.find(opening).expect("opening quotes present") as u32
}

#[test]
fn rule_unterminated_triple_double_quote() {
    let rule = r#"rule a: TestPerson { x = """age >= 18 } where { x }"#;
    let res = Expression::from_rule_str::<TestPerson>(rule);
    let pos = unterminated_at(rule, r#"""""#);
    assert!(matches!(res, Err(Error::UnterminatedString(s, _, _)) if s == pos),
            "expected UnterminatedString at {pos}, got {:?}", res.err());
}

#[test]
fn rule_unterminated_triple_single_quote() {
    let rule = "rule a: TestPerson { x = '''age >= 18 } where { x }";
    let res = Expression::from_rule_str::<TestPerson>(rule);
    let pos = unterminated_at(rule, "'''");
    assert!(matches!(res, Err(Error::UnterminatedString(s, _, _)) if s == pos),
            "expected UnterminatedString at {pos}, got {:?}", res.err());
}

#[test]
fn rule_unterminated_triple_quote_near_end_of_text() {
    // fewer than three characters after the opening quotes
    let rule = r#"rule a: TestPerson { x = """ab"#;
    let res = Expression::from_rule_str::<TestPerson>(rule);
    let pos = unterminated_at(rule, r#"""""#);
    assert!(matches!(res, Err(Error::UnterminatedString(s, _, _)) if s == pos),
            "expected UnterminatedString at {pos}, got {:?}", res.err());
}

#[test]
fn rule_unterminated_single_quote_still_reported() {
    let rule = r#"rule a: TestPerson { x = "age >= 18 } where { x }"#;
    let res = Expression::from_rule_str::<TestPerson>(rule);
    let pos = unterminated_at(rule, "\"");
    assert!(matches!(res, Err(Error::UnterminatedString(s, _, _)) if s == pos),
            "expected UnterminatedString at {pos}, got {:?}", res.err());
}

#[test]
fn rule_triple_quoted_values_still_parse() {
    // regression: a value spanning lines, and one containing the other quote char
    let rule = r#"rule a: TestPerson {
        adult = """age >=
                   18"""
        named = '''label is "x"'''
    } where { adult AND named }"#;
    let ex = Expression::from_rule_str::<TestPerson>(rule).expect("valid rule");
    assert!(ex.matches(&TestPerson { age: 20, label: "x".into() }));
    assert!(!ex.matches(&TestPerson { age: 20, label: "y".into() }));
}

#[test]
fn rule_keyword_must_be_a_whole_word() {
    for rule in ["rulex: TestPerson { x = 'age >= 18' } where { x }",
                 "rule-x: TestPerson { x = 'age >= 18' } where { x }",
                 "rule_x: TestPerson { x = 'age >= 18' } where { x }",
                 "rule2: TestPerson { x = 'age >= 18' } where { x }"] {
        assert!(matches!(Expression::from_rule_str::<TestPerson>(rule),
                         Err(Error::ExpectingRuleKeyword(0, _, _))), "{rule}");
    }
}

#[test]
fn where_keyword_must_be_a_whole_word() {
    for rule in ["rule a: TestPerson { x = 'age >= 18' } wherex { x }",
                 "rule a: TestPerson { x = 'age >= 18' } where-x { x }"] {
        assert!(matches!(Expression::from_rule_str::<TestPerson>(rule),
                         Err(Error::ExpectingWhereKeyword(..))), "{rule}");
    }
}

#[test]
fn keywords_followed_by_whitespace_or_brace_still_work() {
    for rule in ["rule\ta: TestPerson { x = 'age >= 18' } where { x }",
                 "rule\na: TestPerson { x = 'age >= 18' } where { x }",
                 "rule a: TestPerson { x = 'age >= 18' }where{ x }"] {
        assert!(Expression::from_rule_str::<TestPerson>(rule).is_ok(), "{rule}");
    }
}

#[test]
fn rule_key_name_must_start_with_a_letter() {
    let rule = "rule a: TestPerson { _x = 'age >= 18' } where { _x }";
    let pos = rule.find("_x").unwrap() as u32;
    assert!(matches!(Expression::from_rule_str::<TestPerson>(rule),
                     Err(Error::ExpectingKeyName(s, _, _)) if s == pos));
}

#[test]
fn rule_name_must_start_with_a_letter() {
    let rule = "rule _a: TestPerson { x = 'age >= 18' } where { x }";
    let pos = rule.find("_a").unwrap() as u32;
    assert!(matches!(Expression::from_rule_str::<TestPerson>(rule),
                     Err(Error::ExpectingRuleName(s, _, _)) if s == pos));
}

#[test]
fn names_may_contain_underscore_after_the_first_letter() {
    let rule = "rule my_rule: TestPerson { is_adult = 'age >= 18' } where { is_adult }";
    assert!(Expression::from_rule_str::<TestPerson>(rule).is_ok());
}

#[test]
fn rule_where_accepts_symbol_operators() {
    // `&&`, `||`, `!` and `~` are accepted, like in ExpressionBuilder::build
    let symbols = Expression::from_rule_str::<TestPerson>(
        "rule r: TestPerson { a = 'age >= 18' b = 'age > 60' c = 'label is x' } where { (a && !b) || ~c }",
    )
    .expect("symbol operators");
    let words = Expression::from_rule_str::<TestPerson>(
        "rule r: TestPerson { a = 'age >= 18' b = 'age > 60' c = 'label is x' } where { (a AND NOT b) OR NOT c }",
    )
    .expect("word operators");
    for (age, label) in [(20, "x"), (20, "y"), (70, "x"), (70, "y"), (10, "x"), (10, "y")] {
        let p = TestPerson { age, label: label.into() };
        assert_eq!(symbols.evaluate(&p), words.evaluate(&p), "age={age} label={label}");
    }
}

#[test]
fn rule_where_symbol_errors_come_from_the_expression_parser() {
    // the symbols pass the rule parser; a misplaced operator is rejected by build()
    let res = Expression::from_rule_str::<TestPerson>(
        "rule r: TestPerson { a = 'age >= 18' b = 'age > 60' } where { a && && b }",
    );
    assert!(res.is_err());
    assert!(!matches!(res, Err(Error::InvalidCharacterInWhereExpression(..))));
}

// ---------------------------------------------------------------------------
// Expression::parse_rules
// ---------------------------------------------------------------------------

fn rule_names(text: &str) -> Vec<Result<String, Error>> {
    Expression::parse_rules::<TestPerson>(text).map(|r| r.map(|(name, _)| name)).collect()
}

#[test]
fn parse_rules_returns_every_rule_in_order() {
    let text = r#"
        rule adults: TestPerson { adult = "age >= 18" } where { adult }
        rule young:  TestPerson { kid = 'age < 18' } where { kid }
        rule tagged: TestPerson {
            x = """label is x"""
        } where { x }
    "#;
    let rules: Vec<(String, Expression)> =
        Expression::parse_rules::<TestPerson>(text).collect::<Result<_, _>>().expect("all rules compile");
    let names: Vec<&str> = rules.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["adults", "young", "tagged"]);
    let p = TestPerson { age: 20, label: "x".into() };
    assert!(rules[0].1.matches(&p));
    assert!(!rules[1].1.matches(&p));
    assert!(rules[2].1.matches(&p));
}

#[test]
fn parse_rules_collects_into_a_map() {
    let text = "rule a: TestPerson { x = 'age >= 18' } where { x } rule b: TestPerson { y = 'age < 18' } where { y }";
    let map: std::collections::HashMap<String, Expression> =
        Expression::parse_rules::<TestPerson>(text).collect::<Result<_, _>>().expect("compile");
    assert_eq!(map.len(), 2);
    assert!(map["a"].matches(&TestPerson { age: 30, label: String::new() }));
    assert!(map["b"].matches(&TestPerson { age: 10, label: String::new() }));
}

#[test]
fn parse_rules_skips_rules_for_other_types() {
    let text = "
        rule p1: Person     { x = 'age >= 18' } where { x }
        rule t1: TestPerson { x = 'age >= 18' } where { x }
        rule b1: Book       { x = 'price < 10' } where { x }
        rule t2: TestPerson { y = 'age < 18' } where { y }
    ";
    let names: Vec<String> = rule_names(text).into_iter().map(|r| r.expect("compile")).collect();
    assert_eq!(names, ["t1", "t2"]);
}

#[test]
fn parse_rules_type_names_are_compared_exactly() {
    let text = "rule a: testperson { x = 'age >= 18' } where { x }";
    assert!(rule_names(text).is_empty());
}

#[test]
fn parse_rules_continues_after_a_rule_that_does_not_compile() {
    let text = "
        rule good1: TestPerson { x = 'age >= 18' } where { x }
        rule bad1:  TestPerson { x = 'agee >= 18' } where { x }
        rule bad2:  TestPerson { x = 'age >= 18' } where { typo }
        rule good2: TestPerson { y = 'age < 18' } where { y }
    ";
    let items = rule_names(text);
    assert_eq!(items.len(), 4);
    assert_eq!(items[0].as_deref().ok(), Some("good1"));
    assert!(matches!(items[1], Err(Error::UnknownAttribute(..))), "{:?}", items[1]);
    assert!(matches!(items[2], Err(Error::UnknownConditionName(..))), "{:?}", items[2]);
    assert_eq!(items[3].as_deref().ok(), Some("good2"));
}

#[test]
fn parse_rules_stops_after_a_syntax_error() {
    let text = "
        rule good1: TestPerson { x = 'age >= 18' } where { x }
        rule broken TestPerson { x = 'age >= 18' } where { x }
        rule good2: TestPerson { y = 'age < 18' } where { y }
    ";
    let mut it = Expression::parse_rules::<TestPerson>(text);
    assert_eq!(it.next().map(|r| r.map(|(n, _)| n)).and_then(Result::ok).as_deref(), Some("good1"));
    let pos = text.find("TestPerson { x = 'age >= 18' } where { x }\n        rule good2").unwrap() as u32;
    assert!(matches!(it.next(), Some(Err(Error::ExpectingColon(s, _, _))) if s == pos));
    assert!(it.next().is_none());
    assert!(it.next().is_none(), "the iterator stays finished");
}

#[test]
fn parse_rules_reports_syntax_errors_in_rules_for_other_types_too() {
    // the type is only known once the rule has been read
    let text = "rule p: Person { x = 'age >= 18' where { x }";
    let items = rule_names(text);
    assert_eq!(items.len(), 1);
    assert!(items[0].is_err());
}

#[test]
fn parse_rules_allows_duplicate_rule_names() {
    let text = "rule a: TestPerson { x = 'age >= 18' } where { x } rule a: TestPerson { y = 'age < 18' } where { y }";
    let names: Vec<String> = rule_names(text).into_iter().map(|r| r.expect("compile")).collect();
    assert_eq!(names, ["a", "a"]);
    // collecting into a map keeps the last one
    let map: std::collections::HashMap<String, Expression> =
        Expression::parse_rules::<TestPerson>(text).collect::<Result<_, _>>().expect("compile");
    assert_eq!(map.len(), 1);
    assert!(map["a"].matches(&TestPerson { age: 10, label: String::new() }));
}

#[test]
fn parse_rules_needs_no_whitespace_between_rules() {
    let text = "rule a: TestPerson { x = 'age >= 18' } where {x}rule b: TestPerson { y = 'age < 18' } where {y}";
    let names: Vec<String> = rule_names(text).into_iter().map(|r| r.expect("compile")).collect();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn parse_rules_on_empty_text_yields_nothing() {
    assert!(rule_names("").is_empty());
    assert!(rule_names("  \n\t  ").is_empty());
}

#[test]
fn from_rule_str_still_reads_only_the_first_rule() {
    let text = "rule a: TestPerson { x = 'age >= 18' } where { x } rule b: TestPerson { y = 'age < 18' } where { y }";
    let expr = Expression::from_rule_str::<TestPerson>(text).expect("first rule");
    assert!(expr.matches(&TestPerson { age: 30, label: String::new() }));
}

#[test]
fn rule_parser_iter_yields_each_rule() {
    let text = r#"
        rule student-review: Student {
            age-is-prime = "age is-one-of [19, 23, 29, 31]"
            common       = "name.family ends-with 'escu' {ignore-case}"
        } where {
            age-is-prime OR common
        }
        rule adults: Person {
            adult = "age >= 18"
        } where { adult } with meta {
            tags = [1, 2, 3]
            note = "see ] and }"
        }
    "#;
    let mut parser = crate::RuleParser::new(text);
    let mut it = parser.iter();

    let first = it.next().expect("parse").expect("first rule");
    assert_eq!(first.rule_name(), "student-review");
    assert_eq!(first.type_name(), "Student");
    assert_eq!(first.expr(), "age-is-prime OR common");
    assert_eq!(first.conditions_count(), 2);
    assert_eq!(
        first.conditions().collect::<Vec<_>>(),
        vec![
            ("age-is-prime", "age is-one-of [19, 23, 29, 31]"),
            ("common", "name.family ends-with 'escu' {ignore-case}"),
        ]
    );
    assert_eq!(first.meta_count(), 0);
    assert_eq!(first.condition_name(2), None);
    assert_eq!(first.meta_name(0), None);
    drop(first);

    let second = it.next().expect("parse").expect("second rule");
    assert_eq!(second.rule_name(), "adults");
    assert_eq!(second.type_name(), "Person");
    assert_eq!(second.expr(), "adult");
    assert_eq!(second.condition_name(0), Some("adult"));
    assert_eq!(second.condition_value(0), Some("age >= 18"));
    assert_eq!(
        second.meta().collect::<Vec<_>>(),
        vec![("tags", "[1, 2, 3]"), ("note", "see ] and }")]
    );
    drop(second);

    assert!(it.next().expect("parse").is_none());
}

#[test]
fn rule_parser_iter_on_empty_text_yields_nothing() {
    for input in ["", "   ", "\n\t\r\n"] {
        let mut parser = crate::RuleParser::new(input);
        assert!(parser.iter().next().expect("parse").is_none(), "{input:?}");
        assert!(parser.iter().next().expect("parse").is_none(), "a second pass stays empty: {input:?}");
    }
}

#[test]
fn rule_parser_iter_continues_after_parse_next() {
    let text = "rule first: T { a = b } where {a} rule second: U { c = d } where {c}";
    let mut parser = RuleParser::new(text);
    assert_eq!(parser.parse_next(), Ok(true));
    assert_eq!(parser.rule_name(), Some("first"));
    let mut it = parser.iter();
    let second = it.next().expect("parse").expect("second rule");
    assert_eq!(second.rule_name(), "second");
    drop(second);
    assert!(it.next().expect("parse").is_none());
}

#[test]
fn rule_parser_iter_stops_after_a_syntax_error() {
    let text = "rule good: T { a = b } where {a} rule broken T { c = d } where {c} rule later: T { e = f } where {e}";
    let mut parser = RuleParser::new(text);
    let mut it = parser.iter();
    let good = it.next().expect("parse").expect("first rule");
    assert_eq!(good.rule_name(), "good");
    assert_eq!(good.expr(), "a");
    drop(good);
    let pos = text.find("T { c").unwrap() as u32;
    assert!(matches!(it.next(), Err(Error::ExpectingColon(start, _, _)) if start == pos));
    assert!(it.next().expect("finished").is_none());
    assert!(it.next().expect("finished").is_none(), "the iterator stays finished");
    drop(it);
    assert!(parser.iter().next().expect("finished").is_none(), "the broken text is not read again");
}

#[test]
fn rule_parser_iter_for_loop_reads_every_rule() {
    let text = "rule a: T { a = b } where {a}rule b: U { c = d } where {c}";
    let mut parser = RuleParser::new(text);
    let mut it = parser.iter();
    let mut names = Vec::new();
    while let Some(rule) = it.next().expect("parse") {
        names.push(rule.rule_name().to_string());
    }
    assert_eq!(names, ["a", "b"]);
}
