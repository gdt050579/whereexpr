//! Turns the parsed struct into an `impl whereexpr::Attributes for ...` block.

use crate::error::{Error, Result};
use crate::kind::{analyze, Kind, KIND_NAMES};
use crate::meta::{parse_group, Meta};
use crate::parser::{parse_input, Field, Input};
use proc_macro::{Delimiter, Group, Span, TokenStream, TokenTree};

/// One attribute exposed to whereexpr conditions.
struct Attribute {
    name: String,
    name_span: Span,
    kind: Kind,
    /// Code of type `Option<Value<'_>>`, using `self`.
    getter: String,
    /// Span used for the generated getter, so type errors point at the field.
    span: Span,
}

struct Options {
    type_name: Option<String>,
    type_id: Option<u64>,
}

/// Same rule as the condition parser: an ASCII letter, then letters, digits, `_` or `.`.
fn validate_attribute_name(name: &str, span: Span) -> Result<()> {
    let mut chars = name.chars();
    let valid = matches!(chars.next(), Some(c) if c.is_ascii_alphabetic()) && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
    if valid {
        Ok(())
    } else {
        Err(Error::new(
            span,
            format!(
                "`{name}` cannot be used in a whereexpr condition: an attribute name must start with an ASCII letter and contain only letters, digits, `_` and `.`; use `#[whereexpr(name = \"...\")]` or `#[whereexpr(ignore)]`"
            ),
        ))
    }
}

fn parse_kind(meta: &Meta) -> Result<Kind> {
    let name = meta.as_str_or_path()?;
    Kind::from_name(&name).ok_or_else(|| Error::new(meta.value_span, format!("unknown kind `{name}`; expected one of: {KIND_NAMES}")))
}

fn unknown_option(meta: &Meta, allowed: &str) -> Error {
    Error::new(meta.span, format!("unknown option `{}`; expected one of: {allowed}", meta.key))
}

fn parse_computed(items: &[Meta], span: Span, value: &str) -> Result<Attribute> {
    let mut name: Option<(String, Span)> = None;
    let mut kind = None;
    let mut with = None;
    for item in items {
        match item.key.as_str() {
            "name" => name = Some((item.as_str()?, item.value_span)),
            "kind" => kind = Some(parse_kind(item)?),
            "with" => with = Some(item.as_path()?),
            _ => return Err(unknown_option(item, "`name`, `kind`, `with`")),
        }
    }
    let Some((name, name_span)) = name else {
        return Err(Error::new(span, "`computed(...)` needs a `name = \"...\"`"));
    };
    let Some(kind) = kind else {
        return Err(Error::new(span, format!("`computed(...)` needs a `kind = ...` (one of: {KIND_NAMES})")));
    };
    validate_attribute_name(&name, name_span)?;
    let with = match with {
        Some(with) => with,
        None if !name.contains('.') => format!("Self::{name}"),
        None => {
            return Err(Error::new(
                span,
                format!("`computed(...)` needs `with = path::to::function` for `{name}`"),
            ))
        }
    };
    // `with` must be a `fn(&Self) -> Option<T>` where `T` is the exact type of the kind's `Value` variant.
    Ok(Attribute {
        getter: format!("({with})(self).map({value}::{})", kind.variant()),
        name,
        name_span,
        kind,
        span,
    })
}

fn parse_struct_options(input: &Input, value_path: &str) -> Result<(Options, Vec<Attribute>)> {
    let mut options = Options {
        type_name: None,
        type_id: None,
    };
    let mut computed = Vec::new();
    let mut computed_metas = Vec::new();
    for group in &input.attrs {
        for meta in parse_group(group)? {
            match meta.key.as_str() {
                "name" => options.type_name = Some(meta.as_str()?),
                "id" => options.type_id = Some(meta.as_int()?),
                "computed" => computed_metas.push(meta),
                _ => return Err(unknown_option(&meta, "`name`, `id`, `computed(...)`")),
            }
        }
    }
    for meta in &computed_metas {
        computed.push(parse_computed(meta.as_list()?, meta.span, value_path)?);
    }
    Ok((options, computed))
}

fn parse_field(field: &Field, value: &str) -> Result<Option<Attribute>> {
    let mut name_override: Option<(String, Span)> = None;
    let mut kind: Option<Kind> = None;
    let mut ignore = false;
    for group in &field.attrs {
        for meta in parse_group(group)? {
            match meta.key.as_str() {
                "name" => name_override = Some((meta.as_str()?, meta.value_span)),
                "kind" => kind = Some(parse_kind(&meta)?),
                "ignore" => {
                    meta.expect_flag()?;
                    ignore = true;
                }
                _ => return Err(unknown_option(&meta, "`name`, `kind`, `ignore`")),
            }
        }
    }
    if ignore {
        return Ok(None);
    }
    let (name, name_span) = match (name_override, &field.name) {
        (Some(name), _) => name,
        (None, Some(name)) => (name.clone(), field.span),
        (None, None) => {
            return Err(Error::new(
                field.span,
                format!(
                    "tuple field {} needs a name: add `#[whereexpr(name = \"...\")]` or `#[whereexpr(ignore)]`",
                    field.member
                ),
            ))
        }
    };
    validate_attribute_name(&name, name_span)?;

    let ty_span = field.ty.first().map(|t| t.span()).unwrap_or(field.span);
    let ty = analyze(&field.ty);
    let kind = match kind {
        Some(kind) => kind,
        None => ty.default_kind(ty_span)?,
    };
    let conversion = ty.conversion(kind, value, ty_span)?;
    let getter = if ty.optional {
        format!(
            "match &self.{} {{ Some(v) => {conversion}, None => None }}",
            field.member
        )
    } else {
        format!("{{ let v = &self.{}; {conversion} }}", field.member)
    };
    Ok(Some(Attribute {
        name,
        name_span,
        kind,
        getter,
        span: ty_span,
    }))
}

fn parse(code: &str) -> TokenStream {
    code.parse().expect("whereexpr_proc_macro generated invalid code")
}

/// Gives every token (recursively) the span `span`.
fn respan(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|mut token| {
            if let TokenTree::Group(g) = &token {
                let mut group = Group::new(g.delimiter(), respan(g.stream(), span));
                group.set_span(span);
                token = TokenTree::Group(group);
            } else {
                token.set_span(span);
            }
            token
        })
        .collect()
}

fn braces(stream: TokenStream) -> TokenTree {
    TokenTree::Group(Group::new(Delimiter::Brace, stream))
}

fn rust_string(s: &str) -> String {
    format!("{s:?}")
}

/// FNV-1a 64-bit, the same hash `whereexpr` uses for operation and string-list keys.
fn fnv64(s: &str) -> u64 {
    let mut hash = 14695981039346656037u64;
    for &byte in s.as_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211u64);
    }
    hash
}

pub(crate) fn derive(input: TokenStream) -> Result<TokenStream> {
    let input = parse_input(input)?;
    let crate_name = "::whereexpr";
    let value = format!("{crate_name}::Value");
    let (options, computed) = parse_struct_options(&input, &value)?;

    let mut attributes = Vec::new();
    for field in &input.fields {
        if let Some(attribute) = parse_field(field, &value)? {
            attributes.push(attribute);
        }
    }
    attributes.extend(computed);

    for (i, attribute) in attributes.iter().enumerate() {
        if attributes[..i].iter().any(|a| a.name == attribute.name) {
            return Err(Error::new(
                attribute.name_span,
                format!("duplicate whereexpr attribute name `{}`", attribute.name),
            ));
        }
    }
    if attributes.len() > u16::MAX as usize {
        return Err(Error::new(input.name.span(), "too many attributes (the maximum is 65535)"));
    }

    let name = input.name.to_string();
    let type_name = options.type_name.clone().unwrap_or_else(|| name.clone());
    let type_id = match options.type_id {
        Some(id) => format!("{id}u64"),
        None => format!("{}u64", fnv64(&type_name)),
    };

    let mut index_arms = String::new();
    let mut kind_arms = String::new();
    let mut get_arms = TokenStream::new();
    for (i, attribute) in attributes.iter().enumerate() {
        index_arms.push_str(&format!(
            "{} => Some({crate_name}::AttributeIndex::new({i}u16)),",
            rust_string(&attribute.name)
        ));
        kind_arms.push_str(&format!(
            "{i}u16 => Some({crate_name}::ValueKind::{}),",
            attribute.kind.variant()
        ));
        get_arms.extend(parse(&format!("{i}u16 =>")));
        get_arms.extend(respan(parse(&format!("{{ {} }}", attribute.getter)), attribute.span));
    }
    get_arms.extend(parse("_ => None,"));

    let mut body = parse(&format!(
        "const TYPE_ID: u64 = {type_id};
         const TYPE_NAME: &'static str = {type_name};
         #[inline]
         fn kind(idx: {crate_name}::AttributeIndex) -> Option<{crate_name}::ValueKind> {{
             match idx.index() {{ {kind_arms} _ => None, }}
         }}
         #[inline]
         fn index(name: &str) -> Option<{crate_name}::AttributeIndex> {{
             match name {{ {index_arms} _ => None, }}
         }}
         #[inline]
         // the conversions of types the derive cannot see through (aliases, user types) may be no-ops
         #[allow(clippy::useless_conversion, clippy::unnecessary_fallible_conversions)]
         fn get(&self, idx: {crate_name}::AttributeIndex) -> Option<{crate_name}::Value<'_>>",
        type_name = rust_string(&type_name),
    ));
    let mut get_body = parse("match idx.index()");
    get_body.extend([braces(get_arms)]);
    body.extend([braces(get_body)]);

    let mut output = parse(&format!(
        "#[automatically_derived] impl {} {crate_name}::Attributes for {name} {} {}",
        input.impl_generics, input.ty_generics, input.where_clause
    ));
    output.extend([braces(body)]);
    Ok(output)
}
