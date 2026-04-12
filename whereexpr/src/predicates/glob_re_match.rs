use wax::{Any, Glob, Program};
use super::utf8_builder::Utf8Builder;
use crate::{Error, Operation, Value, ValueKind};

#[derive(Debug)]
pub(crate) struct GlobREMatch {
    matcher: Any<'static>,
}

impl GlobREMatch {
    pub(crate) fn with_str(value: &str) -> Result<Self, Error> {
        Glob::new(value)
            .ok()
            .map(|g| g.into_owned())
            .and_then(|g| wax::any([g]).ok())
            .map(|matcher| Self { matcher })
            .ok_or(Error::FailToBuildInternalDataStructure(Operation::GlobREMatch, ValueKind::Path))
    }
    pub(crate) fn with_str_list(list: &[&str]) -> Result<Self, Error> {
        let globs: Vec<Glob<'static>> = list
            .iter()
            .filter_map(|s| Glob::new(s).ok().map(|g| g.into_owned()))
            .collect();

        if globs.is_empty() {
            return Err(Error::EmptyListForGlobREMatch(ValueKind::Path));
        }
        let matcher = wax::any(globs).map_err(|_| Error::FailToBuildInternalDataStructure(Operation::GlobREMatch, ValueKind::Path))?;
        Ok(Self { matcher })
    }
    pub(crate) fn with_value_list<'a, V>(list: &[V]) -> Result<Self, Error>
    where
        V: TryFrom<Value<'a>, Error = Error>,
        V: Into<Value<'a>> + Clone,
    {
        let mut input_list: Vec<&str> = Vec::with_capacity(list.len());
        for value in list {
            let v: Value<'a> = value.clone().into();
            match v {
                Value::String(s) => input_list.push(s),
                Value::Path(p) => {
                    let s = std::str::from_utf8(p).map_err(|_| Error::InvalidUTF8Value(p.to_vec(), ValueKind::Path))?;
                    input_list.push(s);
                }
                _ => return Err(Error::ExpectingADifferentValueKind(v.kind(), ValueKind::String)),
            }
        }
        Self::with_str_list(&input_list)
    }

    pub(crate) fn evaluate(&self, value: &[u8]) -> bool {
        let s = Utf8Builder::<2048>::new(value);
        self.matcher.is_match(s.as_str())
    }
}