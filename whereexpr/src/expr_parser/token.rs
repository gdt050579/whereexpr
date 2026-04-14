#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum TokenKind {
    And,
    Or,
    Not,
    LParen,
    RParen,
    ConditionIndex(u16),
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub(crate) struct TokenSpan {
    start: u16,
    end: u16,
}

impl TokenSpan {
    #[cfg(test)]
    pub(super) fn new(start: usize, end: usize) -> Self {
        Self {
            start: start as u16,
            end: end as u16,
        }
    }
    #[inline(always)]
    pub(crate) fn as_slice<'a>(&self, input: &'a str) -> &'a str {
        &input[self.start as usize..self.end as usize]
    }
    #[inline(always)]
    pub(crate) fn start(&self) -> usize {
        self.start as usize
    }
    #[inline(always)]
    pub(crate) fn end(&self) -> usize {
        self.end as usize
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Token {
    kind: TokenKind,
    span: TokenSpan,
}

impl Token {
    pub(crate) fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Self {
            kind,
            span: TokenSpan {
                start: start as u16,
                end: end as u16,
            },
        }
    }
    pub(crate) fn kind(&self) -> TokenKind {
        self.kind
    }
    pub(crate) fn span(&self) -> TokenSpan {
        self.span
    }
}
