//! Provides [`Sorted`], a slice "wrapper" that is guaranteed to be sorted.

mod traits;
mod iter;

use core::{
    ops::Deref,
    cmp::Ordering,
    slice,
    fmt::{self, Debug},
};

use crate::mem::alloc_vec::{self, Vec32};

use crate::__impl_slice_eq;

pub use traits::*;
pub use iter::*;

/// A vector-like collection that is guaranteed to be sorted.
///
/// Sorting is done through the [`SortByKey`] trait, which is automatically implemented for types
/// that implement [`Ord`]. Sorting is done through this trait so you can use this for types that
/// shouldn't logically be [`Ord`], but that could be sorted by some key (e.g. a single field).
///
/// Sorting is done according to the `sort_unstable` method.
///
/// # Examples
/// ``` rust
///
/// use leimu::{Sorted, sorted::SortByKey};
///
/// #[derive(PartialEq, Eq, Debug)]
/// struct U32(u32);
///
/// impl SortByKey for U32 {
///
///     type Key = u32;
///
///     fn key(&self) -> &Self::Key {
///         &self.0
///     }
/// }
///
/// let mut sorted: Sorted<U32> = [
///     U32(3),
///     U32(2),
///     U32(1),
/// ].into_iter().collect();
/// sorted.insert(U32(0));
/// assert_eq!(sorted, [U32(0), U32(1), U32(2), U32(3)]);
/// ```
#[derive(Clone, Hash)]
pub struct Sorted<T>
{
    data: Vec32<SortByKeyWrap<T>>,
}

impl<T> Sorted<T>
{

    /// Creates an empty slice.
    #[inline]
    pub fn new() -> Self {
        Self {
            data: Vec32::new(),
        }
    }

    /// Extracts the inner slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(
                self.data.as_ptr().cast(),
                self.data.len() as usize,
            )
        }
    }
}

impl<T> Sorted<T>
    where T: SortByKey,
{

    /// Inserts a value into the slice, retaining sortedness.
    #[inline]
    pub fn insert(
        &mut self,
        value: T,
    ) {
        let mut index = self.data.len();
        for (i, item) in self.iter().enumerate() {
            if value.key() < item.key() {
                index = i as u32;
                break;
            }
        }
        self.data.insert(index as usize, SortByKeyWrap(value));
    }

    /// Removes an element at `index` returning the removed value.
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        self.data.remove(index).0
    }
}

impl<T> Deref for Sorted<T> {

    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> AsRef<[T]> for Sorted<T> {
    
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T> IntoIterator for &'a Sorted<T> {

    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for Sorted<T> {

    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.data.into_iter(),
        }
    }
}

impl<T> Default for Sorted<T> {

    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
__impl_slice_eq! {
    [T, U]
    Sorted<T>,
    Sorted<U>,
    where T: PartialEq<U>,
}
__impl_slice_eq! {
    [T, U]
    Sorted<T>,
    &[U],
    where T: PartialEq<U>,
}
__impl_slice_eq! {
    [T, U]
    &[U],
    Sorted<T>,
    where U: PartialEq<T>,
}
__impl_slice_eq! {
    [T, U]
    Sorted<T>,
    &mut [U],
    where T: PartialEq<U>,
}
__impl_slice_eq! {
    [T, U]
    &mut [U],
    Sorted<T>,
    where U: PartialEq<T>,
}
__impl_slice_eq! {
    [T, U]
    Sorted<T>,
    [U],
    where T: PartialEq<U>,
}
__impl_slice_eq! {
    [T, U]
    [U],
    Sorted<T>,
    where U: PartialEq<T>,
}
__impl_slice_eq! {
    [T, U, const N: usize]
    Sorted<T>,
    [U; N],
    where T: PartialEq<U>,
}
__impl_slice_eq! {
    [T, U, const N: usize]
    [U; N],
    Sorted<T>,
    where U: PartialEq<T>,
}
impl<T> Eq for Sorted<T>
    where T: Eq,
{}

impl<A> Extend<A> for Sorted<A>
    where A: SortByKey,
{
    #[inline]
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        self.data.extend(iter.into_iter().map(SortByKeyWrap));
        self.data.sort_unstable();
    }
}

impl<A> FromIterator<A> for Sorted<A>
    where A: SortByKey,
{

    #[inline]
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut sorted = Self::new();
        sorted.extend(iter);
        sorted
    }
}

impl<T> Debug for Sorted<T>
    where T: Debug
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <_ as Debug>::fmt(self.as_slice(), f)
    }
}

/// A macro for creating a [`Sorted`] collection.
///
/// # Example
/// ``` rust
/// let sorted = leimu::sorted![4, 3, 2, 1];
/// assert_eq!(sorted, [1, 2, 3, 4]);
/// ```
#[macro_export]
macro_rules! sorted {
    [$($elem:expr),* $(,)?] => {
        $crate::Sorted::from_iter([$($elem),*])
    }
}
