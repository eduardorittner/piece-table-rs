use std::{collections::VecDeque, fmt::Display, ops::Add};

use crate::interface::EditableText;

pub mod interface;
pub mod baseline;

#[derive(Debug)]
pub struct PieceTable {
    original: String,
    added: String,
    nodes: Vec<Node>,
    history: History,
}

#[derive(Debug, Clone, Copy)]
struct Node {
    kind: NodeKind,
    range: TextRange,
}

#[derive(Debug, Clone, Copy)]
enum NodeKind {
    Original,
    Added,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

/// Represents an insertion or deletion which can be reverted
///
/// An insert needs to remember where it was inserted (`offset`) and where the string contents are
/// stored (`buffer_offset`). Note that since `original` is immutable, we know that any insertions
/// have their content stored in the `added` buffer.
#[derive(Debug, Clone, Copy)]
enum Edit {
    Insert {
        offset: usize,
        buffer_offset: usize,
        len: usize,
    },
    Delete {
        range: TextRange,
        buffer_offset: usize,
    },
}

#[derive(Debug)]
struct History {
    undo: VecDeque<Edit>,
    redo: VecDeque<Edit>,
    depth: usize,
    max_depth: usize,
}

impl Default for History {
    fn default() -> Self {
        History::new(1024)
    }
}

impl History {
    fn new(max_depth: usize) -> Self {
        Self {
            undo: VecDeque::with_capacity(max_depth),
            redo: VecDeque::with_capacity(max_depth),
            depth: 0,
            max_depth,
        }
    }
}

impl Node {
    fn is_in_range(&self, byte_idx: usize, range: &TextRange) -> bool {
        byte_idx <= range.end && byte_idx + self.range.len() >= range.start
    }
}

impl Add<TextRange> for TextRange {
    type Output = TextRange;

    fn add(self, rhs: TextRange) -> Self::Output {
        TextRange {
            start: self.start + rhs.start,
            end: self.end + rhs.end,
        }
    }
}

impl PieceTable {
    pub fn new(string: String) -> Self {
        let mut nodes = Vec::new();
        nodes.push(Node {
            kind: NodeKind::Original,
            range: TextRange {
                start: 0,
                end: string.as_bytes().len(),
            },
        });

        PieceTable {
            original: string,
            added: String::new(),
            nodes,
            history: History::default(),
        }
    }

    pub fn insert(&mut self, data: &str, offset: usize) {
        // The node we'll insert
        let node_range = TextRange {
            start: self.added.as_bytes().len(),
            end: self.added.as_bytes().len() + data.as_bytes().len(),
        };
        self.added.extend(data.chars());
        let node = Node {
            kind: NodeKind::Added,
            range: node_range,
        };

        if let Some((node_idx, node_pos)) = self.find_node(offset) {
            let insert_idx = if self.split_node(node_idx, offset - node_pos) {
                node_idx + 1
            } else {
                node_idx
            };

            self.nodes.insert(insert_idx, node);
        } else {
            self.nodes.push(node);
        }
    }

    pub fn delete(&mut self, range: TextRange) {
        if let Some((start, byte_idx)) = self.find_node(range.start) {
            self.delete_complete_nodes(start, byte_idx, range);

            dbg!(&self);
            if let Some(node) = self.nodes.get(start) {
                assert_eq!(self.find_node(range.start), Some((start, byte_idx)));

                if byte_idx <= range.start && range.end <= byte_idx + node.range.len() {
                    if self.split_node(start, range.end - byte_idx) {
                        self.nodes.get_mut(start).unwrap().range.end -= range.end - range.start;
                    }
                }
            }
        } else {
            unreachable!()
        }
    }

    /// Deletes all nodes which are entirely contained within the specified range
    ///
    /// Any nodes which are only partially in the range will not be deleted and must be dealt with
    /// by the caller
    fn delete_complete_nodes(&mut self, idx: usize, byte_idx: usize, range: TextRange) {
        let mut byte_idx = byte_idx;

        while byte_idx < range.end {
            let node = *self.nodes.get(idx).unwrap();

            if byte_idx >= range.start && byte_idx + node.range.len() <= range.end {
                self.nodes.remove(idx);
            }

            byte_idx += node.range.len();
        }
    }

    /// Replaces a range with the given string
    ///
    /// In order to preserve undo/redo history this is implemented as a delete + insertion, instead
    /// of a direct replacement.
    pub fn replace(&mut self, data: &str, offset: usize) {
        self.delete(TextRange {
            start: offset,
            end: offset + data.len(),
        });
        self.insert(data, offset);
    }

    pub fn undo(&mut self, count: usize) {}

    pub fn redo(&mut self, count: usize) {}

    pub fn clear_history(&mut self) {}

    fn find_node(&self, offset: usize) -> Option<(usize, usize)> {
        let mut byte_idx = 0;

        for (idx, node) in self.nodes.iter().enumerate() {
            if byte_idx + node.range.len() > offset {
                return Some((idx, byte_idx));
            }
            byte_idx += node.range.len();
        }

        None
    }

    /// Returns `true` if the node was split, `false` otherwise
    ///
    /// There are 3 cases:
    /// 1. `offset == 0`:
    ///     In this case, nothing happens since there's nothing to do
    ///
    /// 2. `offset >= node.range.len()`:
    ///     Same as case 1
    ///
    /// 3. `offset != 0 && offset < range.len()`:
    ///     The node is split
    fn split_node(&mut self, piece_idx: usize, offset: usize) -> bool {
        let first_node = self.nodes.get_mut(piece_idx).unwrap();

        if offset == 0 {
            false
        } else if first_node.range.len() > offset {
            let mut second_node = first_node.clone();
            first_node.range.end -= first_node.range.len() - offset;
            second_node.range.start += offset;
            self.nodes.insert(piece_idx + 1, second_node);
            true
        } else {
            first_node.range.end -= offset - 1;
            false
        }
    }
}

impl EditableText for PieceTable {
    fn new(string: String) -> Self {
        PieceTable::new(string)
    }

    fn insert(&mut self, data: &str, offset: usize) {
        self.insert(data, offset)
    }

    fn delete(&mut self, range: TextRange) {
        self.delete(range)
    }
}

impl Display for PieceTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in &self.nodes {
            write!(
                f,
                "{}",
                match node.kind {
                    NodeKind::Original => unsafe {
                        self.original
                            .get_unchecked(node.range.start..node.range.end)
                    },
                    NodeKind::Added => unsafe {
                        self.added.get_unchecked(node.range.start..node.range.end)
                    },
                }
            )?;
        }
        Ok(())
    }
}

impl TextRange {
    fn len(&self) -> usize {
        self.end - self.start
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn display() {
        let string = "hello!";
        let piece_table = PieceTable::new(string.to_string());

        assert_eq!(string, piece_table.to_string());
    }

    #[test]
    fn insert_once() {
        let original = "hello, ";
        let mut piece_table = PieceTable::new(original.to_string());

        let added = "world!";
        piece_table.insert(added, original.len());

        assert_eq!(original.to_owned() + added, piece_table.to_string());
    }

    #[test]
    fn insert_twice() {
        let original = "hello, ";
        let added = "world";
        let second = "!";

        let mut piece_table = PieceTable::new(original.to_string());
        piece_table.insert(added, original.len());
        piece_table.insert(second, original.len() + added.len());

        assert_eq!(
            original.to_owned() + added + second,
            piece_table.to_string()
        );
    }

    #[test]
    fn insert_once_middle() {
        let original = "hello!";
        let added = ", world";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.insert(added, 5);

        assert_eq!(piece_table.nodes.len(), 3);
        assert_eq!("hello, world!", piece_table.to_string());
    }

    #[test]
    fn insert_once_end() {
        let original = "hello";
        let added = ", world!";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.insert(added, 5);

        assert_eq!(piece_table.nodes.len(), 2);
        assert_eq!("hello, world!", piece_table.to_string());
    }

    #[test]
    fn insert_once_start() {
        let original = "bc";
        let added = "a";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.insert(added, 0);

        assert_eq!(piece_table.nodes.len(), 2);
        assert_eq!("abc", piece_table.to_string());
    }

    #[test]
    fn delete_original_whole() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.delete(TextRange { start: 0, end: 2 });

        assert_eq!("", piece_table.to_string());
    }

    #[test]
    fn delete_original_half() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.delete(TextRange { start: 0, end: 1 });

        assert_eq!("b", piece_table.to_string());
    }

    #[test]
    fn delete_original_second_half() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.delete(TextRange { start: 1, end: 2 });

        assert_eq!("a", piece_table.to_string());
    }

    #[test]
    fn delete_original_middle() {
        let original = "abc";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.delete(TextRange { start: 1, end: 2 });

        assert_eq!("ac", piece_table.to_string());
    }

    #[test]
    fn delete_original_two_times() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.delete(TextRange { start: 0, end: 1 });
        piece_table.delete(TextRange { start: 0, end: 1 });

        assert_eq!("", piece_table.to_string());
    }

    #[test]
    fn add_then_delete() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.delete(TextRange { start: 0, end: 1 });
        piece_table.delete(TextRange { start: 0, end: 1 });
        piece_table.insert("ab", 0);

        assert_eq!("ab", piece_table.to_string());
    }

    #[test]
    fn add_at_start() {
        let original = "world!";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.insert("hello", 0);
        piece_table.insert(", ", 5);

        assert_eq!("hello, world!", piece_table.to_string());
        assert_eq!(3, piece_table.nodes.len());
    }

    #[test]
    fn add_delete_add() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.insert("c", 2);
        piece_table.delete(TextRange { start: 0, end: 3 });
        piece_table.insert("ab", 0);

        assert_eq!("ab", piece_table.to_string());
    }

    #[test]
    fn replace() {
        let original = "hello, hello!";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.replace("world", 7);

        assert_eq!("hello, world!", piece_table.to_string());
    }
}

#[cfg(test)]
mod property_tests {
    use crate::baseline::Baseline;
    use crate::interface::EditableText;
    use crate::PieceTable;
    use crate::TextRange;
    use proptest::prelude::*;

    #[derive(Debug, Clone)]
    enum Op {
        Insert(String, usize),
        Delete(usize, usize),
    }

    fn do_op<T: EditableText + std::fmt::Display>(
        doc: &mut T,
        op: &Op,
        current_string: &mut String,
    ) {
        match op {
            Op::Insert(text, offset) => {
                let mut offset = *offset;
                if offset > current_string.len() {
                    offset = current_string.len();
                }
                doc.insert(text, offset);
                current_string.insert_str(offset, text);
            }
            Op::Delete(start, end) => {
                let mut start = *start;
                let mut end = *end;
                if start > current_string.len() {
                    start = current_string.len();
                }
                if end > current_string.len() {
                    end = current_string.len();
                }
                if start > end {
                    std::mem::swap(&mut start, &mut end);
                }
                doc.delete(TextRange { start, end });
                current_string.replace_range(start..end, "");
            }
        }
    }

    proptest! {
        #[test]
        fn compare_implementations(initial_text: String, ops: Vec<Op>) {
            let mut piece_table = PieceTable::new(initial_text.clone());
            let mut baseline = Baseline::new(initial_text.clone());
            let mut current_string = initial_text.clone();

            for op in ops {
                do_op(&mut piece_table, &op, &mut current_string);
                do_op(&mut baseline, &op, &mut current_string);

                prop_assert_eq!(piece_table.to_string(), baseline.to_string());
            }
        }
    }

    impl Arbitrary for Op {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                (any::<String>(), any::<usize>()).prop_map(|(s, i)| Op::Insert(s, i)),
                (any::<usize>(), any::<usize>()).prop_map(|(i, j)| Op::Delete(i, j)),
            ]
            .boxed()
        }
    }
}