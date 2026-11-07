use crate::Error;
use super::{RuleDefinition, Span, MetaEntry};
/*
rule student-review: Student {
    age-is-prime = "age is-one-of [19, 23, 29, 31]"
    common       = "name.family ends-with 'escu' {ignore-case}"
    old-enroll   = "enroll.year in-range [2010, 2020]"
    good-at-math = "grade.math >= 8"
} where {
    age-is-prime OR common OR (old-enroll AND good-at-math)
}

rule adults: Person { adult = "age >= 18" } where { adult }

*/

pub struct RuleParser {
    data: String,
    poz: usize,
    rule_name: Span,
    type_name: Span,
    expr: Span,
    conditions: Vec<(Span, Span)>,
    meta: Vec<MetaEntry>,
    meta_items: Vec<Span>,
    meta_hashes: Vec<(u64, usize)>,
    finished: bool,
}
impl RuleParser {
    pub fn new(s: &str) -> Self {
        Self {
            data: s.to_string(),
            poz: 0,
            rule_name: Span::EMPTY,
            type_name: Span::EMPTY,
            expr: Span::EMPTY,
            conditions: Vec::new(),
            meta: Vec::new(),
            meta_items: Vec::new(),
            meta_hashes: Vec::new(),
            finished: false,
        }
    }
    pub(crate) fn parse_next(&mut self) -> Result<bool, Error> {
        // reset all fields
        self.rule_name = Span::EMPTY;
        self.type_name = Span::EMPTY;
        self.expr = Span::EMPTY;
        self.conditions.clear();
        self.meta.clear();
        self.meta_items.clear();
        self.meta_hashes.clear();
        self.poz = self.skip_whitespace(self.poz);
        if self.poz >= self.data.len() {
            return Ok(false);
        }
        if !self.check_expect(self.poz, "rule", true) {
            return Err(Error::ExpectingRuleKeyword(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        self.poz = self.skip_whitespace(self.poz + 4);
        if !self.check_name_start(self.poz) {
            return Err(Error::ExpectingRuleName(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        (self.rule_name, self.poz) = self.parse_word(self.poz);
        self.poz = self.skip_whitespace(self.poz);

        if !self.check_expect(self.poz, ":", false) {
            return Err(Error::ExpectingColon(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        self.poz = self.skip_whitespace(self.poz + 1);
        if !self.check_word_start(self.poz) {
            return Err(Error::ExpectingTypeName(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        (self.type_name, self.poz) = self.parse_word(self.poz);
        self.poz = self.skip_whitespace(self.poz);
        if !self.check_expect(self.poz, "{", false) {
            return Err(Error::ExpectingOpeningBraceAfterTypeName(
                self.poz as u32,
                self.poz as u32 + 1,
                self.data.clone(),
            ));
        }
        self.poz = self.skip_whitespace(self.poz + 1);
        // loop for conditions
        while let Some((key, value, _)) = self.parse_key_value(false)? {
            self.conditions.push((key, value));
        }
        self.poz = self.skip_whitespace(self.poz);
        if !self.check_expect(self.poz, "where", true) {
            return Err(Error::ExpectingWhereKeyword(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        self.poz = self.skip_whitespace(self.poz + 5);
        if !self.check_expect(self.poz, "{", false) {
            return Err(Error::ExpectingOpeningBraceAfterWhereKeyword(
                self.poz as u32,
                self.poz as u32 + 1,
                self.data.clone(),
            ));
        }
        self.poz = self.skip_whitespace(self.poz + 1);
        // caut } de final
        let buf = self.data.as_bytes();
        let start = self.poz;
        let mut last_expr_char_poz = self.poz;
        while self.poz < buf.len() && buf[self.poz] != b'}' {
            match buf[self.poz] {
                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-'| b'0'..=b'9' | b'(' | b')' | b'&' | b'|' | b'!' | b'~' => {
                    self.poz += 1;
                    last_expr_char_poz = self.poz;
                }
                b' ' | b'\t' | b'\n' | b'\r' => {
                    self.poz += 1;
                }
                _ => {
                    return Err(Error::InvalidCharacterInWhereExpression(
                        self.poz as u32,
                        self.poz as u32 + 1,
                        self.data.clone(),
                    ));
                }
            }
        }
        if self.poz >= buf.len() {
            return Err(Error::ExpectingClosingBraceAfterWhereKeyword(
                self.poz as u32,
                self.poz as u32 + 1,
                self.data.clone(),
            ));
        }
        self.expr = Span::new(start, last_expr_char_poz);
        // step past the closing `}` so the next call starts after this rule
        self.poz = self.skip_whitespace(self.poz + 1);
        if self.check_expect(self.poz, "with", true) {
            // an optional `with meta { ... }` block
            self.poz = self.skip_whitespace(self.poz + 4);
            if !self.check_expect(self.poz, "meta", true) {
                return Err(Error::ExpectingMetaKeyword(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
            }
            self.poz = self.skip_whitespace(self.poz + 4);
            if !self.check_expect(self.poz, "{", false) {
                return Err(Error::ExpectingOpeningBraceAfterMetaKeyword(
                    self.poz as u32,
                    self.poz as u32 + 1,
                    self.data.clone(),
                ));
            }
            self.poz = self.skip_whitespace(self.poz + 1);
            loop {
                let items_start = self.meta_items.len();
                let Some((key, value, is_list)) = self.parse_key_value(true)? else {
                    break;
                };
                let hash_name = super::hasher::hash(&self.data[key.start..key.end]);
                self.meta_hashes.push((hash_name, self.meta.len()));
                if !is_list {
                    self.meta_items.push(value);
                }
                self.meta.push(MetaEntry {
                    key,
                    values_start: items_start as u32,
                    values_end: self.meta_items.len() as u32,
                    is_list,
                });
            }
            self.poz = self.skip_whitespace(self.poz);
        }
        // sort meta_hashes by hash
        self.meta_hashes.sort_by_key(|(hash, _)| *hash);
        // check duplicates (hashes ignore ASCII case, so `owner` and `Owner` collide)
        for i in 1..self.meta_hashes.len() {
            if self.meta_hashes[i].0 == self.meta_hashes[i - 1].0 {
                let key = self.meta[self.meta_hashes[i].1].key;
                return Err(Error::DuplicateMetaKey(key.start as u32, key.end as u32, self.data.clone()));
            }
        }
        Ok(true)
    }

    /// Reads the next rule and returns its definition.
    ///
    /// - `Ok(Some(definition))` is the rule just read. It borrows this parser, so use it
    ///   (for example, compile it with [`RuleDefinition::expression`]) before reading the next rule.
    /// - `Ok(None)` means there are no more rules.
    /// - `Err` is a syntax error. Reading stops there, because the start of the next rule
    ///   is not known, and every later call returns `Ok(None)`.
    ///
    /// ```rust
    /// use whereexpr::{Attributes, RuleParser};
    ///
    /// #[derive(Attributes)]
    /// struct Person {
    ///     age: u32,
    /// }
    ///
    /// let text = r#"
    ///     rule adults:   Person { adult = "age >= 18" } where { adult }
    ///     rule children: Person { child = "age < 18" } where { child }
    /// "#;
    /// let mut parser = RuleParser::new(text);
    /// let mut names = Vec::new();
    /// while let Some(definition) = parser.next_rule_definition().unwrap() {
    ///     let expression = definition.expression::<Person>().unwrap();
    ///     if expression.matches(&Person { age: 40 }) {
    ///         names.push(definition.name().to_string());
    ///     }
    /// }
    /// assert_eq!(names, ["adults"]);
    /// ```
    pub fn next_rule_definition(&mut self) -> Result<Option<RuleDefinition<'_>>, Error> {
        if self.finished {
            return Ok(None);
        }
        match self.parse_next() {
            Ok(true) => Ok(Some(RuleDefinition::new(self))),
            Ok(false) => {
                self.finished = true;
                Ok(None)
            }
            Err(e) => {
                self.finished = true;
                Err(e)
            }
        }
    }
    pub(crate) fn rule_name(&self) -> Option<&str> {
        if self.rule_name.start == usize::MAX {
            None
        } else {
            Some(&self.data[self.rule_name.start..self.rule_name.end])
        }
    }
    pub(crate) fn type_name(&self) -> Option<&str> {
        if self.type_name.start == usize::MAX {
            None
        } else {
            Some(&self.data[self.type_name.start..self.type_name.end])
        }
    }
    pub(crate) fn expr(&self) -> Option<&str> {
        if self.expr.start == usize::MAX {
            None
        } else {
            Some(&self.data[self.expr.start..self.expr.end])
        }
    }
    pub(crate) fn conditions_count(&self) -> usize {
        self.conditions.len()
    }
    pub(crate) fn condition_name(&self, index: usize) -> Option<&str> {
        if index >= self.conditions.len() {
            None
        } else {
            Some(&self.data[self.conditions[index].0.start..self.conditions[index].0.end])
        }
    }
    pub(crate) fn condition_value(&self, index: usize) -> Option<&str> {
        if index >= self.conditions.len() {
            None
        } else {
            Some(&self.data[self.conditions[index].1.start..self.conditions[index].1.end])
        }
    }
    pub(crate) fn meta_count(&self) -> usize {
        self.meta.len()
    }
    pub(super) fn metaentry_by_index(&self, index: usize) -> Option<&MetaEntry> {
        self.meta.get(index)
    }
    pub(super) fn hash_to_index(&self, hash: u64) -> Option<usize> {
        let found = self.meta_hashes.binary_search_by_key(&hash, |(h, _)| *h).ok()?;
        Some(self.meta_hashes[found].1)
    }
    #[inline(always)]
    pub(super) fn span(&self, span: Span) -> &str {
        &self.data[span.start..span.end]
    }
    #[inline(always)]
    pub(crate) fn meta_value(&self, index: u32) -> Option<&str> {
        if let Some(span) = self.meta_items.get(index as usize) {
            Some(&self.data[span.start..span.end])
        } else {
            None
        }
    }
    fn check_word_start(&self, pos: usize) -> bool {
        let buf = self.data.as_bytes();
        if pos >= buf.len() {
            return false;
        }
        return matches!(buf[pos], b'a'..=b'z' | b'A'..=b'Z' | b'_');
    }
    fn check_value_start(&self, pos: usize) -> bool {
        let buf = self.data.as_bytes();
        if pos >= buf.len() {
            return false;
        }
        return matches!(buf[pos], b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-'| b'.' | b'+' | b':' | b'0'..=b'9');
    }
    fn parse_key_value(&mut self, supports_array: bool) -> Result<Option<(Span, Span, bool)>, Error> {
        self.poz = self.skip_whitespace(self.poz);
        if self.check_expect(self.poz, "}", false) {
            self.poz += 1;
            return Ok(None);
        }
        if self.poz >= self.data.len() {
            return Err(Error::ExpectingClosingBrace(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        let key: Span;
        let value: Span;
        let mut is_list = false;
        if !self.check_name_start(self.poz) {
            return Err(Error::ExpectingKeyName(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        (key, self.poz) = self.parse_word(self.poz);
        self.poz = self.skip_whitespace(self.poz);
        if !self.check_expect(self.poz, "=", false) {
            return Err(Error::ExpectingEqualSignAfterKeyName(
                self.poz as u32,
                self.poz as u32 + 1,
                self.data.clone(),
            ));
        }
        self.poz = self.skip_whitespace(self.poz + 1);
        if self.check_string_start(self.poz) {
            (value, self.poz) =
                self.parse_string(self.poz)
                    .ok_or(Error::UnterminatedString(self.poz as u32, self.poz as u32 + 1, self.data.clone()))?;
        } else if self.check_value_start(self.poz) {
            (value, self.poz) = self.parse_value(self.poz);
        } else if supports_array && self.check_expect(self.poz, "[", false) {
            (value, self.poz) = self.parse_array(self.poz)?;
            is_list = true;
        } else {
            return Err(Error::ExpectingKeyValue(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        self.poz = self.skip_whitespace(self.poz);
        Ok(Some((key, value, is_list)))
    }
    /// Reads a `[ ... ]` list starting at `start` (the `[`), adding each value to `meta_items`.
    ///
    /// Values are quoted strings or bare words, separated by commas. `[]` is an empty list,
    /// and a comma after the last value is allowed.
    fn parse_array(&mut self, start: usize) -> Result<(Span, usize), Error> {
        if start >= self.data.len() {
            return Err(Error::UnterminatedArray(start as u32, start as u32 + 1, self.data.clone()));
        }
        let mut i = self.skip_whitespace(start + 1);
        // true at the start of the list and after a comma
        let mut expecting_value = true;
        loop {
            let Some(&c) = self.data.as_bytes().get(i) else {
                return Err(Error::UnterminatedArray(start as u32, start as u32 + 1, self.data.clone()));
            };
            match c {
                b']' => return Ok((Span::new(start, i + 1), i + 1)),
                b'}' => {
                    return Err(Error::UnexpectedClosingBraceInArray(i as u32, i as u32 + 1, self.data.clone()));
                }
                b',' if expecting_value => {
                    return Err(Error::ExpectingArrayValue(i as u32, i as u32 + 1, self.data.clone()));
                }
                b',' => {
                    expecting_value = true;
                    i = self.skip_whitespace(i + 1);
                }
                _ if !expecting_value => {
                    // point at the whole word that follows, not just its first character
                    let end = if self.check_value_start(i) { self.parse_value(i).1 } else { i + 1 };
                    return Err(Error::ExpectingCommaInArray(i as u32, end as u32, self.data.clone()));
                }
                b'"' | b'\'' => {
                    let (span, next) = self
                        .parse_string(i)
                        .ok_or(Error::UnterminatedString(i as u32, i as u32 + 1, self.data.clone()))?;
                    self.meta_items.push(span);
                    expecting_value = false;
                    i = self.skip_whitespace(next);
                }
                _ if self.check_value_start(i) => {
                    let (span, next) = self.parse_value(i);
                    self.meta_items.push(span);
                    expecting_value = false;
                    i = self.skip_whitespace(next);
                }
                _ => return Err(Error::ExpectingArrayValue(i as u32, i as u32 + 1, self.data.clone())),
            }
        }
    }
    fn check_name_start(&self, pos: usize) -> bool {
        matches!(self.data.as_bytes().get(pos), Some(b'a'..=b'z' | b'A'..=b'Z'))
    }
    fn check_string_start(&self, pos: usize) -> bool {
        let buf = self.data.as_bytes();
        if pos >= buf.len() {
            return false;
        }
        return matches!(buf[pos], b'"' | b'\'');
    }
    fn check_expect(&self, pos: usize, expected: &str, check_full_word: bool) -> bool {
        let buf = self.data.as_bytes();
        let b = expected.as_bytes();
        if pos + b.len() > buf.len() {
            false
        } else {
            for i in 0..b.len() {
                if buf[pos + i].to_ascii_lowercase() != b[i].to_ascii_lowercase() {
                    return false;
                }
            }
            if (check_full_word) && (pos + b.len() < buf.len()) {
                let c = buf[pos + b.len()];
                if matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_') {
                    return false;
                }
            }
            true
        }
    }
    fn skip_whitespace(&self, start: usize) -> usize {
        let buf = self.data.as_bytes();
        if start >= buf.len() {
            return start;
        }
        let mut i = start;
        while i < buf.len() && buf[i].is_ascii_whitespace() {
            i += 1;
        }
        i
    }
    fn skip_until_eol(&self, start: usize) -> usize {
        let buf = self.data.as_bytes();
        if start >= buf.len() {
            return start;
        }
        let mut i = start;
        while i < self.data.len() && buf[i] != b'\n' && buf[i] != b'\r' {
            i += 1;
        }
        i
    }
    fn parse_word(&self, start: usize) -> (Span, usize) {
        let buf = self.data.as_bytes();
        if start >= buf.len() {
            return (Span::EMPTY, start);
        }
        let mut i = start;
        while i < self.data.len() && matches!(buf[i], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_') {
            i += 1;
        }
        (Span::new(start, i), i)
    }
    fn parse_value(&self, start: usize) -> (Span, usize) {
        let buf = self.data.as_bytes();
        if start >= buf.len() {
            return (Span::EMPTY, start);
        }
        let mut i = start;
        if buf[i] == b'+' {
            i += 1;
        }
        while i < self.data.len() && matches!(buf[i], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b':') {
            i += 1;
        }
        (Span::new(start, i), i)
    }    
    fn parse_string(&self, start: usize) -> Option<(Span, usize)> {
        let buf = self.data.as_bytes();
        if start >= buf.len() {
            return None;
        }
        let triple_quote = start + 3 <= buf.len() && buf[start] == buf[start + 1] && buf[start + 1] == buf[start + 2];
        let quote_char = buf[start];
        if triple_quote {
            let mut i = start + 3;
            while i + 3 <= buf.len() {
                if buf[i] == quote_char && buf[i + 1] == quote_char && buf[i + 2] == quote_char {
                    return Some((Span::new(start + 3, i), i + 3));
                }
                i += 1;
            }
            None
        } else {
            let mut i = start + 1;
            while i < buf.len() && buf[i] != quote_char {
                i += 1;
            }
            if i >= buf.len() {
                None
            } else {
                Some((Span::new(start + 1, i), i + 1))
            }
        }
    }
}
