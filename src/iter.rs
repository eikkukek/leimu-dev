//! Exports all of [`core::iter`] and provides useful iterator type-defs.

pub use core::iter::*;

use core::slice;

/// [`slice::Iter`] mapped by a function pointer.
pub type IterSliceMap<'a, T, U> = Map<slice::Iter<'a, T>, fn(&'a T) -> U>;

#[macro_export]
#[doc(hidden)]
macro_rules! __iter_wrapper {
    {
        $(#[$meta:meta])*
        $vis:vis struct $name:ident$(<$lt:lifetime>)?($iter:ty): $item:ty;
    } => {
        $(#[$meta])*
        $vis struct $name $(<$lt>)? ($iter);

        impl $(<$lt>)? Iterator for $name $(<$lt>)? {

            type Item = $item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
        }
    };
}

/// [`Copied`] [`slice::Iter`].
pub type IterSliceCopied<'a, T> = Copied<slice::Iter<'a, T>>;

/// Returned by [`enumerate_u32`][IteratorExt::enumerate_u32].
pub struct EnumerateU32<I: Iterator> {
    iter: Enumerate<I>,
}

impl<I> Iterator for EnumerateU32<I>
    where I: Iterator
{

    type Item = (u32, <I as Iterator>::Item);
    
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
            .map(|(i, t)| (i as u32, t))
    }
}

/// An extension trait for [`Iterator`].
pub trait IteratorExt: Iterator + Sized {

    /// Same as [`enumerate`], but yields the count as [`u32`] insted of [`usize`].
    ///
    /// [`enumerate`]: Iterator::enumerate
    #[inline]
    fn enumerate_u32(self) -> EnumerateU32<Self> {
        EnumerateU32 {
            iter: self.enumerate()
        }
    }

    /// Same as [`count`], but returns the count as [`u32`].
    ///
    /// [`count`]: Iterator::count
    #[inline]
    fn count_u32(self) -> u32 {
        self.count() as u32
    }
}

impl<I: Iterator> IteratorExt for I {}
