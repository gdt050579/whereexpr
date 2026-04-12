use whereexpr::*;
fn main() {
    if let Ok(p) = Predicate::with_value(Operation::Is, 100) {
        println!("all ok");
    }
    
}