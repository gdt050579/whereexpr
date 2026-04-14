use whereexpr::*;

struct TextStats {
    text: String,
}

impl TextStats {
    const TEXT: AttributeIndex = AttributeIndex::new(0);
    const LENGTH: AttributeIndex = AttributeIndex::new(1);
    const VOWELS: AttributeIndex = AttributeIndex::new(2);
    const WORDS: AttributeIndex = AttributeIndex::new(3);

    fn length(&self) -> u32 {
        self.text.chars().count() as u32
    }

    fn vowels(&self) -> u32 {
        self.text
            .chars()
            .filter(|c| matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))
            .count() as u32
    }

    fn words(&self) -> u32 {
        self.text.split_whitespace().count() as u32
    }
}

impl Attributes for TextStats {
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::TEXT => Some(Value::String(self.text.as_str())),
            Self::LENGTH => Some(Value::U32(self.length())),
            Self::VOWELS => Some(Value::U32(self.vowels())),
            Self::WORDS => Some(Value::U32(self.words())),
            _ => None,
        }
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::TEXT => Some(ValueKind::String),
            Self::LENGTH => Some(ValueKind::U32),
            Self::VOWELS => Some(ValueKind::U32),
            Self::WORDS => Some(ValueKind::U32),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "text" => Some(Self::TEXT),
            "length" => Some(Self::LENGTH),
            "vowels" => Some(Self::VOWELS),
            "words" => Some(Self::WORDS),
            _ => None,
        }
    }
}

fn main() {
    let sample = TextStats {
        text: "Rust makes expression filtering fast and flexible".to_string(),
    };

    let expr = ExpressionBuilder::<TextStats>::new()
        .add("has_rust", Condition::from_str("text contains Rust"))
        .add("length_ok", Condition::from_str("length in-range [20, 60]"))
        .add("enough_vowels", Condition::from_str("vowels >= 10"))
        .add("enough_words", Condition::from_str("words > 5"))
        .build("has_rust && length_ok && enough_vowels && enough_words")
        .unwrap();

    println!("text   : {}", sample.text);
    println!("length : {}", sample.length());
    println!("vowels : {}", sample.vowels());
    println!("words  : {}", sample.words());
    println!("matches: {}", expr.matches(&sample));
}
