//! `#[derive(Attributes)]` for [whereexpr](https://docs.rs/whereexpr).
//!
//! Normally used through the `derive` feature of `whereexpr`:
//!
//! ```toml
//! [dependencies]
//! whereexpr = { version = "0.2", features = ["derive"] }
//! ```
//!
//! See [`Attributes`](derive@Attributes) for the supported field types and options.
//!
//! This crate has no dependencies: it parses the struct with `proc_macro` only.

extern crate proc_macro;

mod derive;
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
/// ```no_compile
/// use whereexpr::*;
/// use whereexpr_proc_macro::Attributes; // with the `derive` feature: `use whereexpr::Attributes;`
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
///   `i32`, `i64`, `f32`, `f64`, `hash128`, `hash160`, `hash256`, `ip`, `datetime`, `bool`.
///   A number can be widened (`u32` as `u64`) but not narrowed; `usize`/`isize` values and
///   timestamps that do not fit become unknown. For a type the derive does not recognise, the
///   kind decides the conversion: `string` uses `AsRef<str>`, `path` `AsRef<Path>`, `bytes`
///   `AsRef<[u8]>`, `string_list` `AsRef<[&str]>`, the hashes `Borrow<[u8; N]>`, `datetime`
///   `TryInto<u64>` and the others `Clone + Into<T>`.
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
/// ```no_compile
/// use whereexpr::*;
/// use whereexpr_proc_macro::Attributes;
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
