//! A small hand-written parser for the `struct` or `enum` a derive is applied to.
//!
//! Only what the derives need is extracted: the type name, its generics (split into the
//! `impl<...>` and `Type<...>` forms), the `where` clause, the `#[whereexpr(...)]` attributes
//! and, for every field or variant, its name and `#[whereexpr(...)]` attributes (plus the type
//! tokens of a field).

use crate::error::{Error, Result};
use proc_macro::{Delimiter, Group, Ident, Spacing, Span, TokenStream, TokenTree};

pub(crate) struct Input {
    pub(crate) name: Ident,
    pub(crate) impl_generics: String,
    pub(crate) ty_generics: String,
    pub(crate) where_clause: String,
    /// The parenthesized argument groups of every `#[whereexpr(...)]` on the struct.
    pub(crate) attrs: Vec<Group>,
    pub(crate) fields: Vec<Field>,
}

pub(crate) struct Field {
    /// What follows `self.`: the field name, or its position for a tuple struct.
    pub(crate) member: String,
    /// The field name without a leading `r#`, or `None` for a tuple field.
    pub(crate) name: Option<String>,
    pub(crate) span: Span,
    pub(crate) ty: Vec<TokenTree>,
    pub(crate) attrs: Vec<Group>,
}

/// An enum whose variants all have no data and no explicit discriminant.
pub(crate) struct EnumInput {
    pub(crate) name: Ident,
    pub(crate) impl_generics: String,
    pub(crate) ty_generics: String,
    pub(crate) where_clause: String,
    /// The parenthesized argument groups of every `#[whereexpr(...)]` on the enum.
    pub(crate) attrs: Vec<Group>,
    pub(crate) variants: Vec<Variant>,
}

pub(crate) struct Variant {
    pub(crate) name: Ident,
    pub(crate) attrs: Vec<Group>,
}

// ---------------------------------------------------------------------------------------------
// token helpers
// ---------------------------------------------------------------------------------------------

pub(crate) fn is_punct(token: Option<&TokenTree>, c: char) -> bool {
    matches!(token, Some(TokenTree::Punct(p)) if p.as_char() == c)
}

pub(crate) fn is_ident(token: Option<&TokenTree>, name: &str) -> bool {
    matches!(token, Some(TokenTree::Ident(i)) if i.to_string() == name)
}

pub(crate) fn tokens_to_string(tokens: &[TokenTree]) -> String {
    TokenStream::from_iter(tokens.iter().cloned()).to_string()
}

/// Splits `tokens` on commas that are not nested inside `<...>`.
///
/// Groups (`()`, `[]`, `{}`) are single token trees, so only angle brackets need tracking.
/// The `>` of `->` (as in `fn() -> u8`) is not a closing bracket.
pub(crate) fn split_top_level(tokens: &[TokenTree], separator: char) -> Vec<Vec<TokenTree>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0usize;
    let mut previous_is_joint_minus = false;
    for token in tokens {
        let mut joint_minus = false;
        if let TokenTree::Punct(p) = token {
            match p.as_char() {
                '<' => depth += 1,
                '>' if !previous_is_joint_minus => depth = depth.saturating_sub(1),
                '-' => joint_minus = p.spacing() == Spacing::Joint,
                c if c == separator && depth == 0 => {
                    result.push(std::mem::take(&mut current));
                    previous_is_joint_minus = false;
                    continue;
                }
                _ => {}
            }
        }
        previous_is_joint_minus = joint_minus;
        current.push(token.clone());
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

/// A cursor over a flat list of token trees.
pub(crate) struct Cursor {
    tokens: Vec<TokenTree>,
    pos: usize,
    end_span: Span,
}

impl Cursor {
    pub(crate) fn new(tokens: Vec<TokenTree>, end_span: Span) -> Self {
        Self { tokens, pos: 0, end_span }
    }
    pub(crate) fn peek(&self) -> Option<&TokenTree> {
        self.tokens.get(self.pos)
    }
    /// The token after the next one.
    pub(crate) fn peek_next(&self) -> Option<&TokenTree> {
        self.tokens.get(self.pos + 1)
    }
    pub(crate) fn next(&mut self) -> Option<TokenTree> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.pos >= self.tokens.len()
    }
    /// The span of the next token, or of the end of the input.
    pub(crate) fn span(&self) -> Span {
        self.peek().map(|t| t.span()).unwrap_or(self.end_span)
    }
    pub(crate) fn rest(&mut self) -> Vec<TokenTree> {
        let rest = self.tokens[self.pos..].to_vec();
        self.pos = self.tokens.len();
        rest
    }
    fn expect_ident(&mut self, what: &str) -> Result<Ident> {
        match self.next() {
            Some(TokenTree::Ident(i)) => Ok(i),
            Some(other) => Err(Error::new(other.span(), format!("expected {what}"))),
            None => Err(Error::new(self.end_span, format!("expected {what}"))),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// struct parsing
// ---------------------------------------------------------------------------------------------

/// Skips outer attributes, returning the argument group of every `#[whereexpr(...)]`.
fn parse_attributes(cursor: &mut Cursor) -> Result<Vec<Group>> {
    let mut groups = Vec::new();
    while is_punct(cursor.peek(), '#') {
        cursor.next();
        let Some(TokenTree::Group(bracket)) = cursor.next() else {
            return Err(Error::new(cursor.span(), "expected `[` after `#`"));
        };
        let mut inner = Cursor::new(bracket.stream().into_iter().collect(), bracket.span_close());
        if !is_ident(inner.peek(), "whereexpr") {
            continue;
        }
        let name = inner.next().unwrap();
        match inner.next() {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis && inner.is_empty() => groups.push(g),
            _ => {
                return Err(Error::new(
                    name.span(),
                    "expected `#[whereexpr(...)]`, for example `#[whereexpr(name = \"other_name\")]`",
                ))
            }
        }
    }
    Ok(groups)
}

/// Skips `pub`, `pub(crate)`, `pub(in path)` and so on.
fn skip_visibility(cursor: &mut Cursor) {
    if is_ident(cursor.peek(), "pub") {
        cursor.next();
        if matches!(cursor.peek(), Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis) {
            cursor.next();
        }
    }
}

/// Parses `<...>` after the struct name into (`<params with bounds>`, `<param names>`).
fn parse_generics(cursor: &mut Cursor) -> Result<(String, String)> {
    if !is_punct(cursor.peek(), '<') {
        return Ok((String::new(), String::new()));
    }
    let open = cursor.next().unwrap();
    let mut depth = 1usize;
    let mut content = Vec::new();
    let mut previous_is_joint_minus = false;
    loop {
        let Some(token) = cursor.next() else {
            return Err(Error::new(open.span(), "unclosed `<` in the generic parameters"));
        };
        let mut joint_minus = false;
        if let TokenTree::Punct(p) = &token {
            match p.as_char() {
                '<' => depth += 1,
                '>' if !previous_is_joint_minus => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                '-' => joint_minus = p.spacing() == Spacing::Joint,
                _ => {}
            }
        }
        previous_is_joint_minus = joint_minus;
        content.push(token);
    }

    let mut impl_params = Vec::new();
    let mut ty_params = Vec::new();
    for param in split_top_level(&content, ',') {
        let mut p = Cursor::new(param, open.span());
        parse_attributes(&mut p)?; // attributes on generic parameters are dropped
        let param: Vec<TokenTree> = p.rest();
        // drop a default value (`T = u32`, `const N: usize = 4`)
        let without_default = match split_top_level(&param, '=').into_iter().next() {
            Some(before) if before.len() < param.len() => before,
            _ => param.clone(),
        };
        let name = match param.first() {
            Some(TokenTree::Punct(p)) if p.as_char() == '\'' => format!("'{}", param.get(1).map(|t| t.to_string()).unwrap_or_default()),
            Some(TokenTree::Ident(i)) if i.to_string() == "const" => param.get(1).map(|t| t.to_string()).unwrap_or_default(),
            Some(TokenTree::Ident(i)) => i.to_string(),
            _ => continue,
        };
        impl_params.push(tokens_to_string(&without_default));
        ty_params.push(name);
    }
    if ty_params.is_empty() {
        return Ok((String::new(), String::new()));
    }
    Ok((format!("<{}>", impl_params.join(", ")), format!("<{}>", ty_params.join(", "))))
}

/// Collects a `where` clause up to (not including) the struct body or the final `;`.
fn parse_where_clause(cursor: &mut Cursor) -> String {
    if !is_ident(cursor.peek(), "where") {
        return String::new();
    }
    let mut tokens = Vec::new();
    while let Some(token) = cursor.peek() {
        let stop = match token {
            TokenTree::Group(g) => g.delimiter() == Delimiter::Brace,
            TokenTree::Punct(p) => p.as_char() == ';',
            _ => false,
        };
        if stop {
            break;
        }
        tokens.push(cursor.next().unwrap());
    }
    tokens_to_string(&tokens)
}

fn parse_fields(group: &Group, named: bool) -> Result<Vec<Field>> {
    let tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    let mut fields = Vec::new();
    for (position, chunk) in split_top_level(&tokens, ',').into_iter().enumerate() {
        let mut cursor = Cursor::new(chunk, group.span_close());
        let attrs = parse_attributes(&mut cursor)?;
        skip_visibility(&mut cursor);
        let span = cursor.span();
        let (member, name) = if named {
            let ident = cursor.expect_ident("a field name")?;
            if !is_punct(cursor.peek(), ':') {
                return Err(Error::new(cursor.span(), "expected `:` after the field name"));
            }
            cursor.next();
            let member = ident.to_string();
            let name = member.strip_prefix("r#").unwrap_or(&member).to_string();
            (member, Some(name))
        } else {
            (position.to_string(), None)
        };
        let ty = cursor.rest();
        if ty.is_empty() {
            return Err(Error::new(span, "expected a field type"));
        }
        fields.push(Field {
            member,
            name,
            span,
            ty,
            attrs,
        });
    }
    Ok(fields)
}

pub(crate) fn parse_input(input: TokenStream) -> Result<Input> {
    let mut cursor = Cursor::new(input.into_iter().collect(), Span::call_site());
    let attrs = parse_attributes(&mut cursor)?;
    skip_visibility(&mut cursor);

    let keyword = cursor.expect_ident("`struct`")?;
    match keyword.to_string().as_str() {
        "struct" => {}
        "enum" | "union" => {
            return Err(Error::new(
                keyword.span(),
                format!(
                    "`#[derive(Attributes)]` only supports structs, not {}s; implement `Attributes` by hand",
                    keyword
                ),
            ))
        }
        _ => return Err(Error::new(keyword.span(), "expected `struct`")),
    }

    let name = cursor.expect_ident("the struct name")?;
    let (impl_generics, ty_generics) = parse_generics(&mut cursor)?;
    let mut where_clause = parse_where_clause(&mut cursor);

    let fields = match cursor.next() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => parse_fields(&g, true)?,
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
            let fields = parse_fields(&g, false)?;
            where_clause = parse_where_clause(&mut cursor);
            fields
        }
        Some(TokenTree::Punct(p)) if p.as_char() == ';' => Vec::new(),
        Some(other) => return Err(Error::new(other.span(), "expected the struct body")),
        None => return Err(Error::new(name.span(), "expected the struct body")),
    };

    Ok(Input {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        attrs,
        fields,
    })
}

// ---------------------------------------------------------------------------------------------
// enum parsing
// ---------------------------------------------------------------------------------------------

fn parse_variants(group: &Group) -> Result<Vec<Variant>> {
    let tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    let mut variants = Vec::new();
    for chunk in split_top_level(&tokens, ',') {
        let mut cursor = Cursor::new(chunk, group.span_close());
        let attrs = parse_attributes(&mut cursor)?;
        let name = cursor.expect_ident("a variant name")?;
        match cursor.next() {
            None => {}
            Some(TokenTree::Group(g)) if g.delimiter() != Delimiter::Bracket => {
                return Err(Error::new(
                    g.span(),
                    format!("variant `{name}` holds data; `#[derive(WhereexprEnum)]` only supports variants without data"),
                ))
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
                return Err(Error::new(
                    p.span(),
                    format!(
                        "variant `{name}` has an explicit discriminant; `#[derive(WhereexprEnum)]` numbers the variants itself (0, 1, 2, ... in declaration order), so remove `= ...`"
                    ),
                ))
            }
            Some(other) => return Err(Error::new(other.span(), format!("unexpected tokens after variant `{name}`"))),
        }
        variants.push(Variant { name, attrs });
    }
    Ok(variants)
}

pub(crate) fn parse_enum_input(input: TokenStream) -> Result<EnumInput> {
    let mut cursor = Cursor::new(input.into_iter().collect(), Span::call_site());
    let attrs = parse_attributes(&mut cursor)?;
    skip_visibility(&mut cursor);

    let keyword = cursor.expect_ident("`enum`")?;
    match keyword.to_string().as_str() {
        "enum" => {}
        "struct" | "union" => {
            return Err(Error::new(
                keyword.span(),
                format!("`#[derive(WhereexprEnum)]` only supports enums, not {keyword}s; for a struct use `#[derive(Attributes)]`"),
            ))
        }
        _ => return Err(Error::new(keyword.span(), "expected `enum`")),
    }

    let name = cursor.expect_ident("the enum name")?;
    let (impl_generics, ty_generics) = parse_generics(&mut cursor)?;
    let where_clause = parse_where_clause(&mut cursor);
    let variants = match cursor.next() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => parse_variants(&g)?,
        Some(other) => return Err(Error::new(other.span(), "expected the enum body")),
        None => return Err(Error::new(name.span(), "expected the enum body")),
    };

    Ok(EnumInput {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        attrs,
        variants,
    })
}
