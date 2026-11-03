use whereexpr::*;

#[derive(Debug)]
struct Book {
    name: &'static str,
    author: &'static str,
    editure: &'static str,
    year: u32,
    pages: u32,
    is_kids_book: bool,
}

impl Book {
    const NAME: AttributeIndex = AttributeIndex::new(0);
    const AUTHOR: AttributeIndex = AttributeIndex::new(1);
    const EDITURE: AttributeIndex = AttributeIndex::new(2);
    const YEAR: AttributeIndex = AttributeIndex::new(3);
    const PAGES: AttributeIndex = AttributeIndex::new(4);
    const IS_KIDS_BOOK: AttributeIndex = AttributeIndex::new(5);
}

impl Attributes for Book {
    const TYPE_ID: u64 = 1;
    const TYPE_NAME: &'static str = "Book";

    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::NAME => Some(Value::String(self.name)),
            Self::AUTHOR => Some(Value::String(self.author)),
            Self::EDITURE => Some(Value::String(self.editure)),
            Self::YEAR => Some(Value::U32(self.year)),
            Self::PAGES => Some(Value::U32(self.pages)),
            Self::IS_KIDS_BOOK => Some(Value::Bool(self.is_kids_book)),
            _ => None,
        }
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::NAME => Some(ValueKind::String),
            Self::AUTHOR => Some(ValueKind::String),
            Self::EDITURE => Some(ValueKind::String),
            Self::YEAR => Some(ValueKind::U32),
            Self::PAGES => Some(ValueKind::U32),
            Self::IS_KIDS_BOOK => Some(ValueKind::Bool),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "name" => Some(Self::NAME),
            "author" => Some(Self::AUTHOR),
            "editure" => Some(Self::EDITURE),
            "year" => Some(Self::YEAR),
            "pages" => Some(Self::PAGES),
            "is_kids_book" => Some(Self::IS_KIDS_BOOK),
            _ => None,
        }
    }
}

const RULE: &str = r#"
rule shelf: Book {
    modern = "year >= 1990"
    not_too_long = "pages <= 500"
    from_known_editure = "editure is-one-of [Bloomsbury, Crown, Ballantine, Tor]"
    title_has_the = "name contains 'The'"
    kids = "is_kids_book is true"
} where {
    modern AND not_too_long AND from_known_editure AND (kids OR title_has_the)
}
"#;

fn main() {
    let books = [
        Book { name: "The_Hobbit", author: "JRR_Tolkien", editure: "HarperCollins", year: 1937, pages: 310, is_kids_book: false },
        Book { name: "The_Martian", author: "Andy_Weir", editure: "Crown", year: 2011, pages: 369, is_kids_book: false },
        Book { name: "Project_Hail_Mary", author: "Andy_Weir", editure: "Ballantine", year: 2021, pages: 496, is_kids_book: false },
        Book { name: "The_Way_of_Kings", author: "Brandon_Sanderson", editure: "Tor", year: 2010, pages: 1007, is_kids_book: false },
        Book { name: "Gone_Girl", author: "Gillian_Flynn", editure: "Crown", year: 2012, pages: 432, is_kids_book: false },
        Book { name: "Harry_Potter_and_the_Philosophers_Stone", author: "JK_Rowling", editure: "Bloomsbury", year: 1997, pages: 223, is_kids_book: true },
        Book { name: "Coraline", author: "Neil_Gaiman", editure: "Bloomsbury", year: 2002, pages: 208, is_kids_book: true },
        Book { name: "Percy_Jackson_and_the_Lightning_Thief", author: "Rick_Riordan", editure: "Miramax", year: 2005, pages: 377, is_kids_book: true },
    ];

    let expression = Expression::from_rule_str::<Book>(RULE).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    });

    println!("Books that match the rule:");
    for book in &books {
        if expression.matches(book) {
            println!(
                "- {} | author={} | editure={} | year={} | pages={} | kids={}",
                book.name, book.author, book.editure, book.year, book.pages, book.is_kids_book
            );
        }
    }
}
