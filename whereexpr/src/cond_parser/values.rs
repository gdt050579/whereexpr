use crate::Error;

struct Span {
    start: u32,
    end: u32,
    raw: bool,
}
impl Span {
    fn new(start: usize, end: usize, raw: bool) -> Self {
        Self {
            start: start as u32,
            end: end as u32,
            raw,
        }
    }
    fn as_slice<'a>(&self, txt: &'a str) -> &'a str {
        &txt[self.start as usize..self.end as usize]
    }
}
pub(crate) enum ParsedValue<'a> {
    Single(&'a str),
    List(Vec<&'a str>),
}
fn parse_regular_word(buf: &[u8], pos: usize, start: usize, txt: &str) -> Result<(Span, usize), Error> {
    let mut i = pos;
    while i < buf.len() {
        if buf[i].is_ascii_whitespace() || buf[i] == b',' {
            break;
        }
        i += 1;
    }
    Ok((Span::new(start + pos, start + i, true), i))
}
fn parse_single_quoted_string(buf: &[u8], pos: usize, start: usize, txt: &str) -> Result<(Span, usize), Error> {
    let mut i = pos;
    while (i < buf.len()) && (buf[i] != b'\'') {
        i += 1;
    }
    if i < buf.len() {
        Ok((Span::new(start + pos + 1, start + i, true), i + 1))
    } else {
        Err(Error::UnterminatedString(
            (start + pos) as u32,
            (start + buf.len()) as u32,
            txt.to_string(),
        ))
    }
}
fn parse_double_quoted_string(buf: &[u8], pos: usize, start: usize, txt: &str, copy_buffer: &mut String) -> Result<(Span, usize), Error> {
    let mut i = pos;
    let mut needs_unescape = false;
    while i < buf.len() {
        match buf[i] {
            b'\\' => {
                i += 2;
                needs_unescape = true;
            }
            b'"' => {
                if needs_unescape {
                    let raw = &txt[start + pos + 1..start + i];
                    return Ok((unescape(raw, start + pos + 1, txt, copy_buffer)?, i + 1));
                } else {
                    return Ok((Span::new(start + pos + 1, start + i, true), i + 1));
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    Err(Error::UnterminatedString(
        (start + pos) as u32,
        (start + buf.len()) as u32,
        txt.to_string(),
    ))
}
fn unescape<'a>(raw: &str, start: usize, txt: &str, copy_buffer: &'a mut String) -> Result<Span, Error> {
    copy_buffer.reserve(raw.len());
    let bytes = raw.as_bytes();
    let len = bytes.len();
    let mut j = 0;
    let mut span = Span {
        start: copy_buffer.len() as u32,
        end: 0,
        raw: false,
    };
    while j < len {
        match bytes[j] {
            b'\\' if j + 1 < len => match bytes[j + 1] {
                b'"' => {
                    copy_buffer.push('"');
                    j += 2;
                }
                b'\\' => {
                    copy_buffer.push('\\');
                    j += 2;
                }
                b'n' => {
                    copy_buffer.push('\n');
                    j += 2;
                }
                b't' => {
                    copy_buffer.push('\t');
                    j += 2;
                }
                b'r' => {
                    copy_buffer.push('\r');
                    j += 2;
                }
                _ => return Err(Error::InvalidEscapeSequence((start + j) as u32, (start + j + 2) as u32, txt.to_string())),
            },
            _ => {
                copy_buffer.push(bytes[j] as char);
                j += 1;
            }
        }
    }
    span.end = copy_buffer.len() as u32;
    Ok(span)
}
fn parse_list<'a>(buf: &'a [u8], start: usize, txt: &'a str, copy_buffer: &'a mut String) -> Result<ParsedValue<'a>, Error> {
    if buf.trim_ascii().is_empty() {
        return Err(Error::EmptyArrayList((start - 1) as u32, (start + buf.len()) as u32, txt.to_string()));
    }
    let sep_count = buf.iter().filter(|&b| *b == b',').count();
    let mut span_list: Vec<Span> = Vec::with_capacity(sep_count + 1);
    todo!();
    // populate the list
    let mut values: Vec<&'a str> = Vec::with_capacity(span_list.len());
    for span in span_list {
        values.push(&txt[span.start as usize..span.end as usize]);
    }
    Ok(ParsedValue::List(values))
}
fn parse_single<'a>(buf: &[u8], start: usize, txt: &'a str, copy_buffer: &'a mut String) -> Result<ParsedValue<'a>, Error> {
    if buf.trim_ascii().is_empty() {
        return Err(Error::ExpectingAValue((start - 1) as u32, (start + buf.len()) as u32, txt.to_string()));
    }
    // code already starts at the first non-whitespace character
    let (span, mut next) = match buf[0] {
        b'\'' => parse_single_quoted_string(buf, 0, start, txt)?,
        b'"' => parse_double_quoted_string(buf, 0, start, txt, copy_buffer)?,
        _ => parse_regular_word(buf, 0, start, txt)?,
    };
    while next < buf.len()  && buf[next].is_ascii_whitespace() {
        next += 1;
    }
    if next < buf.len() {
        return Err(Error::ExpectingASingleValue((start + next) as u32, (start + buf.len()) as u32, txt.to_string()));
    }
    Ok(ParsedValue::Single(span.as_slice(txt)))
}
pub(crate) fn parse<'a>(txt: &'a str, start: usize, end: usize, copy_buffer: &'a mut String) -> Result<ParsedValue<'a>, Error> {
    let bytes = (&txt[start..end]).as_bytes();
    let first = bytes
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .ok_or_else(|| Error::ExpectingAValue(start as u32, end as u32, txt.to_string()))?;

    let last = bytes.iter().rposition(|&b| !b.is_ascii_whitespace()).unwrap(); // safe - we already found at least one non-whitespace
    match (bytes[first], bytes[last]) {
        (b'[', b']') => parse_list(&bytes[first + 1..last], start + first + 1, txt, copy_buffer),
        (_, b']') => Err(Error::MissingStartingBracket(
            (start + first) as u32,
            (start + end + 1) as u32,
            txt.to_string(),
        )),
        (b'[', _) => Err(Error::MissingEndingBracket(
            (start + first) as u32,
            (start + end + 1) as u32,
            txt.to_string(),
        )),
        _ => parse_single(&bytes[first..last], start + first, txt, copy_buffer),
    }
}
