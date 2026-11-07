pub(super) fn hash(name: &str) -> u64 {
    // fnv64 hash
    let mut hash = 14695981039346656037u64;
    for &byte in name.as_bytes() {
        hash ^= byte.to_ascii_lowercase() as u64;
        hash = hash.wrapping_mul(1099511628211u64);
    }
    hash
}