//! `#[derive(Attributes)]` instead of a hand-written `impl Attributes`.
//!
//! cargo run --example derive

use std::path::PathBuf;
use whereexpr::*;

#[derive(Attributes)]
#[whereexpr(property = "fn extension -> string")]
struct File {
    path: PathBuf,
    size: u64,
    #[whereexpr(kind = datetime)]
    modified: u64,
    #[whereexpr(name = "owner")]
    owner_name: Option<String>,
    #[whereexpr(ignore)]
    _contents: Vec<u8>,
}

impl File {
    fn extension(&self) -> Option<&str> {
        self.path.extension()?.to_str()
    }
}

fn file(path: &str, size: u64, modified: u64, owner: Option<&str>) -> File {
    File {
        path: PathBuf::from(path),
        size,
        modified,
        owner_name: owner.map(str::to_string),
        _contents: Vec::new(),
    }
}

fn main() {
    let files = [
        file("/var/log/app.log", 4_096, 1_735_000_000, Some("root")),
        file("/var/log/old.log", 128, 1_600_000_000, Some("root")),
        file("/home/ana/notes.md", 900, 1_740_000_000, Some("ana")),
        file("/tmp/upload.bin", 10_000, 1_741_000_000, None),
    ];

    let rule = r#"
        rule big_recent_logs: File {
            log    = "extension is log"
            big    = "size > 1000"
            recent = "modified > 2024-01-01"
        } where {
            log AND big AND recent
        }
        rule not_root: File {
            root = "owner is root"
        } where {
            NOT root
        }
    "#;

    for item in Expression::parse_rules::<File>(rule) {
        let (name, expr) = item.expect("rule compiles");
        println!("{name}:");
        for f in &files {
            let result = match expr.evaluate(f) {
                Some(true) => "match",
                Some(false) => "no match",
                None => "unknown (owner not set)",
            };
            println!("  {:<22} {result}", f.path.display());
        }
    }
}
