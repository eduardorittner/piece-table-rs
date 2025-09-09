use crate::interface::EditableText;
use crate::TextRange;
use std::fmt;

pub struct Baseline {
    text: String,
}

impl EditableText for Baseline {
    fn new(string: String) -> Self {
        Baseline { text: string }
    }

    fn insert(&mut self, data: &str, offset: usize) {
        self.text.insert_str(offset, data);
    }

    fn delete(&mut self, range: TextRange) {
        self.text.replace_range(range.start..range.end, "");
    }
}

impl fmt::Display for Baseline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
