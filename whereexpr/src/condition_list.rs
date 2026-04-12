use super::Condition;
use std::collections::BTreeMap;

pub(crate) struct ConditionList {
    conditions: Vec<Condition>,
    indexes: BTreeMap<u64, u16>,
}

impl ConditionList {
    pub(crate) fn new() -> Self {
        Self {
            conditions: Vec::new(),
            indexes: BTreeMap::new(),
        }
    }
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            conditions: Vec::with_capacity(capacity),
            indexes: BTreeMap::new(),
        }
    }
    #[inline(always)]
    fn fnv64(s: &str) -> u64 {
        let mut h = 14695981039346656037u64;
        for &byte in s.as_bytes() {
            h ^= byte.to_ascii_lowercase() as u64;
            h = h.wrapping_mul(1099511628211u64);
        }
        h
    }
    pub(crate) fn add(&mut self, name: &str, cond: Condition) -> bool {
        let hash = Self::fnv64(name);
        if self.indexes.contains_key(&hash) {
            return false;
        }
        self.indexes.insert(hash, self.conditions.len() as u16);
        self.conditions.push(cond);
        true
    }
    pub(crate) fn from_name(&self, name: &str) -> Option<u16> {
        let hash = Self::fnv64(name);
        self.indexes.get(&hash).copied()
    }
    pub(crate) fn get(&self, index: u16) -> Option<&Condition> {
        self.conditions.get(index as usize)
    }
}
