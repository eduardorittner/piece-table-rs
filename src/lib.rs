use std::{collections::VecDeque, fmt::Display, ops::Range};

use crate::interface::EditableText;

pub mod baseline;
pub mod interface;

#[derive(Debug, Clone)]
pub struct PieceTable<'a> {
    original: &'a str,
    added: String,
    nodes: VecDeque<Node>,
}

#[derive(Debug, Clone)]
struct Node {
    kind: NodeKind,
    range: Range<usize>,
}

#[derive(Debug, Clone, Copy)]
enum NodeKind {
    Original,
    Added,
}

/// An immutable view into a PieceTable
///
/// A slice lives as long as its corresponding PieceTable, and will not be affected by any changes
/// made to the PieceTable after the slice was created.
#[derive(Debug)]
pub struct PTableSlice<'ptable> {
    nodes: Vec<Node>,
    original: *const str,
    added: *const String, // SAFETY: must never be mutated, only read
    _marker: std::marker::PhantomData<&'ptable ()>,
}

impl<'ptable> PTableSlice<'ptable> {
    /// Get the length of the text in bytes
    pub fn len(&self) -> usize {
        self.nodes.iter().map(|n| n.range.len()).sum()
    }

    /// Create a sub-slice of this slice
    ///
    /// The range used to create the slice is in reference to the `PTableSlice`, not the
    /// `PieceTable`.
    pub fn slice(&self, range: Range<usize>) -> Option<PTableSlice<'ptable>> {
        let mut new_nodes = Vec::new();
        let mut byte_idx = 0;
        let mut remaining = range.end - range.start;
        let start_offset = range.start;

        for node in &self.nodes {
            let node_len = node.range.len();

            if byte_idx + node_len > start_offset {
                let node_start = if byte_idx < start_offset {
                    start_offset - byte_idx
                } else {
                    0
                };

                let node_end = if remaining < node_len - node_start {
                    node_start + remaining
                } else {
                    node_len
                };

                if node_start < node_end {
                    let mut new_node = node.clone();
                    new_node.range.start += node_start;
                    new_node.range.end = new_node.range.start + (node_end - node_start);
                    new_nodes.push(new_node);
                    remaining -= node_end - node_start;
                }
            }

            byte_idx += node_len;
            if remaining == 0 {
                break;
            }
        }

        if new_nodes.is_empty() {
            return None;
        }

        Some(PTableSlice {
            nodes: new_nodes,
            original: self.original,
            added: self.added,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<'ptable> From<&PTableSlice<'ptable>> for String {
    fn from(value: &PTableSlice<'ptable>) -> Self {
        let mut result = String::new();
        for node in &value.nodes {
            match node.kind {
                // SAFETY: Since value still valid, then its corresponding `PieceTable` is still
                // valid and thus `added` is still valid.
                NodeKind::Original => unsafe {
                    result.push_str(&(&*value.original)[node.range.clone()])
                },
                NodeKind::Added => unsafe {
                    result.push_str(str::from_utf8_unchecked(
                        &(*value.added).as_bytes()[node.range.clone()],
                    ));
                },
            }
        }
        result
    }
}

impl<'ptable> Display for PTableSlice<'ptable> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl<'ptable> PieceTable<'ptable> {
    /// Create an immutable slice of the current state
    pub fn create_slice(&self) -> PTableSlice<'_> {
        PTableSlice {
            nodes: self.nodes.iter().cloned().collect(),
            original: self.original as *const str,
            added: &self.added as *const _,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn new(string: &'ptable str) -> Self {
        let mut nodes = VecDeque::new();
        nodes.push_back(Node {
            kind: NodeKind::Original,
            range: 0..string.as_bytes().len(),
        });

        PieceTable {
            original: string,
            added: String::new(),
            nodes,
        }
    }

    pub fn len(&self) -> usize {
        self.to_string().len()
    }

    pub fn insert_char(&mut self, offset: usize, c: char) {
        // The node we'll insert
        let node_range = self.added.as_bytes().len()..self.added.as_bytes().len() + c.len_utf8();
        self.added.push(c);
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
            self.nodes.push_back(node);
        }
    }

    pub fn insert(&mut self, data: &str, offset: usize) {
        // The node we'll insert
        let node_range =
            self.added.as_bytes().len()..self.added.as_bytes().len() + data.as_bytes().len();
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
            self.nodes.push_back(node);
        }
    }

    pub fn delete(&mut self, range: Range<usize>) {
        if let Some((start, byte_idx)) = self.find_node(range.start) {
            self.delete_complete_nodes(start, byte_idx, &range);

            if let Some(node) = self.nodes.get(start) {
                if byte_idx <= range.start && range.end <= byte_idx + node.range.len() {
                    if self.split_node(start, range.end - byte_idx) {
                        self.nodes.get_mut(start).unwrap().range.end -= range.end - range.start;
                    }
                }
            }
        }
    }

    /// Deletes all nodes which are entirely contained within the specified range
    ///
    /// Any nodes which are only partially in the range will not be deleted and must be dealt with
    /// by the caller.
    fn delete_complete_nodes(&mut self, idx: usize, byte_idx: usize, range: &Range<usize>) {
        let mut start = None;
        let mut end = None;
        let mut byte_idx = byte_idx;

        for i in idx..self.nodes.len() {
            let node = &self.nodes[i];
            let node_len = node.range.len();

            if byte_idx >= range.start && byte_idx + node_len <= range.end {
                if start.is_none() {
                    start = Some(i);
                }
                end = Some(i);
            }

            byte_idx += node_len;
            if byte_idx >= range.end {
                break;
            }
        }

        if let (Some(first), Some(last)) = (start, end) {
            self.nodes.drain(first..=last);
        }
    }

    /// Replaces a range with the given string
    ///
    /// In order to preserve undo/redo history this is implemented as a delete + insertion, instead
    /// of a direct replacement.
    pub fn replace(&mut self, data: &str, offset: usize) {
        self.delete(offset..offset + data.len());
        self.insert(data, offset);
    }

    pub fn undo(&mut self, _count: usize) {}

    pub fn redo(&mut self, _count: usize) {}

    pub fn clear_history(&mut self) {}

    /// Create a slice of the piece table for the given range
    pub fn slice(&self, range: Range<usize>) -> PTableSlice<'ptable> {
        let mut nodes = Vec::new();
        let mut byte_idx = 0;
        let mut found_start = false;

        for node in &self.nodes {
            let node_len = node.range.len();
            let node_start = byte_idx;
            let node_end = byte_idx + node_len;

            // Check if node intersects with the range
            if node_end > range.start && node_start < range.end {
                if found_start && node_end < range.end {
                    // Middle node - copy as-is
                    nodes.push(node.clone());
                }
                if !found_start {
                    // First node in range - may need to adjust start
                    let start_offset = range.start.saturating_sub(node_start);
                    let mut new_node = node.clone();
                    new_node.range.start += start_offset;

                    if node_end >= range.end {
                        new_node.range.end -= node_end - range.end;
                        nodes.push(new_node);
                        found_start = true;
                    } else {
                        nodes.push(new_node);
                        found_start = true;
                    }
                } else if node_end >= range.end {
                    // Last node in range - may need to adjust end
                    let end_offset = node_end - range.end;
                    let mut new_node = node.clone();
                    new_node.range.end -= end_offset;
                    nodes.push(new_node);
                    break;
                }
            }

            byte_idx = node_end;
        }

        PTableSlice {
            nodes,
            original: self.original as *const str,
            added: &self.added as *const _,
            _marker: std::marker::PhantomData,
        }
    }

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

impl<'a> EditableText<'a> for PieceTable<'a> {
    fn new(string: &'a str) -> Self {
        PieceTable::new(string)
    }

    fn insert(&mut self, data: &str, offset: usize) {
        self.insert(data, offset)
    }

    fn delete(&mut self, range: Range<usize>) {
        self.delete(range)
    }
}

impl<'a> Display for PieceTable<'a> {
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

impl<'a> From<&'a str> for PieceTable<'a> {
    fn from(s: &'a str) -> Self {
        PieceTable::new(s)
    }
}

impl<'a> From<PieceTable<'a>> for String {
    fn from(p: PieceTable<'a>) -> Self {
        p.to_string()
    }
}

impl<'a> PartialEq for PieceTable<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl<'a> Eq for PieceTable<'a> {}

impl<'a> PartialEq<String> for PieceTable<'a> {
    fn eq(&self, other: &String) -> bool {
        &self.to_string() == other
    }
}

impl<'a> PartialEq<&str> for PieceTable<'a> {
    fn eq(&self, other: &&str) -> bool {
        &self.to_string() == other
    }
}

impl<'a> PartialOrd for PieceTable<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.to_string().cmp(&other.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn display() {
        let string = "hello!";
        let piece_table = PieceTable::new(string);

        assert_eq!(string, piece_table.to_string());
    }

    #[test]
    fn insert_once() {
        let original = "hello, ";
        let mut piece_table = PieceTable::new(original);

        let added = "world!";
        piece_table.insert(added, original.len());

        assert_eq!(original.to_owned() + added, piece_table.to_string());
    }

    #[test]
    fn insert_twice() {
        let original = "hello, ";
        let added = "world";
        let second = "!";

        let mut piece_table = PieceTable::new(original);
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

        let mut piece_table = PieceTable::new(original);

        piece_table.insert(added, 5);

        assert_eq!(piece_table.nodes.len(), 3);
        assert_eq!("hello, world!", piece_table.to_string());
    }

    #[test]
    fn insert_once_end() {
        let original = "hello";
        let added = ", world!";

        let mut piece_table = PieceTable::new(original);

        piece_table.insert(added, 5);

        assert_eq!(piece_table.nodes.len(), 2);
        assert_eq!("hello, world!", piece_table.to_string());
    }

    #[test]
    fn insert_once_start() {
        let original = "bc";
        let added = "a";

        let mut piece_table = PieceTable::new(original);

        piece_table.insert(added, 0);

        assert_eq!(piece_table.nodes.len(), 2);
        assert_eq!("abc", piece_table.to_string());
    }

    #[test]
    fn delete_original_whole() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original);

        piece_table.delete(0..2);

        assert_eq!("", piece_table.to_string());
    }

    #[test]
    fn delete_original_half() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original);

        piece_table.delete(0..1);

        assert_eq!("b", piece_table.to_string());
    }

    #[test]
    fn delete_original_second_half() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original);

        piece_table.delete(1..2);

        assert_eq!("a", piece_table.to_string());
    }

    #[test]
    fn delete_original_middle() {
        let original = "abc";

        let mut piece_table = PieceTable::new(original);

        piece_table.delete(1..2);

        assert_eq!("ac", piece_table.to_string());
    }

    #[test]
    fn delete_original_two_times() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original);

        piece_table.delete(0..1);
        piece_table.delete(0..1);

        assert_eq!("", piece_table.to_string());
    }

    #[test]
    fn add_then_delete() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original);

        piece_table.delete(0..1);
        piece_table.delete(0..1);
        piece_table.insert("ab", 0);

        assert_eq!("ab", piece_table.to_string());
    }

    #[test]
    fn add_at_start() {
        let original = "world!";

        let mut piece_table = PieceTable::new(original);

        piece_table.insert("hello", 0);
        piece_table.insert(", ", 5);

        assert_eq!("hello, world!", piece_table.to_string());
        assert_eq!(3, piece_table.nodes.len());
    }

    #[test]
    fn add_delete_add() {
        let original = "ab";

        let mut piece_table = PieceTable::new(original);

        piece_table.insert("c", 2);
        piece_table.delete(0..3);
        piece_table.insert("ab", 0);

        assert_eq!("ab", piece_table.to_string());
    }

    #[test]
    fn replace() {
        let original = "hello, hello!";

        let mut piece_table = PieceTable::new(original);

        piece_table.replace("world", 7);

        assert_eq!("hello, world!", piece_table.to_string());
    }

    #[test]
    fn eq_same_piecetables() {
        let text = "test";
        let pt1 = PieceTable::new(text);
        let pt2 = PieceTable::new(text);
        assert_eq!(pt1, pt2);
    }

    #[test]
    fn eq_different_piecetables() {
        let pt1 = PieceTable::new("hello");
        let pt2 = PieceTable::new("world");
        assert_ne!(pt1, pt2);
    }

    #[test]
    fn eq_piecetable_and_string() {
        let text = "test";
        let pt = PieceTable::new(text);
        assert_eq!(pt, text.to_string());
    }

    #[test]
    fn ne_piecetable_and_string() {
        let pt = PieceTable::new("hello");
        assert_ne!(pt, "world".to_string());
    }

    #[test]
    fn eq_piecetable_and_str() {
        let text = "test";
        let pt = PieceTable::new(text);
        assert_eq!(pt, text);
    }

    #[test]
    fn ne_piecetable_and_str() {
        let pt = PieceTable::new("hello");
        assert_ne!(pt, "world");
    }

    #[test]
    fn partial_ord_less() {
        let pt1 = PieceTable::new("apple");
        let pt2 = PieceTable::new("banana");
        assert!(pt1 < pt2);
    }

    #[test]
    fn partial_ord_greater() {
        let pt1 = PieceTable::new("banana");
        let pt2 = PieceTable::new("apple");
        assert!(pt1 > pt2);
    }

    #[test]
    fn partial_ord_equal() {
        let pt1 = PieceTable::new("apple");
        let pt2 = PieceTable::new("apple");
        assert!(pt1 <= pt2);
        assert!(pt1 >= pt2);
    }
}

#[cfg(test)]
mod ptable_slice_tests {
    use std::iter::repeat_n;

    use super::*;

    #[test]
    fn create_slice() {
        let table = PieceTable::new("hello");
        let slice = table.create_slice();
        assert_eq!(slice.to_string(), "hello");
    }

    #[test]
    fn slice_immutability() {
        let mut table = PieceTable::new("hello");
        {
            let slice = table.create_slice();
            assert_eq!(slice.to_string(), "hello");
        }
        table.insert(" world", 5);
        assert_eq!(table.to_string(), "hello world");
    }

    #[test]
    fn slice_conversion() {
        let table = PieceTable::new("test");
        let slice = table.create_slice();

        assert_eq!(slice.to_string(), "test");
    }

    #[test]
    fn slice_range() {
        let mut table = PieceTable::new("hello world");
        table.insert(" beautiful", 5);
        let slice = table.slice(6..16);

        assert_eq!(slice.len(), 10);
        assert_eq!(slice.to_string(), "beautiful ");
    }

    #[test]
    fn slice_contiguous_range() {
        let table = PieceTable::new("hello world");
        let slice = table.slice(0..5);
        assert_eq!(slice.to_string(), "hello");

        assert_eq!(slice.len(), 5);
    }

    #[test]
    fn multiple_slices() {
        let table = PieceTable::new("hello");
        let slice1 = table.create_slice();
        let slice2 = table.create_slice();
        let slice3 = table.slice(0..5);

        assert_eq!(slice1.to_string(), "hello");
        assert_eq!(slice2.to_string(), "hello");
        assert_eq!(slice3.to_string(), "hello");
    }

    #[test]
    fn nested_slices() {
        let table = PieceTable::new("hello world");
        let outer = table.create_slice();
        let inner = table.slice(0..5);

        assert_eq!(outer.to_string(), "hello world");
        assert_eq!(inner.to_string(), "hello");
    }

    #[test]
    fn empty_slice() {
        let table = PieceTable::new("");
        let slice = table.create_slice();
        assert_eq!(slice.len(), 0);
        assert_eq!(slice.to_string(), "");
    }

    #[test]
    fn slice_of_slice() {
        let table = PieceTable::new("hello world");
        let slice1 = table.slice(0..5);
        let slice2 = slice1.slice(1..4).unwrap();

        assert_eq!(slice1.to_string(), "hello");
        assert_eq!(slice2.to_string(), "ell");
    }

    #[test]
    fn slice_of_slice_of_slice() {
        let table = PieceTable::new("hello world");
        let slice1 = table.slice(0..11);
        let slice2 = slice1.slice(0..5).unwrap();
        let slice3 = slice2.slice(0..4).unwrap();

        assert_eq!(slice1.to_string(), "hello world");
        assert_eq!(slice2.to_string(), "hello");
        assert_eq!(slice3.to_string(), "hell");
    }

    #[test]
    fn slice_after_modifing() {
        let mut table = PieceTable::new("hello world");
        let slice = table.slice(0..11);

        table.delete(0..11);

        assert_eq!(slice.to_string(), "hello world");
    }

    #[test]
    fn slice_after_reallocation() {
        // NOTE this test is trying to test that even if the underlying data for `added` is
        // reallocated the slice is still valid
        let mut table = PieceTable::new("hello world");
        table.delete(0..11);
        table.insert("helloworld!", 0);
        table.insert(", ", 5);

        let slice = table.slice(0..13);

        let string: String = repeat_n('a', 1024).collect();
        table.insert(&string, 5);

        assert_eq!(slice.to_string(), "hello, world!");
    }
}

#[cfg(test)]
mod property_tests {
    use crate::PieceTable;
    use crate::baseline::Baseline;
    use crate::interface::EditableText;
    use proptest::prelude::*;

    #[derive(Debug, Clone)]
    enum Op {
        Insert(String, usize),
        Delete(usize, usize),
    }

    fn do_op<'a, T: EditableText<'a> + std::fmt::Display>(
        doc: &mut T,
        op: &Op,
        string_before_op: &String,
    ) {
        match op {
            Op::Insert(text, offset) => {
                let mut offset = *offset;
                if offset > string_before_op.len() {
                    offset = string_before_op.len();
                }
                while !string_before_op.is_char_boundary(offset) {
                    offset = offset.saturating_sub(1);
                }
                doc.insert(text, offset);
            }
            Op::Delete(start, end) => {
                let mut start = *start;
                let mut end = *end;
                if start > string_before_op.len() {
                    start = string_before_op.len();
                }
                if end > string_before_op.len() {
                    end = string_before_op.len();
                }
                if start > end {
                    std::mem::swap(&mut start, &mut end);
                }

                while !string_before_op.is_char_boundary(start) {
                    start = start.saturating_sub(1);
                }
                while !string_before_op.is_char_boundary(end) {
                    end = end.saturating_sub(1);
                }
                doc.delete(start..end);
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

    proptest! {
        #[test]
        fn compare_implementations(initial_text: String, ops: Vec<Op>) {
            let mut piece_table = PieceTable::new(&initial_text);
            let mut baseline = Baseline::new(&initial_text);

            for op in ops {
                let s = piece_table.to_string();
                do_op(&mut piece_table, &op, &s);
                do_op(&mut baseline, &op, &s);

                prop_assert_eq!(baseline.to_string(), piece_table.to_string());
            }
        }
    }
}
