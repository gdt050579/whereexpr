#[derive(Clone, Copy)]
pub(super) struct Span {
    pub(super) start: usize,
    pub(super) end: usize,
}
impl Span {
    pub(super) const EMPTY: Self = Self {
        start: usize::MAX,
        end: usize::MAX,
    };
    pub(super) fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}