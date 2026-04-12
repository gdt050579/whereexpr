use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use crate::{Error, ValueKind, Operation};

use super::lower_case_builder::LowerCaseBuilder;

#[derive(Debug)]
pub(crate) struct ContainsOneOf {
    ac: AhoCorasick,
    ignore_case: bool,
}

impl ContainsOneOf {
    pub(crate) fn with_str_list(list: &[&str], ignore_case: bool) -> Result<Self, Error> {
        let ac = if ignore_case {
            // Normalize patterns to lowercase once at build time
            let lowered: Vec<String> = list.iter().map(|s| s.to_lowercase()).collect();
            AhoCorasickBuilder::new().match_kind(MatchKind::LeftmostFirst).build(&lowered)
        } else {
            AhoCorasickBuilder::new().match_kind(MatchKind::LeftmostFirst).build(list)
        };
        if let Ok(ac) = ac {    
            Ok(Self { ac, ignore_case })
        } else {
            Err(Error::FailToBuildInternalDataStructure(Operation::ContainsOneOf, ValueKind::String))
        }
    }

    pub(crate) fn evaluate(&self, value: &str) -> bool {
        if self.ignore_case {
            let lower_case = LowerCaseBuilder::<2048>::new(value);
            self.ac.is_match(lower_case.as_str())
        } else {
            self.ac.is_match(value)
        }
    }
}
