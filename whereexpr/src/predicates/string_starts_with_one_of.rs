use super::lower_case_builder::LowerCaseBuilder;
use crate::{Error, Operation, Value, ValueKind};
use fst::raw::Fst;

pub(crate) struct StartsWithOneOf {
    fst: Fst<Vec<u8>>,
    ignore_case: bool,
}

impl StartsWithOneOf {
    fn new(mut patterns: Vec<String>, ignore_case: bool) -> Result<Self, Error> {
        patterns.sort();
        patterns.dedup();
        let set = fst::Set::from_iter(&patterns)
            .map_err(|e| Error::FailToBuildInternalDataStructure(Operation::EndsWithOneOf, ValueKind::String, e.to_string()))?;
        Ok(Self {
            fst: set.into_fst(),
            ignore_case,
        })
    }
    pub(crate) fn with_str_list(list: &[&str], ignore_case: bool) -> Result<Self, Error> {
        let patterns: Vec<String> = if ignore_case {
            list.iter().map(|s| s.to_lowercase()).collect()
        } else {
            list.iter().map(|s| String::from(*s)).collect()
        };
        Self::new(patterns, ignore_case)
    }
    pub(crate) fn with_value_list(list: &[Value<'_>]) -> Result<Self, Error> {
        let mut input_list: Vec<String> = Vec::with_capacity(list.len());
        for value in list {
            match value {
                Value::String(s) => input_list.push(s.to_string()),
                _ => return Err(Error::ExpectingADifferentValueKind(value.kind(), ValueKind::String)),
            }
        }
        Self::new(input_list, false)
    }

    pub(crate) fn evaluate(&self, value: &str) -> bool {
        if self.ignore_case {
            let lower = LowerCaseBuilder::<2048>::new(value);
            self.check(lower.as_str())
        } else {
            self.check(value)
        }
    }
    #[inline]
    fn check(&self, value: &str) -> bool {
        let mut node = self.fst.root();
        for &b in value.as_bytes() {
            if node.is_final() {
                return true;
            }
            match node.find_input(b) {
                None => return false,
                Some(idx) => {
                    node = self.fst.node(node.transition(idx).addr);
                }
            }
        }
        node.is_final()
    }
}

impl std::fmt::Debug for StartsWithOneOf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StartsWithOneOf {{ ignore_case: {} }}", self.ignore_case)
    }
}
