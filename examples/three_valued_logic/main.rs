//! Three-valued (Kleene) logic in whereexpr.
//!
//! A condition evaluates to:
//!   - `Some(true)`  -> true
//!   - `Some(false)` -> false
//!   - `None`        -> unknown (e.g. the attribute is missing for this object)
//!
//! `Expression::try_matches` combines these with Kleene logic:
//!   - AND: any false -> false; otherwise any unknown -> unknown; otherwise true
//!   - OR : any true  -> true;  otherwise any unknown -> unknown; otherwise false
//!   - NOT: true <-> false, unknown stays unknown
//!
//! The result never depends on the order of the operands.
//!
//! Run with: cargo run --example three_valued_logic

use whereexpr::*;

/// Four optional flags. `None` means "unknown": `get` returns `None` for it.
struct Flags {
    values: [Option<bool>; 4],
}

impl Flags {
    const A: AttributeIndex = AttributeIndex::new(0);
    const B: AttributeIndex = AttributeIndex::new(1);
    const C: AttributeIndex = AttributeIndex::new(2);
    const D: AttributeIndex = AttributeIndex::new(3);
}

impl Attributes for Flags {
    const TYPE_ID: u64 = 0x7A1F_0001; // unique ID for Flags type
    const TYPE_NAME: &'static str = "Flags";

    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        let value = match idx {
            Self::A => self.values[0],
            Self::B => self.values[1],
            Self::C => self.values[2],
            Self::D => self.values[3],
            _ => return None,
        };
        value.map(Value::Bool)
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::A | Self::B | Self::C | Self::D => Some(ValueKind::Bool),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "a" => Some(Self::A),
            "b" => Some(Self::B),
            "c" => Some(Self::C),
            "d" => Some(Self::D),
            _ => None,
        }
    }
}

const T: Option<bool> = Some(true);
const F: Option<bool> = Some(false);
const U: Option<bool> = None;

fn build(expr: &str) -> Expression {
    ExpressionBuilder::<Flags>::new()
        .add("a", Condition::from_str("a is true"))
        .add("b", Condition::from_str("b is true"))
        .add("c", Condition::from_str("c is true"))
        .add("d", Condition::from_str("d is true"))
        .build(expr)
        .unwrap()
}

fn show(v: Option<bool>) -> &'static str {
    match v {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn show_values(values: &[Option<bool>; 4]) -> String {
    let names = ["a", "b", "c", "d"];
    names
        .iter()
        .zip(values)
        .map(|(n, v)| format!("{n}={}", show(*v)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() {
    // (expression, [a, b, c, d], expected result)
    let cases: &[(&str, [Option<bool>; 4], Option<bool>)] = &[
        // AND: a single false decides the result, wherever it is
        ("a && b && c && d", [F, F, U, F], F),
        ("a && b && c && d", [F, F, U, T], F),
        ("a && b && c && d", [U, T, T, F], F),
        ("a && b && c && d", [T, T, U, T], U),
        ("a && b && c && d", [T, T, T, T], T),
        ("a && b", [U, F, U, U], F),
        ("a && b", [F, U, U, U], F),
        ("a && b", [U, U, U, U], U),
        // OR: a single true decides the result, wherever it is
        ("a || b || c || d", [F, F, U, F], U),
        ("a || b || c || d", [F, F, U, T], T),
        ("a || b || c || d", [U, F, F, T], T),
        ("a || b || c || d", [F, F, F, F], F),
        ("a || b", [U, T, U, U], T),
        ("a || b", [T, U, U, U], T),
        ("a || b", [U, U, U, U], U),
        // NOT
        ("!a", [T, U, U, U], F),
        ("!a", [F, U, U, U], T),
        ("!a", [U, U, U, U], U),
        // Nested
        ("(a || b) && c", [U, T, T, U], T),
        ("(a && b) || c", [U, F, F, U], F),
        ("!(a && b)", [U, T, U, U], U),
        ("!(a || b)", [U, T, U, U], F),
    ];

    let mut failures = 0;
    for (expr, values, expected) in cases {
        let actual = build(expr).try_matches(&Flags { values: *values });
        let ok = actual == *expected;
        if !ok {
            failures += 1;
        }
        println!(
            "[{}] {:<18} with {:<44} => {:<7} (expected {})",
            if ok { " OK " } else { "FAIL" },
            expr,
            show_values(values),
            show(actual),
            show(*expected),
        );
    }

    println!();
    if failures == 0 {
        println!("All {} cases follow three-valued logic.", cases.len());
    } else {
        println!("{failures} of {} cases FAILED.", cases.len());
        std::process::exit(1);
    }
}
