use whereexpr::*;

struct TextSample {
    content: String,
}

impl TextSample {
    const CONTENT: AttributeIndex = AttributeIndex::new(0);
}

impl Attributes for TextSample {
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::CONTENT => Some(Value::String(self.content.as_str())),
            _ => None,
        }
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::CONTENT => Some(ValueKind::String),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "content" => Some(Self::CONTENT),
            _ => None,
        }
    }
}

fn main() {
    let rule = "content contains-one-of ['best of times', 'worst of times', 'age of wisdom', 'age of foolishness', 'epoch of belief', 'epoch of incredulity', 'season of Light', 'season of Darkness', 'spring of hope', 'winter of despair']";

    let expr = ExpressionBuilder::<TextSample>::new()
        .add("contains_known_phrase", Condition::from_str(rule))
        .build("contains_known_phrase")
        .unwrap();

    let passage = TextSample {
        content: "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness."
            .to_string(),
    };

    println!("Rule: {}", rule);
    println!("Passage: {}", passage.content);
    println!("Matches rule: {}", expr.matches(&passage));
}
