use whereexpr::*;
fn main() {
    if let Ok(p) = Predicate::with_list(Operation::IsOneOf, &[100, 200, 300]) {
        println!("all ok");
    }
    
}