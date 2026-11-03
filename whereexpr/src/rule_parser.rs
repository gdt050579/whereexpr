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
    const EMPTY: Self = Self { start: usize::MAX, end: usize::MAX };
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
struct RuleParser {
    data: Vec<u8>,
    poz: usize,
    rule_name: Span,
    type_name: Span,
}
impl RuleParser {
    pub(crate) fn new(s: &str) -> Self {
        Self {
            data: s.as_bytes().to_vec(),
            poz: 0,
            rule_name: Span::EMPTY,
            type_name: Span::EMPTY,
        }
    }
    fn parse(&mut self) -> Result<(), Error> {
        // reset all fields
        self.rule_name = Span::EMPTY;
        self.type_name = Span::EMPTY;
        self.poz = self.skip_whitespace(self.poz);
        if !self.check_expect(self.poz, "rule") {
            return Err(Error::ExpectingRuleKeyword(self.build_error_string(self.poz, 4)));
        }
        self.poz = self.skip_whitespace(self.poz + 4);
        if !self.check_word_start(self.poz) {
            return Err(Error::ExpectingRuleName(self.build_error_string(self.poz, 1)));
        }
        self.rule_name.start = self.poz;
        self.poz = self.parse_word(self.poz);
        self.rule_name.end = self.poz;
        self.poz = self.skip_whitespace(self.poz);
        
        if !self.check_expect(self.poz, ":") {
            return Err(Error::ExpectingColon(self.build_error_string(self.poz, 1)));
        }
        self.poz = self.skip_whitespace(self.poz + 1);
        if !self.check_word_start(self.poz) {
            return Err(Error::ExpectingTypeName(self.build_error_string(self.poz, 1)));
        }
        self.type_name.start = self.poz;
        self.poz = self.parse_word(self.poz);
        self.type_name.end = self.poz;
        self.poz = self.skip_whitespace(self.poz);
        if !self.check_expect(self.poz, "{") {
            return Err(Error::ExpectingOpeningBraceAfterTypeName(self.build_error_string(self.poz, 1)));
        }
        self.poz = self.skip_whitespace(self.poz + 1);


        Ok(())
    }
    fn check_word_start(&self, pos: usize) -> bool {
        if pos >= self.data.len() {
            return false;
        }
        return matches!(self.data[pos], b'a'..=b'z' | b'A'..=b'Z' | b'_');
    }
    fn check_string_start(&self, pos: usize) -> bool {
        if pos >= self.data.len() {
            return false;
        }
        return matches!(self.data[pos], b'"' | b'\'');
    }
    fn check_expect(&self, pos: usize, expected: &str) -> bool {
        let b = expected.as_bytes();
        if pos + b.len() > self.data.len() {
            false
        } else {
            for i in 0..b.len() {
                if self.data[pos + i].to_ascii_lowercase() != b[i].to_ascii_lowercase() {
                    return false;
                }
            }
            true
        }
    }
    fn skip_whitespace(&self, start: usize) -> usize {
        if start >= self.data.len() {
            return start;
        }
        let mut i = start;
        while i < self.data.len() && self.data[i].is_ascii_whitespace() {
            i += 1;
        }
        i
    }
    fn skip_until_eol(&self, start: usize) -> usize {
        if start >= self.data.len() {
            return start;
        }
        let mut i = start;
        while i < self.data.len() && self.data[i] != b'\n' && self.data[i] != b'\r' {
            i += 1;
        }
        i
    }
    fn parse_word(&self, start: usize) -> usize {
        if start >= self.data.len() {
            return start;
        }
        let mut i = start;
        while i < self.data.len() && matches!(self.data[i], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_') {
            i += 1;
        }
        i
    }
    fn parse_string(&self, start: usize) -> Option<(usize, u8)> {
        if start >= self.data.len() {
            return None;
        }
        let triple_quote = start + 3 <= self.data.len() && self.data[start] == self.data[start + 1] && self.data[start + 1] == self.data[start + 2];
        let quote_char = self.data[start];
        if triple_quote {
            let mut i = start + 3;
            while i + 3 < self.data.len() && (self.data[i] != quote_char || self.data[i + 1] != quote_char || self.data[i + 2] != quote_char) {
                i += 1;
            }
            if i >= self.data.len() {
                None
            } else {
                Some((i + 3, 3))
            }
        } else {
            let mut i = start + 1;
            while i < self.data.len() && self.data[i] != quote_char {
                i += 1;
            }
            if i >= self.data.len() {
                None
            } else {
                Some((i + 1, 1))
            }
        }
    }
    fn build_error_string(&self, pos: usize, size: usize) -> String {
        todo!()
    }
}
