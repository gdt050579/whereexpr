use super::Error;
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
struct Span {
    start: usize,
    end: usize,
}
impl Span {
    const EMPTY: Self = Self {
        start: usize::MAX,
        end: usize::MAX,
    };
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
pub(crate) struct RuleParser {
    data: String,
    poz: usize,
    rule_name: Span,
    type_name: Span,
    expr: Span,
    conditions: Vec<(Span, Span)>,
    meta: Vec<(Span, Span)>,
}
impl RuleParser {
    pub(crate) fn new(s: &str) -> Self {
        Self {
            data: s.to_string(),
            poz: 0,
            rule_name: Span::EMPTY,
            type_name: Span::EMPTY,
            expr: Span::EMPTY,
            conditions: Vec::new(),
            meta: Vec::new(),
        }
    }
    pub(crate) fn parse_next(&mut self) -> Result<bool, Error> {
        // reset all fields
        self.rule_name = Span::EMPTY;
        self.type_name = Span::EMPTY;
        self.expr = Span::EMPTY;
        self.conditions.clear();
        self.meta.clear();
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
        loop {
            let (key, value, should_continue) = self.parse_key_value()?;
            self.conditions.push((key, value));
            if !should_continue {
                break;
            }
        }
        self.poz = self.skip_whitespace(self.poz);
        if !self.check_expect(self.poz, "where", true) {
            return Err(Error::ExpectingWhereKeyword(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        self.poz = self.skip_whitespace(self.poz + 5);
        if !self.check_expect(self.poz, "{", false) {
            return Err(Error::ExpectingOpeningBraceAfterWhereKeyword(self.poz as u32, self.poz as u32 + 1, self.data.clone())); 
        }
        self.poz = self.skip_whitespace(self.poz + 1);
        // caut } de final 
        let buf = self.data.as_bytes(); 
        let start = self.poz;
        while self.poz < buf.len() && buf[self.poz] != b'}' {
            if !matches!(buf[self.poz], b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-'| b'0'..=b'9' | b'(' | b')' | b'&' | b'|' | b'!' | b'~' | b' ' | b'\t' | b'\n' | b'\r') {
                return Err(Error::InvalidCharacterInWhereExpression(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
            }
            self.poz += 1;
        }
        if self.poz >= buf.len() {
            return Err(Error::ExpectingClosingBraceAfterWhereKeyword(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        self.expr = Span::new(start, self.poz);
        // step past the closing `}` so the next call starts after this rule
        self.poz = self.skip_whitespace(self.poz + 1);
        if self.check_expect(self.poz, "with", true) {
            // avem si posibile meta
            self.poz = self.skip_whitespace(self.poz + 4);
            if !self.check_expect(self.poz, "meta", true) {
                return Err(Error::ExpectingMetaKeyword(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
            }
            self.poz = self.skip_whitespace(self.poz + 4);
            if !self.check_expect(self.poz, "{", false) {
                return Err(Error::ExpectingOpeningBraceAfterMetaKeyword(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
            }
            self.poz = self.skip_whitespace(self.poz + 1);
            // loop for meta
            loop {
                let (key, value, should_continue) = self.parse_key_value()?;
                self.meta.push((key, value));
                if !should_continue {
                    break;
                }                
            }
            self.poz = self.skip_whitespace(self.poz);
        }
        Ok(true)
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
    fn check_word_start(&self, pos: usize) -> bool {
        let buf = self.data.as_bytes();
        if pos >= buf.len() {
            return false;
        }
        return matches!(buf[pos], b'a'..=b'z' | b'A'..=b'Z' | b'_');
    }
    fn parse_key_value(&mut self) -> Result<(Span, Span, bool), Error> {
        let key: Span;
        let value: Span;
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
        } else if self.check_word_start(self.poz) {
            (value, self.poz) = self.parse_word(self.poz);
        } else {
            return Err(Error::ExpectingKeyValue(self.poz as u32, self.poz as u32 + 1, self.data.clone()));
        }
        self.poz = self.skip_whitespace(self.poz);
        let should_continue = if self.check_expect(self.poz, "}", false) {
            self.poz += 1;
            true
        } else {
            false
        };
        Ok((key, value, should_continue))
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
                Some((Span::new(start+1, i), i + 1))
            }
        }
    }
}
