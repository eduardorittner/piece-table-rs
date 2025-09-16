// TODO move TextRange definition and impl to here
use crate::TextRange;

pub trait EditableText<'a> {
    fn new(string: &'a str) -> Self;

    fn insert(&mut self, data: &str, offset: usize);

    fn delete(&mut self, range: TextRange);
}
