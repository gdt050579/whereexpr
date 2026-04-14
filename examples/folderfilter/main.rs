use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use whereexpr::*;

#[derive(Debug)]
struct FileEntry {
    full_path: String,
    name: String,
    size: u64,
    modified_at: u64,
}

impl FileEntry {
    const FULL_PATH: AttributeIndex = AttributeIndex::new(0);
    const NAME: AttributeIndex = AttributeIndex::new(1);
    const SIZE: AttributeIndex = AttributeIndex::new(2);
    const MODIFIED_AT: AttributeIndex = AttributeIndex::new(3);
}

impl Attributes for FileEntry {
    const TYPE_ID: u64 = 0x1f257651; // unique ID for FileEntry type
    const TYPE_NAME: &'static str = "FileEntry";
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::FULL_PATH => Some(Value::Path(self.full_path.as_bytes())),
            Self::NAME => Some(Value::String(self.name.as_str())),
            Self::SIZE => Some(Value::U64(self.size)),
            Self::MODIFIED_AT => Some(Value::DateTime(self.modified_at)),
            _ => None,
        }
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::FULL_PATH => Some(ValueKind::Path),
            Self::NAME => Some(ValueKind::String),
            Self::SIZE => Some(ValueKind::U64),
            Self::MODIFIED_AT => Some(ValueKind::DateTime),
            _ => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "full_path" => Some(Self::FULL_PATH),
            "name" => Some(Self::NAME),
            "size" => Some(Self::SIZE),
            "modified_at" => Some(Self::MODIFIED_AT),
            _ => None,
        }
    }
}

fn collect_files(root: &Path, out: &mut Vec<FileEntry>) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries {
        let entry = match entry {
            Ok(v) => v,
            Err(_) => continue,
        };
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_dir() {
            collect_files(&path, out);
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|st| st.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());

        out.push(FileEntry {
            full_path: path.to_string_lossy().to_string(),
            name,
            size: metadata.len(),
            modified_at,
        });
    }
}

fn main() {
    let folder = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut files = Vec::new();
    collect_files(&folder, &mut files);

    let expr = ExpressionBuilder::<FileEntry>::new()
        .add("size_window", Condition::from_str("size in-range [1024, 65536]"))
        .add("recent", Condition::from_str("modified_at > 2024-01-01"))
        .add("name_filter", Condition::from_str("full_path glob-match  [**/*.rs , **/*.md , **/Cargo.*]"))
        .build("size_window && recent && name_filter")
        .unwrap();

    println!("Scanning folder: {}", folder.display());
    println!("Matched files:");
    for file in &files {
        if expr.matches(file) {
            println!(
                "- {} | size={} bytes | modified_at={} ",
                file.full_path, file.size, file.modified_at
            );
        }
    }
}
