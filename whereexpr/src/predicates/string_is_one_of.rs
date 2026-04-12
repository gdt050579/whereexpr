use super::lower_case_builder::LowerCaseBuilder;
use crate::{Error, Value, ValueKind};

#[inline(always)]
fn fnv64(s: &str) -> u64 {
    let mut h = 14695981039346656037u64;
    for &byte in s.as_bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(1099511628211u64);
    }
    h
}

#[derive(Debug)]
pub(crate) struct IsOneOf {
    hashes: Vec<u64>,
    strings: Vec<String>,
    ignore_case: bool,
}

impl IsOneOf {
    fn new(mut pairs: Vec<(u64, String)>, ignore_case: bool) -> Self {
        pairs.sort_unstable_by(|a, b| a.1.cmp(&b.1));
        pairs.dedup_by(|a, b| a.1 == b.1);
        pairs.sort_unstable_by_key(|&(h, _)| h);
        let (hashes, strings) = pairs.into_iter().unzip();
        Self {hashes,strings,ignore_case,}
    }
    pub(crate) fn with_str_list(list: &[&str], ignore_case: bool) -> Result<Self, Error> {
        let pairs: Vec<(u64, String)> = list
            .iter()
            .map(|s| {
                let normalized = if ignore_case { s.to_lowercase() } else { String::from(*s) };
                (fnv64(&normalized), normalized)
            })
            .collect();
        Ok(Self::new(pairs, ignore_case))
    }

    pub(crate) fn with_value_list(list: &[Value<'_>]) -> Result<Self, Error>
    {
        let mut pairs: Vec<(u64, String)> = Vec::with_capacity(list.len());
        for value in list {
            match value {
                Value::String(s) => pairs.push((fnv64(s), s.to_string())),
                _ => return Err(Error::ExpectingADifferentValueKind(value.kind(), ValueKind::String)),
            }
        }
        Ok(Self::new(pairs, false))      
    }
    pub(crate) fn evaluate(&self, value: &str) -> bool {
        if self.ignore_case {
            let lower = LowerCaseBuilder::<512>::new(value);
            self.check(lower.as_str())
        } else {
            self.check(value)
        }
    }

    #[inline]
    fn check(&self, value: &str) -> bool {
        let h = fnv64(value);

        if self.hashes.len() < 16 {
            for (i, &stored) in self.hashes.iter().enumerate() {
                if stored == h && self.strings[i] == value {
                    return true;
                }
            }
            return false;
        } else {
            match self.hashes.binary_search(&h) {
                Err(_) => false,
                Ok(idx) => self.verify_match(idx, value, h),
            }
        }
    }

    #[inline]
    fn verify_match(&self, idx: usize, value: &str, hash: u64) -> bool {
        // Check the found index first
        if self.strings[idx] == value {
            return true;
        }
        // Scan left for collision neighbours
        let mut i = idx;
        while i > 0 && self.hashes[i - 1] == hash {
            i -= 1;
            if self.strings[i] == value {
                return true;
            }
        }
        // Scan right for collision neighbours
        let mut i = idx + 1;
        while i < self.hashes.len() && self.hashes[i] == hash {
            if self.strings[i] == value {
                return true;
            }
            i += 1;
        }
        false
    }
}
