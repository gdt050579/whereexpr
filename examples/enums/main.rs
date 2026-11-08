//! Enum attributes: `#[derive(WhereexprEnum)]` on the enum, `#[whereexpr(kind = enum)]` on the field.
//!
//! cargo run --example enums

use whereexpr::*;

#[derive(WhereexprEnum, Clone, Copy, Debug)]
enum Color {
    #[whereexpr(aliases = ["red", "rosu"])]
    Red,
    Green,
    Blue,
}

#[derive(WhereexprEnum, Clone, Copy, Debug)]
enum Fuel {
    Petrol,
    Diesel,
    #[whereexpr(aliases = ["electric", "ev"])]
    Electric,
}

#[derive(Attributes, Debug)]
struct Car {
    model: &'static str,
    #[whereexpr(kind = enum)]
    color: Color,
    #[whereexpr(kind = enum)]
    fuel: Option<Fuel>, // None: not known yet
    doors: u8,
}

fn main() {
    let cars = [
        Car { model: "Spark", color: Color::Red, fuel: Some(Fuel::Electric), doors: 3 },
        Car { model: "Atlas", color: Color::Blue, fuel: Some(Fuel::Diesel), doors: 5 },
        Car { model: "Fiesta", color: Color::Green, fuel: Some(Fuel::Petrol), doors: 3 },
        Car { model: "Prototype", color: Color::Red, fuel: None, doors: 2 },
    ];

    let rules = r#"
        rule small_warm: Car {
            warm  = "color is-one-of [rosu, green]"
            small = "doors <= 3"
        } where { warm AND small }

        rule clean: Car {
            ev = "fuel is ev"
        } where { ev }

        rule not_red: Car {
            red = "color is RED"
        } where { NOT red }
    "#;

    let mut parser = RuleParser::new(rules);
    while let Some(def) = parser.next_rule_definition().unwrap() {
        let name = def.name().to_string();
        let expr = def.expression::<Car>().unwrap();
        println!("{name}:");
        for car in &cars {
            // `evaluate` shows unknown results: the prototype's fuel is not known
            let result = match expr.evaluate(car) {
                Some(true) => "yes",
                Some(false) => "no",
                None => "unknown",
            };
            println!("  {:<10} {:?}/{:?} -> {result}", car.model, car.color, car.fuel);
        }
    }

    // a name that is not a value of the enum is an error when the expression is built
    match Expression::from_rule_str::<Car>(r#"rule x: Car { c = "color is purple" } where { c }"#) {
        Ok(_) => println!("unexpected"),
        Err(e) => println!("\nerror: {e}"),
    }
}
