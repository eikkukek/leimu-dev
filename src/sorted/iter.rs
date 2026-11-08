use super::*;

/// Iterator that moves out of [`Sorted`].
pub struct IntoIter<T> {
    pub(super) iter: alloc_vec::IntoIter<SortByKeyWrap<T>, (), u32>,
}

impl<T> Iterator for IntoIter<T> {

    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|s| s.0)
    }
}
