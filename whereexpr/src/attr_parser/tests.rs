use super::parse;
use crate::Error;

#[test]
fn parse_simple_attribute() {
    let input = "status == active";
    let (attribute, next_index) = parse(input).expect("attribute should parse");

    assert_eq!(attribute, "status");
    assert_eq!(next_index, 7);
}

#[test]
fn parse_attribute_after_leading_spaces() {
    let input = "   user_name   eq admin";
    let (attribute, next_index) = parse(input).expect("attribute should parse after spaces");

    assert_eq!(attribute, "user_name");
    assert_eq!(next_index, 15);
}

#[test]
fn parse_attribute_accepts_digits_underscore_and_dot() {
    let input = "a1_b.c  >= 42";
    let (attribute, next_index) = parse(input).expect("attribute should allow valid characters");

    assert_eq!(attribute, "a1_b.c");
    assert_eq!(next_index, 8);
}

#[test]
fn parse_attribute_stops_at_first_invalid_char() {
    let input = "name-value";
    let (attribute, next_index) = parse(input).expect("attribute should parse until invalid char");

    assert_eq!(attribute, "name");
    assert_eq!(next_index, 4);
}

#[test]
fn parse_returns_error_for_empty_input() {
    let input = "";
    let err = parse(input).expect_err("empty input should fail");

    assert_eq!(err, Error::EmptyCondition);
}

#[test]
fn parse_returns_error_when_first_non_space_is_not_letter() {
    let input = " 1name eq value";
    let err = parse(input).expect_err("attribute cannot start with non-letter");

    assert_eq!(err, Error::InvalidAttributeName(1, 2, input.to_string()));
}

#[test]
fn parse_returns_error_for_whitespace_only_input() {
    let input = "   ";
    let err = parse(input).expect_err("whitespace-only input should fail");

    assert_eq!(err, Error::EmptyCondition);
}
