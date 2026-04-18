use chrono::{Datelike, NaiveDate};
use whereexpr::*;

struct Student {
    name: String,
    math_grade: u32,
    english_grade: u32,
    age: u32,
    enrolled: NaiveDate,
    birthday: NaiveDate,
}

impl Student {
    const NAME: AttributeIndex = AttributeIndex::new(0);
    const GRADE_MATH: AttributeIndex = AttributeIndex::new(1);
    const GRADE_ENGLISH: AttributeIndex = AttributeIndex::new(2);
    const GRADE_AVG: AttributeIndex = AttributeIndex::new(3);
    const AGE: AttributeIndex = AttributeIndex::new(4);
    const FAMILY_NAME: AttributeIndex = AttributeIndex::new(5);
    const ENROLL_YEAR: AttributeIndex = AttributeIndex::new(6);
    const BIRTHDAY_MONTH: AttributeIndex = AttributeIndex::new(7);

    fn family_name(&self) -> &str {
        self.name.split(' ').next().unwrap()
    }

    fn average_grade(&self) -> u32 {
        (self.math_grade + self.english_grade) / 2
    }
}

impl Attributes for Student {
    const TYPE_ID: u64 = 0x12345678; // unique ID for Student type
    const TYPE_NAME: &'static str = "Student";

    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::NAME => Some(Value::String(self.name.as_str())),
            Self::GRADE_MATH => Some(Value::U32(self.math_grade)),
            Self::GRADE_ENGLISH => Some(Value::U32(self.english_grade)),
            Self::GRADE_AVG => Some(Value::U32(self.average_grade())),
            Self::AGE => Some(Value::U32(self.age)),
            Self::FAMILY_NAME => Some(Value::String(self.family_name())),
            Self::ENROLL_YEAR => Some(Value::I32(self.enrolled.year())),
            Self::BIRTHDAY_MONTH => Some(Value::U32(self.birthday.month())),
            _ => None,
        }
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::NAME => Some(ValueKind::String),
            Self::GRADE_MATH => Some(ValueKind::U32),
            Self::GRADE_ENGLISH => Some(ValueKind::U32),
            Self::GRADE_AVG => Some(ValueKind::U32),
            Self::AGE => Some(ValueKind::U32),
            Self::FAMILY_NAME => Some(ValueKind::String),
            Self::ENROLL_YEAR => Some(ValueKind::I32),
            Self::BIRTHDAY_MONTH => Some(ValueKind::U32),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name.to_ascii_lowercase().as_str() {
            "name" => Some(Self::NAME),
            "grade.math" => Some(Self::GRADE_MATH),
            "grade.english" => Some(Self::GRADE_ENGLISH),
            "grade.average" => Some(Self::GRADE_AVG),
            "age" => Some(Self::AGE),
            "name.family" => Some(Self::FAMILY_NAME),
            "enroll.year" => Some(Self::ENROLL_YEAR),
            "birthday.month" => Some(Self::BIRTHDAY_MONTH),
            _ => None,
        }
    }
}

fn main() {
    let expr = ExpressionBuilder::<Student>::new()
        .add("age-is-prime", Condition::from_str("age is-one-of [19,23,29,31]"))
        .add("common", Condition::from_str("name.family ends-with 'escu' {ignore-case}"))
        .add("old-enroll", Condition::from_str("enroll.year in-range [2010, 2020]"))
        .add("passes-courses", Condition::from_str("grade.average > 5"))
        .add("born-in-dec", Condition::from_str("birthday.month is 12"))
        .add("vowel-name", Condition::from_str("name starts-with-one-of [A, E, I, O, U] {ignore-case}"))
        .add("good-average", Condition::from_str("grade.math >= 8"))
        .build("Age-Is-Prime OR Common OR (Old-Enroll AND Good-at-math) OR NOT (Passes-Courses AND NOT (Born-in-dec OR Vowel-Name))")
        .unwrap();

    let student = Student {
        name: "Popescu Ion".to_string(),
        math_grade: 8,
        english_grade: 9,
        age: 20,
        enrolled: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        birthday: NaiveDate::from_ymd_opt(1990, 12, 1).unwrap(),
    };

    println!("matches: {}", expr.matches(&student));
}
