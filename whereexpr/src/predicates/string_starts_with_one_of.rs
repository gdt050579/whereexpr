use super::lower_case_builder::LowerCaseBuilder;
use fst::raw::Fst;

pub(crate) struct StartsWithOneOf {
    fst: Fst<Vec<u8>>,
    ignore_case: bool,
}

impl StartsWithOneOf {
    pub(crate) fn new(list: &[String], ignore_case: bool) -> Option<Self> {
        let mut patterns: Vec<String> = if ignore_case {
            list.iter().map(|s| s.to_lowercase()).collect()
        } else {
            list.to_vec()
        };
        patterns.sort();
        patterns.dedup();

        fst::Set::from_iter(&patterns).ok().map(|set| Self {
            fst: set.into_fst(),
            ignore_case,
        })
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