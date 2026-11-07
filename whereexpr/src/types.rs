mod basic;
mod hash;
mod datetime;
mod enum_type;

pub(crate) trait IntoValueKind {
    const VALUE_KIND: super::ValueKind;
}
pub(crate) trait FromRepr: Sized {
    fn from_repr(repr: &str) -> Result<Self, crate::Error>;
}

pub(crate) use hash::*;
pub(crate) use datetime::DateTime;

pub use enum_type::WhereexprEnum;