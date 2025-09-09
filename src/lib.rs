use std::{fmt::Display, ops::Add, slice::SliceIndex};

#[derive(Debug)]
struct PieceTable {
    original: String,
    added: String,
    nodes: Vec<Node>,
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
struct TextRange {
    start: usize,
    end: usize,
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
    // Public API
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

        let mut byte_idx = 0;
        let mut node_idx = 0;
        for (idx, node) in self.nodes.iter().enumerate() {
            if byte_idx + node.range.len() > offset {
                node_idx = idx;
            }
            byte_idx += node.range.len();
        }

        // To update
        let original_node = self.nodes.get(node_idx).unwrap().clone();

        // Update original node
        self.nodes.get_mut(node_idx).unwrap().range.end = offset;

        // Insert new node
        self.nodes.insert(node_idx + 1, node);

        // Insert second half of original node
        let mut second_half = Node {
            kind: original_node.kind,
            range: original_node.range,
        };

        second_half.range.start += offset - original_node.range.start;

        self.nodes.insert(node_idx + 2, second_half);
    }

    pub fn insert_end(&mut self, data: &str) {
        let range = TextRange {
            start: self.added.as_bytes().len(),
            end: self.added.as_bytes().len() + data.as_bytes().len(),
        };
        self.nodes.push(Node {
            kind: NodeKind::Added,
            range,
        });
        self.added.extend(data.chars());
    }

    pub fn delete(&mut self, range: TextRange) {
        let mut byte_idx = 0;
        let mut start_byte_idx = 0;

        // Since a delete operation can operate on more that one node, we find the range of nodes
        // which are inside that range.
        let mut start_idx = 0;
        let mut end_idx = None;
        for (idx, node) in self.nodes.iter().enumerate() {
            if byte_idx + node.range.len() > range.start {
                start_idx = idx;
                start_byte_idx = byte_idx;
            }

            if byte_idx + node.range.len() >= range.end && end_idx == None {
                end_idx = Some(idx);
            }

            byte_idx += node.range.len();
        }

        if let Some(end_idx) = end_idx {
            while start_idx <= end_idx {
                if self.nodes.get(start_idx).unwrap().range.end < range.end {
                    println!(
                        "{} {}",
                        self.nodes.get(start_idx).unwrap().range.end,
                        range.end
                    );
                    self.nodes.remove(start_idx);
                } else {
                    let node = self.nodes.get_mut(start_idx).unwrap();

                    if node.range.start == range.start {
                        // update range.end
                        node.range.start = range.end;
                    } else {
                        node.range.end += range.end - start_byte_idx - 1;
                    }
                }

                start_idx += 1;
            }
        } else {
            // TODO is there any meaningful case where end_idx = None?
            unreachable!()
        }
    }

    pub fn replace(&mut self, data: &str, offset: TextRange) {}

    pub fn undo(&mut self, count: usize) {}

    pub fn redo(&mut self, count: usize) {}

    pub fn clear_history(&mut self) {}
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
        piece_table.insert_end("world!");

        assert_eq!(original.to_owned() + added, piece_table.to_string());
    }

    #[test]
    fn insert_twice() {
        let original = "hello, ";
        let added = "world";
        let second = "!";

        let mut piece_table = PieceTable::new(original.to_string());
        piece_table.insert_end(added);
        piece_table.insert_end(second);

        assert_eq!(
            original.to_owned() + added + second,
            piece_table.to_string()
        );
    }

    #[test]
    fn insert_once_in_the_middle() {
        let original = "hello!";
        let added = ", world";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.insert(added, 5);

        assert_eq!("hello, world!", piece_table.to_string());
    }

    #[test]
    fn insert_once_in_the_end() {
        let original = "hello";
        let added = ", world!";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.insert(added, 5);

        assert_eq!("hello, world!", piece_table.to_string());
    }

    #[test]
    fn insert_once_start() {
        let original = "bc";
        let added = "a";

        let mut piece_table = PieceTable::new(original.to_string());

        piece_table.insert(added, 0);

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
}
