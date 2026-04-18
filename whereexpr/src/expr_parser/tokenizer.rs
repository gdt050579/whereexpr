use super::{Token, TokenKind};
use crate::Error;

pub(crate) fn tokenize(input: &str) -> Result<Vec<Token>, Error> {
    if input.trim().is_empty() {
        return Err(Error::EmptyExpression);
    }
    if input.len() > 0x7FFF {
        return Err(Error::ExpressioTooLong);
    }

    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut tokens = Vec::with_capacity((len / 4) + 1);
    let mut i = 0;

    while i < len {
        match bytes[i] {
            b' ' | b'\t' | b'\n' | b'\r' => {
                i += 1;
            }
            b'(' => {
                tokens.push(Token::new(TokenKind::LParen, i, i + 1));
                i += 1;
            }
            b')' => {
                tokens.push(Token::new(TokenKind::RParen, i, i + 1));
                i += 1;
            }
            b'!' | b'~' => {
                tokens.push(Token::new(TokenKind::Not, i, i + 1));
                i += 1;
            }
            b'&' => {
                let start = i;
                i += 1;
                if i < len && bytes[i] == b'&' {
                    i += 1;
                }
                tokens.push(Token::new(TokenKind::And, start, i)); // & or &&
            }
            b'|' => {
                let start = i;
                i += 1;
                if i < len && bytes[i] == b'|' {
                    i += 1;
                }
                tokens.push(Token::new(TokenKind::Or, start, i)); // | or ||
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                let start = i;
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-') {
                    i += 1;
                }
                let word_len = i - start;
                let kind = match word_len {
                    2 if (bytes[start] | 0x20) == b'o' && (bytes[start + 1] | 0x20) == b'r' => TokenKind::Or,
                    3 if (bytes[start] | 0x20) == b'a' && (bytes[start + 1] | 0x20) == b'n' && (bytes[start + 2] | 0x20) == b'd' => TokenKind::And,
                    3 if (bytes[start] | 0x20) == b'n' && (bytes[start + 1] | 0x20) == b'o' && (bytes[start + 2] | 0x20) == b't' => TokenKind::Not,
                    _ => TokenKind::ConditionIndex(u16::MAX),
                };
                tokens.push(Token::new(kind, start, i));
            }
            _ => {
                return Err(Error::UnexpectedChar(i as u32, (i + 1) as u32, input.to_string()));
            }
        }
    }

    Ok(tokens)
}
