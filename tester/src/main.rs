use whereexpr::*;
struct Person {
    name: String,
    surname: String,
    age: u32,
}
impl Person {
    const NAME: AttributeIndex = AttributeIndex::new(0);
    const SURNAME: AttributeIndex = AttributeIndex::new(1);
    const AGE: AttributeIndex = AttributeIndex::new(2);
}
impl Attributes for Person {
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::NAME => Some(Value::String(self.name.as_str())),
            Self::SURNAME => Some(Value::String(self.surname.as_str())),
            Self::AGE => Some(Value::U32(self.age)),
            _ => None,
        }
    }
    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::NAME => Some(ValueKind::String),
            Self::SURNAME => Some(ValueKind::String),
            Self::AGE => Some(ValueKind::U32),
            _ => None,
        }
    }
    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "name" => Some(Self::NAME),
            "surname" => Some(Self::SURNAME),
            "age" => Some(Self::AGE),
            _ => None,
        }
    }
}

fn main() {
    let mut builder = ExpressionBuilder::<Person>::new();
    builder.add_condition("cond_1", "name", Predicate::with_value(Operation::Is, "John").unwrap());
    builder.add_condition("cond_2", "surname", Predicate::with_value(Operation::Is, "Doe").unwrap());
    builder.add_condition("cond_3", "age", Predicate::with_value(Operation::GreaterThan, 10u32).unwrap());
    let ex = builder.build("cond_1 && cond_2 && cond_3").unwrap();
    let person = Person {
        name: "John".to_string(),
        surname: "Doe".to_string(),
        age: 33,
    };
    println!("matches: {}", ex.matches(&person));
    println!("all ok");
}
