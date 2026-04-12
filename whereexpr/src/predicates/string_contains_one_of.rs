use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use crate::{Error, Operation, Value, ValueKind};

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
                _ => return Err(Error::ExpectingADifferentValueKind(v.kind(), ValueKind::String)),
            }
        }
        if let Ok(ac) = AhoCorasickBuilder::new().match_kind(MatchKind::LeftmostFirst).build(input_list) {    
            Ok(Self { ac, ignore_case: false })
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
