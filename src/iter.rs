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
