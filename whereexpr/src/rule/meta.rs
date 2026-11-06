use std::fmt::Display;
use std::iter::FusedIterator;
use std::ops::Range;
use std::str::FromStr;

use crate::Error;
use crate::RuleParser;

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
/// assert_eq!(meta.get("owner").unwrap().value(), Some("ops"));
/// assert_eq!(meta.get("tags").unwrap().values().collect::<Vec<_>>(), ["disk", "high load"]);
/// for index in 0..meta.len() {
///     let item = meta.get_by_index(index).unwrap();
///     println!("{} has {} value(s)", item.key(), item.len());
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

    /// The entry with this key, if there is one. Keys are case-sensitive.
    pub fn get(&self, key: &str) -> Option<MetaItem<'a>> {
        self.parser.meta_find(key).map(|index| MetaItem { parser: self.parser, index })
    }

    /// The entry at `index`, counting from 0 in the order they are written,
    /// or `None` when `index` is not less than [`len`](Self::len).
    pub fn get_by_index(&self, index: usize) -> Option<MetaItem<'a>> {
        if index < self.len() {
            Some(MetaItem { parser: self.parser, index })
        } else {
            None
        }
    }
}

/// One `key = value` entry of a [`Meta`] block.
///
/// The entry holds a list of values: one for `key = value`, any number for `key = [ ... ]`.
/// Whether a key should hold one value or several is up to the reader, so `tags = disk`
/// can be read as a list of one, and [`value`](Self::value) rejects a list of two.
#[derive(Clone, Copy)]
pub struct MetaItem<'a> {
    parser: &'a RuleParser,
    index: usize,
}

impl<'a> MetaItem<'a> {
    /// The key.
    pub fn key(&self) -> &'a str {
        self.parser.meta_key(self.index).unwrap()
    }

    /// The value, when the entry holds exactly one: `Some` for `key = value` and for a
    /// one-value list `key = [value]`, `None` for `[]` or a list of two or more values.
    pub fn value(&self) -> Option<&'a str> {
        let range = self.parser.meta_items_range(self.index);
        if range.len() == 1 {
            Some(self.parser.meta_item(range.start))
        } else {
            None
        }
    }

    /// All the values, in the order they are written.
    pub fn values(&self) -> MetaValues<'a> {
        MetaValues {
            parser: self.parser,
            items: self.parser.meta_items_range(self.index),
        }
    }

    /// Number of values.
    pub fn len(&self) -> usize {
        self.parser.meta_items_range(self.index).len()
    }

    /// `true` for an empty list (`key = []`).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `true` when the value was written as a `[ ... ]` list, even of one value.
    /// Useful only to insist on brackets; [`values`](Self::values) works either way.
    pub fn is_list(&self) -> bool {
        self.parser.meta_is_list(self.index)
    }

    /// Parses the single value with [`FromStr`].
    ///
    /// Returns [`Error::InvalidMetaValue`], pointing at the value in the rule text, when the
    /// entry does not hold exactly one value or the value does not parse.
    ///
    /// ```rust
    /// use whereexpr::RuleParser;
    ///
    /// let text = r#"rule r: T { a = "x is 1" } where { a } with meta { priority = 3 owner = ops }"#;
    /// let mut parser = RuleParser::new(text);
    /// let definition = parser.next_rule_definition().unwrap().unwrap();
    /// let meta = definition.meta();
    ///
    /// assert_eq!(meta.get("priority").unwrap().parse::<i32>().unwrap(), 3);
    /// assert!(meta.get("owner").unwrap().parse::<i32>().is_err());
    /// ```
    pub fn parse<T: FromStr>(&self) -> Result<T, Error>
    where
        T::Err: Display,
    {
        let range = self.parser.meta_items_range(self.index);
        if range.len() != 1 {
            return Err(self.error(&format!("expected a single value, found {}", Self::describe_count(range.len()))));
        }
        self.parse_item(range.start)
    }

    /// Parses every value with [`FromStr`], in order. An empty list gives an empty `Vec`.
    ///
    /// Returns [`Error::InvalidMetaValue`] pointing at the first value that does not parse.
    pub fn parse_all<T: FromStr>(&self) -> Result<Vec<T>, Error>
    where
        T::Err: Display,
    {
        self.parser.meta_items_range(self.index).map(|item| self.parse_item(item)).collect()
    }

    /// An [`Error::InvalidMetaValue`] for this entry with your own message, pointing at
    /// its value in the rule text. Use it to report checks of your own, so they read
    /// like the library's errors.
    ///
    /// ```rust
    /// use whereexpr::RuleParser;
    ///
    /// let text = r#"rule r: T { a = "x is 1" } where { a } with meta { severity = urgent }"#;
    /// let mut parser = RuleParser::new(text);
    /// let definition = parser.next_rule_definition().unwrap().unwrap();
    /// let severity = definition.meta().get("severity").unwrap();
    ///
    /// if !matches!(severity.value(), Some("low" | "high")) {
    ///     let error = severity.error("expected `low` or `high`");
    ///     # let _ = error;
    /// }
    /// ```
    pub fn error(&self, message: &str) -> Error {
        let (start, end) = self.parser.meta_value_position(self.index);
        self.error_at(start, end, message)
    }

    fn parse_item<T: FromStr>(&self, item: usize) -> Result<T, Error>
    where
        T::Err: Display,
    {
        let text = self.parser.meta_item(item);
        text.parse::<T>().map_err(|e| {
            let (start, end) = self.parser.meta_item_position(item);
            self.error_at(start, end, &format!("cannot parse `{text}`: {e}"))
        })
    }

    fn error_at(&self, start: u32, end: u32, message: &str) -> Error {
        Error::InvalidMetaValue(
            start,
            end,
            self.parser.text().to_string(),
            format!("Invalid value for meta key '{}': {message}", self.key()),
        )
    }

    fn describe_count(count: usize) -> String {
        match count {
            0 => "an empty list".to_string(),
            n => format!("a list of {n} values"),
        }
    }
}

/// Iterator over the values of a [`MetaItem`], in the order they are written.
pub struct MetaValues<'a> {
    parser: &'a RuleParser,
    items: Range<usize>,
}

impl<'a> Iterator for MetaValues<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        self.items.next().map(|item| self.parser.meta_item(item))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}

impl DoubleEndedIterator for MetaValues<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.items.next_back().map(|item| self.parser.meta_item(item))
    }
}

impl ExactSizeIterator for MetaValues<'_> {}
impl FusedIterator for MetaValues<'_> {}
