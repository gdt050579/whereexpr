use crate::Error;

pub(crate) fn parse(text: &str) -> Result<(&str, usize), Error> {
    if text.is_empty() {
        return Err(Error::EmptyAttribute);
    }

    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // first character must be a letter
    if !bytes[0].is_ascii_alphabetic() {
        return Err(Error::InvalidAttributeName);
    }

    while i < len {
        match bytes[i] {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' => {
                i += 1;
            }
            b'(' => {
                let mut depth = 1usize;
                i += 1;
                while i < len && depth > 0 {
                    match bytes[i] {
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                if depth != 0 {
                    return Err(Error::UnbalancedParentheses);
                }
                // after closing parenthesis - attribute name is done
                break;
            }
            _ => break,
        }
    }

    if i == 0 {
        return Err(Error::EmptyAttribute);
    }

    let attr_end = i;

    // skip optional whitespace
    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    Ok((&text[..attr_end], i))
}