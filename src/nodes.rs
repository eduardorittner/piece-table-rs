use std::{
    ops::{Add, Range, RangeBounds},
    simd::{num::SimdUint, usizex8, usizex64},
};

use crate::nodekind_vec::NodeKindVec;
#[cfg(all(feature = "simd", target_arch = "aarch64"))]
use crate::simd::{ByteChunk, Chunk};

/// What buffer the data from this `Node` is stored in
///
/// TODO: Add comment saying that like `Node` this shouldn't be used too often
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeKind {
    Original,
    Added,
}

/// Represents a continuous slice of text in one of the two buffers
///
/// This type is mostly ever used to add nodes onto a `Nodes` struct, but for most cases only the
/// required fields should be accessed directly from a `Nodes` instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Node {
    pub kind: NodeKind,
    pub start: usize,
    pub len: usize,
}

impl Node {
    pub(crate) fn new(kind: NodeKind, start: usize, len: usize) -> Node {
        Node { kind, start, len }
    }

    #[inline(always)]
    pub(crate) fn end(&self) -> usize {
        self.start + self.len
    }

    pub(crate) fn range(&self) -> Range<usize> {
        Range {
            start: self.start,
            end: self.end(),
        }
    }
}

/// Represents all nodes of a `PieceTable`
///
/// This is a SoA (Struct of Arrays) instead of an AoS (Array of Structs) in order to speed up the
/// most common operations, like finding the node for a given byte offset, etc.
#[derive(Debug, Clone)]
pub(crate) struct Nodes {
    pub(crate) kinds: NodeKindVec,
    pub(crate) starts: Vec<usize>,
    pub(crate) lens: Vec<usize>,
}

impl Nodes {
    pub(crate) fn new() -> Nodes {
        Nodes {
            kinds: NodeKindVec::new(),
            starts: Vec::new(),
            lens: Vec::new(),
        }
    }

    #[inline(always)]
    pub(crate) fn len(&self) -> usize {
        // Make sure that all lengths are the same
        debug_assert!(
            self.kinds.len() == self.starts.len() && self.starts.len() == self.lens.len()
        );

        self.starts.len()
    }

    #[inline(always)]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Updates node at `idx`
    ///
    /// This is kind of awkward since we can't have a reasonable `get_mut()` method on `Nodes`,
    /// since we'd need to return a reference to a `Node` that doesnt't actually exist.
    pub(crate) fn set(&mut self, idx: usize, node: Node) {
        if idx >= self.len() {
            panic!("Index out of bounds");
        }

        self.kinds.set(idx, node.kind);
        self.starts[idx] = node.start;
        self.lens[idx] = node.len;
    }

    pub(crate) fn push(&mut self, node: Node) {
        self.kinds.push_back(node.kind);
        self.starts.push(node.start);
        self.lens.push(node.len);
    }

    pub(crate) fn push_back(&mut self, node: Node) {
        self.push(node);
    }

    pub(crate) fn insert(&mut self, idx: usize, node: Node) {
        self.kinds.insert(idx, node.kind);
        self.starts.insert(idx, node.start);
        self.lens.insert(idx, node.len);
    }

    pub(crate) fn remove_range<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        use std::ops::Bound::*;

        let start = match range.start_bound() {
            Included(&s) => s,
            Excluded(&s) => s + 1,
            Unbounded => 0,
        };

        let end = match range.end_bound() {
            Included(&e) => e + 1,
            Excluded(&e) => e,
            Unbounded => self.len(),
        };

        if start < end && start < self.len() {
            let end = end.min(self.len());
            self.kinds.remove_range(start..end);
            self.starts.drain(start..end);
            self.lens.drain(start..end);
        }
    }

    pub(crate) fn get(&self, idx: usize) -> Option<Node> {
        if self.len() <= idx {
            return None;
        }

        unsafe {
            Some(Node::new(
                self.kinds.get_unchecked(idx),
                *self.starts.get_unchecked(idx),
                *self.lens.get_unchecked(idx),
            ))
        }
    }

    pub(crate) fn find_node(&self, offset: usize) -> Option<NodeHandle> {
        let (lanes, leftover) = self.lens.as_chunks();

        let (len, idx) = match find_node_64(lanes, offset) {
            NodeSearch::Found(NodeHandle { idx, text_idx }) => {
                return Some(NodeHandle { idx, text_idx });
            }
            NodeSearch::FirstRemainingNode(NodeHandle { idx, text_idx }) => (text_idx, idx),
        };

        let (lanes, leftover) = leftover.as_chunks();

        let (len, idx) = match find_node_8(lanes, offset) {
            NodeSearch::Found(NodeHandle { idx, text_idx }) => {
                return Some((NodeHandle { idx, text_idx }) + NodeHandle { text_idx: len, idx });
            }
            NodeSearch::FirstRemainingNode(NodeHandle { idx, text_idx }) => (text_idx, idx),
        };

        match find_node_1(leftover, offset) {
            NodeSearch::Found(NodeHandle { idx, text_idx }) => Some(NodeHandle { idx, text_idx }),
            NodeSearch::FirstRemainingNode(NodeHandle { idx, text_idx }) => None,
        }
    }
}

#[derive(Clone, Copy)]
/// The result of a `Node` search
enum NodeSearch {
    /// Found `Node`
    Found(NodeHandle),
    /// First `Node` which wasn't searched
    FirstRemainingNode(NodeHandle),
}

#[derive(Debug, Clone, Copy)]
/// A Handle to a `Node`
// TODO: I'd like to think of a better/consisten naming scheme? Not sure about this
pub struct NodeHandle {
    pub idx: usize,      // Index where node is stored in `Nodes`
    pub text_idx: usize, // Byte index of the first byte of the `Node` in the final text
}

impl Add<NodeHandle> for NodeHandle {
    type Output = NodeHandle;

    fn add(self, rhs: NodeHandle) -> Self::Output {
        NodeHandle {
            idx: self.idx + rhs.idx,
            text_idx: self.text_idx + rhs.text_idx,
        }
    }
}

impl Add<NodeHandle> for NodeSearch {
    type Output = NodeSearch;

    fn add(self, rhs: NodeHandle) -> Self::Output {
        match self {
            NodeSearch::Found(NodeHandle { idx, text_idx }) => NodeSearch::Found(NodeHandle {
                idx: idx + rhs.idx,
                text_idx: text_idx + rhs.text_idx,
            }),
            NodeSearch::FirstRemainingNode(NodeHandle { idx, text_idx }) => {
                NodeSearch::FirstRemainingNode(NodeHandle {
                    idx: idx + rhs.idx,
                    text_idx: text_idx + rhs.text_idx,
                })
            }
        }
    }
}

fn find_node_64(elems: &[[usize; 64]], offset: usize) -> NodeSearch {
    const LANE_SIZE: usize = 64;
    debug_assert!(elems.is_empty() || elems[0].len() == LANE_SIZE);

    let mut byte_idx = 0;
    let mut idx = 0;

    for lane in elems {
        let simd = usizex64::from_array(*lane);

        let sum = simd.reduce_sum();

        if byte_idx + sum > offset {
            let (lanes, leftover) = lane.as_chunks();
            let (first_node) = match find_node_8(lanes, offset).add(NodeHandle {
                idx,
                text_idx: byte_idx,
            }) {
                found @ NodeSearch::Found { .. } => return found,
                NodeSearch::FirstRemainingNode(node) => node,
            };

            return find_node_1(leftover, offset).add(first_node);
        }

        byte_idx += sum;
        idx += LANE_SIZE;
    }

    NodeSearch::FirstRemainingNode(NodeHandle {
        text_idx: byte_idx,
        idx,
    })
}

fn find_node_8(elems: &[[usize; 8]], offset: usize) -> NodeSearch {
    const LANE_SIZE: usize = 8;
    debug_assert!(elems.is_empty() || elems[0].len() == LANE_SIZE);

    let mut byte_idx = 0;
    let mut idx = 0;

    for lane in elems {
        let simd = usizex8::from_array(*lane);

        let sum = simd.reduce_sum();

        if byte_idx + sum > offset {
            return find_node_1(lane, offset).add(NodeHandle {
                idx,
                text_idx: byte_idx,
            });
        }

        byte_idx += sum;
        idx += LANE_SIZE;
    }

    NodeSearch::FirstRemainingNode(NodeHandle {
        text_idx: byte_idx,
        idx,
    })
}

fn find_node_1(elems: &[usize], offset: usize) -> NodeSearch {
    let mut byte_idx = 0;
    let mut idx = 0;

    for len in elems {
        if byte_idx + len > offset {
            return NodeSearch::Found(NodeHandle {
                text_idx: byte_idx,
                idx: idx,
            });
        }

        byte_idx += len;
        idx += 1;
    }

    NodeSearch::FirstRemainingNode(NodeHandle {
        text_idx: byte_idx,
        idx,
    })
}

impl<'nodes> IntoIterator for &'nodes Nodes {
    type Item = Node;

    type IntoIter = NodesIter<'nodes>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            nodes: self,
            idx: 0,
        }
    }
}

pub(crate) struct NodesIter<'nodes> {
    nodes: &'nodes Nodes,
    idx: usize,
}

impl Iterator for NodesIter<'_> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.nodes.get(self.idx) {
            self.idx += 1;
            Some(node)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nodes_new() {
        let nodes = Nodes::new();
        assert_eq!(nodes.len(), 0);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_nodes_push_and_get() {
        let mut nodes = Nodes::new();

        let node1 = Node::new(NodeKind::Original, 0, 10);
        let node2 = Node::new(NodeKind::Added, 10, 5);

        nodes.push(node1);
        nodes.push(node2);

        assert_eq!(nodes.len(), 2);
        assert!(!nodes.is_empty());

        let retrieved1 = nodes.get(0).unwrap();
        assert_eq!(retrieved1.kind, NodeKind::Original);
        assert_eq!(retrieved1.start, 0);
        assert_eq!(retrieved1.len, 10);

        let retrieved2 = nodes.get(1).unwrap();
        assert_eq!(retrieved2.kind, NodeKind::Added);
        assert_eq!(retrieved2.start, 10);
        assert_eq!(retrieved2.len, 5);
    }

    #[test]
    fn test_nodes_push_back() {
        let mut nodes = Nodes::new();

        let node = Node::new(NodeKind::Original, 0, 10);
        nodes.push_back(node);

        assert_eq!(nodes.len(), 1);
        let retrieved = nodes.get(0).unwrap();
        assert_eq!(retrieved.kind, NodeKind::Original);
        assert_eq!(retrieved.start, 0);
        assert_eq!(retrieved.len, 10);
    }

    #[test]
    fn test_nodes_set() {
        let mut nodes = Nodes::new();

        let node1 = Node::new(NodeKind::Original, 0, 10);
        let node2 = Node::new(NodeKind::Added, 5, 15);

        nodes.push(node1);
        assert_eq!(nodes.get(0).unwrap().kind, NodeKind::Original);
        assert_eq!(nodes.get(0).unwrap().start, 0);
        assert_eq!(nodes.get(0).unwrap().len, 10);

        nodes.set(0, node2);
        assert_eq!(nodes.get(0).unwrap().kind, NodeKind::Added);
        assert_eq!(nodes.get(0).unwrap().start, 5);
        assert_eq!(nodes.get(0).unwrap().len, 15);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_nodes_set_out_of_bounds() {
        let mut nodes = Nodes::new();
        let node = Node::new(NodeKind::Original, 0, 10);
        nodes.set(0, node); // Should panic
    }

    #[test]
    fn test_nodes_insert() {
        let mut nodes = Nodes::new();

        let node1 = Node::new(NodeKind::Original, 0, 10);
        let node2 = Node::new(NodeKind::Added, 10, 5);
        let node3 = Node::new(NodeKind::Original, 15, 3);

        nodes.push(node1);
        nodes.push(node2);
        nodes.insert(1, node3); // Insert in the middle

        assert_eq!(nodes.len(), 3);

        // Check order: Original(0,10), Original(15,3), Added(10,5)
        let first = nodes.get(0).unwrap();
        assert_eq!(first.kind, NodeKind::Original);
        assert_eq!(first.start, 0);
        assert_eq!(first.len, 10);

        let second = nodes.get(1).unwrap();
        assert_eq!(second.kind, NodeKind::Original);
        assert_eq!(second.start, 15);
        assert_eq!(second.len, 3);

        let third = nodes.get(2).unwrap();
        assert_eq!(third.kind, NodeKind::Added);
        assert_eq!(third.start, 10);
        assert_eq!(third.len, 5);
    }

    #[test]
    fn test_nodes_get_out_of_bounds() {
        let nodes = Nodes::new();
        assert_eq!(nodes.get(0), None);

        let mut nodes = Nodes::new();
        let node = Node::new(NodeKind::Original, 0, 10);
        nodes.push(node);

        assert_eq!(nodes.get(0), Some(Node::new(NodeKind::Original, 0, 10)));
        assert_eq!(nodes.get(1), None);
    }

    #[test]
    fn test_nodes_into_iter() {
        let mut nodes = Nodes::new();

        let node1 = Node::new(NodeKind::Original, 0, 10);
        let node2 = Node::new(NodeKind::Added, 10, 5);

        nodes.push(node1);
        nodes.push(node2);

        let mut iter = nodes.into_iter();
        let first = iter.next().unwrap();
        assert_eq!(first.kind, NodeKind::Original);
        assert_eq!(first.start, 0);
        assert_eq!(first.len, 10);

        let second = iter.next().unwrap();
        assert_eq!(second.kind, NodeKind::Added);
        assert_eq!(second.start, 10);
        assert_eq!(second.len, 5);

        assert_eq!(iter.next(), None);
    }
}
