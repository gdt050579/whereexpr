use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

/// A compile error with the span it should point at.
pub(crate) struct Error {
    span: Span,
    message: String,
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }

    /// Builds `::core::compile_error!("...");` with every token carrying the error span,
    /// so the compiler underlines the offending part of the user's code.
    pub(crate) fn into_compile_error(self) -> TokenStream {
        let span = self.span;
        let punct = |c: char, spacing: Spacing| {
            let mut p = Punct::new(c, spacing);
            p.set_span(span);
            TokenTree::Punct(p)
        };
        let mut message = Literal::string(&self.message);
        message.set_span(span);
        let mut args = Group::new(Delimiter::Parenthesis, TokenStream::from(TokenTree::Literal(message)));
        args.set_span(span);

        TokenStream::from_iter([
            punct(':', Spacing::Joint),
            punct(':', Spacing::Alone),
            TokenTree::Ident(Ident::new("core", span)),
            punct(':', Spacing::Joint),
            punct(':', Spacing::Alone),
            TokenTree::Ident(Ident::new("compile_error", span)),
            punct('!', Spacing::Alone),
            TokenTree::Group(args),
            punct(';', Spacing::Alone),
        ])
    }
}
