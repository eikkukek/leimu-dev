use super::*;

/// A trait used by [`Sorted`] that is used to extract a [`Ord`] key.
///
/// Implemented automatically for types that implement [`Ord`].
pub trait SortByKey {

    /// The type of the key.
    type Key: Ord;

    /// Extracts the key used for sorting.
    fn key(&self) -> &Self::Key;
}

impl<T: Ord> SortByKey for T {

    type Key = Self;

    #[inline]
    fn key(&self) -> &Self::Key {
        self
    }
}

#[repr(transparent)]
#[derive(Clone, Hash)]
pub(super) struct SortByKeyWrap<T>(pub T);

impl<T> PartialEq for SortByKeyWrap<T>
    where
        T: SortByKey,
{

    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.key().eq(other.0.key())
    }
}

impl<T> Eq for SortByKeyWrap<T>
    where
        T: SortByKey,
{}

impl<T> PartialOrd for SortByKeyWrap<T>
    where
        T: SortByKey,
{

    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for SortByKeyWrap<T>
    where
        T: SortByKey,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.key().cmp(other.0.key())
    }
}
