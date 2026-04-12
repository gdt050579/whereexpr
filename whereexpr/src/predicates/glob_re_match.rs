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
    pub(crate) fn with_value_list(list: &[Value<'_>]) -> Result<Self, Error> {
        let mut input_list: Vec<&str> = Vec::with_capacity(list.len());
        for value in list {
            match value {
                Value::Path(bytes) => {
                    if let Ok(s) = std::str::from_utf8(bytes) {
                        input_list.push(s);
                    } else {
                        return Err(Error::InvalidUTF8Value(bytes.to_vec(), ValueKind::Path));
                    }
                }
                _ => return Err(Error::ExpectingADifferentValueKind(value.kind(), ValueKind::Path)),
            }
        }
        Self::with_str_list(&input_list)
    }

    pub(crate) fn evaluate(&self, value: &[u8]) -> bool {
        let s = Utf8Builder::<2048>::new(value);
        self.matcher.is_match(s.as_str())
    }
}