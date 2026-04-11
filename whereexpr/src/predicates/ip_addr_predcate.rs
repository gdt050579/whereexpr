use crate::Operation;
use super::list_search::ListSearch;
use std::net::IpAddr;

#[derive(Debug)]
struct Equals {
    value: IpAddr,
}
impl Equals {
    fn new(value: &str) -> Option<Self> {
        Some(Self { value: value.parse().ok()? })
    }
    fn evaluate(&self, value: IpAddr) -> bool {
        self.value == value
    }
}

#[derive(Debug)]
struct Different {
    value: IpAddr,
}
impl Different {
    fn new(value: &str) -> Option<Self> {
        Some(Self { value: value.parse().ok()? })
    }
    fn evaluate(&self, value: IpAddr) -> bool {
        self.value != value
    }
}

#[derive(Debug)]
struct InRange {
    start: IpAddr,
    end: IpAddr,
}
impl InRange {
    fn new(values: &[String]) -> Option<Self> {
        if values.len() != 2 {
            return None;
        }
        let start_str = values[0].trim();
        let end_str = values[1].trim();
        let start = start_str.parse().ok()?;
        let end = end_str.parse().ok()?;
        if start > end {
            return None;
        }
        Some(Self { start, end })
    }
    fn evaluate(&self, value: IpAddr) -> bool {
        value >= self.start && value <= self.end
    }
}

#[derive(Debug)]
pub(crate) enum IpAddrPredicate {
    Equals(Equals),
    Different(Different),
    InRange(InRange),
    IsOneOf(ListSearch<IpAddr>),
}

impl IpAddrPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: IpAddr) -> bool {
        match self {
            IpAddrPredicate::Equals(p)    => p.evaluate(value),
            IpAddrPredicate::Different(p) => p.evaluate(value),
            IpAddrPredicate::InRange(p)   => p.evaluate(value),
            IpAddrPredicate::IsOneOf(p)   => p.evaluate(value),
        }
    }
    pub(crate) fn new(operation: Operation, value: &str) -> Option<Self> {
        match operation {
            Operation::Is      => Some(IpAddrPredicate::Equals(Equals::new(value)?)),
            Operation::IsNot   => Some(IpAddrPredicate::Different(Different::new(value)?)),
            _                  => None,
        }
    }
    pub(crate) fn new_with_values(operation: Operation, values: &[String]) -> Option<Self> {
        match operation {
            Operation::IsOneOf => Some(IpAddrPredicate::IsOneOf(ListSearch::new(values)?)),
            Operation::InRange => Some(IpAddrPredicate::InRange(InRange::new(values)?)),
            _                  => None,
        }
    }
}