mod basic;
mod hash;
mod datetime;

pub(crate) trait ValueKindConst {
    const VALUE_KIND: super::ValueKind;
}
pub(crate) trait FromRepr<T> {
    fn from_repr(repr: &str) -> Result<T, crate::Error>;
}

pub use basic::*;
pub(crate) use hash::*;
pub(crate) use datetime::DateTime;