use whereexpr::*;
fn main() {
    let mut builder = ExpressionBuilder::new();
    builder.add_condition("cond_1", 0, Predicate::with_value(Operation::Is, "John").unwrap());
    builder.add_condition("cond_2", 1, Predicate::with_value(Operation::Is, "Doe").unwrap());
    builder.add_condition("cond_3", 2, Predicate::with_value(Operation::GreaterThan, 10u32).unwrap());
    let ex = builder.build("cond_1 && cond_2 && cond_3").unwrap();
    println!("all ok");
}
