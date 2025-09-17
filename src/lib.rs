use std::{collections::VecDeque, fmt::Display, ops::Range};

use crate::interface::EditableText;

pub mod baseline;
pub mod interface;

/// A piece table data structure for efficient string manipulation.
///
/// The `PieceTable` is designed for scenarios requiring frequent insertions and deletions,
/// such as text editors. It achieves efficiency by maintaining the original text immutable
/// and storing all modifications (insertions) in a separate, append-only buffer. The logical
/// text is represented as a sequence of "pieces" (or spans), where each piece is a reference
/// to either a segment of the original text or a segment of the added text buffer.
///
/// This approach avoids costly copying of large portions of text during edits. Instead,
/// operations like insert and delete are performed by adding, removing, or adjusting these
/// pieces. This makes the piece table particularly effective for applications with an
/// initial large body of text that undergoes many incremental changes.
///
/// # Invariants
/// - The `original` string reference is immutable and must outlive the `PieceTable`.
/// - The `added` string buffer is append-only; text is never removed or modified from it,
///   only referenced by `Node`s.
/// - The sequence of `Node`s in `nodes` always represents the current, correct state of the
///   entire text. Concatenating the text from all nodes, in order, yields the full document.
#[derive(Debug, Clone)]
pub struct PieceTable<'a> {
    original: &'a str,
    added: String,
    nodes: VecDeque<Node>,
    len: usize,
}

/// Represents a continuous slice of text in one of the two buffers
#[derive(Debug, Clone)]
struct Node {
    kind: NodeKind,
    range: Range<usize>,
}

/// What buffer the data from this `Node` is stored in
#[derive(Debug, Clone, Copy)]
enum NodeKind {
    Original,
    Added,
}

/// An immutable view into a PieceTable.
///
/// A `PTableSlice` provides a snapshot of the `PieceTable`'s content at a specific point in time.
/// It lives as long as its corresponding `PieceTable` and will not be affected by any changes
/// made to the `PieceTable` after the slice was created. This makes it useful for operations
/// that require a stable view of the text, such as iteration or complex transformations.
#[derive(Debug)]
pub struct PTableSlice<'ptable> {
    nodes: Vec<Node>,
    original: *const str,
    added: *const String, // SAFETY: must never be mutated, only read
    _marker: std::marker::PhantomData<&'ptable ()>,
}

impl<'ptable> PieceTable<'ptable> {
    /// Creates a new `PieceTable` from an initial string slice.
    ///
    /// This is the primary constructor for the `PieceTable`. It initializes the table with the
    /// provided `string`, which becomes the "original" text. The entire original text is
    /// represented by a single `Node` of kind `Original`.
    ///
    /// The `PieceTable` takes a reference to the input string, so the original data must outlive
    /// the `PieceTable`. Any modifications (insertions) will be stored in a separate internal buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let initial_text = "This is the original text.";
    /// let pt = PieceTable::new(initial_text);
    /// assert_eq!(pt.to_string(), initial_text);
    /// ```
    pub fn new(string: &'ptable str) -> Self {
        let mut nodes = VecDeque::new();
        nodes.push_back(Node {
            kind: NodeKind::Original,
            range: 0..string.len(),
        });

        PieceTable {
            original: string,
            added: String::new(),
            nodes,
            len: string.len(),
        }
    }

    /// Returns the total length of the text in the `PieceTable`, in bytes.
    ///
    /// The length is in bytes, not characters. For multi-byte UTF-8 characters, the byte length
    /// will be greater than the character count.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let pt = PieceTable::new("hello");
    /// assert_eq!(pt.len(), 5);
    ///
    /// let mut pt = PieceTable::new("héllo"); // 'é' is 2 bytes
    /// assert_eq!(pt.len(), 6);
    /// ```
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.len, self.to_string().len());
        self.len
    }

    /// Checks if the `PieceTable` is empty.
    ///
    /// Returns `true` if the `PieceTable` contains no text, `false` otherwise.
    /// This is equivalent to checking if `len()` returns 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let mut pt = PieceTable::new("hello");
    /// assert!(!pt.is_empty());
    ///
    /// pt.delete(0..5);
    /// assert!(pt.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Inserts a single character at the specified byte offset.
    ///
    /// This method inserts the given character `c` into the text at the byte `offset`.
    /// The character is appended to the internal "added" buffer, and a new `Node` of kind `Added`
    /// is created to reference it. This new node is then inserted into the sequence of pieces
    /// at the correct logical position.
    ///
    /// If the `offset` falls within an existing piece, that piece may be split into two to
    /// accommodate the new character. If the `offset` is at the end of the text, the new
    /// character is appended.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let mut pt = PieceTable::new("helo");
    /// pt.insert_char(2, 'l'); // Insert 'l' at byte offset 2
    /// assert_eq!(pt.to_string(), "hello");
    ///
    /// let mut pt = PieceTable::new("world");
    /// pt.insert_char(0, ' '); // Insert space at the beginning
    /// assert_eq!(pt.to_string(), " world");
    /// ```
    pub fn insert_char(&mut self, offset: usize, c: char) {
        // The node we'll insert
        let node_range = self.added.len()..self.added.len() + c.len_utf8();
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

        self.len += c.len_utf8();
    }

    /// Inserts a string slice at the specified byte offset.
    ///
    /// This is a core method for modifying the `PieceTable`. It inserts the provided `data`
    /// string at the given byte `offset`. The `data` is appended to the internal "added" buffer,
    /// and a single new `Node` of kind `Added` is created to reference the entire inserted string.
    /// This new node is then spliced into the sequence of existing pieces.
    ///
    /// If the `offset` is within the bounds of an existing piece, that piece is split into two
    /// pieces, and the new piece (representing the inserted data) is placed between them.
    /// If the `offset` is at the end of the text, the new data is appended.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let mut pt = PieceTable::new("hello, !");
    /// pt.insert("world", 7);
    /// assert_eq!(pt.to_string(), "hello, world!");
    ///
    /// let mut pt = PieceTable::new("start");
    /// pt.insert("beginning ", 0);
    /// assert_eq!(pt.to_string(), "beginning start");
    /// ```
    pub fn insert(&mut self, data: &str, offset: usize) {
        // The node we'll insert
        let node_range = self.added.len()..self.added.len() + data.len();
        self.added.push_str(data);
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

        self.len += data.len();
    }

    /// Deletes a range of text specified by byte offsets.
    ///
    /// This method removes the text within the given `range` (inclusive of `range.start` and
    /// exclusive of `range.end`). The deletion process involves:
    /// 1. Identifying the `Node`(s) that overlap with the specified range.
    /// 2. Removing any `Node`s that are entirely contained within the range.
    /// 3. Adjusting the boundaries of `Node`s that are partially covered by the range. This may
    ///    involve splitting a `Node` if the deletion range starts or ends in its middle.
    ///
    /// Deletion never modifies any of the data in `original` or `added` buffers, only the nodes
    /// themselves.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let mut pt = PieceTable::new("hello, world!");
    /// pt.delete(5..7); // Delete ", "
    /// assert_eq!(pt.to_string(), "helloworld!");
    ///
    /// let mut pt = PieceTable::new("abcde");
    /// pt.delete(1..4); // Delete "bcd"
    /// assert_eq!(pt.to_string(), "ae");
    /// ```
    pub fn delete(&mut self, range: Range<usize>) {
        if let Some((start, byte_idx)) = self.find_node(range.start) {
            self.delete_complete_nodes(start, byte_idx, &range);

            if let Some(node) = self.nodes.get(start)
                && byte_idx <= range.start
                && range.end <= byte_idx + node.range.len()
                && self.split_node(start, range.end - byte_idx)
            {
                self.nodes.get_mut(start).unwrap().range.end -= range.end - range.start;
            }
        }

        self.len -= range.len();
    }

    /// Replaces a range of text with a new string.
    ///
    /// This method first deletes the text starting at `offset` up to`data.len()` bytes, and then
    /// inserts the new `data` at the same `offset`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let mut pt = PieceTable::new("hello, world!");
    /// // Replace "world" (7 bytes) with "cruel world"
    /// pt.replace("cruel world", 7);
    /// assert_eq!(pt.to_string(), "hello, cruel world!");
    /// ```
    pub fn replace(&mut self, data: &str, offset: usize) {
        let end = (offset + data.len()).min(self.len() - 1);
        self.delete(offset..end);
        self.insert(data, offset);
    }

    /// Creates an immutable snapshot of the `PieceTable`'s current state.
    ///
    /// This method captures the entire content of the `PieceTable` at the moment it is called
    /// and returns it as a `PTableSlice`. The returned slice is immutable and will not reflect
    /// any subsequent modifications (insertions, deletions) made to the original `PieceTable`.
    /// This is useful for operations that require a consistent, unchanging view of the text,
    /// such as for rendering, saving, or complex analysis.
    ///
    /// The lifetime of the returned slice is tied to the lifetime of the `PieceTable` itself,
    /// ensuring that the underlying text data remains valid while still alowing for mutation
    /// of the original `PieceTable`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let mut pt = PieceTable::new("hello");
    /// let snapshot = pt.create_slice();
    /// assert_eq!(snapshot.to_string(), "hello");
    ///
    /// pt.insert(" world", 5);
    /// assert_eq!(pt.to_string(), "hello world");
    /// assert_eq!(snapshot.to_string(), "hello");
    /// ```
    pub fn create_slice(&self) -> PTableSlice<'ptable> {
        PTableSlice {
            nodes: self.nodes.iter().cloned().collect(),
            original: self.original as *const str,
            added: &self.added as *const _,
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates an immutable slice of the `PieceTable` for a given byte range.
    ///
    /// This method returns a `PTableSlice` that represents a portion of the `PieceTable`'s
    /// current content, as defined by the `range`. The `range` is specified by byte offsets
    /// relative to the entire `PieceTable`.
    ///
    /// The resulting slice is constructed by finding the `Node`s that intersect with the
    /// specified range and creating new, adjusted `Node`s that fit precisely within the range.
    /// For example, if the range starts in the middle of an `Original` node, a new `Node`
    /// is created with an adjusted start index. The same logic applies to the end of the range.
    ///
    /// The returned `PTableSlice` is an immutable snapshot and will not be affected by
    /// subsequent modifications to the original `PieceTable`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let pt = PieceTable::new("hello world");
    /// let slice = pt.slice(0..5);
    /// assert_eq!(slice.to_string(), "hello");
    ///
    /// let mut pt = PieceTable::new("abc");
    /// pt.insert("def", 3);
    /// pt.insert("ghi", 6);
    /// let slice = pt.slice(1..7); // "bcdefg"
    /// assert_eq!(slice.to_string(), "bcdefg");
    /// ```
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

    pub fn byte(&self, at: usize) -> Option<u8> {
        if let Some((idx, byte_idx)) = self.find_node(at) {
            let offset = at - byte_idx;
            let node = &self.nodes[idx];

            let byte = match node.kind {
                NodeKind::Original => self.original.as_bytes()[node.range.start + offset],
                NodeKind::Added => self.added.as_bytes()[node.range.start + offset],
            };

            Some(byte)
        } else {
            None
        }
    }

    pub fn char(&self, at: usize) -> Option<char> {
        if let Some((idx, byte_idx)) = self.find_node(at) {
            let offset = at - byte_idx;
            let node = &self.nodes[idx];

            if let Some(byte) = match node.kind {
                NodeKind::Original => self.original[node.range.start + offset..].chars().next(),
                NodeKind::Added => self.added[node.range.start + offset..].chars().next(),
            } {
                Some(byte)
            } else {
                None
            }
        } else {
            None
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

    /// Internal helper method to find the node that contains the char at `offset`
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

    /// Tries to split a node and returns `true` if succeeded and `false` otherwise
    ///
    /// There are 3 cases:
    /// 1. `offset == 0`:
    ///    In this case, nothing happens since there's nothing to do
    ///
    /// 2. `offset >= node.range.len()`:
    ///    Same as case 1
    ///
    /// 3. `offset != 0 && offset < range.len()`:
    ///    The node is split
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
        *other == self.to_string()
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

impl<'ptable> PTableSlice<'ptable> {
    /// Returns the total length of the text in the slice, in bytes.
    ///
    /// This method iterates through all the nodes in the slice and sums up their individual lengths.
    /// The length is calculated based on the byte ranges of the underlying text pieces, not on the
    /// number of characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let pt = PieceTable::new("hello world");
    /// let slice = pt.slice(0..5);
    /// assert_eq!(slice.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.nodes.iter().map(|n| n.range.len()).sum()
    }

    /// Checks if the slice is empty.
    ///
    /// Returns `true` if the slice contains no text, `false` otherwise.
    /// This is equivalent to checking if `len()` returns 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let mut pt = PieceTable::new("hello");
    /// pt.delete(0..5);
    /// let slice = pt.create_slice();
    /// assert!(slice.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a sub-slice from this slice.
    ///
    /// This method allows you to create a new `PTableSlice` that represents a portion of the current slice.
    /// The `range` argument specifies the byte offsets within *this slice* (not the original `PieceTable`)
    /// that the new slice should cover.
    ///
    /// If the specified range is invalid (e.g., `start` > `end`, or the range is out of bounds),
    /// this method returns `None`. It also returns `None` if the resulting slice would be empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use piece_table::PieceTable;
    /// let pt = PieceTable::new("hello world");
    /// let slice = pt.slice(0..11); // "hello world"
    /// let sub_slice = slice.slice(0..5).unwrap(); // "hello"
    /// assert_eq!(sub_slice.to_string(), "hello");
    /// ```
    pub fn slice(&self, range: Range<usize>) -> Option<PTableSlice<'ptable>> {
        let mut new_nodes = Vec::new();
        let mut byte_idx = 0;
        let mut remaining = range.end - range.start;
        let start_offset = range.start;

        for node in &self.nodes {
            let node_len = node.range.len();

            if byte_idx + node_len > start_offset {
                let node_start = start_offset.saturating_sub(byte_idx);

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

    #[test]
    fn byte() {
        let pt = PieceTable::new("abcd");

        assert_eq!(Some('a' as u8), pt.byte(0));
        assert_eq!(Some('b' as u8), pt.byte(1));
        assert_eq!(Some('c' as u8), pt.byte(2));
        assert_eq!(Some('d' as u8), pt.byte(3));
        assert_eq!(None, pt.byte(4));
        assert_eq!(None, pt.byte(5));
        assert_eq!(None, pt.byte(usize::MAX));
    }

    #[test]
    fn byte_of_added() {
        let mut pt = PieceTable::new("abcd");
        pt.replace("hello!", 0);

        assert_eq!(Some('h' as u8), pt.byte(0));
        assert_eq!(Some('e' as u8), pt.byte(1));
        assert_eq!(Some('l' as u8), pt.byte(2));
        assert_eq!(Some('l' as u8), pt.byte(3));
        assert_eq!(Some('o' as u8), pt.byte(4));
        assert_eq!(Some('!' as u8), pt.byte(5));
    }

    #[test]
    fn char() {
        let pt = PieceTable::new("abcd");

        assert_eq!(Some('a'), pt.char(0));
        assert_eq!(Some('b'), pt.char(1));
        assert_eq!(Some('c'), pt.char(2));
        assert_eq!(Some('d'), pt.char(3));
        assert_eq!(None, pt.char(4));
    }

    #[test]
    fn char_of_added() {
        let mut pt = PieceTable::new("abcd");
        pt.replace("hello!", 0);

        assert_eq!(Some('h'), pt.char(0));
        assert_eq!(Some('e'), pt.char(1));
        assert_eq!(Some('l'), pt.char(2));
        assert_eq!(Some('l'), pt.char(3));
        assert_eq!(Some('o'), pt.char(4));
        assert_eq!(Some('!'), pt.char(5));
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
