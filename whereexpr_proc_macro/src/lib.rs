//! Derive macros for [whereexpr](https://docs.rs/whereexpr):
//!
//! * [`Attributes`](derive@Attributes) implements `whereexpr::Attributes` for a struct, so its
//!   fields (and properties) can be tested by conditions and rules.
//! * [`WhereexprEnum`](derive@WhereexprEnum) implements `whereexpr::WhereexprEnum` for an enum,
//!   so a field of that enum can be compared by variant (`color is red`) with
//!   `#[whereexpr(kind = enum)]`.
//!
//! There is no need to depend on this crate directly: `whereexpr` re-exports both derives, so
//! adding `whereexpr` is enough.
//!
//! ```toml
//! [dependencies]
//! whereexpr = "0.3"
//! ```
//!
//! ```ignore
//! use whereexpr::*;
//!
//! #[derive(WhereexprEnum)]
//! enum Color { Red, Green, Blue }
//!
//! #[derive(Attributes)]
//! struct Car {
//!     #[whereexpr(kind = enum)]
//!     color: Color,
//!     doors: u8,
//! }
//! ```
//!
//! This crate has no dependencies: it parses the code with `proc_macro` only.

extern crate proc_macro;

mod derive;
mod enum_derive;
mod error;
mod kind;
mod meta;
mod parser;

use proc_macro::TokenStream;

/// Implements `whereexpr::Attributes` for a struct.
///
/// Every field becomes an attribute that conditions can test, named after the field and with a
/// `ValueKind` inferred from its type. The generated `get`, `kind` and `index` are the same
/// `match` statements you would write by hand, so evaluation is just as fast.
///
/// ```ignore
/// use whereexpr::*;
///
/// #[derive(Attributes)]
/// struct Person {
///     name: String,
///     #[whereexpr(name = "last_name")]
///     surname: String,
///     age: Option<u32>,          // `None` makes conditions on `age` "unknown"
///     #[whereexpr(kind = datetime)]
///     created: u64,              // a Unix timestamp, not a plain number
///     #[whereexpr(ignore)]
///     cache: Vec<u64>,
/// }
///
/// let expr = ExpressionBuilder::<Person>::new()
///     .add("is_john", Condition::from_str("name is John"))
///     .add("adult", Condition::from_str("age >= 18"))
///     .add("recent", Condition::from_str("created > 2024-01-01"))
///     .build("is_john && adult && recent")
///     .unwrap();
///
/// let john = Person { name: "John".into(), surname: "Doe".into(), age: Some(40), created: 1_750_000_000, cache: vec![] };
/// assert!(expr.matches(&john));
/// ```
///
/// # Inferred kinds
///
/// | Field type | Kind |
/// | --- | --- |
/// | `String`, `&str`, `Box<str>`, `Rc<str>`, `Arc<str>`, `Cow<str>` | `string` |
/// | `PathBuf`, `&Path`, `Box<Path>`, `Cow<Path>` | `path` (a non UTF-8 path is unknown) |
/// | `u8` … `u64`, `i8` … `i64`, `f32`, `f64` | the same numeric kind |
/// | `usize`, `isize` | `u64`, `i64` |
/// | `bool` | `bool` |
/// | `IpAddr`, `Ipv4Addr`, `Ipv6Addr` | `ip` |
/// | `SystemTime` | `datetime` (a time before 1970 is unknown) |
/// | `[u8; 16]`, `[u8; 20]`, `[u8; 32]` | `hash128`, `hash160`, `hash256` |
/// | `Vec<u8>`, `&[u8]`, other `[u8; N]` | `bytes` |
/// | `Vec<&str>`, `&[&str]` | `string_list` |
/// | `Option<T>` | the kind of `T`; `None` is unknown |
///
/// Types are recognised by name (`std::string::String` and `String` are the same), so a type
/// alias is not recognised: give it a `kind`.
///
/// # Field options
///
/// * `#[whereexpr(name = "name")]` - the name used in conditions. Names must start with an
///   ASCII letter and contain only letters, digits, `_` and `.` (the condition parser's rule).
/// * `#[whereexpr(kind = datetime)]` - the kind, when the type is ambiguous or not recognised.
///   Accepted: `string`, `string_list`, `path`, `bytes`, `u8`, `u16`, `u32`, `u64`, `i8`, `i16`,
///   `i32`, `i64`, `f32`, `f64`, `hash128`, `hash160`, `hash256`, `ip`, `datetime`, `bool`,
///   `enum`.
///   A number can be widened (`u32` as `u64`) but not narrowed; `usize`/`isize` values and
///   timestamps that do not fit become unknown. For a type the derive does not recognise, the
///   kind decides the conversion: `string` uses `AsRef<str>`, `path` `AsRef<Path>`, `bytes`
///   `AsRef<[u8]>`, `string_list` `AsRef<[&str]>`, the hashes `Borrow<[u8; N]>`, `datetime`
///   `TryInto<u64>` and the others `Clone + Into<T>`.
/// * `#[whereexpr(kind = enum)]` - an enum implementing `WhereexprEnum` (usually through
///   `#[derive(WhereexprEnum)]`), also as `Option<E>` or `&E`. It is never inferred. Conditions
///   compare variants by id (`color is red`, `color is-one-of [red, blue]`): the names are turned
///   into ids once, when the expression is built, through the generated `enum_value`.
///   Properties cannot be enums.
/// * `#[whereexpr(ignore)]` - not exposed. Required for fields of types that cannot be inferred.
///
/// Tuple structs are supported when every field that is not ignored has a `name`.
///
/// # Struct options
///
/// * `#[whereexpr(name = "Person")]` - `TYPE_NAME`, the type name used by rules
///   (`rule adults: Person { ... }`). Defaults to the struct name.
/// * `#[whereexpr(id = 42)]` - `TYPE_ID`. Defaults to the FNV-64 hash of `TYPE_NAME`. All
///   instantiations of a generic struct share it.
/// * `#[whereexpr(property = "fn <name> [as <attribute>] -> <kind>")]` - a property: an
///   attribute that is not a stored field but is returned by a method or function, which is
///   called only when a condition on it is evaluated.
///   * `fn extension -> string` calls the method `self.extension()`; a path with `::`
///     (`fn util::doc_len -> u64`) calls the function `util::doc_len(self)`.
///   * `as <attribute>` sets the name used in conditions (it may contain `.`, as in
///     `as meta.len`); the default is the method or function name.
///   * `<kind>` is one of the kinds above. The method or function returns `Option<T>`, where `T`
///     is the type held by the kind's `Value` variant: `&str` for `string` and `path`, `u64` for
///     `datetime`, `&[u8; 32]` for `hash256`, `std::net::IpAddr` for `ip`, the number type for
///     numeric kinds. `None` makes conditions on the property unknown.
///
///   Repeat the option for several properties, in one `#[whereexpr(...)]` or in several.
///
/// ```ignore
/// use whereexpr::*;
///
/// #[derive(Attributes)]
/// #[whereexpr(property = "fn word_count as words -> u32")]
/// struct Text<'a> {
///     text: &'a str,
/// }
///
/// impl Text<'_> {
///     fn word_count(&self) -> Option<u32> {
///         Some(self.text.split_whitespace().count() as u32)
///     }
/// }
///
/// let expr = Expression::from_rule_str::<Text>(
///     r#"rule long: Text { many = "words > 3" } where { many }"#,
/// ).unwrap();
/// assert!(expr.matches(&Text { text: "one two three four" }));
/// assert!(!expr.matches(&Text { text: "one two" }));
/// ```
#[proc_macro_derive(Attributes, attributes(whereexpr))]
pub fn derive_attributes(input: TokenStream) -> TokenStream {
    match derive::derive(input) {
        Ok(output) => output,
        Err(error) => error.into_compile_error(),
    }
}

/// Implements `whereexpr::WhereexprEnum` for an enum whose variants hold no data.
///
/// Variants are numbered in declaration order (`as_u16` returns 0, 1, 2, ...), and
/// `as_u16_from_str` turns a name back into that number. A variant's name is its identifier in
/// lower case, unless `#[whereexpr(aliases = [...])]` lists the names to use instead. Names
/// ignore ASCII case and must be unique across the enum.
///
/// ```ignore
/// use whereexpr::WhereexprEnum;
///
/// #[derive(WhereexprEnum)]
/// enum Color {
///     #[whereexpr(aliases = ["red", "rosu"])]
///     Red,      // 0, from "red" or "rosu"
///     Green,    // 1, from "green"
///     Blue,     // 2, from "blue"
/// }
///
/// assert_eq!(Color::Green.as_u16(), 1);
/// assert_eq!(Color::as_u16_from_str("Rosu"), Some(0));
/// assert_eq!(Color::as_u16_from_str("Red"), Some(0));
/// assert_eq!(Color::as_u16_from_str("yellow"), None);
/// ```
///
/// Rejected: variants with data (`Red(u8)`, `Red { level: u8 }`) and explicit discriminants
/// (`Red = 1`), since the numbering is the derive's own.
///
/// `as_u16_from_str` is a `match` on the FNV-1a hash of the lower-cased name, with one arm per
/// name or alias computed when the derive runs. Only the hashes are in the generated code, not
/// the names, so a text is accepted when its hash matches the hash of a name; two names with
/// the same hash are a compile error.
#[proc_macro_derive(WhereexprEnum, attributes(whereexpr))]
pub fn derive_whereexpr_enum(input: TokenStream) -> TokenStream {
    match enum_derive::derive(input) {
        Ok(output) => output,
        Err(error) => error.into_compile_error(),
    }
}
