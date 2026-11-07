use crate::Error;
use crate::Operation;
use crate::Value;
use crate::ValueKind;

#[derive(Debug)]
pub(crate) enum EnumPredicate {
    Is(u16),
    IsOneOfSmall(u64),
    IsOneOf(Box<[u64]>),
}

impl EnumPredicate {
    #[inline(always)]
    pub(crate) fn evaluate(&self, value: u16) -> bool {
        match self {
            Self::Is(id) => *id == value,
            Self::IsOneOfSmall(mask) => value < 64 && (mask >> value) & 1 != 0,
            Self::IsOneOf(bits) => bits
                .get((value >> 6) as usize)
                .is_some_and(|word| (word >> (value & 63)) & 1 != 0),
        }
    }

    pub(crate) fn with_value(operation: Operation, value: u16) -> Result<Self, Error> {
        match operation {
            Operation::Is => Ok(Self::Is(value)),
            _ => Err(Error::InvalidOperationForValue(operation, ValueKind::Enum)),
        }
    }

    pub(crate) fn with_ids(operation: Operation, ids: &[u16]) -> Result<Self, Error> {
        if !matches!(operation, Operation::IsOneOf) {
            return Err(Error::InvalidOperationForValue(operation, ValueKind::Enum));
        }
        let Some(&max) = ids.iter().max() else {
            return Err(Error::EmptyListForOperation(operation));
        };
        if ids.len() == 1 {
            return Ok(Self::Is(ids[0]));
        }
        if max < 64 {
            return Ok(Self::IsOneOfSmall(ids.iter().fold(0u64, |mask, &id| mask | (1u64 << id))));
        }
        let mut bits = vec![0u64; (max as usize >> 6) + 1];
        for &id in ids {
            bits[(id >> 6) as usize] |= 1u64 << (id & 63);
        }
        Ok(Self::IsOneOf(bits.into_boxed_slice()))
    }

    pub(crate) fn with_value_list(operation: Operation, values: &[Value<'_>]) -> Result<Self, Error> {
        let mut ids = Vec::with_capacity(values.len());
        for value in values {
            match value {
                Value::Enum(id) => ids.push(*id),
                other => return Err(Error::ExpectingADifferentValueKind(other.kind(), ValueKind::Enum)),
            }
        }
        Self::with_ids(operation, &ids)
    }
}