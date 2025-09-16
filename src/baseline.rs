use crate::interface::EditableText;
use std::ops::Range;
use std::fmt;

pub struct Baseline {
    text: String,
}

impl EditableText<'_> for Baseline {
    fn new(string: &str) -> Self {
        Baseline { text: string.to_string() }
    }

    fn insert(&mut self, data: &str, offset: usize) {
        self.text.insert_str(offset, data);
    }

    fn delete(&mut self, range: Range<usize>) {
        self.text.replace_range(range.start..range.end, "");
    }
}

impl fmt::Display for Baseline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
