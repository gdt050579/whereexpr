// ===== shared helpers =====

fn starts_with_ignore_case(value: &str, pattern: &str, ascii_only: bool) -> bool {
    if ascii_only {
        value.len() >= pattern.len()
            && value[..pattern.len()].eq_ignore_ascii_case(pattern)
    } else {
        let mut value_chars = value.chars();
        for tc in pattern.chars() {
            let mut vc_lower = match value_chars.next() {
                None => return false,
                Some(vc) => vc.to_lowercase(),
            };
            let mut tc_lower = tc.to_lowercase();
            loop {
                match (vc_lower.next(), tc_lower.next()) {
                    (Some(a), Some(b)) if a == b => continue,
                    (None, None)                 => break,
                    _                            => return false,
                }
            }
        }
        true
    }
}

fn ends_with_ignore_case(value: &str, pattern: &str, ascii_only: bool) -> bool {
    if ascii_only {
        value.len() >= pattern.len()
            && value[value.len() - pattern.len()..].eq_ignore_ascii_case(pattern)
    } else {
        let mut value_chars = value.chars().rev();
        for tc in pattern.chars().rev() {
            let mut vc_lower = match value_chars.next() {
                None => return false,
                Some(vc) => vc.to_lowercase(),
            };
            let mut tc_lower = tc.to_lowercase();
            loop {
                match (vc_lower.next(), tc_lower.next()) {
                    (Some(a), Some(b)) if a == b => continue,
                    (None, None)                 => break,
                    _                            => return false,
                }
            }
        }
        true
    }
}

fn contains_ignore_case(value: &str, pattern: &str, ascii_only: bool) -> bool {
    if ascii_only {
        // slide a window of pattern.len() bytes across value
        let plen = pattern.len();
        if value.len() < plen { return false; }
        value.as_bytes()
            .windows(plen)
            .any(|w| {
                // SAFETY: value is valid UTF-8 and we're slicing on byte boundaries
                // that are guaranteed valid for ASCII-only patterns
                unsafe { std::str::from_utf8_unchecked(w) }
                    .eq_ignore_ascii_case(pattern)
            })
    } else {
        // collect pattern char count once
        let pattern_char_count = pattern.chars().count();
        let value_chars: Vec<char> = value.chars().collect();
        if value_chars.len() < pattern_char_count { return false; }
        value_chars
            .windows(pattern_char_count)
            .any(|w| {
                w.iter().zip(pattern.chars()).all(|(vc, tc)| {
                    vc.to_lowercase().eq(tc.to_lowercase())
                })
            })
    }
}

fn equals_ignore_case(value: &str, pattern: &str, ascii_only: bool) -> bool {
    if ascii_only {
        value.eq_ignore_ascii_case(pattern)
    } else {
        let mut value_chars = value.chars();
        for tc in pattern.chars() {
            let mut vc_lower = match value_chars.next() {
                None => return false,
                Some(vc) => vc.to_lowercase(),
            };
            let mut tc_lower = tc.to_lowercase();
            loop {
                match (vc_lower.next(), tc_lower.next()) {
                    (Some(a), Some(b)) if a == b => continue,
                    (None, None) => break,
                    _ => return false,
                }
            }
        }
        // ensure value has no remaining characters
        value_chars.next().is_none()
    }
}

// ===== shared constructor helper =====

fn prepare_pattern(text: &str, ignore_case: bool) -> (String, bool) {
    let ascii_only = text.is_ascii();
    let text = if ignore_case {
        if ascii_only { text.to_ascii_lowercase() } else { text.to_lowercase() }
    } else {
        text.to_string()
    };
    (text, ascii_only)
}

// ===== predicates =====

#[derive(Debug)]
pub(crate) struct StartsWith {
    text: String,
    ignore_case: bool,
    ascii_only: bool,
}

impl StartsWith {
    pub(crate) fn new(text: &str, ignore_case: bool) -> Self {
        let (text, ascii_only) = prepare_pattern(text, ignore_case);
        Self { text, ignore_case, ascii_only }
    }
    pub(crate) fn evaluate(&self, value: &str) -> bool {
        if !self.ignore_case { return value.starts_with(&self.text); }
        starts_with_ignore_case(value, &self.text, self.ascii_only)
    }
}

#[derive(Debug)]
pub(crate) struct EndsWith {
    text: String,
    ignore_case: bool,
    ascii_only: bool,
}

impl EndsWith {
    pub(crate) fn new(text: &str, ignore_case: bool) -> Self {
        let (text, ascii_only) = prepare_pattern(text, ignore_case);
        Self { text, ignore_case, ascii_only }
    }
    pub(crate) fn evaluate(&self, value: &str) -> bool {
        if !self.ignore_case { return value.ends_with(&self.text); }
        ends_with_ignore_case(value, &self.text, self.ascii_only)
    }
}

#[derive(Debug)]
pub(crate) struct Contains {
    text: String,
    ignore_case: bool,
    ascii_only: bool,
}

impl Contains {
    pub(crate) fn new(text: &str, ignore_case: bool) -> Self {
        let (text, ascii_only) = prepare_pattern(text, ignore_case);
        Self { text, ignore_case, ascii_only }
    }
    pub(crate) fn evaluate(&self, value: &str) -> bool {
        if !self.ignore_case { return value.contains(&self.text as &str); }
        contains_ignore_case(value, &self.text, self.ascii_only)
    }
}

#[derive(Debug)]
pub(crate) struct Equals {
    text: String,
    ignore_case: bool,
    ascii_only: bool,
}

impl Equals {
    pub(crate) fn new(text: &str, ignore_case: bool) -> Self {
        let (text, ascii_only) = prepare_pattern(text, ignore_case);
        Self { text, ignore_case, ascii_only }
    }
    pub(crate) fn evaluate(&self, value: &str) -> bool {
        if !self.ignore_case { return value == self.text; }
        equals_ignore_case(value, &self.text, self.ascii_only)
    }
}