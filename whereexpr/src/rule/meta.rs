use crate::RuleParser;
use super::Span;
use super::hasher;

/// The `with meta { ... }` block of a rule: its entries, in the order they are written.
///
/// Every entry holds a list of values. A single value (`owner = ops`) is a list of one,
/// and a `[ ... ]` list can hold any number of values, including none (`tags = []`).
/// Quotes are removed, so `owner = ops` and `owner = "ops"` give the same value.
///
/// `Meta` borrows the [`RuleParser`], like the [`RuleDefinition`](crate::RuleDefinition)
/// it comes from, so read what you need before reading the next rule.
///
/// ```rust
/// use whereexpr::RuleParser;
///
/// let text = r#"rule r: T { a = "x is 1" } where { a } with meta { owner = ops tags = [disk, "high load"] }"#;
/// let mut parser = RuleParser::new(text);
/// let definition = parser.next_rule_definition().unwrap().unwrap();
/// let meta = definition.meta();
///
/// assert_eq!(meta.len(), 2);
/// assert_eq!(meta.get("owner").unwrap().value(), "ops");
/// assert_eq!(meta.get("tags").unwrap().array_value(0), Some("disk"));
/// assert_eq!(meta.get("tags").unwrap().array_value(1), Some("high load"));
/// for index in 0..meta.len() {
///     let item = meta.get(index).unwrap();
///     println!("{} has {} value(s)", item.key(), item.values_count());
/// }
/// ```
#[derive(Clone, Copy)]
pub struct Meta<'a> {
    parser: &'a RuleParser,
}

impl<'a> Meta<'a> {
    pub(crate) fn new(parser: &'a RuleParser) -> Self {
        Self { parser }
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.parser.meta_count()
    }

    /// `true` when the rule has no `with meta` block, or an empty one.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The entry at this index, or with this key. Key lookup ignores ASCII case.
    pub fn get<T>(&self, index: T) -> Option<MetaItem<'a>> where T: MetaEntryIndex {
        if let Some(index) = index.as_index(self.parser) {
            self.parser.metaentry_by_index(index).map(|entry| MetaItem { parser: self.parser, entry })
        } else {
            None
        }
    }
}

pub(super) struct MetaEntry {
    pub(super) key: Span,
    pub(super) values_start: u32,
    pub(super) values_end: u32,
    pub(super) is_list: bool,
}

#[derive(Clone, Copy)]
pub struct MetaItem<'a> {
    parser: &'a RuleParser,
    entry: &'a MetaEntry,
}

impl<'a> MetaItem<'a> {
    /// The key.
    pub fn key(&self) -> &'a str {
        self.parser.span(self.entry.key)
    }

    /// The first value: the bare or quoted value of `key = value`, or the first element of a list.
    ///
    /// An empty list (`key = []`) has no value. Check [`values_count`](Self::values_count) first.
    pub fn value(&self) -> &'a str {
        self.parser.meta_value(self.entry.values_start).unwrap()
    }

    /// The value at `index`, counting from 0, or `None` when `index` is past the last value.
    pub fn array_value(&self, index: u32) -> Option<&'a str> {
        let count = self.values_count();
        if index >= count {
            None
        } else {
            self.parser.meta_value(self.entry.values_start + index)
        }
    }

    /// Number of values.
    #[inline(always)]
    pub fn values_count(&self) -> u32 {
        self.entry.values_end - self.entry.values_start
    }

    /// `true` when the value was written as a `[ ... ]` list, even of one value.
    /// Useful only to insist on brackets; [`array_value`](Self::array_value) works either way.
    pub fn is_list(&self) -> bool {
        self.entry.is_list
    }
}

pub(crate) trait MetaEntryIndex {
    fn as_index(&self, parser: &RuleParser) -> Option<usize>;
}
impl MetaEntryIndex for usize {
    fn as_index(&self, _: &RuleParser) -> Option<usize> {
        Some(*self)
    }
}
impl MetaEntryIndex for u64 {
    fn as_index(&self, _: &RuleParser) -> Option<usize> {
        Some(*self as usize)
    }
}
impl MetaEntryIndex for u32 {
    fn as_index(&self, _: &RuleParser) -> Option<usize> {
        Some(*self as usize)
    }
}
impl MetaEntryIndex for u16 {
    fn as_index(&self, _: &RuleParser) -> Option<usize> {
        Some(*self as usize)
    }
}
impl MetaEntryIndex for u8 {
    fn as_index(&self, _: &RuleParser) -> Option<usize> {
        Some(*self as usize)
    }
}
impl MetaEntryIndex for &str {
    fn as_index(&self, parser: &RuleParser) -> Option<usize> {
        parser.hash_to_index(hasher::hash(self))
    }
}
impl MetaEntryIndex for String {
    fn as_index(&self, parser: &RuleParser) -> Option<usize> {
        parser.hash_to_index(hasher::hash(self.as_str()))
    }
}