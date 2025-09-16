// TODO move TextRange definition and impl to here
use std::ops::Range;

pub trait EditableText<'a> {
    fn new(string: &'a str) -> Self;

    fn insert(&mut self, data: &str, offset: usize);

    fn delete(&mut self, range: Range<usize>);
}
