use crate::Error;
use crate::Operation;

const FNV_OFFSET: u64 = 14695981039346656037;
const FNV_PRIME: u64 = 1099511628211;

const fn fnv_operation(s: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    let mut i = 0;
    while i < s.len() {
        let b = match s[i] {
            b'A'..=b'Z' => s[i] + 32, // to lowercase
            b'-' | b'_' => {
                i += 1;
                continue;
            } // skip - and _
            b => b,
        };
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    h
}

// (hash, Operation)
static OPERATIONS: &[(u64, Operation)] = &[
    // Is
    (fnv_operation(b"is"), Operation::Is),
    (fnv_operation(b"=="), Operation::Is),
    (fnv_operation(b"eq"), Operation::Is),
    (fnv_operation(b"equals"), Operation::Is),
    // IsNot
    (fnv_operation(b"isnot"), Operation::IsNot),
    (fnv_operation(b"!="), Operation::IsNot),
    (fnv_operation(b"neq"), Operation::IsNot),
    (fnv_operation(b"notequals"), Operation::IsNot),
    // IsOneOf
    (fnv_operation(b"isoneof"), Operation::IsOneOf),
    (fnv_operation(b"in"), Operation::IsOneOf),
    // IsNotOneOf
    (fnv_operation(b"isnotoneof"), Operation::IsNotOneOf),
    (fnv_operation(b"notin"), Operation::IsNotOneOf),
    // StartsWith
    (fnv_operation(b"startswith"), Operation::StartsWith),
    // NotStartsWith
    (fnv_operation(b"notstartswith"), Operation::NotStartsWith),
    // StartsWithOneOf
    (fnv_operation(b"startswithoneof"), Operation::StartsWithOneOf),
    // NotStartsWithOneOf
    (fnv_operation(b"notstartswithoneof"), Operation::NotStartsWithOneOf),
    // EndsWith
    (fnv_operation(b"endswith"), Operation::EndsWith),
    // NotEndsWith
    (fnv_operation(b"notendswith"), Operation::NotEndsWith),
    // EndsWithOneOf
    (fnv_operation(b"endswithoneof"), Operation::EndsWithOneOf),
    // NotEndsWithOneOf
    (fnv_operation(b"notendswithoneof"), Operation::NotEndsWithOneOf),
    // Contains
    (fnv_operation(b"contains"), Operation::Contains),
    // NotContains
    (fnv_operation(b"notcontains"), Operation::NotContains),
    // ContainsOneOf
    (fnv_operation(b"containsoneof"), Operation::ContainsOneOf),
    // NotContainsOneOf
    (fnv_operation(b"notcontainsoneof"), Operation::NotContainsOneOf),
    // Has
    (fnv_operation(b"has"), Operation::Has),
    // NotHas
    (fnv_operation(b"nothas"), Operation::NotHas),
    // HasOneOf
    (fnv_operation(b"hasoneof"), Operation::HasOneOf),
    // NotHasOneOf
    (fnv_operation(b"nothasoneof"), Operation::NotHasOneOf),
    // GlobREMatch
    (fnv_operation(b"glob"), Operation::GlobREMatch),
    (fnv_operation(b"globmatch"), Operation::GlobREMatch),
    // NotGlobREMatch
    (fnv_operation(b"notglob"), Operation::NotGlobREMatch),
    (fnv_operation(b"notglobmatch"), Operation::NotGlobREMatch),
    // GreaterThan
    (fnv_operation(b">"), Operation::GreaterThan),
    (fnv_operation(b"gt"), Operation::GreaterThan),
    (fnv_operation(b"greaterthan"), Operation::GreaterThan),
    // GreaterThanOrEqual
    (fnv_operation(b">="), Operation::GreaterThanOrEqual),
    (fnv_operation(b"gte"), Operation::GreaterThanOrEqual),
    (fnv_operation(b"greaterthanorequal"), Operation::GreaterThanOrEqual),
    // LessThan
    (fnv_operation(b"<"), Operation::LessThan),
    (fnv_operation(b"lt"), Operation::LessThan),
    (fnv_operation(b"lessthan"), Operation::LessThan),
    // LessThanOrEqual
    (fnv_operation(b"<="), Operation::LessThanOrEqual),
    (fnv_operation(b"lte"), Operation::LessThanOrEqual),
    (fnv_operation(b"lessthanorequal"), Operation::LessThanOrEqual),
    // InRange
    (fnv_operation(b"inrange"), Operation::InRange),
    // NotInRange
    (fnv_operation(b"notinrange"), Operation::NotInRange),
];

#[inline(always)]
fn lookup_operation(token: &[u8]) -> Option<Operation> {
    let hash = fnv_operation(token);
    OPERATIONS.iter().find(|(h, _)| *h == hash).map(|(_, op)| *op)
}

pub(crate) fn parse(txt: &str, start: usize, end: usize) -> Result<(Operation, usize), Error> {
    let slice = &txt[start..end];
    let bytes = slice.as_bytes();
    let len = slice.len();

    // skip leading whitespace
    let op_start = bytes.iter().position(|&b| !b.is_ascii_whitespace()).unwrap_or(len);

    if op_start == len {
        return Err(Error::ExpectingOperation(start as u32, end as u32, txt.to_string()));
    }

    // operation is formed out of letters, _, -, >, <, !, =
    let op_end = bytes[op_start..]
        .iter()
        .position(|&b| {
            !matches!(b,
                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-' |
                b'>' | b'<' | b'!' | b'='
            )
        })
        .map(|p| op_start + p)
        .unwrap_or(len);

    if op_end == op_start {
        return Err(Error::ExpectingOperation(
            (start + op_start) as u32,
            (start + op_end + 1) as u32,
            txt.to_string(),
        ));
    }

    let op_token = &bytes[op_start..op_end];

    let operation =
        lookup_operation(op_token).ok_or_else(|| Error::UnknownOperation((start + op_start) as u32, (start + op_end) as u32, txt.to_string()))?;

    // skip whitespace after operation to find value start
    let value_start = start + op_end + bytes[op_end..].iter().take_while(|&&b| b == b' ' || b == b'\t').count();

    Ok((operation, value_start))
}
