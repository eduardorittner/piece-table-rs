#[derive(Debug)]
pub(crate) struct BooleanVec {
    inner: Vec<usize>,
}

impl BooleanVec {
    pub(crate) fn new() -> BooleanVec {
        BooleanVec { inner: Vec::new() }
    }
}
