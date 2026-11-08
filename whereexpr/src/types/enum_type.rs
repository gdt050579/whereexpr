/// An enum whose variants can be used in conditions (`color is red`,
/// `color is-one-of [red, blue]`).
///
/// Each variant has a numeric id, and every name a condition may use maps to one of those
/// ids. Names are turned into ids once, when an expression is built (through
/// [`Attributes::enum_value`](crate::Attributes::enum_value)), so evaluating a condition
/// only compares ids.
///
/// Usually implemented with `#[derive(WhereexprEnum)]`, which numbers the variants in
/// declaration order (0, 1, 2, ...) and names each one after its identifier in lower case,
/// or after the names listed in `#[whereexpr(aliases = [...])]`. The generated `impl` names
/// the trait without its path, so the trait must be in scope where the derive is used:
///
/// ```rust
/// use whereexpr::WhereexprEnum;
///
/// #[derive(WhereexprEnum)]
/// enum Color {
///     #[whereexpr(aliases = ["red", "rosu"])]
///     Red,      // 0
///     Green,    // 1
///     Blue,     // 2
/// }
///
/// assert_eq!(Color::Green.as_u16(), 1);
/// assert_eq!(Color::as_u16_from_str("Rosu"), Some(0));
/// assert_eq!(Color::as_u16_from_str("yellow"), None);
/// ```
///
/// A struct field of such an enum is exposed with `#[whereexpr(kind = enum)]` in
/// `#[derive(Attributes)]`. The trait can also be implemented by hand, as long as
/// `as_u16_from_str` returns the id of the same variant `as_u16` returns for it.
pub trait WhereexprEnum {
    /// The id of this value's variant.
    fn as_u16(&self) -> u16;

    /// The id of the variant called `s`, or `None` if `s` is not one of its names.
    ///
    /// With the derive, names ignore ASCII case and are recognised by a 64-bit hash only (the
    /// names are not stored in the program), so a text that is not a name returns `None`
    /// unless its hash equals the hash of a name.
    fn as_u16_from_str(s: &str) -> Option<u16>;
}