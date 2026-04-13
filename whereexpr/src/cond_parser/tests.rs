use super::attribute;
use super::modifiers;
use super::operation;
use crate::Error;
use crate::Operation;

#[test]
fn parse_simple_attribute() {
    let input = "status == active";
    let (attribute, next_index) = attribute::parse(input).expect("attribute should parse");

    assert_eq!(attribute, "status");
    assert_eq!(next_index, 7);
}

#[test]
fn parse_attribute_after_leading_spaces() {
    let input = "   user_name   eq admin";
    let (attribute, next_index) = attribute::parse(input).expect("attribute should parse after spaces");

    assert_eq!(attribute, "user_name");
    assert_eq!(next_index, 15);
}

#[test]
fn parse_attribute_accepts_digits_underscore_and_dot() {
    let input = "a1_b.c  >= 42";
    let (attribute, next_index) = attribute::parse(input).expect("attribute should allow valid characters");

    assert_eq!(attribute, "a1_b.c");
    assert_eq!(next_index, 8);
}

#[test]
fn parse_attribute_stops_at_first_invalid_char() {
    let input = "name-value";
    let (attribute, next_index) = attribute::parse(input).expect("attribute should parse until invalid char");

    assert_eq!(attribute, "name");
    assert_eq!(next_index, 4);
}

#[test]
fn parse_returns_error_for_empty_input() {
    let input = "";
    let err = attribute::parse(input).expect_err("empty input should fail");

    assert_eq!(err, Error::EmptyCondition);
}

#[test]
fn parse_returns_error_when_first_non_space_is_not_letter() {
    let input = " 1name eq value";
    let err = attribute::parse(input).expect_err("attribute cannot start with non-letter");

    assert_eq!(err, Error::InvalidAttributeName(1, 2, input.to_string()));
}

#[test]
fn parse_returns_error_for_whitespace_only_input() {
    let input = "   ";
    let err = attribute::parse(input).expect_err("whitespace-only input should fail");

    assert_eq!(err, Error::EmptyCondition);
}

#[test]
fn parse_modifiers_returns_default_when_missing() {
    let input = "status == active";
    let (parsed, start_pos) = modifiers::parse(input).expect("missing modifiers should be accepted");

    assert!(!parsed.ignore_case);
    assert_eq!(start_pos, input.len());
}

#[test]
fn parse_modifiers_ignore_case() {
    let input = "status == active {ignore-case}";
    let (parsed, start_pos) = modifiers::parse(input).expect("ignore-case should parse");

    assert!(parsed.ignore_case);
    assert_eq!(start_pos, input.rfind('{').unwrap());
}

#[test]
fn parse_modifiers_ignore_case_with_trailing_spaces() {
    let input = "status == active {ignore-case}   ";
    let (parsed, start_pos) = modifiers::parse(input).expect("trailing spaces should be ignored");

    assert!(parsed.ignore_case);
    assert_eq!(start_pos, input.find('{').unwrap());
}

#[test]
fn parse_modifiers_returns_error_for_unmatched_closing_bracket() {
    let input = "status == active}";
    let err = match modifiers::parse(input) {
        Ok(_) => panic!("unmatched close bracket should fail"),
        Err(err) => err,
    };

    assert_eq!(
        err,
        Error::UnmatchedModifierBracket(
            (input.len() - 1) as u16,
            input.len() as u16,
            input.to_string()
        )
    );
}

#[test]
fn parse_modifiers_returns_error_for_empty_block() {
    let input = "status == active {}";
    let err = match modifiers::parse(input) {
        Ok(_) => panic!("empty modifier block should fail"),
        Err(err) => err,
    };

    let start = input.find('{').unwrap() as u16;
    let end = (input.rfind('}').unwrap() + 1) as u16;
    assert_eq!(err, Error::EmptyModifierBlock(start, end, input.to_string()));
}

#[test]
fn parse_modifiers_returns_error_for_unknown_modifier() {
    let input = "status == active {unknown}";
    let err = match modifiers::parse(input) {
        Ok(_) => panic!("unknown modifier should fail"),
        Err(err) => err,
    };

    let start = (input.find('{').unwrap() + 1) as u16;
    let end = (input.rfind('}').unwrap()) as u16;
    assert_eq!(err, Error::UnknownModifier(start, end, input.to_string()));
}

#[test]
fn parse_modifiers_returns_error_for_unknown_modifier_with_valid_modifier() {
    let input = "status == active {ignore-case, unknown}";
    let err = match modifiers::parse(input) {
        Ok(_) => panic!("unknown modifier should fail"),
        Err(err) => err,
    };

    let end = input.rfind('}').unwrap() as u16;
    let start = (end-7) as u16;
    assert_eq!(&input[start as usize..end as usize], "unknown");
    assert_eq!(err, Error::UnknownModifier(start, end, input.to_string()));
}

/// Every `(alias, Operation)` pair must match `cond_parser::operation::OPERATIONS` (same aliases, same mapping).
const OPERATION_ALIAS_CASES: &[(&str, Operation)] = &[
    // Is
    ("is", Operation::Is),
    ("==", Operation::Is),
    ("eq", Operation::Is),
    ("equals", Operation::Is),
    // IsNot
    ("isnot", Operation::IsNot),
    ("!=", Operation::IsNot),
    ("neq", Operation::IsNot),
    ("notequals", Operation::IsNot),
    // IsOneOf
    ("isoneof", Operation::IsOneOf),
    ("in", Operation::IsOneOf),
    // IsNotOneOf (includes typo alias `isnotonoeof` from the operation table)
    ("isnotonoeof", Operation::IsNotOneOf),
    ("notin", Operation::IsNotOneOf),
    // StartsWith
    ("startswith", Operation::StartsWith),
    // NotStartsWith
    ("notstartswith", Operation::NotStartsWith),
    // StartsWithOneOf
    ("startswithoneof", Operation::StartsWithOneOf),
    // NotStartsWithOneOf
    ("notstartswithoneof", Operation::NotStartsWithOneOf),
    // EndsWith
    ("endswith", Operation::EndsWith),
    // NotEndsWith
    ("notendswith", Operation::NotEndsWith),
    // EndsWithOneOf
    ("endswithoneof", Operation::EndsWithOneOf),
    // NotEndsWithOneOf
    ("notendswithoneof", Operation::NotEndsWithOneOf),
    // Contains
    ("contains", Operation::Contains),
    // NotContains
    ("notcontains", Operation::NotContains),
    // ContainsOneOf
    ("containsoneof", Operation::ContainsOneOf),
    // NotContainsOneOf
    ("notcontainsoneof", Operation::NotContainsOneOf),
    // GlobREMatch
    ("glob", Operation::GlobREMatch),
    ("globmatch", Operation::GlobREMatch),
    // NotGlobREMatch
    ("notglob", Operation::NotGlobREMatch),
    ("notglobmatch", Operation::NotGlobREMatch),
    // GreaterThan
    (">", Operation::GreaterThan),
    ("gt", Operation::GreaterThan),
    ("greaterthan", Operation::GreaterThan),
    // GreaterThanOrEqual
    (">=", Operation::GreaterThanOrEqual),
    ("gte", Operation::GreaterThanOrEqual),
    ("greaterthanorequal", Operation::GreaterThanOrEqual),
    // LessThan
    ("<", Operation::LessThan),
    ("lt", Operation::LessThan),
    ("lessthan", Operation::LessThan),
    // LessThanOrEqual
    ("<=", Operation::LessThanOrEqual),
    ("lte", Operation::LessThanOrEqual),
    ("lessthanorequal", Operation::LessThanOrEqual),
    // InRange
    ("inrange", Operation::InRange),
    // NotInRange
    ("notinrange", Operation::NotInRange),
];

#[test]
fn parse_operation_each_alias_maps_to_expected_operation() {
    for &(alias, expected) in OPERATION_ALIAS_CASES {
        let input = format!("a {alias} v");
        let start = 2usize;
        let end = input.len();
        let (op, value_start) = operation::parse(&input, start, end)
            .unwrap_or_else(|e| panic!("alias {alias:?} should parse: {e:?}"));

        assert_eq!(op, expected, "alias {alias:?}");
        assert_eq!(&input[value_start..], "v", "alias {alias:?} value_start");
    }
}

#[test]
fn parse_operation_each_variant_has_at_least_one_alias() {
    let all_variants = [
        Operation::Is,
        Operation::IsNot,
        Operation::IsOneOf,
        Operation::IsNotOneOf,
        Operation::StartsWith,
        Operation::NotStartsWith,
        Operation::StartsWithOneOf,
        Operation::NotStartsWithOneOf,
        Operation::EndsWith,
        Operation::NotEndsWith,
        Operation::EndsWithOneOf,
        Operation::NotEndsWithOneOf,
        Operation::Contains,
        Operation::NotContains,
        Operation::ContainsOneOf,
        Operation::NotContainsOneOf,
        Operation::GlobREMatch,
        Operation::NotGlobREMatch,
        Operation::GreaterThan,
        Operation::GreaterThanOrEqual,
        Operation::LessThan,
        Operation::LessThanOrEqual,
        Operation::InRange,
        Operation::NotInRange,
    ];

    for op in all_variants {
        assert!(
            OPERATION_ALIAS_CASES.iter().any(|(_, o)| *o == op),
            "Operation::{op:?} has no alias in OPERATION_ALIAS_CASES"
        );
    }
}

#[test]
fn parse_operation_skips_leading_whitespace_in_slice() {
    let input = "status   eq   value";
    let start = 6;
    let end = input.len();
    let (op, value_start) = operation::parse(input, start, end).expect("eq should parse");

    assert_eq!(op, Operation::Is);
    assert_eq!(&input[value_start..], "value");
}

#[test]
fn parse_operation_returns_error_for_whitespace_only_slice() {
    let input = "ab   cd";
    let start = 2;
    let end = 5;
    let err = match operation::parse(input, start, end) {
        Ok(_) => panic!("whitespace-only slice should fail"),
        Err(e) => e,
    };

    assert_eq!(err, Error::ExpectingOperation(start as u16, end as u16, input.to_string()));
}

#[test]
fn parse_operation_returns_error_for_unknown_token() {
    let input = "x unknown y";
    let start = 2;
    let end = input.len();
    let err = match operation::parse(input, start, end) {
        Ok(_) => panic!("unknown operation should fail"),
        Err(e) => e,
    };

    assert_eq!(
        err,
        Error::UnknownOperation(start as u16, (start + "unknown".len()) as u16, input.to_string())
    );
}

#[test]
fn parse_operation_returns_error_when_operation_starts_with_invalid_char() {
    let input = "a 1bad";
    let start = 1;
    let end = input.len();
    let err = match operation::parse(input, start, end) {
        Ok(_) => panic!("invalid leading char should fail"),
        Err(e) => e,
    };

    assert_eq!(
        err,
        Error::ExpectingOperation(2, 3, input.to_string())
    );
}
