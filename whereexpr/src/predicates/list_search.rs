use std::fmt::Debug;
use std::str::FromStr;
use crate::Error;
use crate::types::{ValueKindConst, FromRepr};

#[derive(Debug)]
pub(crate) struct ListSearch<T>
where
    T: Copy + Eq + FromStr + Debug + Ord + ValueKindConst + FromRepr<T>,
{
    list: Vec<T>,
}

impl<T> ListSearch<T>
where
    T: Copy + Eq + FromStr + Debug + Ord + ValueKindConst + FromRepr<T>,
{
    pub(crate) fn with_str_list(list: &[&str]) -> Result<Self, Error> {
        let mut obj_list: Vec<T> = Vec::with_capacity(list.len());
        for s in list {
            let value = T::from_repr(s)?;
            obj_list.push(value);
        }
        if obj_list.is_empty() {
            return Err(Error::EmptyListForIsOneOf(<T>::VALUE_KIND));
        }
        obj_list.sort();
        obj_list.dedup();
        Ok(Self { list: obj_list })
    }
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: T) -> bool {
        if self.list.len() <= 8 {
            self.list.contains(&value)
        } else {
            self.list.binary_search(&value).is_ok()
        }
    }
}
