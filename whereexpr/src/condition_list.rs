use super::CompiledCondition;

const LINEAR_THRESHOLD: usize = 16;

enum ConditionIndex {
    Linear(Vec<(u64, u16)>),
    Sorted(Vec<(u64, u16)>),
}

impl ConditionIndex {
    fn with_capacity(capacity: usize) -> Self {
        Self::Linear(Vec::with_capacity(capacity))
    }

    fn contains(&self, hash: u64) -> bool {
        match self {
            Self::Linear(vec) => vec.iter().any(|(h, _)| *h == hash),
            Self::Sorted(vec) => vec.binary_search_by_key(&hash, |(h, _)| *h).is_ok(),
        }
    }

    fn insert(&mut self, hash: u64, index: u16) {
        match self {
            Self::Linear(vec) => {
                vec.push((hash, index));
                if vec.len() == LINEAR_THRESHOLD {
                    vec.sort_unstable_by_key(|(h, _)| *h);
                    let sorted = std::mem::take(vec);
                    *self = Self::Sorted(sorted);
                }
            }
            Self::Sorted(vec) => {
                let pos = vec.partition_point(|(h, _)| *h < hash);
                vec.insert(pos, (hash, index));
            }
        }
    }

    fn get(&self, hash: u64) -> Option<u16> {
        match self {
            Self::Linear(vec) => vec.iter().find(|(h, _)| *h == hash).map(|(_, i)| *i),
            Self::Sorted(vec) => vec.binary_search_by_key(&hash, |(h, _)| *h)
                .ok()
                .map(|pos| vec[pos].1),
        }
    }
}

pub(crate) struct ConditionList {
    conditions: Vec<CompiledCondition>,
    index: ConditionIndex,
}

impl ConditionList {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self::with_capacity(0)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.conditions.len()
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    #[inline(always)]
    fn hash(s: &str) -> u64 {
        let mut h = 14695981039346656037u64;
        for &byte in s.as_bytes() {
            h ^= byte.to_ascii_lowercase() as u64;
            h = h.wrapping_mul(1099511628211u64);
        }
        h
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            conditions: Vec::with_capacity(capacity),
            index: ConditionIndex::with_capacity(capacity),
        }
    }

    pub(crate) fn add(&mut self, name: &str, cond: CompiledCondition) -> bool {
        let hash = Self::hash(name);
        if self.index.contains(hash) {
            return false;
        }
        let cond_index = self.conditions.len() as u16;
        self.conditions.push(cond);
        self.index.insert(hash, cond_index);
        true
    }

    pub(crate) fn index_of(&self, name: &str) -> Option<u16> {
        self.index.get(Self::hash(name))
    }

    pub(crate) fn get(&self, index: u16) -> Option<&CompiledCondition> {
        self.conditions.get(index as usize)
    }
}