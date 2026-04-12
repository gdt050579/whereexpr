use super::list_search::ListSearch;
use crate::Error;
use crate::Operation;
use crate::ValueKind;
use crate::Value;
use std::net::IpAddr;

#[derive(Debug)]
struct Equals {
    value: IpAddr,
}
impl Equals {
    fn new(value: IpAddr) -> Self {
        Self { value }
    }
    fn evaluate(&self, value: IpAddr) -> bool {
        self.value == value
    }
}

#[derive(Debug)]
struct InRange {
    start: IpAddr,
    end: IpAddr,
}
impl InRange {
    fn with_str_list(values: &[&str]) -> Result<Self, Error> {
        if values.len() != 2 {
            return Err(Error::ExpectingTwoValuesForRange(ValueKind::IpAddr));
        }
        let start_str = values[0].trim();
        let end_str = values[1].trim();
        let start = start_str
            .parse()
            .map_err(|_| Error::FailToParseValue(start_str.to_string(), ValueKind::IpAddr))?;
        let end = end_str
            .parse()
            .map_err(|_| Error::FailToParseValue(end_str.to_string(), ValueKind::IpAddr))?;
        if start > end {
            return Err(Error::ExpectingMinToBeLessThanMax(ValueKind::IpAddr));
        }
        Ok(Self { start, end })
    }

    pub(crate) fn with_value_list(values: &[Value<'_>]) -> Result<Self, Error>
    {
        if values.len() != 2 {
            return Err(Error::ExpectingTwoValuesForRange(ValueKind::IpAddr));
        }
        let start = IpAddr::try_from(values[0].clone())?;
        let end = IpAddr::try_from(values[1].clone())?;
        if start > end {
            return Err(Error::ExpectingMinToBeLessThanMax(ValueKind::IpAddr));
        }
        Ok(Self { start, end })
    }

    fn evaluate(&self, value: IpAddr) -> bool {
        value >= self.start && value <= self.end
    }
}

#[derive(Debug)]
pub(crate) enum IpAddrPredicate {
    Equals(Equals),
    InRange(InRange),
    IsOneOf(ListSearch<IpAddr>),
}

impl IpAddrPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: IpAddr) -> bool {
        match self {
            IpAddrPredicate::Equals(p) => p.evaluate(value),
            IpAddrPredicate::InRange(p) => p.evaluate(value),
            IpAddrPredicate::IsOneOf(p) => p.evaluate(value),
        }
    }

    pub(crate) fn with_value(operation: Operation, value: IpAddr) -> Result<Self, Error> {
        match operation {
            Operation::Is => Ok(IpAddrPredicate::Equals(Equals::new(value))),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::IpAddr)),
        }
    }

    pub(crate) fn with_str(operation: Operation, value: &str) -> Result<Self, Error> {
        Self::with_value(
            operation,
            value.parse().map_err(|_| Error::FailToParseValue(value.to_string(), ValueKind::IpAddr))?,
        )
    }

    pub(crate) fn with_str_list(operation: Operation, values: &[&str]) -> Result<Self, Error> {
        match operation {
            Operation::IsOneOf => Ok(IpAddrPredicate::IsOneOf(ListSearch::with_str_list(values)?)),
            Operation::InRange => Ok(IpAddrPredicate::InRange(InRange::with_str_list(values)?)),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::IpAddr)),
        }
    }

    pub(crate) fn with_value_list(operation: crate::Operation, values: &[Value<'_>]) ->  Result<Self, Error> 
    {
        match operation {
            Operation::IsOneOf => Ok(IpAddrPredicate::IsOneOf(ListSearch::with_value_list(values)?)),
            Operation::InRange => Ok(IpAddrPredicate::InRange(InRange::with_value_list(values)?)),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::IpAddr)),
        }
    }  

}
