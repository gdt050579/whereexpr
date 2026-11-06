use crate::RuleParser;
pub struct Meta<'a> {
    parser: &'a RuleParser,
}
impl<'a> Meta<'a> {
    pub(crate) fn new(parser: &'a RuleParser) -> Self {
        Self { parser }
    }
    pub fn len(&self) -> usize {
        self.parser.meta_count()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
}
pub struct MetaItem<'a> {
    parser: &'a RuleParser,
    index: usize,
}
impl<'a> MetaItem<'a> {
    pub fn key(&self) -> &str {
        self.parser.meta_key(self.index).unwrap()
    }
    pub fn value(&self) -> &str {
        self.parser.meta_value(self.index).unwrap()
    }
}