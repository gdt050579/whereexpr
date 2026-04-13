use crate::Error;

pub(crate) fn parse(text: &str) -> Result<(&str, usize), Error> {
    if text.trim().is_empty() {
        return Err(Error::EmptyCondition);
    }

    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // skip whitespaces
    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    // first character must be a letter
    if !bytes[i].is_ascii_alphabetic() {
        return Err(Error::InvalidAttributeName(i as u16, (i + 1) as u16, text.to_string()));
    }
    let start_attr = i;
    while i < len {
        match bytes[i] {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' => {
                i += 1;
            }
            _ => break,
        }
    }
    let attr_end = i;
    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    Ok((&text[start_attr..attr_end], i))
}