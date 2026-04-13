use super::attribute;
use super::modifiers;
use super::operation;
use super::values::{self, ParsedValue};
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
            (input.len() - 1) as u32,
            input.len() as u32,
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

    let start = input.find('{').unwrap() as u32;
    let end = (input.rfind('}').unwrap() + 1) as u32;
    assert_eq!(err, Error::EmptyModifierBlock(start, end, input.to_string()));
}

#[test]
fn parse_modifiers_returns_error_for_unknown_modifier() {
    let input = "status == active {unknown}";
    let err = match modifiers::parse(input) {
        Ok(_) => panic!("unknown modifier should fail"),
        Err(err) => err,
    };

    let start = (input.find('{').unwrap() + 1) as u32;
    let end = (input.rfind('}').unwrap()) as u32;
    assert_eq!(err, Error::UnknownModifier(start, end, input.to_string()));
}

#[test]
fn parse_modifiers_returns_error_for_unknown_modifier_with_valid_modifier() {
    let input = "status == active {ignore-case, unknown}";
    let err = match modifiers::parse(input) {
        Ok(_) => panic!("unknown modifier should fail"),
        Err(err) => err,
    };

    let end = input.rfind('}').unwrap() as u32;
    let start = (end-7) as u32;
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

    assert_eq!(err, Error::ExpectingOperation(start as u32, end as u32, input.to_string()));
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
        Error::UnknownOperation(start as u32, (start + "unknown".len()) as u32, input.to_string())
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

// --- cond_parser::values::parse ---

fn values_parse(txt: &str, start: usize, end: usize) -> Result<ParsedValue, Error> {
    let mut copy = String::new();
    values::parse(txt, start, end, &mut copy)
}

fn single_slice(txt: &str, start: usize, end: usize) -> String {
    let mut copy = String::new();
    match values::parse(txt, start, end, &mut copy).expect("parse ok") {
        ParsedValue::Single(sp) => sp.as_slice(txt, &copy).to_string(),
        ParsedValue::List(_) => panic!("expected ParsedValue::Single"),
    }
}

fn list_slices(txt: &str, start: usize, end: usize) -> Vec<String> {
    let mut copy = String::new();
    match values::parse(txt, start, end, &mut copy).expect("parse ok") {
        ParsedValue::List(spans) => spans.iter().map(|sp| sp.as_slice(txt, &copy).to_string()).collect(),
        ParsedValue::Single(_) => panic!("expected ParsedValue::List"),
    }
}

#[test]
fn values_parse_empty_slice_expects_value() {
    let err = values_parse("", 0, 0).unwrap_err();
    assert_eq!(err, Error::ExpectingAValue(0, 0, "".to_string()));
}

#[test]
fn values_parse_whitespace_only_expects_value() {
    let txt = "  \t\n  ";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(err, Error::ExpectingAValue(0, txt.len() as u32, txt.to_string()));
}

#[test]
fn values_parse_single_regular_word() {
    assert_eq!(single_slice("active", 0, 6), "active");
}

#[test]
fn values_parse_single_trims_outer_whitespace() {
    let txt = "  \tfoo\n  ";
    assert_eq!(single_slice(txt, 0, txt.len()), "foo");
}

#[test]
fn values_parse_single_regular_stops_at_whitespace_extra_is_error() {
    let txt = "foo bar";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::ExpectingASingleValue(4, txt.len() as u32, txt.to_string())
    );
}

#[test]
fn values_parse_single_regular_stops_at_comma_extra_is_error() {
    let txt = "a,b";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::ExpectingASingleValue(1, txt.len() as u32, txt.to_string())
    );
}

#[test]
fn values_parse_single_quoted_empty() {
    assert_eq!(single_slice("''", 0, 2), "");
}

#[test]
fn values_parse_single_quoted_with_spaces_and_apostrophe_like_content() {
    assert_eq!(single_slice("'a b'", 0, 5), "a b");
}

#[test]
fn values_parse_single_quoted_unterminated() {
    let txt = "'abc";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::UnterminatedString(0, txt.len() as u32, txt.to_string())
    );
}

#[test]
fn values_parse_single_quoted_only_opening_quote() {
    let txt = "'";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::UnterminatedString(0, txt.len() as u32, txt.to_string())
    );
}

#[test]
fn values_parse_double_quoted_empty() {
    assert_eq!(single_slice("\"\"", 0, 2), "");
}

#[test]
fn values_parse_double_quoted_plain() {
    assert_eq!(single_slice("\"hello\"", 0, 7), "hello");
}

#[test]
fn values_parse_double_quoted_unterminated() {
    let txt = "\"hello";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::UnterminatedString(0, txt.len() as u32, txt.to_string())
    );
}

#[test]
fn values_parse_double_quoted_backslash_at_end() {
    let txt = "\"x\\";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::UnterminatedString(2, txt.len() as u32, txt.to_string())
    );
}

#[test]
fn values_parse_double_quoted_invalid_escape_sequence() {
    let txt = "\"a\\z\"";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidEscapeSequence(2, 4, txt.to_string())
    );
}

#[test]
fn values_parse_double_quoted_unescape_quote_backslash_and_escapes() {
    let mut copy = String::new();
    let txt = "\"a\\\"b\\\\c\\n\\t\\r\"";
    match values::parse(txt, 0, txt.len(), &mut copy).unwrap() {
        ParsedValue::Single(sp) => assert_eq!(sp.as_slice(txt, &copy), "a\"b\\c\n\t\r"),
        ParsedValue::List(_) => panic!("expected single"),
    }
}

#[test]
fn values_parse_double_quoted_escaped_quote_uses_copy_buffer() {
    let mut copy = String::new();
    let txt = "\"a\\\"b\"";
    match values::parse(txt, 0, txt.len(), &mut copy).unwrap() {
        ParsedValue::Single(sp) => {
            assert_eq!(sp.as_slice(txt, &copy), "a\"b");
            assert_eq!(copy, "a\"b");
        }
        ParsedValue::List(_) => panic!("expected single"),
    }
}

#[test]
fn values_parse_double_quoted_without_backslash_leaves_copy_buffer_empty() {
    let mut copy = String::new();
    let txt = "\"plain\"";
    match values::parse(txt, 0, txt.len(), &mut copy).unwrap() {
        ParsedValue::Single(sp) => {
            assert_eq!(sp.as_slice(txt, &copy), "plain");
            assert!(copy.is_empty());
        }
        ParsedValue::List(_) => panic!("expected single"),
    }
}

#[test]
fn values_parse_list_two_unquoted_elements() {
    assert_eq!(list_slices("[a,b]", 0, 5), vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn values_parse_list_with_whitespace_and_three_elements() {
    let txt = "[ a , bb , ccc ]";
    assert_eq!(
        list_slices(txt, 0, txt.len()),
        vec!["a".to_string(), "bb".to_string(), "ccc".to_string()]
    );
}

#[test]
fn values_parse_empty_brackets() {
    let txt = "[]";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::EmptyArrayList(0, 1, txt.to_string())
    );
}

#[test]
fn values_parse_list_whitespace_only_inside_brackets() {
    let txt = "[ \t ]";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::EmptyArrayList(0, 4, txt.to_string())
    );
}

#[test]
fn values_parse_list_missing_comma_between_items() {
    let txt = "[a b]";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::ExpectedCommaOrEnd(3, 4, txt.to_string())
    );
}

#[test]
fn values_parse_missing_starting_bracket_before_close() {
    let txt = "x]";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::MissingStartingBracket(0, 3, txt.to_string())
    );
}

#[test]
fn values_parse_missing_ending_bracket() {
    let txt = "[x";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::MissingEndingBracket(0, 3, txt.to_string())
    );
}

#[test]
fn values_parse_slice_offset_in_full_expression() {
    let txt = "attr == hello";
    let start = 8;
    let end = txt.len();
    assert_eq!(single_slice(txt, start, end), "hello");
}

#[test]
fn values_parse_list_offset_in_full_expression() {
    let txt = "x in [a,b]";
    let start = 5;
    let end = txt.len();
    assert_eq!(list_slices(txt, start, end), vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn values_parse_list_single_quoted_value_with_comma_inside() {
    assert_eq!(list_slices("['a,b',c]", 0, 9), vec!["a,b".to_string(), "c".to_string()]);
}

#[test]
fn values_parse_list_double_quoted_with_comma_unescaped() {
    assert_eq!(list_slices("[\"x,y\",z]", 0, 9), vec!["x,y".to_string(), "z".to_string()]);
}

#[test]
fn values_parse_list_trailing_comma_after_one_element_ok() {
    assert_eq!(list_slices("[a,]", 0, 4), vec!["a".to_string()]);
}

#[test]
fn values_parse_list_leading_comma_yields_empty_first_element() {
    assert_eq!(list_slices("[,a]", 0, 4), vec!["".to_string(), "a".to_string()]);
}

#[test]
fn values_parse_regular_word_includes_non_ascii_bytes() {
    let txt = "café";
    assert_eq!(single_slice(txt, 0, txt.len()), "café");
}

#[test]
fn values_parse_copy_buffer_accumulates_across_escaped_double_quoted_parses() {
    let mut copy = String::new();
    let t1 = "\"a\\n\"";
    let t2 = "\"b\\t\"";
    let v1 = values::parse(t1, 0, t1.len(), &mut copy).unwrap();
    let v2 = values::parse(t2, 0, t2.len(), &mut copy).unwrap();
    match (v1, v2) {
        (ParsedValue::Single(s1), ParsedValue::Single(s2)) => {
            assert_eq!(s1.as_slice(t1, &copy), "a\n");
            assert_eq!(s2.as_slice(t2, &copy), "b\t");
        }
        _ => panic!("expected two singles"),
    }
    assert_eq!(copy, "a\nb\t");
}

#[test]
fn values_parse_outer_whitespace_around_bracketed_list() {
    let txt = "  [ 1 , 2 ]  ";
    assert_eq!(list_slices(txt, 0, txt.len()), vec!["1".to_string(), "2".to_string()]);
}

#[test]
fn values_parse_list_unterminated_single_quoted_element() {
    let txt = "[a,'bc]";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::UnterminatedString(3, 6, txt.to_string())
    );
}

#[test]
fn values_parse_list_unterminated_double_quoted_element() {
    let txt = "[\"x]";
    let err = values_parse(txt, 0, txt.len()).unwrap_err();
    assert_eq!(
        err,
        Error::UnterminatedString(1, 3, txt.to_string())
    );
}

#[test]
fn values_parse_list_double_quoted_element_with_escapes() {
    let mut copy = String::new();
    let txt = "[\"a\\nb\"]";
    match values::parse(txt, 0, txt.len(), &mut copy).unwrap() {
        ParsedValue::List(spans) => {
            assert_eq!(spans.len(), 1);
            assert_eq!(spans[0].as_slice(txt, &copy), "a\nb");
        }
        ParsedValue::Single(_) => panic!("expected list"),
    }
}

#[test]
fn values_parse_single_double_quoted_only_needs_unescape_flag_from_backslash_before_close() {
    let mut copy = String::new();
    let txt = "\"\\\\\"";
    match values::parse(txt, 0, txt.len(), &mut copy).unwrap() {
        ParsedValue::Single(sp) => assert_eq!(sp.as_slice(txt, &copy), "\\"),
        ParsedValue::List(_) => panic!("expected single"),
    }
}
