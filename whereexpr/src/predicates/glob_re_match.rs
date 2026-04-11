use wax::{Any, Glob, Program};
use super::utf8_builder::Utf8Builder;

#[derive(Debug)]
pub(crate) struct GlobREMatch {
    matcher: Any<'static>,
}

impl GlobREMatch {
    pub(crate) fn with_value(value: &str) -> Option<Self> {
        Glob::new(value)
        .ok()
        .map(|g| g.into_owned())
        .and_then(|g| wax::any([g]).ok())
        .map(|matcher| Self { matcher })
    }
    pub(crate) fn new(list: &[String]) -> Option<Self> {
        let globs: Vec<Glob<'static>> = list
            .iter()
            .filter_map(|s| Glob::new(s.as_str()).ok().map(|g| g.into_owned()))
            .collect();

        if globs.is_empty() {
            return None;
        }
        wax::any(globs)
            .ok()
            .map(|matcher| Self { matcher })
    }

    pub(crate) fn evaluate(&self, value: &[u8]) -> bool {
        let s = Utf8Builder::<2048>::new(value);
        self.matcher.is_match(s.as_str())
    }
}