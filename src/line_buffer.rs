use crate::interface::EditableText;
use crate::TextRange;
use std::fmt;

pub struct LineBuffer {
    lines: Vec<String>,
}

impl LineBuffer {
    fn find_pos(&self, offset: usize) -> (usize, usize) {
        let mut cumulative_offset = 0;
        for (i, line) in self.lines.iter().enumerate() {
            let line_len = line.len();
            if cumulative_offset + line_len >= offset {
                return (i, offset - cumulative_offset);
            }
            cumulative_offset += line_len + 1; // for newline
        }
        if self.lines.is_empty() {
            return (0, 0);
        }
        let last_line_idx = self.lines.len() - 1;
        (last_line_idx, self.lines[last_line_idx].len())
    }
}

impl EditableText for LineBuffer {
    fn new(string: String) -> Self {
        let lines = string.lines().map(|s| s.to_string()).collect();
        LineBuffer { lines }
    }

    fn insert(&mut self, data: &str, offset: usize) {
        let (line_idx, col_idx) = self.find_pos(offset);

        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        let original_line = self.lines.remove(line_idx);
        let (before, after) = original_line.split_at(col_idx);

        let mut new_content = String::from(before);
        new_content.push_str(data);
        new_content.push_str(after);

        let new_lines = new_content.lines().map(|s| s.to_string());
        self.lines.splice(line_idx..line_idx, new_lines);
    }

    fn delete(&mut self, range: TextRange) {
        if range.start >= range.end {
            return;
        }

        let (start_line, start_col) = self.find_pos(range.start);
        let (end_line, end_col) = self.find_pos(range.end);

        if start_line == end_line {
            self.lines[start_line].replace_range(start_col..end_col, "");
        } else {
            let end_of_start_line = self.lines[start_line].split_at(start_col).0.to_string();
            let start_of_end_line = self.lines[end_line].split_at(end_col).1.to_string();

            let merged_line = end_of_start_line + &start_of_end_line;
            
            self.lines[start_line] = merged_line;
            self.lines.drain((start_line + 1)..=end_line);
        }
    }
}

impl fmt::Display for LineBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}
