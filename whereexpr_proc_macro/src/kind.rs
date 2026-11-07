//! Mapping from Rust field types to `whereexpr::ValueKind`, and the code that turns a
//! reference to a field into a `whereexpr::Value`.

use crate::error::{Error, Result};
use crate::parser::{is_ident, is_punct, split_top_level, tokens_to_string};
use proc_macro::{Delimiter, Span, TokenTree};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    String,
    StringList,
    Path,
    Bytes,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Hash128,
    Hash160,
    Hash256,
    IpAddr,
    DateTime,
    Bool,
}

pub(crate) const KIND_NAMES: &str =
    "string, string_list, path, bytes, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, hash128, hash160, hash256, ip, datetime, bool";

impl Kind {
    pub(crate) fn from_name(name: &str) -> Option<Kind> {
        Some(match name.to_ascii_lowercase().as_str() {
            "string" | "str" => Kind::String,
            "string_list" | "string-list" | "stringlist" => Kind::StringList,
            "path" => Kind::Path,
            "bytes" => Kind::Bytes,
            "u8" => Kind::U8,
            "u16" => Kind::U16,
            "u32" => Kind::U32,
            "u64" => Kind::U64,
            "i8" => Kind::I8,
            "i16" => Kind::I16,
            "i32" => Kind::I32,
            "i64" => Kind::I64,
            "f32" => Kind::F32,
            "f64" => Kind::F64,
            "hash128" => Kind::Hash128,
            "hash160" => Kind::Hash160,
            "hash256" => Kind::Hash256,
            "ip" | "ipaddr" | "ip_addr" => Kind::IpAddr,
            "datetime" | "date_time" => Kind::DateTime,
            "bool" => Kind::Bool,
            _ => return None,
        })
    }

    /// The variant name, shared by `Value` and `ValueKind`.
    pub(crate) fn variant(self) -> &'static str {
        match self {
            Kind::String => "String",
            Kind::StringList => "StringList",
            Kind::Path => "Path",
            Kind::Bytes => "Bytes",
            Kind::U8 => "U8",
            Kind::U16 => "U16",
            Kind::U32 => "U32",
            Kind::U64 => "U64",
            Kind::I8 => "I8",
            Kind::I16 => "I16",
            Kind::I32 => "I32",
            Kind::I64 => "I64",
            Kind::F32 => "F32",
            Kind::F64 => "F64",
            Kind::Hash128 => "Hash128",
            Kind::Hash160 => "Hash160",
            Kind::Hash256 => "Hash256",
            Kind::IpAddr => "IpAddr",
            Kind::DateTime => "DateTime",
            Kind::Bool => "Bool",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Kind::String => "string",
            Kind::StringList => "string_list",
            Kind::Path => "path",
            Kind::Bytes => "bytes",
            Kind::U8 => "u8",
            Kind::U16 => "u16",
            Kind::U32 => "u32",
            Kind::U64 => "u64",
            Kind::I8 => "i8",
            Kind::I16 => "i16",
            Kind::I32 => "i32",
            Kind::I64 => "i64",
            Kind::F32 => "f32",
            Kind::F64 => "f64",
            Kind::Hash128 => "hash128",
            Kind::Hash160 => "hash160",
            Kind::Hash256 => "hash256",
            Kind::IpAddr => "ip",
            Kind::DateTime => "datetime",
            Kind::Bool => "bool",
        }
    }

    fn is_integer(self) -> bool {
        matches!(
            self,
            Kind::U8 | Kind::U16 | Kind::U32 | Kind::U64 | Kind::I8 | Kind::I16 | Kind::I32 | Kind::I64
        )
    }

    fn is_number(self) -> bool {
        self.is_integer() || matches!(self, Kind::F32 | Kind::F64)
    }
}

fn number_kind(name: &str) -> Option<Kind> {
    Some(match name {
        "u8" => Kind::U8,
        "u16" => Kind::U16,
        "u32" => Kind::U32,
        "u64" => Kind::U64,
        "i8" => Kind::I8,
        "i16" => Kind::I16,
        "i32" => Kind::I32,
        "i64" => Kind::I64,
        "f32" => Kind::F32,
        "f64" => Kind::F64,
        _ => return None,
    })
}

/// Whether every value of `from` fits in `to` (mirrors the std `From` impls).
fn is_lossless(from: Kind, to: Kind) -> bool {
    use Kind::*;
    if from == to {
        return true;
    }
    matches!(
        (from, to),
        (U8, U16 | U32 | U64 | I16 | I32 | I64 | F32 | F64)
            | (U16, U32 | U64 | I32 | I64 | F32 | F64)
            | (U32, U64 | I64 | F64)
            | (I8, I16 | I32 | I64 | F32 | F64)
            | (I16, I32 | I64 | F32 | F64)
            | (I32, I64 | F64)
            | (F32, F64)
    )
}

/// What the derive recognised a field type as.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Base {
    /// `String`, `str`, `Box<str>`, `Rc<str>`, `Arc<str>`, `Cow<str>`
    Str,
    /// `PathBuf`, `Path`, `Box<Path>`, `Cow<Path>`, ...
    Path,
    Number(Kind),
    Usize,
    Isize,
    Bool,
    IpAddr,
    /// `Ipv4Addr` or `Ipv6Addr`
    IpV4V6,
    SystemTime,
    /// `[u8; N]`
    ByteArray(usize),
    /// `Vec<u8>`, `[u8]`
    Bytes,
    /// `Vec<&str>`, `[&str]`
    StrList,
    Unknown,
}

pub(crate) struct FieldType {
    pub(crate) optional: bool,
    /// Number of `&` in front of the (inner) type.
    refs: usize,
    base: Base,
    /// The (inner) type as written, for error messages.
    text: String,
}

/// Strips leading references (`&`, `&'a`, `&mut`, `&&`) and returns how many there were.
fn strip_refs(mut tokens: &[TokenTree]) -> (usize, &[TokenTree]) {
    let mut refs = 0;
    loop {
        if is_punct(tokens.first(), '&') {
            let double =
                matches!(tokens.first(), Some(TokenTree::Punct(p)) if p.spacing() == proc_macro::Spacing::Joint) && is_punct(tokens.get(1), '&');
            if double {
                refs += 2;
                tokens = &tokens[2..];
            } else {
                refs += 1;
                tokens = &tokens[1..];
            }
            if is_punct(tokens.first(), '\'') {
                tokens = &tokens[2.min(tokens.len())..];
            }
            if is_ident(tokens.first(), "mut") {
                tokens = &tokens[1..];
            }
        } else {
            return (refs, tokens);
        }
    }
}

/// For a path type such as `std::option::Option<T>` returns (`Option`, Some(generic args)).
fn last_segment(tokens: &[TokenTree]) -> Option<(String, Option<Vec<TokenTree>>)> {
    let lt = tokens.iter().position(|t| is_punct(Some(t), '<'));
    let path = &tokens[..lt.unwrap_or(tokens.len())];
    // a path is idents separated by `::`
    for token in path {
        match token {
            TokenTree::Ident(_) => {}
            TokenTree::Punct(p) if p.as_char() == ':' => {}
            _ => return None,
        }
    }
    let TokenTree::Ident(last) = path.last()? else { return None };
    let args = match lt {
        Some(start) => {
            if !is_punct(tokens.last(), '>') {
                return None;
            }
            Some(tokens[start + 1..tokens.len() - 1].to_vec())
        }
        None => None,
    };
    Some((last.to_string(), args))
}

/// The last generic type argument, skipping lifetimes (`Cow<'a, str>` → `str`).
fn last_type_argument(args: &[TokenTree]) -> Option<Vec<TokenTree>> {
    split_top_level(args, ',').into_iter().rev().find(|arg| !is_punct(arg.first(), '\''))
}

fn classify(tokens: &[TokenTree]) -> Base {
    let (_, tokens) = strip_refs(tokens);
    if let [TokenTree::Group(g)] = tokens {
        if g.delimiter() == Delimiter::Bracket {
            let inner: Vec<TokenTree> = g.stream().into_iter().collect();
            let parts = split_top_level(&inner, ';');
            return match parts.as_slice() {
                [element] => match classify_element(element) {
                    Some(Base::Number(Kind::U8)) => Base::Bytes,
                    Some(Base::Str) if is_punct(element.first(), '&') => Base::StrList,
                    _ => Base::Unknown,
                },
                [element, len] if matches!(classify_element(element), Some(Base::Number(Kind::U8))) && !is_punct(element.first(), '&') => {
                    match tokens_to_string(len).replace('_', "").trim_end_matches("usize").parse::<usize>() {
                        Ok(n) => Base::ByteArray(n),
                        Err(_) => Base::Unknown,
                    }
                }
                _ => Base::Unknown,
            };
        }
        if g.delimiter() == Delimiter::Parenthesis || g.delimiter() == Delimiter::None {
            // `(T)` or an invisible group coming from a macro expansion
            let inner: Vec<TokenTree> = g.stream().into_iter().collect();
            return classify(&inner);
        }
    }
    let Some((name, args)) = last_segment(tokens) else {
        return Base::Unknown;
    };
    match (name.as_str(), args) {
        ("String" | "str", None) => Base::Str,
        ("PathBuf" | "Path", None) => Base::Path,
        ("usize", None) => Base::Usize,
        ("isize", None) => Base::Isize,
        ("bool", None) => Base::Bool,
        ("IpAddr", None) => Base::IpAddr,
        ("Ipv4Addr" | "Ipv6Addr", None) => Base::IpV4V6,
        ("SystemTime", None) => Base::SystemTime,
        ("Box" | "Rc" | "Arc" | "Cow", Some(args)) => match last_type_argument(&args).map(|a| classify(&a)) {
            Some(Base::Str) => Base::Str,
            Some(Base::Path) => Base::Path,
            Some(Base::Bytes) => Base::Bytes,
            Some(Base::StrList) => Base::StrList,
            _ => Base::Unknown,
        },
        ("Vec", Some(args)) => {
            let arg = last_type_argument(&args).unwrap_or_default();
            match classify_element(&arg) {
                Some(Base::Number(Kind::U8)) if !is_punct(arg.first(), '&') => Base::Bytes,
                Some(Base::Str) if is_punct(arg.first(), '&') => Base::StrList,
                _ => Base::Unknown,
            }
        }
        (other, None) => number_kind(other).map(Base::Number).unwrap_or(Base::Unknown),
        _ => Base::Unknown,
    }
}

fn classify_element(tokens: &[TokenTree]) -> Option<Base> {
    let base = classify(tokens);
    (base != Base::Unknown).then_some(base)
}

/// `std :: vec :: Vec < u8 >` → `std::vec::Vec<u8>`, for error messages.
fn pretty(tokens: &[TokenTree]) -> String {
    let mut text = tokens_to_string(tokens);
    for (from, to) in [
        (" :: ", "::"),
        (":: ", "::"),
        (" < ", "<"),
        ("< ", "<"),
        (" <", "<"),
        (" >", ">"),
        (" ,", ","),
        ("& ", "&"),
        (" ;", ";"),
    ] {
        text = text.replace(from, to);
    }
    text
}

pub(crate) fn analyze(tokens: &[TokenTree]) -> FieldType {
    // `Option<T>` (by its last path segment, so `std::option::Option<T>` works too)
    if let Some((name, Some(args))) = last_segment(tokens) {
        if name == "Option" {
            let inner = last_type_argument(&args).unwrap_or_default();
            let (refs, _) = strip_refs(&inner);
            return FieldType {
                optional: true,
                refs,
                base: classify(&inner),
                text: pretty(&inner),
            };
        }
    }
    let (refs, _) = strip_refs(tokens);
    FieldType {
        optional: false,
        refs,
        base: classify(tokens),
        text: pretty(tokens),
    }
}

impl FieldType {
    pub(crate) fn default_kind(&self, span: Span) -> Result<Kind> {
        let kind = match self.base {
            Base::Str => Kind::String,
            Base::Path => Kind::Path,
            Base::Number(kind) => kind,
            Base::Usize => Kind::U64,
            Base::Isize => Kind::I64,
            Base::Bool => Kind::Bool,
            Base::IpAddr | Base::IpV4V6 => Kind::IpAddr,
            Base::SystemTime => Kind::DateTime,
            Base::ByteArray(16) => Kind::Hash128,
            Base::ByteArray(20) => Kind::Hash160,
            Base::ByteArray(32) => Kind::Hash256,
            Base::ByteArray(_) | Base::Bytes => Kind::Bytes,
            Base::StrList => Kind::StringList,
            Base::Unknown => {
                return Err(Error::new(
                    span,
                    format!(
                        "cannot infer a whereexpr kind for type `{}`; add `#[whereexpr(kind = ...)]` (one of: {KIND_NAMES}) or `#[whereexpr(skip)]`",
                        self.text
                    ),
                ))
            }
        };
        Ok(kind)
    }

    /// Code of type `Option<Value<'_>>` built from a variable `v: &T`, where `T` is the
    /// (inner) field type. `value` is the path of `whereexpr::Value`.
    pub(crate) fn conversion(&self, kind: Kind, value: &str, span: Span) -> Result<String> {
        let some = "::core::option::Option::Some";
        let variant = format!("{value}::{}", kind.variant());
        // `v` dereferenced down to the value itself (for `Copy` types)
        let copied = format!("{}v", "*".repeat(self.refs + 1));
        // `v` dereferenced to a reference to the value itself
        let by_ref = format!("{}v", "*".repeat(self.refs));
        let incompatible = || {
            Error::new(
                span,
                format!("type `{}` cannot be used as a whereexpr `{}` attribute", self.text, kind.name()),
            )
        };
        let convert_from = format!("{some}({variant}(::core::convert::From::from({copied})))");
        let try_from = format!("::core::convert::TryFrom::try_from({copied}).ok().map({variant})");
        let clone_from = format!("{some}({variant}(::core::convert::From::from(::core::clone::Clone::clone({by_ref}))))");

        let code = match kind {
            Kind::String | Kind::Path => {
                let as_str = format!("{some}({variant}(::core::convert::AsRef::<str>::as_ref(v)))");
                // non UTF-8 paths become "unknown"
                let path_to_str = format!("::std::path::Path::to_str(::core::convert::AsRef::<::std::path::Path>::as_ref(v)).map({variant})");
                match (kind, self.base) {
                    (_, Base::Str) | (Kind::String, Base::Unknown) => as_str,
                    (_, Base::Path) | (Kind::Path, Base::Unknown) => path_to_str,
                    _ => return Err(incompatible()),
                }
            }
            Kind::StringList => match self.base {
                Base::StrList | Base::Unknown => format!("{some}({variant}(::core::convert::AsRef::<[&str]>::as_ref(v)))"),
                _ => return Err(incompatible()),
            },
            Kind::Bytes => match self.base {
                Base::Bytes | Base::ByteArray(_) | Base::Unknown => {
                    format!("{some}({variant}(::core::convert::AsRef::<[u8]>::as_ref(v)))")
                }
                _ => return Err(incompatible()),
            },
            Kind::Hash128 | Kind::Hash160 | Kind::Hash256 => {
                let len = match kind {
                    Kind::Hash128 => 16,
                    Kind::Hash160 => 20,
                    _ => 32,
                };
                match self.base {
                    Base::ByteArray(n) if n == len => {}
                    Base::Unknown => {}
                    _ => return Err(incompatible()),
                }
                format!("{some}({variant}(::core::borrow::Borrow::<[u8; {len}]>::borrow(v)))")
            }
            k if k.is_number() => match self.base {
                Base::Number(from) if from == k => format!("{some}({variant}({copied}))"),
                Base::Number(from) if is_lossless(from, k) => convert_from,
                Base::Number(from) => {
                    return Err(Error::new(
                        span,
                        format!(
                            "a `{}` field cannot be exposed as `{}` without losing values; use a wider kind or a computed attribute",
                            from.name(),
                            k.name()
                        ),
                    ))
                }
                // out of range values become "unknown"
                Base::Usize | Base::Isize if k.is_integer() => try_from,
                Base::Unknown => clone_from,
                _ => return Err(incompatible()),
            },
            Kind::Bool => match self.base {
                Base::Bool => format!("{some}({variant}({copied}))"),
                Base::Unknown => clone_from,
                _ => return Err(incompatible()),
            },
            Kind::IpAddr => match self.base {
                Base::IpAddr => format!("{some}({variant}({copied}))"),
                Base::IpV4V6 => convert_from,
                Base::Unknown => clone_from,
                _ => return Err(incompatible()),
            },
            Kind::DateTime => match self.base {
                Base::Number(Kind::U64) => format!("{some}({variant}({copied}))"),
                Base::Number(Kind::U8 | Kind::U16 | Kind::U32) => convert_from,
                // negative timestamps become "unknown"
                Base::Number(k) if k.is_integer() => try_from,
                Base::Usize | Base::Isize => try_from,
                // times before 1970 become "unknown"
                Base::SystemTime => format!("v.duration_since(::std::time::UNIX_EPOCH).ok().map(|d| {variant}(d.as_secs()))"),
                Base::Unknown => format!("::core::convert::TryFrom::try_from(::core::clone::Clone::clone({by_ref})).ok().map({variant})"),
                _ => return Err(incompatible()),
            },
            _ => unreachable!(),
        };
        Ok(code)
    }
}
