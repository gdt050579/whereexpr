use whereexpr::*;

pub trait TestTrait {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
    fn init()->Self;
    fn run_test(&mut self, count: usize);
}

pub struct StringData {
    pub value: String,
}
impl Attributes for StringData {
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        if idx == AttributeIndex::new(0) {
            Some(Value::String(&self.value))
        } else {
            None
        }
    }
    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        if idx == AttributeIndex::new(0) {
            Some(ValueKind::String)
        } else {
            None
        }
    }
    
    fn index(name: &str) -> Option<AttributeIndex> {
        if name == "value" {
            Some(AttributeIndex::new(0))
        } else {
            None
        }
    }
    const TYPE_ID: u64 = 0x517652f2;
    const TYPE_NAME: &'static str = "StringData";
}


pub struct BufferData {
    pub value: String,
}
impl Attributes for BufferData {
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        if idx == AttributeIndex::new(0) {
            Some(Value::Path(&self.value))
        } else {
            None
        }
    }
    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        if idx == AttributeIndex::new(0) {
            Some(ValueKind::Path)
        } else {
            None
        }
    }
    
    fn index(name: &str) -> Option<AttributeIndex> {
        if name == "value" {
            Some(AttributeIndex::new(0))
        } else {
            None
        }
    }
    const TYPE_ID: u64 = 0x517652f3;
    const TYPE_NAME: &'static str = "BufferData";
}