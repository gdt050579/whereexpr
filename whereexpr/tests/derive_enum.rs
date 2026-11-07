//! `#[derive(Attributes)]` with `#[whereexpr(kind = enum)]` fields.

use whereexpr::{Attributes, Error, Expression, ExpressionBuilder, Condition, WhereexprEnum};

#[derive(WhereexprEnum, Clone, Copy)]
enum Color {
    #[whereexpr(aliases = ["red", "rosu"])]
    Red,
    Green,
    Blue,
}

#[derive(WhereexprEnum, Clone, Copy)]
enum Fuel {
    Petrol,
    Diesel,
    #[whereexpr(aliases = ["electric", "ev"])]
    Electric,
}

#[derive(Attributes)]
struct Car {
    #[whereexpr(kind = enum)]
    color: Color,
    #[whereexpr(kind = enum)]
    fuel: Option<Fuel>,
    doors: u8,
}

#[derive(Attributes)]
struct CarRef<'a> {
    #[whereexpr(kind = enum)]
    color: &'a Color,
    #[whereexpr(kind = enum, name = "paint")]
    second: Option<&'a Color>,
}

fn car(color: Color, fuel: Option<Fuel>) -> Car {
    Car { color, fuel, doors: 4 }
}

fn build<T: Attributes>(condition: &str) -> Result<Expression, Error> {
    ExpressionBuilder::<T>::new().add("c", Condition::from_str(condition)).build("c")
}

#[test]
fn enum_field_is() {
    let e = build::<Car>("color is rosu").unwrap();
    assert!(e.matches(&car(Color::Red, None)));
    assert!(!e.matches(&car(Color::Blue, None)));
}

#[test]
fn enum_fields_use_their_own_enum() {
    // "electric" is a Fuel name, not a Color name
    assert!(matches!(build::<Car>("color is electric"), Err(Error::UnknownEnumValue(..))));
    assert!(matches!(build::<Car>("fuel is red"), Err(Error::UnknownEnumValue(..))));
    let e = build::<Car>("fuel is-one-of [ev, diesel]").unwrap();
    assert!(e.matches(&car(Color::Red, Some(Fuel::Electric))));
    assert!(e.matches(&car(Color::Red, Some(Fuel::Diesel))));
    assert!(!e.matches(&car(Color::Red, Some(Fuel::Petrol))));
}

#[test]
fn optional_enum_field_none_is_unknown() {
    let e = build::<Car>("fuel is petrol").unwrap();
    assert_eq!(e.evaluate(&car(Color::Red, None)), None);
    assert_eq!(e.evaluate(&car(Color::Red, Some(Fuel::Petrol))), Some(true));
}

#[test]
fn enum_and_other_fields() {
    let e = ExpressionBuilder::<Car>::new()
        .add("green", Condition::from_str("color is green"))
        .add("four", Condition::from_str("doors is 4"))
        .build("green && four")
        .unwrap();
    assert!(e.matches(&car(Color::Green, None)));
    assert!(!e.matches(&car(Color::Blue, None)));
}

#[test]
fn enum_field_by_reference() {
    let red = Color::Red;
    let blue = Color::Blue;
    let c = CarRef { color: &red, second: Some(&blue) };
    assert!(build::<CarRef>("color is red").unwrap().matches(&c));
    assert!(build::<CarRef>("paint is blue").unwrap().matches(&c));
    let c = CarRef { color: &blue, second: None };
    assert_eq!(build::<CarRef>("paint is blue").unwrap().evaluate(&c), None);
}

#[test]
fn enum_field_in_rule() {
    let e = Expression::from_rule_str::<Car>(
        "rule green_ev: Car { g = \"color is green\" e = \"fuel is ev\" } where { g AND e }",
    )
    .unwrap();
    assert!(e.matches(&car(Color::Green, Some(Fuel::Electric))));
    assert!(!e.matches(&car(Color::Green, Some(Fuel::Diesel))));
}

#[test]
fn derived_enum_value() {
    // color = 0, fuel = 1, doors = 2
    let color = Car::index("color").unwrap();
    let fuel = Car::index("fuel").unwrap();
    let doors = Car::index("doors").unwrap();
    assert_eq!(Car::enum_value(color, "ROSU"), Some(0));
    assert_eq!(Car::enum_value(fuel, "ev"), Some(2));
    assert_eq!(Car::enum_value(doors, "red"), None);
    assert_eq!(Car::kind(color), Some(whereexpr::ValueKind::Enum));
}
