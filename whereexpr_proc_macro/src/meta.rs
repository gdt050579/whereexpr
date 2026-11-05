use crate::error::{Error, Result};
use crate::parser::{split_top_level, tokens_to_string, Cursor};
use proc_macro::{Delimiter, Group, Span, TokenTree};

pub(crate) enum MetaValue {
    Flag,
    Str(String),
    Int(u64),
    Path(String),
    List(Vec<Meta>),
}

pub(crate) struct Meta {
    pub(crate) key: String,
    pub(crate) span: Span,
    pub(crate) value_span: Span,
    pub(crate) value: MetaValue,
}

impl Meta {
    fn describe(&self) -> &'static str {
        match self.value {
            MetaValue::Flag => "a flag",
            MetaValue::Str(_) => "a string",
            MetaValue::Int(_) => "an integer",
            MetaValue::Path(_) => "a path",
            MetaValue::List(_) => "a list",
        }
    }

    pub(crate) fn as_str_or_path(&self) -> Result<String> {
        match &self.value {
            MetaValue::Str(s) | MetaValue::Path(s) => Ok(s.clone()),
            _ => Err(self.wrong_value("a string")),
        }
    }

    pub(crate) fn as_str(&self) -> Result<String> {
        match &self.value {
            MetaValue::Str(s) => Ok(s.clone()),
            _ => Err(self.wrong_value("a string literal")),
        }
    }

    pub(crate) fn as_int(&self) -> Result<u64> {
        match &self.value {
            MetaValue::Int(v) => Ok(*v),
            _ => Err(self.wrong_value("an integer literal")),
        }
    }

    pub(crate) fn as_path(&self) -> Result<String> {
        match &self.value {
            MetaValue::Path(s) => Ok(s.clone()),
            MetaValue::Str(s) => Ok(s.clone()),
            _ => Err(self.wrong_value("a path")),
        }
    }

    pub(crate) fn as_list(&self) -> Result<&[Meta]> {
        match &self.value {
            MetaValue::List(items) => Ok(items),
            _ => Err(Error::new(self.span, format!("expected `{}(...)`", self.key))),
        }
    }

    pub(crate) fn expect_flag(&self) -> Result<()> {
        match self.value {
            MetaValue::Flag => Ok(()),
            _ => Err(Error::new(self.value_span, format!("`{}` takes no value", self.key))),
        }
    }

    fn wrong_value(&self, expected: &str) -> Error {
        let span = match self.value {
            MetaValue::Flag => self.span,
            _ => self.value_span,
        };
        Error::new(span, format!("`{}` expects {expected}, found {}", self.key, self.describe()))
    }
}

pub(crate) fn parse_group(group: &Group) -> Result<Vec<Meta>> {
    let tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    let mut items = Vec::new();
    for chunk in split_top_level(&tokens, ',') {
        let mut cursor = Cursor::new(chunk, group.span_close());
        let key = match cursor.next() {
            Some(TokenTree::Ident(i)) => i,
            Some(other) => return Err(Error::new(other.span(), "expected an option name")),
            None => continue,
        };
        let key_name = key.to_string();
        let key_name = key_name.strip_prefix("r#").unwrap_or(&key_name).to_string();
        let (value, value_span) = match cursor.next() {
            None => (MetaValue::Flag, key.span()),
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                if !cursor.is_empty() {
                    return Err(Error::new(cursor.span(), "unexpected tokens"));
                }
                (MetaValue::List(parse_group(&g)?), g.span())
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
                let value_span = cursor.span();
                let rest = cursor.rest();
                (parse_value(&key_name, &rest, value_span)?, value_span)
            }
            Some(other) => return Err(Error::new(other.span(), format!("expected `=` or `,` after `{key_name}`"))),
        };
        items.push(Meta {
            key: key_name,
            span: key.span(),
            value_span,
            value,
        });
    }
    Ok(items)
}

fn parse_value(key: &str, tokens: &[TokenTree], span: Span) -> Result<MetaValue> {
    if tokens.is_empty() {
        return Err(Error::new(span, format!("expected a value after `{key} =`")));
    }
    if let [TokenTree::Literal(literal)] = tokens {
        let text = literal.to_string();
        if let Some(s) = parse_string_literal(&text) {
            return Ok(MetaValue::Str(s));
        }
        if let Some(v) = parse_int_literal(&text) {
            return Ok(MetaValue::Int(v));
        }
        return Err(Error::new(literal.span(), format!("unsupported literal for `{key}`")));
    }
    // A path: identifiers separated by `::`, possibly with a leading `::`.
    for token in tokens {
        match token {
            TokenTree::Ident(_) => {}
            TokenTree::Punct(p) if p.as_char() == ':' => {}
            other => return Err(Error::new(other.span(), format!("expected a literal or a path as the value of `{key}`"))),
        }
    }
    Ok(MetaValue::Path(tokens_to_string(tokens).replace(' ', "")))
}
fn parse_string_literal(text: &str) -> Option<String> {
    if let Some(raw) = text.strip_prefix('r') {
        let hashes = raw.len() - raw.trim_start_matches('#').len();
        let body = &raw[hashes..];
        let body = body.strip_prefix('"')?;
        let body = body.strip_suffix(&"#".repeat(hashes))?;
        return body.strip_suffix('"').map(str::to_string);
    }
    let body = text.strip_prefix('"')?.strip_suffix('"')?;
    let mut result = String::with_capacity(body.len());
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        match chars.next()? {
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            '0' => result.push('\0'),
            '\\' => result.push('\\'),
            '"' => result.push('"'),
            '\'' => result.push('\''),
            'x' => {
                let hex: String = chars.by_ref().take(2).collect();
                result.push(u8::from_str_radix(&hex, 16).ok()? as char);
            }
            'u' => {
                let rest: String = chars.by_ref().take_while(|&c| c != '}').collect();
                let code = u32::from_str_radix(rest.strip_prefix('{')?, 16).ok()?;
                result.push(char::from_u32(code)?);
            }
            '\n' => {
                let rest: String = chars.collect();
                result.push_str(&parse_string_literal(&format!("\"{}\"", rest.trim_start()))?);
                return Some(result);
            }
            _ => return None,
        }
    }
    Some(result)
}

fn parse_int_literal(text: &str) -> Option<u64> {
    let text = text.replace('_', "");
    let (digits, radix) = if let Some(hex) = text.strip_prefix("0x") {
        (hex.to_string(), 16)
    } else if let Some(bin) = text.strip_prefix("0b") {
        (bin.to_string(), 2)
    } else if let Some(oct) = text.strip_prefix("0o") {
        (oct.to_string(), 8)
    } else {
        (text, 10)
    };
    let end = digits.find(|c: char| !c.is_digit(radix)).unwrap_or(digits.len());
    let suffix = &digits[end..];
    if !suffix.is_empty()
        && !matches!(
            suffix,
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
        )
    {
        return None;
    }
    u64::from_str_radix(&digits[..end], radix).ok()
}
