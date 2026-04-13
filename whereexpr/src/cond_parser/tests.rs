use super::attribute;
use super::modifiers;
use crate::Error;

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
