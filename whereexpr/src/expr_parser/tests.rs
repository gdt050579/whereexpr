use crate::Error;
use crate::condition_list::ConditionList;
use crate::expression::{Composition, EvaluationNode};
use crate::{AttributeIndex, CompiledCondition, Operation, Predicate};
use super::parser::parse;
use super::redundancy_optimizations::{reduce_extra_wrapping, reduce_outermost_wrapping, reduce_parentheses, reduce_single_rule_wrapping};
use super::token::{Token, TokenKind, TokenSpan};
use super::tokenizer::tokenize;
use super::tokens_validator::{resolve_rule_names, validate_parentheses, validate_same_operation_per_level, validate_token_pairs};

/// Unresolved rule name as produced by `tokenize`.
const RN: TokenKind = TokenKind::ConditionIndex(u16::MAX);

fn condition_list_for_rule_tests(names: &[&str]) -> ConditionList {
    let mut list = ConditionList::new();
    for name in names {
        let p = Predicate::with_value(Operation::Is, "x").expect("predicate");
        assert!(list.add(name, CompiledCondition::new(AttributeIndex::new(0), p)), "duplicate name {name}");
    }
    list
}

fn parse_tokens(input: &str) -> Vec<Token> {
    tokenize(input).expect("tokenize")
}

fn kind_seq(tokens: &[Token]) -> Vec<TokenKind> {
    tokens.iter().map(|t| t.kind()).collect()
}

fn assert_tokens(input: &str, expected: &[(TokenKind, &str)]) {
    let got = tokenize(input).expect("tokenize");
    assert_eq!(got.len(), expected.len(), "token count mismatch for input {input:?}");
    for (i, (tok, &(kind, slice))) in got.iter().zip(expected.iter()).enumerate() {
        assert_eq!(tok.kind(), kind, "wrong kind at token {i} for input {input:?}");
        assert_eq!(tok.span().as_slice(input), slice, "wrong span text at token {i} for input {input:?}");
    }
}

#[test]
fn empty_and_whitespace_only_yields_empty_input() {
    for s in ["", " ", "\t", "\n", "\r", "\r\n", "  \t\n\r  "] {
        assert!(matches!(tokenize(s), Err(Error::EmptyExpression)), "input: {s:?}");
    }
}

#[test]
fn rule_too_long_when_len_exceeds_0x7fff() {
    let s = "a".repeat(0x7FFF + 1);
    assert_eq!(s.len(), 0x8000);
    assert!(matches!(tokenize(&s), Err(Error::ExpressioTooLong)));
}

#[test]
fn max_allowed_len_succeeds() {
    let s = "a".repeat(0x7FFF);
    assert_eq!(s.len(), 0x7FFF);
    let tokens = tokenize(&s).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind(), TokenKind::ConditionIndex(u16::MAX));
    assert_eq!(tokens[0].span().as_slice(&s), &s);
}

#[test]
fn unexpected_char_for_invalid_byte() {
    let cases = [
        ("0", 0usize),
        ("9", 0),
        (".", 0),
        (",", 0),
        ("@", 0),
        ("%", 0),
        ("[", 0),
        ("\"", 0),
        ("a+", 1),
        ("foo bar#", 7),
    ];
    for (input, pos) in cases {
        match tokenize(input) {
            Err(Error::UnexpectedChar(start,end,text)) => {
                assert_eq!(&text[start as usize..end as usize], &input[pos..pos + 1]);
            }
            other => panic!("expected UnexpectedChar at {pos} for {input:?}, got {other:?}"),
        }
    }
}

#[test]
fn parentheses() {
    assert_tokens("()", &[(TokenKind::LParen, "("), (TokenKind::RParen, ")")]);
    assert_tokens("( )", &[(TokenKind::LParen, "("), (TokenKind::RParen, ")")]);
}

#[test]
fn not_operators_bang_and_tilde() {
    assert_tokens("!", &[(TokenKind::Not, "!")]);
    assert_tokens("~", &[(TokenKind::Not, "~")]);
    assert_tokens("! ~", &[(TokenKind::Not, "!"), (TokenKind::Not, "~")]);
}

#[test]
fn and_ascii_single_and_double() {
    assert_tokens("&", &[(TokenKind::And, "&")]);
    assert_tokens("&&", &[(TokenKind::And, "&&")]);
    assert_tokens("& &", &[(TokenKind::And, "&"), (TokenKind::And, "&")]);
    assert_tokens("&&&", &[(TokenKind::And, "&&"), (TokenKind::And, "&")]);
}

#[test]
fn or_ascii_single_and_double() {
    assert_tokens("|", &[(TokenKind::Or, "|")]);
    assert_tokens("||", &[(TokenKind::Or, "||")]);
    assert_tokens("| |", &[(TokenKind::Or, "|"), (TokenKind::Or, "|")]);
    assert_tokens("|||", &[(TokenKind::Or, "||"), (TokenKind::Or, "|")]);
}

#[test]
fn keyword_or_two_letters_case_insensitive() {
    for word in ["or", "OR", "Or", "oR"] {
        assert_tokens(word, &[(TokenKind::Or, word)]);
    }
}

#[test]
fn keyword_and_three_letters_case_insensitive() {
    for word in ["and", "AND", "And", "aNd"] {
        assert_tokens(word, &[(TokenKind::And, word)]);
    }
}

#[test]
fn keyword_not_three_letters_case_insensitive() {
    for word in ["not", "NOT", "Not", "nOt"] {
        assert_tokens(word, &[(TokenKind::Not, word)]);
    }
}

#[test]
fn longer_words_are_rule_names_not_keywords() {
    assert_tokens("orange", &[(TokenKind::ConditionIndex(u16::MAX), "orange")]);
    assert_tokens("android", &[(TokenKind::ConditionIndex(u16::MAX), "android")]);
    assert_tokens("note", &[(TokenKind::ConditionIndex(u16::MAX), "note")]);
    assert_tokens("nothing", &[(TokenKind::ConditionIndex(u16::MAX), "nothing")]);
    assert_tokens("or_", &[(TokenKind::ConditionIndex(u16::MAX), "or_")]);
    assert_tokens("_or", &[(TokenKind::ConditionIndex(u16::MAX), "_or")]);
    assert_tokens("and_", &[(TokenKind::ConditionIndex(u16::MAX), "and_")]);
    assert_tokens("orca", &[(TokenKind::ConditionIndex(u16::MAX), "orca")]);
}

#[test]
fn rule_names_alphanumeric_and_underscore() {
    assert_tokens("a", &[(TokenKind::ConditionIndex(u16::MAX), "a")]);
    assert_tokens("_", &[(TokenKind::ConditionIndex(u16::MAX), "_")]);
    assert_tokens("_foo", &[(TokenKind::ConditionIndex(u16::MAX), "_foo")]);
    assert_tokens("Rule_123", &[(TokenKind::ConditionIndex(u16::MAX), "Rule_123")]);
    assert_tokens("x", &[(TokenKind::ConditionIndex(u16::MAX), "x")]);
}

#[test]
fn keyword_vs_operator_precedence_in_stream() {
    // "or" tokenizes as keyword Or, not | |
    assert_tokens(
        "a or b",
        &[
            (TokenKind::ConditionIndex(u16::MAX), "a"),
            (TokenKind::Or, "or"),
            (TokenKind::ConditionIndex(u16::MAX), "b"),
        ],
    );
    assert_tokens(
        "a|b",
        &[
            (TokenKind::ConditionIndex(u16::MAX), "a"),
            (TokenKind::Or, "|"),
            (TokenKind::ConditionIndex(u16::MAX), "b"),
        ],
    );
}

#[test]
fn adjacent_tokens_without_whitespace() {
    assert_tokens(
        "a&b",
        &[
            (TokenKind::ConditionIndex(u16::MAX), "a"),
            (TokenKind::And, "&"),
            (TokenKind::ConditionIndex(u16::MAX), "b"),
        ],
    );
    assert_tokens("!foo", &[(TokenKind::Not, "!"), (TokenKind::ConditionIndex(u16::MAX), "foo")]);
    assert_tokens(
        "(x)",
        &[(TokenKind::LParen, "("), (TokenKind::ConditionIndex(u16::MAX), "x"), (TokenKind::RParen, ")")],
    );
}

#[test]
fn double_keyword_lookalikes_are_rule_names() {
    assert_tokens("oror", &[(TokenKind::ConditionIndex(u16::MAX), "oror")]);
    assert_tokens("notnot", &[(TokenKind::ConditionIndex(u16::MAX), "notnot")]);
    assert_tokens("andand", &[(TokenKind::ConditionIndex(u16::MAX), "andand")]);
}

#[test]
fn leading_trailing_whitespace_around_rule_name_skipped() {
    assert_tokens("  foo  ", &[(TokenKind::ConditionIndex(u16::MAX), "foo")]);
}

#[test]
fn mixed_expression() {
    assert_tokens(
        "!(a && b) || c",
        &[
            (TokenKind::Not, "!"),
            (TokenKind::LParen, "("),
            (TokenKind::ConditionIndex(u16::MAX), "a"),
            (TokenKind::And, "&&"),
            (TokenKind::ConditionIndex(u16::MAX), "b"),
            (TokenKind::RParen, ")"),
            (TokenKind::Or, "||"),
            (TokenKind::ConditionIndex(u16::MAX), "c"),
        ],
    );
}

#[test]
fn all_whitespace_variants_skipped() {
    assert_tokens(
        "a \t\n\r b",
        &[(TokenKind::ConditionIndex(u16::MAX), "a"), (TokenKind::ConditionIndex(u16::MAX), "b")],
    );
}

#[test]
fn utf8_non_ascii_first_byte_is_unexpected() {
    // First UTF-8 byte is not a valid token; span is a single byte (may not align with char boundaries).
    assert!(matches!(tokenize("€"), Err(Error::UnexpectedChar(_,_,_))));
}

// --- validate_parentheses ---

#[test]
fn validate_parentheses_empty_token_list_ok() {
    assert!(validate_parentheses(&[],  "").is_ok());
}

#[test]
fn validate_parentheses_no_parentheses_ok() {
    for input in ["a", "a && b", "!x", "not foo"] {
        let tokens = tokenize(input).expect("tokenize");
        validate_parentheses(&tokens,  input).expect("parens");
    }
}

#[test]
fn validate_parentheses_balanced_simple_and_nested_ok() {
    for input in ["()", "( )", "((a))", "((a) && b)", "(a)(b)", "(()())", "!(a || (b && c))"] {
        let tokens = tokenize(input).expect("tokenize");
        validate_parentheses(&tokens,  input).unwrap_or_else(|e| panic!("{input:?}: {e:?}"));
    }
}

#[test]
fn validate_parentheses_unclosed_reports_opening_span() {
    let input = "(";
    let tokens = tokenize(input).unwrap();
    match validate_parentheses(&tokens,  input) {
        Err(Error::UnclosedParenthesis(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "("),
        e => panic!("expected UnclosedParenthesis, got {e:?}"),
    }

    let input = "(a";
    let tokens = tokenize(input).unwrap();
    match validate_parentheses(&tokens,  input) {
        Err(Error::UnclosedParenthesis(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "("),
        e => panic!("expected UnclosedParenthesis, got {e:?}"),
    }

    // `( ( )` → outer `(` still open; error points at that outer `(`.
    let input = "(()";
    let tokens = tokenize(input).unwrap();
    match validate_parentheses(&tokens,  input) {
        Err(Error::UnclosedParenthesis(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "("),
        e => panic!("expected UnclosedParenthesis, got {e:?}"),
    }

    let input = "(a)(b";
    let tokens = tokenize(input).unwrap();
    let _second_open = tokens.iter().filter(|t| t.kind() == TokenKind::LParen).nth(1).expect("second (");
    match validate_parentheses(&tokens,  input) {
        Err(Error::UnclosedParenthesis(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "("),
        e => panic!("expected UnclosedParenthesis, got {e:?}"),
    }
}

#[test]
fn validate_parentheses_unexpected_close_reports_closing_span() {
    let input = ")";
    let tokens = tokenize(input).unwrap();
    match validate_parentheses(&tokens,  input) {
        Err(Error::UnexpectedCloseParen(start,end,text)) => assert_eq!(&text[start as usize..end as usize], ")"),
        e => panic!("expected UnexpectedCloseParen, got {e:?}"),
    }

    let input = "a)";
    let tokens = tokenize(input).unwrap();
    match validate_parentheses(&tokens,  input) {
        Err(Error::UnexpectedCloseParen(start,end,text)) => assert_eq!(&text[start as usize..end as usize], ")"),
        e => panic!("expected UnexpectedCloseParen, got {e:?}"),
    }

    let input = "())";
    let tokens = tokenize(input).unwrap();
    match validate_parentheses(&tokens,  input) {
        Err(Error::UnexpectedCloseParen(start,end,text)) => assert_eq!(&text[start as usize..end as usize], ")"),
        e => panic!("expected UnexpectedCloseParen, got {e:?}"),
    }

    let input = ")(";
    let tokens = tokenize(input).unwrap();
    match validate_parentheses(&tokens,  input) {
        Err(Error::UnexpectedCloseParen(start,end,text)) => assert_eq!(&text[start as usize..end as usize], ")"),
        e => panic!("expected UnexpectedCloseParen, got {e:?}"),
    }
}

#[test]
fn validate_parentheses_max_nesting_depth_ok() {
    // Exactly 8 nested `(` before content, then 8 `)` — depth tops out at 8 opens.
    let input = "((((((((a))))))))";
    let tokens = tokenize(input).unwrap();
    validate_parentheses(&tokens,  input).expect("8-deep nesting should be valid");
}

#[test]
fn validate_parentheses_ninth_open_exceeds_max_depth() {
    let input = "(((((((((a))))))))"; // 9 opens, 8 closes — 9th `(` is rejected
    let tokens = tokenize(input).unwrap();
    match validate_parentheses(&tokens,  input) {
        Err(Error::MaxParenDepthExceeded(start,end,text)) => assert_eq!(&text[start as usize..end as usize], &input[8..9]),
        e => panic!("expected MaxParenDepthExceeded, got {e:?}"),
    }
}

#[test]
fn validate_parentheses_token_stream_with_only_parens_matches_depth_logic() {
    // Direct token slice: no rule names needed for paren validation.
    let t = |kind, i| Token::new(kind, i, i + 1);
    assert!(validate_parentheses(&[t(TokenKind::LParen, 0), t(TokenKind::RParen, 1)],  "").is_ok());
    assert!(matches!(
        validate_parentheses(&[t(TokenKind::RParen, 0)],  ""),
        Err(Error::UnexpectedCloseParen(_,_,_))
    ));
    assert!(matches!(
        validate_parentheses(&[t(TokenKind::LParen, 0)],  ""),
        Err(Error::UnclosedParenthesis(_,_,_))
    ));
}

// --- resolve_rule_names ---

#[test]
fn resolve_rule_names_empty_slice_ok() {
    let mut tokens: Vec<Token> = Vec::new();
    resolve_rule_names(&mut tokens,  "", &ConditionList::new()).unwrap();
}

#[test]
fn resolve_rule_names_no_unresolved_rule_tokens_resolve_not_called() {
    let input = "||";
    let mut tokens = tokenize(input).unwrap();
    resolve_rule_names(&mut tokens,  input, &ConditionList::new()).unwrap();
}

#[test]
fn resolve_rule_names_single_rule_replaces_sentinel_with_index() {
    let input = "foo";
    let mut tokens = tokenize(input).unwrap();
    let conditions = condition_list_for_rule_tests(&["foo"]);
    resolve_rule_names(&mut tokens, input, &conditions).unwrap();
    assert_eq!(tokens[0].kind(), TokenKind::ConditionIndex(0));
    assert_eq!(tokens[0].span().as_slice(input), "foo");
}

#[test]
fn resolve_rule_names_multiple_distinct_names() {
    let input = "foo && bar";
    let mut tokens = tokenize(input).unwrap();
    let conditions = condition_list_for_rule_tests(&["foo", "bar"]);
    resolve_rule_names(&mut tokens, input, &conditions).unwrap();
    assert_eq!(tokens[0].kind(), TokenKind::ConditionIndex(0));
    assert_eq!(tokens[1].kind(), TokenKind::And);
    assert_eq!(tokens[2].kind(), TokenKind::ConditionIndex(1));
}

#[test]
fn resolve_rule_names_unknown_rule_reports_that_span() {
    let input = "known && mystery";
    let mut tokens = tokenize(input).unwrap();
    let conditions = condition_list_for_rule_tests(&["known"]);
    match resolve_rule_names(&mut tokens, input, &conditions) {
        Err(Error::UnknownConditionName(start, end, text)) => assert_eq!(&text[start as usize..end as usize], "mystery"),
        o => panic!("expected UnknownRuleName, got {o:?}"),
    }
}

#[test]
fn resolve_rule_names_fails_on_first_unknown_in_scan_order() {
    let input = "bad && good";
    let mut tokens = tokenize(input).unwrap();
    let conditions = condition_list_for_rule_tests(&["good"]);
    match resolve_rule_names(&mut tokens, input, &conditions) {
        Err(Error::UnknownConditionName(start, end, text)) => assert_eq!(&text[start as usize..end as usize], "bad"),
        o => panic!("expected UnknownRuleName, got {o:?}"),
    }
}

#[test]
fn resolve_rule_names_leaves_keyword_or_untouched() {
    let input = "a or b";
    let mut tokens = tokenize(input).unwrap();
    let conditions = condition_list_for_rule_tests(&["a", "b"]);
    resolve_rule_names(&mut tokens, input, &conditions).unwrap();
    assert_eq!(tokens[0].kind(), TokenKind::ConditionIndex(0));
    assert_eq!(tokens[1].kind(), TokenKind::Or);
    assert_eq!(tokens[2].kind(), TokenKind::ConditionIndex(1));
}

#[test]
fn resolve_rule_names_skips_already_resolved_rule_indices() {
    let input = "x";
    let mut tokens = vec![Token::new(TokenKind::ConditionIndex(7), 0, 1)];
    resolve_rule_names(&mut tokens, input, &ConditionList::new()).unwrap();
    assert_eq!(tokens[0].kind(), TokenKind::ConditionIndex(7));
}

#[test]
fn resolve_rule_names_repeated_name_resolved_each_occurrence() {
    let input = "x && x";
    let mut tokens = tokenize(input).unwrap();
    let conditions = condition_list_for_rule_tests(&["x"]);
    resolve_rule_names(&mut tokens, input, &conditions).unwrap();
    assert_eq!(tokens[0].kind(), TokenKind::ConditionIndex(0));
    assert_eq!(tokens[2].kind(), TokenKind::ConditionIndex(0));
}

#[test]
fn resolve_rule_names_complex_expression() {
    let input = "!(a && b) || c";
    let mut tokens = tokenize(input).unwrap();
    let conditions = condition_list_for_rule_tests(&["a", "b", "c"]);
    resolve_rule_names(&mut tokens, input, &conditions).unwrap();
    assert_eq!(tokens[2].kind(), TokenKind::ConditionIndex(0));
    assert_eq!(tokens[4].kind(), TokenKind::ConditionIndex(1));
    assert_eq!(tokens[7].kind(), TokenKind::ConditionIndex(2));
}

// --- validate_token_pairs ---

fn assert_pairs_ok(input: &str) {
    let tokens = tokenize(input).expect("tokenize");
    validate_token_pairs(&tokens,  input).unwrap_or_else(|e| panic!("{input:?}: {e:?}"));
}

#[test]
fn validate_token_pairs_single_rule_ok() {
    assert_pairs_ok("x");
}

#[test]
fn validate_token_pairs_valid_expressions_ok() {
    for input in [
        "a && b",
        "a || b",
        "a or b",
        "not x",
        "!a",
        "! a",
        "(a)",
        "!(a)",
        "!(a && b)",
        "(a) && (b)",
        "((a))",
        "a && b || c",
        "(a || b) && c",
        "! ( a && b )",
    ] {
        assert_pairs_ok(input);
    }
}

#[test]
fn validate_token_pairs_unexpected_token_at_start() {
    for (input, expected_slice) in [
        ("& a", "&"),
        ("&& a", "&&"),
        ("| x", "|"),
        ("|| x", "||"),
        ("or z", "or"),
        ("and z", "and"),
        (")", ")"),
    ] {
        let tokens = tokenize(input).unwrap();
        match validate_token_pairs(&tokens,  input) {
            Err(Error::UnexpectedTokenAtStart(start,end,text)) => {
                assert_eq!(&text[start as usize..end as usize], expected_slice, "input {input:?}");
            }
            e => panic!("input {input:?}: expected UnexpectedTokenAtStart, got {e:?}"),
        }
    }
}

#[test]
fn validate_token_pairs_unexpected_token_at_end() {
    let input = "a &&";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::UnexpectedTokenAtEnd(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "&&"),
        e => panic!("{e:?}"),
    }

    let input = "x or";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::UnexpectedTokenAtEnd(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "or"),
        e => panic!("{e:?}"),
    }

    let input = "x !";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::UnexpectedTokenAtEnd(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "!"),
        e => panic!("{e:?}"),
    }

    let input = "x && (";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::UnexpectedTokenAtEnd(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "("),
        e => panic!("{e:?}"),
    }
}

#[test]
fn validate_token_pairs_missing_operator() {
    let input = "a b";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::MissingOperator(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "b"),
        e => panic!("{e:?}"),
    }

    let input = "a (b)";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::MissingOperator(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "("),
        e => panic!("{e:?}"),
    }

    let input = "a not b";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::MissingOperator(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "not"),
        e => panic!("{e:?}"),
    }

    let input = "(a)b";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::MissingOperator(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "b"),
        e => panic!("{e:?}"),
    }

    // `!` after `)` needs an operator between sub-expressions; `(a)!` alone fails earlier (ends with `!`).
    let input = "(a) ! b";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::MissingOperator(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "!"),
        e => panic!("{e:?}"),
    }

    let input = "(a)(b)";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::MissingOperator(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "("),
        e => panic!("{e:?}"),
    }
}

#[test]
fn validate_token_pairs_missing_operand() {
    let input = "a && && b";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::MissingOperand(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "&&"),
        e => panic!("{e:?}"),
    }

    let input = "a || || b";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::MissingOperand(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "||"),
        e => panic!("{e:?}"),
    }

    let input = "(a) && )";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::MissingOperand(start,end,text)) => assert_eq!(&text[start as usize..end as usize], ")"),
        e => panic!("{e:?}"),
    }

    let input = "(a) || )";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::MissingOperand(start,end,text)) => assert_eq!(&text[start as usize..end as usize], ")"),
        e => panic!("{e:?}"),
    }
}

#[test]
fn validate_token_pairs_double_negation() {
    let input = "! ! a";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::DoubleNegation(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "!"),
        e => panic!("{e:?}"),
    }
}

#[test]
fn validate_token_pairs_negation_of_operator() {
    let input = "! && a";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::NegationOfOperator(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "&&"),
        e => panic!("{e:?}"),
    }

    let input = "! or a";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::NegationOfOperator(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "or"),
        e => panic!("{e:?}"),
    }
}

#[test]
fn validate_token_pairs_negation_of_close_paren() {
    let input = "!)";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::NegationOfCloseParen(start,end,text)) => assert_eq!(&text[start as usize..end as usize], ")"),
        e => panic!("{e:?}"),
    }
}

#[test]
fn validate_token_pairs_empty_parenthesis() {
    let input = "()";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::EmptyParenthesis(start,end,text)) => assert_eq!(&text[start as usize..end as usize], ")"),
        e => panic!("{e:?}"),
    }
}

#[test]
fn validate_token_pairs_operator_after_open_paren() {
    let input = "( && a";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::OperatorAfterOpenParen(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "&&"),
        e => panic!("{e:?}"),
    }

    let input = "( || a";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::OperatorAfterOpenParen(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "||"),
        e => panic!("{e:?}"),
    }

    let input = "( or a";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::OperatorAfterOpenParen(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "or"),
        e => panic!("{e:?}"),
    }

    let input = "( and a";
    let tokens = tokenize(input).unwrap();
    match validate_token_pairs(&tokens,  input) {
        Err(Error::OperatorAfterOpenParen(start,end,text)) => assert_eq!(&text[start as usize..end as usize], "and"),
        e => panic!("{e:?}"),
    }
}

// --- parser::parse ---

fn parser_tok(kind: TokenKind, i: usize) -> Token {
    Token::new(kind, i, i + 1)
}

#[test]
fn parse_single_rule() {
    let tokens = [parser_tok(TokenKind::ConditionIndex(0), 0)];
    assert_eq!(parse(&tokens), EvaluationNode::Condition(0));
}

#[test]
fn parse_and_chain() {
    let tokens = [
        parser_tok(TokenKind::ConditionIndex(1), 0),
        parser_tok(TokenKind::And, 1),
        parser_tok(TokenKind::ConditionIndex(2), 2),
        parser_tok(TokenKind::And, 3),
        parser_tok(TokenKind::ConditionIndex(3), 4),
    ];
    assert_eq!(
        parse(&tokens),
        EvaluationNode::Group {
            composition: Composition::And,
            children: vec![
                EvaluationNode::Condition(1),
                EvaluationNode::Condition(2),
                EvaluationNode::Condition(3),
            ],
        }
    );
}

#[test]
fn parse_or_chain() {
    let tokens = [
        parser_tok(TokenKind::ConditionIndex(10), 0),
        parser_tok(TokenKind::Or, 1),
        parser_tok(TokenKind::ConditionIndex(11), 2),
    ];
    assert_eq!(
        parse(&tokens),
        EvaluationNode::Group {
            composition: Composition::Or,
            children: vec![EvaluationNode::Condition(10), EvaluationNode::Condition(11)],
        }
    );
}

#[test]
fn parse_or_binds_looser_left_of_and() {
    // a || b && c  →  Or { a, And { b, c } }
    let tokens = [
        parser_tok(TokenKind::ConditionIndex(0), 0),
        parser_tok(TokenKind::Or, 1),
        parser_tok(TokenKind::ConditionIndex(1), 2),
        parser_tok(TokenKind::And, 3),
        parser_tok(TokenKind::ConditionIndex(2), 4),
    ];
    assert_eq!(
        parse(&tokens),
        EvaluationNode::Group {
            composition: Composition::Or,
            children: vec![
                EvaluationNode::Condition(0),
                EvaluationNode::Group {
                    composition: Composition::And,
                    children: vec![EvaluationNode::Condition(1), EvaluationNode::Condition(2)],
                },
            ],
        }
    );
}

#[test]
fn parse_parentheses_override_precedence() {
    // (a || b) && c
    let tokens = [
        parser_tok(TokenKind::LParen, 0),
        parser_tok(TokenKind::ConditionIndex(0), 1),
        parser_tok(TokenKind::Or, 2),
        parser_tok(TokenKind::ConditionIndex(1), 3),
        parser_tok(TokenKind::RParen, 4),
        parser_tok(TokenKind::And, 5),
        parser_tok(TokenKind::ConditionIndex(2), 6),
    ];
    assert_eq!(
        parse(&tokens),
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
fn parse_not_rule_wraps_single_child_and() {
    let tokens = [parser_tok(TokenKind::Not, 0), parser_tok(TokenKind::ConditionIndex(7), 1)];
    assert_eq!(
        parse(&tokens),
        EvaluationNode::Not {
            child: Box::new(EvaluationNode::Condition(7)),
        }
    );
}

#[test]
fn parse_not_and_group() {
    // !(a && b)
    let tokens = [
        parser_tok(TokenKind::Not, 0),
        parser_tok(TokenKind::LParen, 1),
        parser_tok(TokenKind::ConditionIndex(0), 2),
        parser_tok(TokenKind::And, 3),
        parser_tok(TokenKind::ConditionIndex(1), 4),
        parser_tok(TokenKind::RParen, 5),
    ];
    assert_eq!(
        parse(&tokens),
        EvaluationNode::Not {
            child: Box::new(EvaluationNode::Group {
                composition: Composition::And,
                children: vec![EvaluationNode::Condition(0), EvaluationNode::Condition(1)],
            }),
        }
    );
}

#[test]
fn parse_not_or_group() {
    // !(a || b)
    let tokens = [
        parser_tok(TokenKind::Not, 0),
        parser_tok(TokenKind::LParen, 1),
        parser_tok(TokenKind::ConditionIndex(0), 2),
        parser_tok(TokenKind::Or, 3),
        parser_tok(TokenKind::ConditionIndex(1), 4),
        parser_tok(TokenKind::RParen, 5),
    ];
    assert_eq!(
        parse(&tokens),
        EvaluationNode::Not {
            child: Box::new(EvaluationNode::Group {
                composition: Composition::Or,
                children: vec![EvaluationNode::Condition(0), EvaluationNode::Condition(1)],
            }),
        }
    );
}

#[test]
fn parse_nested_parens_single_rule() {
    let tokens = [
        parser_tok(TokenKind::LParen, 0),
        parser_tok(TokenKind::LParen, 1),
        parser_tok(TokenKind::ConditionIndex(99), 2),
        parser_tok(TokenKind::RParen, 3),
        parser_tok(TokenKind::RParen, 4),
    ];
    assert_eq!(parse(&tokens), EvaluationNode::Condition(99));
}

#[test]
fn parse_end_to_end_after_resolve() {
    let input = "!(a && b) || c";
    let mut tokens = tokenize(input).unwrap();
    let conditions = condition_list_for_rule_tests(&["_pad", "a", "b", "c"]);
    resolve_rule_names(&mut tokens, input, &conditions).unwrap();
    assert_eq!(
        parse(&tokens),
        EvaluationNode::Group {
            composition: Composition::Or,
            children: vec![
                EvaluationNode::Not {
                    child: Box::new(EvaluationNode::Group {
                        composition: Composition::And,
                        children: vec![EvaluationNode::Condition(1), EvaluationNode::Condition(2)],
                    }),
                },
                EvaluationNode::Condition(3),
            ],
        }
    );
}

// --- redundancy_optimizations ---

#[test]
fn reduce_single_rule_wrapping_strips_parens_around_rule_name() {
    for input in ["(x)", "((x))", "(((x)))"] {
        let mut t = parse_tokens(input);
        reduce_single_rule_wrapping(&mut t);
        assert_eq!(kind_seq(&t), vec![RN], "{input:?}");
    }
}

#[test]
fn reduce_single_rule_wrapping_strips_parens_around_not_rule() {
    for input in ["(!x)", "((!x))", "(((!x)))"] {
        let mut t = parse_tokens(input);
        reduce_single_rule_wrapping(&mut t);
        assert_eq!(kind_seq(&t), vec![TokenKind::Not, RN], "{input:?}");
    }
}

#[test]
fn reduce_single_rule_wrapping_leaves_inner_expression_unchanged() {
    let input = "(a && b)";
    let mut t = parse_tokens(input);
    let before = kind_seq(&t);
    reduce_single_rule_wrapping(&mut t);
    assert_eq!(kind_seq(&t), before, "single-rule optimization must not apply");
}

#[test]
fn reduce_outermost_wrapping_strips_full_span_layers() {
    let input = "(a && b)";
    let mut t = parse_tokens(input);
    reduce_outermost_wrapping(&mut t);
    assert_eq!(kind_seq(&t), vec![RN, TokenKind::And, RN]);

    let input = "((a && b))";
    let mut t = parse_tokens(input);
    reduce_outermost_wrapping(&mut t);
    assert_eq!(kind_seq(&t), vec![RN, TokenKind::And, RN]);
}

#[test]
fn reduce_outermost_wrapping_noop_when_parens_do_not_span_whole_input() {
    let input = "a && (b || c)";
    let mut t = parse_tokens(input);
    let before = kind_seq(&t);
    reduce_outermost_wrapping(&mut t);
    assert_eq!(kind_seq(&t), before);
}

#[test]
fn reduce_outermost_wrapping_noop_without_wrapping_parens() {
    let input = "a && b";
    let mut t = parse_tokens(input);
    let before = kind_seq(&t);
    reduce_outermost_wrapping(&mut t);
    assert_eq!(kind_seq(&t), before);
}

#[test]
fn reduce_extra_wrapping_collapses_redundant_nested_parens() {
    let input = "(((a && b)))";
    let mut t = parse_tokens(input);
    reduce_extra_wrapping(&mut t);
    assert_eq!(kind_seq(&t), vec![TokenKind::LParen, RN, TokenKind::And, RN, TokenKind::RParen,]);
}

#[test]
fn reduce_extra_wrapping_keeps_single_pair_around_expression() {
    let input = "(a && b)";
    let mut t = parse_tokens(input);
    let before = kind_seq(&t);
    reduce_extra_wrapping(&mut t);
    assert_eq!(kind_seq(&t), before);
}

#[test]
fn reduce_parentheses_fully_flattens_deep_rule_wrapping() {
    let input = "(((x)))";
    let mut t = parse_tokens(input);
    reduce_parentheses(&mut t);
    assert_eq!(kind_seq(&t), vec![RN]);
}

#[test]
fn reduce_parentheses_flattens_not_rule_and_outer_expression() {
    let input = "(((!x)))";
    let mut t = parse_tokens(input);
    reduce_parentheses(&mut t);
    assert_eq!(kind_seq(&t), vec![TokenKind::Not, RN]);
}

#[test]
fn reduce_parentheses_expression_with_inner_groups() {
    // 3 outer pairs + inner `(b || c)` — `reduce_parentheses` strips redundant outer layers only.
    let input = "(((a && (b || c))))";
    let mut t = parse_tokens(input);
    reduce_parentheses(&mut t);
    assert_eq!(
        kind_seq(&t),
        vec![RN, TokenKind::And, TokenKind::LParen, RN, TokenKind::Or, RN, TokenKind::RParen,]
    );
}

#[test]
fn check_same_operator_per_level() {
    let input = "(((A and B and (C or D) or E)))";
    let t = parse_tokens(input);
    let res = validate_same_operation_per_level(&t,  input);
    assert_eq!(res, Err(Error::MixedOperators(24, 26, input.to_string())));
    assert_eq!(TokenSpan::new(24, 26).as_slice(input), "or");
}

#[test]
fn check_same_operator_per_level_no_pharantheses() {
    let input = "A or B and C";
    let t = parse_tokens(input);
    let res = validate_same_operation_per_level(&t,  input);
    assert_eq!(res, Err(Error::MixedOperators(7, 10, input.to_string())));
    assert_eq!(TokenSpan::new(7, 10).as_slice(input), "and");
}
