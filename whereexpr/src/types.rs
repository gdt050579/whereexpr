mod basic;
mod hash;
mod datetime;

pub(crate) trait IntoValueKind {
    const VALUE_KIND: super::ValueKind;
}
pub(crate) trait FromRepr: Sized {
    fn from_repr(repr: &str) -> Result<Self, crate::Error>;
}

pub use basic::*;
pub(crate) use hash::*;
pub(crate) use datetime::DateTime;