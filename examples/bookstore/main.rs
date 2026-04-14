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

fn main() {
    let books = vec![
        Book { name: "The_Hobbit", author: "JRR_Tolkien", editure: "HarperCollins", year: 1937, pages: 310, is_kids_book: false },
        Book { name: "The_Lord_of_the_Rings", author: "JRR_Tolkien", editure: "Allen_Unwin", year: 1954, pages: 1178, is_kids_book: false },
        Book { name: "Dune", author: "Frank_Herbert", editure: "Chilton_Books", year: 1965, pages: 412, is_kids_book: false },
        Book { name: "Foundation", author: "Isaac_Asimov", editure: "Gnome_Press", year: 1951, pages: 255, is_kids_book: false },
        Book { name: "Neuromancer", author: "William_Gibson", editure: "Ace", year: 1984, pages: 271, is_kids_book: false },
        Book { name: "Snow_Crash", author: "Neal_Stephenson", editure: "Bantam", year: 1992, pages: 470, is_kids_book: false },
        Book { name: "The_Martian", author: "Andy_Weir", editure: "Crown", year: 2011, pages: 369, is_kids_book: false },
        Book { name: "Project_Hail_Mary", author: "Andy_Weir", editure: "Ballantine", year: 2021, pages: 496, is_kids_book: false },
        Book { name: "1984", author: "George_Orwell", editure: "Secker_Warburg", year: 1949, pages: 328, is_kids_book: false },
        Book { name: "Animal_Farm", author: "George_Orwell", editure: "Secker_Warburg", year: 1945, pages: 112, is_kids_book: false },
        Book { name: "Brave_New_World", author: "Aldous_Huxley", editure: "Chatto_Windus", year: 1932, pages: 311, is_kids_book: false },
        Book { name: "Fahrenheit_451", author: "Ray_Bradbury", editure: "Ballantine", year: 1953, pages: 194, is_kids_book: false },
        Book { name: "The_Name_of_the_Wind", author: "Patrick_Rothfuss", editure: "DAW", year: 2007, pages: 662, is_kids_book: false },
        Book { name: "The_Wise_Mans_Fear", author: "Patrick_Rothfuss", editure: "DAW", year: 2011, pages: 994, is_kids_book: false },
        Book { name: "Mistborn", author: "Brandon_Sanderson", editure: "Tor", year: 2006, pages: 541, is_kids_book: false },
        Book { name: "The_Way_of_Kings", author: "Brandon_Sanderson", editure: "Tor", year: 2010, pages: 1007, is_kids_book: false },
        Book { name: "Words_of_Radiance", author: "Brandon_Sanderson", editure: "Tor", year: 2014, pages: 1087, is_kids_book: false },
        Book { name: "The_Girl_with_the_Dragon_Tattoo", author: "Stieg_Larsson", editure: "Norstedts", year: 2005, pages: 465, is_kids_book: false },
        Book { name: "Gone_Girl", author: "Gillian_Flynn", editure: "Crown", year: 2012, pages: 432, is_kids_book: false },
        Book { name: "The_Da_Vinci_Code", author: "Dan_Brown", editure: "Doubleday", year: 2003, pages: 454, is_kids_book: false },
        Book { name: "The_Alchemist", author: "Paulo_Coelho", editure: "HarperOne", year: 1988, pages: 208, is_kids_book: false },
        Book { name: "To_Kill_a_Mockingbird", author: "Harper_Lee", editure: "J_B_Lippincott", year: 1960, pages: 281, is_kids_book: false },
        Book { name: "Pride_and_Prejudice", author: "Jane_Austen", editure: "T_Egerton", year: 1813, pages: 279, is_kids_book: false },
        Book { name: "Moby_Dick", author: "Herman_Melville", editure: "Richard_Bentley", year: 1851, pages: 635, is_kids_book: false },
        Book { name: "Harry_Potter_and_the_Philosophers_Stone", author: "JK_Rowling", editure: "Bloomsbury", year: 1997, pages: 223, is_kids_book: true },
        Book { name: "Harry_Potter_and_the_Chamber_of_Secrets", author: "JK_Rowling", editure: "Bloomsbury", year: 1998, pages: 251, is_kids_book: true },
        Book { name: "Matilda", author: "Roald_Dahl", editure: "Jonathan_Cape", year: 1988, pages: 240, is_kids_book: true },
        Book { name: "Charlie_and_the_Chocolate_Factory", author: "Roald_Dahl", editure: "Alfred_A_Knopf", year: 1964, pages: 155, is_kids_book: true },
        Book { name: "The_Little_Prince", author: "Antoine_de_Saint_Exupery", editure: "Reynal_Hitchcock", year: 1943, pages: 96, is_kids_book: true },
        Book { name: "The_Cat_in_the_Hat", author: "Dr_Seuss", editure: "Random_House", year: 1957, pages: 61, is_kids_book: true },
        Book { name: "Charlotte_s_Web", author: "EB_White", editure: "Harper_Brothers", year: 1952, pages: 192, is_kids_book: true },
        Book { name: "Percy_Jackson_and_the_Lightning_Thief", author: "Rick_Riordan", editure: "Miramax", year: 2005, pages: 377, is_kids_book: true },
        Book { name: "The_Hunger_Games", author: "Suzanne_Collins", editure: "Scholastic", year: 2008, pages: 374, is_kids_book: true },
        Book { name: "Coraline", author: "Neil_Gaiman", editure: "Bloomsbury", year: 2002, pages: 208, is_kids_book: true },
        Book { name: "The_Giver", author: "Lois_Lowry", editure: "Houghton_Mifflin", year: 1993, pages: 180, is_kids_book: true },
    ];

    let expression = ExpressionBuilder::<Book>::new()
        .add("modern", Condition::from_str("year >= 1990"))
        .add("not_too_long", Condition::from_str("pages <= 500"))
        .add("from_known_editure", Condition::from_str("editure is-one-of [Bloomsbury, Crown, Ballantine, Tor]"))
        .add("title_has_the", Condition::from_str("name contains 'The'"))
        .add("kids", Condition::from_str("is_kids_book is true"))
        .build("modern && not_too_long && from_known_editure && (kids || title_has_the)")
        .unwrap();

    println!("Books that match the search filter:");
    for book in &books {
        if expression.matches(book) {
            println!(
                "- {} | author={} | editure={} | year={} | pages={} | kids={}",
                book.name, book.author, book.editure, book.year, book.pages, book.is_kids_book
            );
        }
    }
}