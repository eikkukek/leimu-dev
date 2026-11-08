use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    slice,
    hash::{self, Hash}
};

use smallvec::SmallVec;

#[doc(hidden)]
pub trait VecType:
    Default +
    Extend<Self::T> +
    FromIterator<Self::T> +
    IntoIterator<Item = Self::T> +
    AsRef<[Self::T]> +
    AsMut<[Self::T]> +
    Deref<Target = [Self::T]> +
    DerefMut +
{

    type T;

    fn _capacity(&self) -> usize;
    fn _with_len(len: usize, value: Self::T) -> Self
        where Self::T: Clone;
    fn _reserve(&mut self, new_capacity: usize);
    fn _resize_with<F>(&mut self, new_len: usize, f: F)
        where F: FnMut() -> Self::T;
    fn _push(&mut self, value: Self::T);
    fn _push_mut(&mut self, value: Self::T) -> &mut Self::T;
    fn _insert(&mut self, index: usize, element: Self::T);
    fn _insert_mut(&mut self, index: usize, element: Self::T) -> &mut Self::T;
    fn _remove(&mut self, index: usize) -> Self::T;
    fn _retain_mut<F>(&mut self, f: F)
        where F: FnMut(&mut Self::T) -> bool;
    fn _dedup_by<F>(&mut self, same_bucket: F)
        where F: FnMut(&mut Self::T, &mut Self::T) -> bool;
    fn _swap_remove(&mut self, index: usize) -> Self::T;
    fn _pop(&mut self) -> Option<Self::T>;
    fn _clear(&mut self);
}

/// A generic wrapper around a vector.
///
/// Supports basic vector operations.
#[derive(Clone)]
pub struct VecWrapper<V: VecType, M = ()>(V, PhantomData<M>);

impl<V, M> VecWrapper<V, M>
    where V: VecType
{

    /// Creates an empty vector.
    #[inline]
    pub fn new() -> Self {
        Self(V::default(), PhantomData)
    }

    #[doc(hidden)]
    #[inline]
    pub fn with_len(len: usize, value: V::T) -> Self
        where V::T: Clone
    {
        Self(V::_with_len(len, value), PhantomData)
    }

    /// Returns the length of the vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Returns the allocated capacity of the vector.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0._capacity()
    }

    /// Returns a slice over the contents of the vector.
    #[inline]
    pub fn as_slice(&self) -> &[V::T] {
        self.0.as_ref()
    }

    /// Returns a mutable slice over the contents of the vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [V::T] {
        self.0.as_mut()
    }

    /// Resizes the vector with the given closure.
    ///
    /// If `new_len` is less than the current length, this shrinks the vector.
    #[inline]
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
        where F: FnMut() -> V::T
    {
        self.0._resize_with(new_len, f);
    }

    /// Reserves space for the vector.
    #[inline]
    pub fn reserve(&mut self, new_capacity: usize) {
        self.0._reserve(new_capacity);
    }

    /// Resizes the vector with a clonable value.
    ///
    /// If `new_len` is less than the current length, this shrinks the vector.
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: V::T)
        where V::T: Clone
    {
        self.0._resize_with(new_len, || value.clone());
    }

    /// Pushes an element to the end of the vector.
    #[inline]
    pub fn push(&mut self, value: V::T) {
        self.0._push(value);
    }

    /// Pushes an element to the end of the vector, returning a mutable refernce to the newly
    /// written value.
    #[inline]
    pub fn push_mut(&mut self, value: V::T) -> &mut V::T {
        self.0._push_mut(value)
    }

    /// Removes the last element of the vector and returns it, or `None` if the vector is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<V::T> {
        self.0._pop()
    }

    /// Removes all elements from the vector preserving its allocated capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.0._clear();
    }

    /// Inserts an element to the vector at the specified index.
    ///
    /// This panics if `index` is greater than the length of the vector.
    #[inline]
    pub fn insert(&mut self, index: usize, element: V::T) {
        self.0._insert(index, element);
    }

    /// Inserts an element to the vector at the specified index, returning a mutable reference to
    /// the newly written value.
    ///
    /// This panics if `index` is greater than the length of the vector.
    #[inline]
    pub fn insert_mut(&mut self, index: usize, element: V::T) -> &mut V::T {
        self.0._insert_mut(index, element)
    }

    #[inline]
    pub fn extend_from_slice(&mut self, slice: &[V::T])
        where V::T: Clone
    {
        self.0.extend(slice.iter().cloned());
    }

    /// Removes an element from the specified index.
    ///
    /// Panics if `index` is greater than or equal to the length of the vector.
    #[inline]
    pub fn remove(&mut self, index: usize) -> V::T {
        self.0._remove(index)
    }

    /// Retains all elements that satisfy the predicate, preserving the order of elements.
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
        where F: FnMut(&V::T) -> bool
    {
        self.0._retain_mut(|v| f(v));
    }

    /// Removes consecutive repeated elements from the vector according to a closure defined
    /// equality.
    #[inline]
    pub fn dedup_by<F>(&mut self, same_bucket: F)
        where F: FnMut(&mut V::T, &mut V::T) -> bool
    {
        self.0._dedup_by(same_bucket);
    }

    /// Removes consecutive repeated elements from the vector according to [`PartialEq`].
    #[inline]
    pub fn dedup(&mut self)
        where V::T: PartialEq
    {
        self.0._dedup_by(|a, b| (&*a).eq(&*b));
    }

    /// Removes consecutive repeated elements that resolve to a key implementing [`PartialEq`].
    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
        where
            F: FnMut(&mut V::T) -> K,
            K: PartialEq,
    {
        self.0._dedup_by(|a, b| key(a).eq(&key(b)));
    }

    /// Retains all elements that satisfy the predicate that takes a mutable reference,
    /// preserving the order of elements.
    #[inline]
    pub fn retain_mut<F>(&mut self, f: F)
        where F: FnMut(&mut V::T) -> bool
    {
        self.0._retain_mut(f);
    }

    /// Removes an element from the specified index and replaces the value of that index with the
    /// last element of the vector if index is not the last index.
    ///
    /// This may be faster than [`remove`][Self::remove] but this doesn’t preserve the order of elements.
    ///
    /// Panics if `index` is greater than or equal to the length of the vector.
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> V::T {
        self.0._remove(index)
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, V::T> {
        self.as_slice().iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, V::T> {
        self.as_mut_slice().iter_mut()
    }
}
impl<V, M> Default for VecWrapper<V, M>
    where V: VecType
{
    #[inline]
    fn default() -> Self {
        Self(V::default(), PhantomData)
    }
}
impl<V, M> FromIterator<V::T> for VecWrapper<V, M>
    where V: VecType
{
    
    #[inline]
    fn from_iter<T: IntoIterator<Item = V::T>>(iter: T) -> Self {
        Self(iter.into_iter().collect(), PhantomData)
    }
}
impl<V, M> Extend<V::T> for VecWrapper<V, M>
    where V: VecType,
{

    #[inline]
    fn extend<T: IntoIterator<Item = V::T>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}
impl<'a, V, M> IntoIterator for &'a VecWrapper<V, M>  
    where V: VecType
{

    type Item = &'a V::T;
    type IntoIter = slice::Iter<'a, V::T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, V, M> IntoIterator for &'a mut VecWrapper<V, M>  
    where V: VecType
{

    type Item = &'a mut V::T;
    type IntoIter = slice::IterMut<'a, V::T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
impl<V, M> IntoIterator for VecWrapper<V, M>
    where V: VecType
{

    type Item = V::T;
    type IntoIter = V::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<V, M> AsRef<[V::T]> for VecWrapper<V, M>
    where V: VecType
{
    #[inline]
    fn as_ref(&self) -> &[V::T] {
        self.as_slice()
    }
}
impl<V, M> AsMut<[V::T]> for VecWrapper<V, M>
    where V: VecType
{
    #[inline]
    fn as_mut(&mut self) -> &mut [V::T] {
        self.as_mut_slice()
    }
}
impl<V, M> Deref for VecWrapper<V, M>
    where V: VecType
{
    type Target = [V::T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<V, M> DerefMut for VecWrapper<V, M>
    where V: VecType
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
trait SmallVecExt<T> {

    fn with_len(len: usize, value: T) -> Self
        where T: Clone;

    fn push_mut(&mut self, value: T) -> &mut T;

    fn insert_mut(&mut self, index: usize, element: T) -> &mut T;
}

impl<A: smallvec::Array> SmallVecExt<A::Item> for SmallVec<A> {

    fn with_len(len: usize, value: A::Item) -> Self
        where A::Item: Clone
    {
        smallvec::smallvec![value; len]
    }

    fn push_mut(&mut self, value: A::Item) -> &mut A::Item {
        let index = self.len();
        self.push(value);
        unsafe {
            self.get_unchecked_mut(index)
        }
    }

    fn insert_mut(&mut self, index: usize, element: A::Item) -> &mut A::Item {
        self.insert(index, element);
        unsafe {
            self.get_unchecked_mut(index)
        }
    }
}

macro_rules! vec_impl {
    ([$($bounds:tt)*] type T = $ty:ty; $t:ty 
    ) => {
        impl<$($bounds)*> VecType for $t
        {
            type T = $ty;
            #[inline]
            fn _capacity(&self) -> usize {
                self.capacity()
            }
            #[inline]
            fn _with_len(len: usize, value: Self::T) -> Self
                where Self::T: Clone
            {
                Self::with_len(len, value)
            }
            #[inline]
            fn _reserve(&mut self, new_capacity: usize) {
                self.reserve(new_capacity);
            }
            #[inline]
            fn _resize_with<F>(&mut self, new_len: usize, f: F)
                where F: FnMut() -> Self::T {
                self.resize_with(new_len, f);
            }
            #[inline]
            fn _push(&mut self, value: Self::T) {
                self.push(value);
            }
            #[inline]
            fn _push_mut(&mut self, value: Self::T) -> &mut Self::T {
                self.push_mut(value)
            }
            #[inline]
            fn _insert(&mut self, index: usize, element: Self::T) {
                self.insert(index, element)
            }
            #[inline]
            fn _insert_mut(&mut self, index: usize, element: Self::T) -> &mut Self::T {
                self.insert_mut(index, element)
            }
            #[inline]
            fn _remove(&mut self, index: usize) -> Self::T {
                self.remove(index)
            }
            #[inline]
            fn _retain_mut<F>(&mut self, f: F)
                where F: FnMut(&mut Self::T) -> bool
            {
                self.retain_mut(f);
            }
            #[inline]
            fn _dedup_by<F>(&mut self, same_bucket: F)
                where F: FnMut(&mut Self::T, &mut Self::T) -> bool
            {
                self.dedup_by(same_bucket);
            }
            #[inline]
            fn _swap_remove(&mut self, index: usize) -> Self::T
            {
                self.swap_remove(index)
            }
            #[inline]
            fn _pop(&mut self) -> Option<Self::T>
            {
                self.pop()
            }
            #[inline]
            fn _clear(&mut self) {
                self.clear();
            }
        }
    };
}

vec_impl! {
    [A: smallvec::Array] type T = A::Item; SmallVec<A>
}

use crate::__impl_slice_eq;

__impl_slice_eq! {
    [V1: VecType, M1, V2: VecType, M2]
    VecWrapper<V1, M1>, VecWrapper<V2, M2>,
    where V1::T: PartialEq<V2::T>
}
impl<V: VecType, M> Eq for VecWrapper<V, M>
    where V::T: Eq {}
__impl_slice_eq! {
    [V1: VecType, M, U, const N: usize]
    VecWrapper<V1, M>, [U; N],
    where V1::T: PartialEq<U>
}
__impl_slice_eq! {
    [V1: VecType, M, U, const N: usize]
    [U; N], VecWrapper<V1, M>,
    where U: PartialEq<V1::T>
}
__impl_slice_eq! {
    [V1: VecType, M, U]
    VecWrapper<V1, M>, &[U],
    where V1::T: PartialEq<U>
}
__impl_slice_eq! {
    [V1: VecType, M, U]
    &[U], VecWrapper<V1, M>,
    where U: PartialEq<V1::T>
}
__impl_slice_eq! {
    [V1: VecType, M, U]
    VecWrapper<V1, M>, &mut [U],
    where V1::T: PartialEq<U>
}
__impl_slice_eq! {
    [V1: VecType, M, U]
    &mut [U], VecWrapper<V1, M>,
    where U: PartialEq<V1::T>
}
__impl_slice_eq! {
    [V1: VecType, M, U]
    VecWrapper<V1, M>, [U],
    where V1::T: PartialEq<U>
}
__impl_slice_eq! {
    [V1: VecType, M, U]
    [U], VecWrapper<V1, M>,
    where U: PartialEq<V1::T>
}
impl<V: VecType, M> Hash for VecWrapper<V, M>
    where V::T: Hash
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
