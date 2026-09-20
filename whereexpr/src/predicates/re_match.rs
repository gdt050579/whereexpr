use crate::{Error, Operation, ValueKind};
use regex::Regex;


#[derive(Debug)]
pub(crate) struct ReMatch {
    matcher: Regex,
}

impl ReMatch {
    pub(crate) fn with_str(value: &str) -> Result<Self, Error> {
        Regex::new(value)
            .map(|matcher| Self { matcher })
            .map_err(|e| {
                Error::FailToBuildInternalDataStructure(
                    Operation::ReMatch,
                    ValueKind::String,
                    format!("\nRegex: {}\nError: {}", value, e),
                )
            })
    }

    pub(crate) fn evaluate(&self, value: &str) -> bool {
        self.matcher.is_match(value)
    }
}
