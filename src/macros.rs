/// Implements [`Display`][1] for [`bitflags`].
///
/// [1]: ::core::fmt::Display
#[doc(hidden)]
#[macro_export]
macro_rules! __bitflags_display {
    ($name:ident) => {
        impl ::core::fmt::Display for $name {

            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut iter = self.iter_names();
                let unknown_bits = self.bits() & !Self::all().bits();
                if let Some((first, _)) = iter.next() {
                    write!(f, "{first}")?;
                    for (rest, _) in iter {
                        write!(f, " | {rest}")?;
                    }
                    if unknown_bits != 0 {
                        write!(f, "| {unknown_bits:b}")?;
                    }
                    Ok(())
                } else {
                    if unknown_bits != 0 {
                        write!(f, "{unknown_bits:b}")
                    } else {
                        write!(f, "empty")
                    }
                }
            }
        }       
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_slice_eq {
    (
        [$($vars:tt)*]
        $lhs:ty,
        $rhs:ty $(,)?
        $(where $($where:tt)*)?
    ) => {
        impl<$($vars)*> PartialEq<$rhs> for $lhs
            $(where $($where)*)?
        {

            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                self[..] == other[..]
            }
        }
    };
}
