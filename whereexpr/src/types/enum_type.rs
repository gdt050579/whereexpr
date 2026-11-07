pub trait WhereexprEnum {
    fn as_u16(&self) -> u16;
    fn as_u16_from_str(s: &str) -> Option<u16>;
}