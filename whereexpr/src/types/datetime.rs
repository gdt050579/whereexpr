use chrono::NaiveDate;

use crate::{ValueKind, types::FromRepr};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DateTime {
    value: u64,
}

impl DateTime {
    pub(crate) fn from_unix_timestamp(timestamp: u64) -> Self {
        Self { value: timestamp }
    }

    pub(crate) fn from_str_representation(value: &str) -> Option<Self> {
        let b = value.as_bytes();

        if b.len() < 10 {
            return None;
        }

        let year = parse_u32(b, 0, 4)?;
        if b[4] != b'-' && b[4] != b'/' {
            return None;
        }
        let sep = b[4];
        let month = parse_u32(b, 5, 7)?;
        if b[7] != sep {
            return None;
        }
        let day = parse_u32(b, 8, 10)?;

        if b.len() == 10 {
            let timestamp = calendar_to_unix(year, month, day, 0, 0, 0)?;
            return Some(Self { value: timestamp });
        }

        // skip 'T' or one-or-more spaces
        let time_start = if b[10] == b'T' {
            11
        } else if b[10] == b' ' {
            let mut i = 10;
            while i < b.len() && b[i] == b' ' {
                i += 1;
            }
            i
        } else {
            return None;
        };

        let b = &b[time_start..];

        if b.len() < 5 {
            return None;
        }
        let hour = parse_u32(b, 0, 2)?;
        if b[2] != b':' {
            return None;
        }
        let minute = parse_u32(b, 3, 5)?;

        let second = if b.len() >= 8 && b[5] == b':' {
            let s = parse_u32(b, 6, 8)?;
            if b.len() > 8 && b[8] != b'Z' {
                return None;
            }
            s
        } else if b.len() == 5 {
            0
        } else {
            return None;
        };

        if month < 1 || month > 12 {
            return None;
        }
        if day < 1 || day > 31 {
            return None;
        }
        if hour > 23 {
            return None;
        }
        if minute > 59 {
            return None;
        }
        if second > 59 {
            return None;
        }

        let timestamp = calendar_to_unix(year, month, day, hour, minute, second)?;
        Some(Self { value: timestamp })
    }
}

impl From<DateTime> for u64 {
    #[inline(always)]
    fn from(datetime: DateTime) -> Self {
        datetime.value
    }
}
impl FromRepr for DateTime {
    fn from_repr(repr: &str) -> Result<Self, crate::Error> {
        Self::from_str_representation(repr).ok_or(crate::Error::FailToParseValue(repr.to_string(), ValueKind::DateTime))
    }
}

fn parse_u32(b: &[u8], start: usize, end: usize) -> Option<u32> {
    if b.len() < end {
        return None;
    }
    let mut result = 0u32;
    for &byte in &b[start..end] {
        if byte < b'0' || byte > b'9' {
            return None;
        }
        result = result * 10 + (byte - b'0') as u32;
    }
    Some(result)
}

fn calendar_to_unix(year: u32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Option<u64> {
    let date = NaiveDate::from_ymd_opt(year as i32, month, day)?;
    let dt = date.and_hms_opt(hour, min, sec)?;
    let timestamp = dt.and_utc().timestamp();
    if timestamp < 0 {
        return None;
    }
    Some(timestamp as u64)
}
