use std::ops::RangeBounds;

use crate::nodes::NodeKind;

const WORD_SIZE: usize = size_of::<usize>() * 8;

#[derive(Debug, Clone)]
pub(crate) struct NodeKindVec {
    inner: Vec<usize>,
    len: usize,
}

impl NodeKindVec {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            inner: Vec::new(),
            len: 0,
        }
    }

    #[inline(always)]
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn offset(&self, idx: usize) -> Option<(usize, usize)> {
        if self.len <= idx {
            return None;
        }

        Some((idx / WORD_SIZE, idx % WORD_SIZE))
    }

    fn increment_len(&mut self) {
        if self.len() % WORD_SIZE == 0 {
            self.inner.push(0);
        }

        self.len += 1;
    }

    pub(crate) fn push_back(&mut self, value: NodeKind) {
        self.increment_len();
        self.set(self.len() - 1, value);
    }

    pub(crate) fn set(&mut self, idx: usize, value: NodeKind) {
        let (word_idx, offset) = self.offset(idx).unwrap();

        let mask = 1 << offset;

        if value == NodeKind::Original {
            self.inner[word_idx] |= mask;
        } else {
            self.inner[word_idx] &= !mask;
        }
    }

    /// Shifts all values starting from `idx`, the value at `idx` is left unchanged
    fn shift_right(&mut self, idx: usize) {
        self.increment_len();

        let (start_word, start_offset) = self.offset(idx).unwrap();
        let (end_word, _end_offset) = self.offset(self.len - 1).unwrap();

        // Handle the case where the insertion point is within the same word
        if start_word == end_word {
            let word = self.inner[start_word];
            // Shift bits to the right within the same word
            let mask = usize::MAX << start_offset;
            self.inner[start_word] = (word & !mask) | ((word & mask) << 1);
        } else {
            // Shift bits across word boundaries
            // Start from the end and work backwards to avoid overwriting
            for word_idx in (start_word..end_word).rev() {
                let current_word = self.inner[word_idx];
                let next_word = self.inner[word_idx + 1];

                // Shift the current word left by 1 bit
                // The MSB of current word becomes the LSB of next word
                self.inner[word_idx + 1] = (current_word >> (WORD_SIZE - 1)) | (next_word << 1);
            }

            // Handle the starting word
            let start_word_content = self.inner[start_word];
            let mask = usize::MAX << start_offset;
            self.inner[start_word] =
                (start_word_content & !mask) | ((start_word_content & mask) << 1);
        }
    }

    pub(crate) fn insert(&mut self, idx: usize, value: NodeKind) {
        if idx > self.len() {
            panic!("Index out of bounds");
        } else if idx == self.len() {
            self.push_back(value);
            return;
        }

        self.shift_right(idx);
        self.set(idx, value);
    }

    /// Removes elements from the vector in the given range
    ///
    /// Similar in spirit to `Vec`'s `drain()`, just simpler because we just want to remove a range
    /// without actually looking at the removed elements.
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

        if start >= end || start >= self.len() {
            return;
        }

        let end = end.min(self.len());
        let count = end - start;

        // Shift elements left to fill the gap
        for i in end..self.len() {
            // TODO: this could be more optimized if we implemented a `shift_left` function, but
            // it's probably not going to be a problem?
            let value = self.get(i).unwrap();
            self.set(i - count, value);
        }

        self.len -= count;

        // Remove unused words if possible
        let new_word_count = (self.len + WORD_SIZE - 1) / WORD_SIZE;
        self.inner.truncate(new_word_count);
    }

    pub(crate) fn get(&self, idx: usize) -> Option<NodeKind> {
        if let Some((word_idx, offset)) = self.offset(idx) {
            let word = self.inner[word_idx];
            Some(((word >> offset) & 1 == 1).into())
        } else {
            None
        }
    }

    pub(crate) unsafe fn get_unchecked(&self, idx: usize) -> NodeKind {
        let (word_idx, offset) = self
            .offset(idx)
            .expect("Tried to index past the end of array");

        let word = self.inner[word_idx];
        ((word >> offset) & 1 == 1).into()
    }
}

impl From<bool> for NodeKind {
    fn from(value: bool) -> Self {
        if value { Self::Original } else { Self::Added }
    }
}

#[cfg(test)]
mod proptests {
    use crate::{nodekind_vec::NodeKindVec, nodes::NodeKind::*};
    use proptest::prelude::*;

    #[derive(Debug, Clone)]
    enum Operation {
        PushBack(bool), // true = Original, false = Added
        Insert(usize, bool),
        Set(usize, bool),
        Get(usize),
    }

    fn operation_strategy() -> impl Strategy<Value = Operation> {
        prop_oneof![
            (any::<bool>()).prop_map(|b| Operation::PushBack(b)),
            (0..100usize, any::<bool>()).prop_map(|(idx, b)| Operation::Insert(idx, b)),
            (0..100usize, any::<bool>()).prop_map(|(idx, b)| Operation::Set(idx, b)),
            (0..100usize).prop_map(|idx| Operation::Get(idx)),
        ]
    }

    proptest! {
        #[test]
        fn nodekind_vec_matches_vec(operations in prop::collection::vec(operation_strategy(), 0..100)) {
            let mut nodekind_vec = NodeKindVec::new();
            let mut std_vec: Vec<crate::nodes::NodeKind> = Vec::new();

            for op in operations {
                match op {
                    Operation::PushBack(is_original) => {
                        let kind = if is_original { Original } else { Added };
                        nodekind_vec.push_back(kind);
                        std_vec.push(kind);
                        prop_assert_eq!(nodekind_vec.len(), std_vec.len());
                    }
                    Operation::Insert(idx, is_original) => {
                        let kind = if is_original { Original } else { Added };
                        if idx <= std_vec.len() {
                            nodekind_vec.insert(idx, kind);
                            std_vec.insert(idx, kind);
                            prop_assert_eq!(nodekind_vec.len(), std_vec.len());
                        }
                    }
                    Operation::Set(idx, is_original) => {
                        let kind = if is_original { Original } else { Added };
                        if idx < std_vec.len() {
                            nodekind_vec.set(idx, kind);
                            std_vec[idx] = kind;
                        }
                    }
                    Operation::Get(idx) => {
                        let nodekind_result = nodekind_vec.get(idx);
                        let std_result = std_vec.get(idx).copied();
                        prop_assert_eq!(nodekind_result, std_result);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{nodekind_vec::NodeKindVec, nodes::NodeKind::*};

    #[test]
    fn is_empty() {
        let vec = NodeKindVec::new();
        assert!(vec.is_empty());
    }

    #[test]
    fn push_back_one_original() {
        let mut vec = NodeKindVec::new();

        vec.push_back(Original);

        assert!(!vec.is_empty());
        assert_eq!(Original, vec.get(0).unwrap());
    }

    #[test]
    fn push_back_one_added() {
        let mut vec = NodeKindVec::new();

        vec.push_back(Added);

        assert!(!vec.is_empty());
        assert_eq!(Added, vec.get(0).unwrap());
    }

    #[test]
    fn push_back_two() {
        let mut vec = NodeKindVec::new();

        vec.push_back(Added);
        vec.push_back(Added);

        assert_eq!(Added, vec.get(0).unwrap());
        assert_eq!(Added, vec.get(1).unwrap());
    }

    #[test]
    fn insert_at_the_end() {
        let mut vec = NodeKindVec::new();

        vec.insert(0, Added);
        vec.insert(1, Added);

        assert_eq!(Added, vec.get(0).unwrap());
        assert_eq!(Added, vec.get(1).unwrap());
    }

    #[test]
    fn set() {
        let mut vec = NodeKindVec::new();

        vec.insert(0, Added);
        vec.set(0, Original);

        assert_eq!(Original, vec.get(0).unwrap());
    }

    #[test]
    fn set_two_times() {
        let mut vec = NodeKindVec::new();

        vec.insert(0, Added);
        vec.set(0, Original);
        vec.set(0, Added);

        assert_eq!(Added, vec.get(0).unwrap());
    }

    #[test]
    fn shift_right() {
        let mut vec = NodeKindVec::new();

        vec.insert(0, Added);
        vec.shift_right(0);

        assert_eq!(Added, vec.get(0).unwrap());
        assert_eq!(Added, vec.get(1).unwrap());
    }

    #[test]
    fn shift_right_same_value() {
        let mut vec = NodeKindVec::new();

        vec.insert(0, Added);
        vec.insert(1, Added);
        vec.shift_right(0);

        assert_eq!(Added, vec.get(0).unwrap());
        assert_eq!(Added, vec.get(1).unwrap());
        assert_eq!(Added, vec.get(2).unwrap());
    }

    #[test]
    fn shift_right_start() {
        let mut vec = NodeKindVec::new();

        vec.insert(0, Added);
        vec.insert(1, Original);
        vec.shift_right(0);

        assert_eq!(Added, vec.get(0).unwrap());
        assert_eq!(Added, vec.get(1).unwrap());
        assert_eq!(Original, vec.get(2).unwrap());
    }

    #[test]
    fn shift_right_more_than_one_word() {
        let mut vec = NodeKindVec::new();

        for i in 0..63 {
            let kind = if i % 2 == 0 { Added } else { Original };
            vec.push_back(kind);
        }

        vec.shift_right(0);

        assert_eq!(Added, vec.get(0).unwrap());
        for i in 1..vec.len() {
            let kind = if (i - 1) % 2 == 0 { Added } else { Original };
            assert_eq!(kind, vec.get(i).unwrap());
        }
    }

    #[test]
    fn push_back_then_insert() {
        let mut vec = NodeKindVec::new();

        vec.push_back(Added);
        vec.push_back(Added);
        vec.insert(0, Added);

        assert_eq!(Added, vec.get(0).unwrap());
        assert_eq!(Added, vec.get(1).unwrap());
    }

    #[test]
    fn insert_at_beginning() {
        let mut vec = NodeKindVec::new();

        vec.push_back(Original);
        vec.push_back(Added);
        vec.insert(0, Added);

        assert_eq!(Added, vec.get(0).unwrap());
        assert_eq!(Original, vec.get(1).unwrap());
        assert_eq!(Added, vec.get(2).unwrap());
    }

    #[test]
    fn insert_in_middle() {
        let mut vec = NodeKindVec::new();

        vec.push_back(Added);
        vec.push_back(Original);
        vec.push_back(Added);
        vec.insert(1, Added);

        assert_eq!(Added, vec.get(0).unwrap());
        assert_eq!(Added, vec.get(1).unwrap());
        assert_eq!(Original, vec.get(2).unwrap());
        assert_eq!(Added, vec.get(3).unwrap());
    }

    #[test]
    fn insert_multiple_times() {
        let mut vec = NodeKindVec::new();

        vec.push_back(Added);
        vec.insert(0, Original);
        vec.insert(0, Added);
        vec.insert(1, Original);

        assert_eq!(Added, vec.get(0).unwrap());
        assert_eq!(Original, vec.get(1).unwrap());
        assert_eq!(Original, vec.get(2).unwrap());
        assert_eq!(Added, vec.get(3).unwrap());
    }

    #[test]
    fn insert_beyond_word_boundary() {
        let mut vec = NodeKindVec::new();

        for i in 0..64 {
            vec.push_back(if i % 2 == 0 { Added } else { Original });
        }

        vec.insert(0, Original);

        assert_eq!(Original, vec.get(0).unwrap());
        assert_eq!(Added, vec.get(1).unwrap());
        assert_eq!(Original, vec.get(2).unwrap());
        assert_eq!(Added, vec.get(3).unwrap());

        for i in 1..65 {
            let expected = if (i - 1) % 2 == 0 { Added } else { Original };
            assert_eq!(expected, vec.get(i).unwrap(), "Mismatch at index {}", i);
        }
    }

    #[test]
    fn large_insertion_sequence() {
        let mut vec = NodeKindVec::new();

        vec.push_back(Added);
        vec.push_back(Added);

        vec.insert(0, Original);
        assert_eq!(Original, vec.get(0).unwrap());
        assert_eq!(Added, vec.get(1).unwrap());
        assert_eq!(Added, vec.get(2).unwrap());

        vec.insert(2, Original);
        assert_eq!(Original, vec.get(0).unwrap());
        assert_eq!(Added, vec.get(1).unwrap());
        assert_eq!(Original, vec.get(2).unwrap());
        assert_eq!(Added, vec.get(3).unwrap());
    }

    #[test]
    fn get_out_of_bounds() {
        let mut vec = NodeKindVec::new();

        vec.push_back(Added);
        vec.push_back(Original);

        assert_eq!(Some(Added), vec.get(0));
        assert_eq!(Some(Original), vec.get(1));
        assert_eq!(None, vec.get(2));
        assert_eq!(None, vec.get(100));
    }

    #[test]
    fn empty_vector_operations() {
        let mut vec = NodeKindVec::new();

        assert_eq!(0, vec.len());
        assert!(vec.is_empty());
        assert_eq!(None, vec.get(0));

        vec.push_back(Original);
        assert_eq!(1, vec.len());
        assert!(!vec.is_empty());
        assert_eq!(Some(Original), vec.get(0));
    }

    #[test]
    fn alternating_pattern_insert() {
        let mut vec = NodeKindVec::new();

        for i in 0..6 {
            vec.push_back(if i % 2 == 0 { Added } else { Original });
        }

        vec.insert(2, Original);

        assert_eq!(Added, vec.get(0).unwrap());
        assert_eq!(Original, vec.get(1).unwrap());
        assert_eq!(Original, vec.get(2).unwrap());
        assert_eq!(Added, vec.get(3).unwrap());
        assert_eq!(Original, vec.get(4).unwrap());
        assert_eq!(Added, vec.get(5).unwrap());
        assert_eq!(Original, vec.get(6).unwrap());
    }

    #[test]
    fn insert_at_end_equivalent_to_push() {
        let mut vec1 = NodeKindVec::new();
        let mut vec2 = NodeKindVec::new();

        vec1.push_back(Added);
        vec1.push_back(Original);
        vec2.push_back(Added);
        vec2.push_back(Original);

        vec1.insert(2, Added);
        vec2.push_back(Added);

        assert_eq!(vec1.len(), vec2.len());
        for i in 0..vec1.len() {
            assert_eq!(vec1.get(i), vec2.get(i), "Mismatch at index {}", i);
        }
    }
}
