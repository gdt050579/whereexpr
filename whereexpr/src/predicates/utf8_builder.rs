pub(crate) struct Utf8Builder<'a, const N: usize> {
    inner: Utf8Inner<'a, N>,
}

enum Utf8Inner<'a, const N: usize> {
    Borrowed(&'a str),
    Stack { buf: [u8; N], len: usize },
    Heap(String),
}

impl<'a, const N: usize> Utf8Builder<'a, N> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        // Fast path: valid UTF-8 — borrow directly, zero copy
        if let Ok(s) = std::str::from_utf8(bytes) {
            return Self {
                inner: Utf8Inner::Borrowed(s),
            };
        }

        // Repair: try stack first
        let mut buf = [0u8; N];
        let mut len = 0usize;

        'stack: {
            for chunk in bytes.utf8_chunks() {
                let valid = chunk.valid();
                if len + valid.len() > N {
                    break 'stack;
                }
                buf[len..len + valid.len()].copy_from_slice(valid.as_bytes());
                len += valid.len();
            }
            return Self {
                inner: Utf8Inner::Stack { buf, len },
            };
        }

        // Repair: heap fallback
        let mut heap = String::with_capacity(bytes.len());
        for chunk in bytes.utf8_chunks() {
            heap.push_str(chunk.valid());
        }

        Self {
            inner: Utf8Inner::Heap(heap),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match &self.inner {
            Utf8Inner::Borrowed(s) => s,
            Utf8Inner::Stack { buf, len } => {
                unsafe { std::str::from_utf8_unchecked(&buf[..*len]) }
            }
            Utf8Inner::Heap(s) => s.as_str(),
        }
    }
}