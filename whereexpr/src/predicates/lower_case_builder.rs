pub(crate) struct LowerCaseBuilder<const N: usize> {
    buf: [u8; N],
    len: usize,
    heap: String,
}

impl<const N: usize> LowerCaseBuilder<N> {
    pub(crate) fn new(text: &str) -> Self {
        let mut buf = [0u8; N];
        let mut len = 0usize;

        'stack: {
            for ch in text.chars() {
                for lower_ch in ch.to_lowercase() {
                    let char_len = lower_ch.len_utf8();
                    if len + char_len > N {
                        break 'stack;
                    }
                    lower_ch.encode_utf8(&mut buf[len..]);
                    len += char_len;
                }
            }
            return Self {
                buf,
                len,
                heap: String::new(),
            };
        }
        Self {
            buf: [0u8; N],
            len: usize::MAX,
            heap: text.to_lowercase(),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        if self.len != usize::MAX {
            unsafe { std::str::from_utf8_unchecked(&self.buf[..self.len]) }
        } else {
            &self.heap
        }
    }
}