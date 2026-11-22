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
                        write!(f, "0x0")
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

#[doc(hidden)]
#[macro_export]
macro_rules! __id {
    (
        $(#[$outer:meta])*
        $name:ident
    ) => {
        $(#[$outer])*
        #[repr(transparent)]
        #[derive(Clone, Copy, $crate::bytemuck::ByteEq, $crate::bytemuck::ByteHash, Debug)]
        pub struct $name(u64);
        unsafe impl $crate::bytemuck::NoUninit for $name {}

        impl $name {

            fn next() -> Self {
                use $crate::sync::atomic;
                static NEXT_ID: atomic::AtomicU64 = atomic::AtomicU64::new(0);
                Self(NEXT_ID.fetch_add(1, atomic::Ordering::AcqRel))
            }
        }
    };
}

#[doc(hidden)]
#[inline]
pub const fn module_name(module_path: &'static str) -> &'static str {
    let bytes = module_path.as_bytes();
    let mut i = bytes.len() - 1;
    while i != 0 {
        let b = bytes[i];
        if b == b':' {
            let start = i + 1;
            return unsafe {
                str::from_utf8_unchecked(
                    ::core::slice::from_raw_parts(
                        bytes.as_ptr().add(start),
                        bytes.len() - start,
                    )
                )
            }
        }
        i -= 1;
    }
    module_path
}

#[doc(hidden)]
#[macro_export]
macro_rules! __module_name {
    () => {
        $crate::macros::module_name(module_path!())
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __variant_some {
    ($e_first:ident $(::$e:ident)* = $value:expr) => {
        {
            if let $e_first$(::$e)*(x) = $value {
                Some(x)
            } else {
                None
            }
        }
    };
}
