use crate::boolean_vec::BooleanVec;

/// What buffer the data from this `Node` is stored in
///
/// TODO: Add comment saying that like `Node` this shouldn't be used too often
#[derive(Debug, Clone, Copy)]
pub(crate) enum NodeKind {
    Original,
    Added,
}

/// Represents a continuous slice of text in one of the two buffers
///
/// This type is mostly ever used to add nodes onto a `Nodes` struct, but for most cases only the
/// required fields should be accessed directly from a `Nodes` instance.
#[derive(Debug, Clone)]
pub(crate) struct Node {
    kind: NodeKind,
    start: usize,
    len: usize,
}

impl Node {
    pub(crate) fn new(kind: NodeKind, start: usize, len: usize) -> Node {
        Node { kind, start, len }
    }
}

/// Represents all nodes of a `PieceTable`
///
/// This is a SoA (Struct of Arrays) instead of an AoS (Array of Structs) in order to speed up the
/// most common operations, like finding the node for a given byte offset, etc.
#[derive(Debug)]
pub(crate) struct Nodes {
    pub(crate) kinds: BooleanVec,
    pub(crate) starts: Vec<usize>,
    pub(crate) lens: Vec<usize>,
}

impl Nodes {
    pub(crate) fn new() -> Nodes {
        Nodes {
            kinds: BooleanVec::new(),
            starts: Vec::new(),
            lens: Vec::new(),
        }
    }

    pub(crate) fn push_back(&mut self, node: Node) {}

    pub(crate) fn insert(&mut self, idx: usize, node: Node) {
        self.kinds.insert(idx, node.kind);
        self.start.insert(idx, node.start);
        self.lens.insert(idx, node.len);
    }
}
