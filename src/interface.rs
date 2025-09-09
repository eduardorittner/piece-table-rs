// TODO move TextRange definition and impl to here
use crate::TextRange;

pub trait EditableText {
    fn new(string: String) -> Self;

    fn insert(&mut self, data: &str, offset: usize);

    fn delete(&mut self, range: TextRange);
}
