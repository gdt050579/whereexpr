use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug)]
pub(crate) struct ListSearch<T>
where
    T: Copy + Eq + FromStr + Debug + Ord
{
    list: Vec<T>,
}

impl<T> ListSearch<T>
where
    T: Copy + Eq + FromStr + Debug + Ord,
{
    pub(crate) fn new(list: &[String]) -> Option<Self> {
        let mut obj_list: Vec<T> = Vec::with_capacity(list.len());
        for s in list {
            let value = T::from_str(s.as_str()).ok()?;
            obj_list.push(value);
        }
        if obj_list.is_empty() {
            return None;
        }
        obj_list.sort();
        obj_list.dedup();
        Some(Self { list: obj_list })
    }
    pub(crate) fn evaluate(&self, value: T) -> bool {
        if self.list.len() <= 8 {
            self.list.contains(&value)
        } else {
            self.list.binary_search(&value).is_ok()
        }
    }
}
