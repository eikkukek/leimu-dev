use std::sync::Arc;

use core::{
    ffi::CStr,
    num::NonZero,
    ops::Deref,
    fmt::{self, Debug},
    hash::{self, Hash},
    borrow::Borrow,
};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Stack {
    first: NonZero<u8>,
    rest: [u8; 23],
}

impl Stack {

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        let ptr: *const [u8; 24] = <*const _>::cast(self);
        unsafe {
            let bytes = &*ptr;
            bytes.as_slice()
        }
    }
}

#[derive(Clone)]
pub enum Inner {
    Stack(Stack),
    Heap(Arc<CStr>),
}

/// Similar to `SmolStr` and `CompactString`, but for an immutable [`CStr`].
///
/// The size of this string type is 24 bytes, which is the same as [`CString`][std::ffi::CString] on
/// 64-bit platforms.
///
/// Strings are stack-allocated if their length is 23 bytes long, otherwise the `CStr` is stored in a
/// clonable [`Arc`].
///
/// Cloning this string doesn't allocate any extra memory.
#[derive(Clone)]
pub struct CompactCStr(Inner);

impl CompactCStr {

    /// Creates a new string from a [`CStr`] reference.
    #[inline]
    pub fn new(c_str: &CStr) -> Self {
        let n_bytes = c_str.count_bytes();
        if n_bytes > 23 {
            return Self(Inner::Heap(c_str.into()))
        }
        if n_bytes == 0 {
            let mut rest = [0u8; 23];
            rest[1] = u8::MAX;
            return Self(Inner::Stack(Stack {
                first: NonZero::<u8>::MAX,
                rest,
            }))
        }
        let mut data = [0u8; 24];
        data[0..n_bytes]
            .copy_from_slice(
            c_str.to_bytes()
        );
        unsafe { Self(Inner::Stack(
            ::core::mem::transmute::<
                [u8; 24],
                Stack,
            >(data)
        ))}
    }

    /// Extracts a [`CStr`] containing the entire string.
    #[inline]
    pub fn as_c_str(&self) -> &CStr {
        match &self.0 {
            Inner::Stack(stack) => {
                if stack.first.get() == u8::MAX &&
                    stack.rest[0] == 0 &&
                    stack.rest[1] == u8::MAX
                {
                    return c""
                }
                CStr::from_bytes_until_nul(stack.as_bytes())
                    .unwrap()
            },
            Inner::Heap(c_str) => c_str,
        }
    }
}

impl Default for CompactCStr {

    #[inline]
    fn default() -> Self {
        let mut rest = [0u8; 23];
        rest[1] = u8::MAX;
        Self(Inner::Stack(Stack {
            first: NonZero::<u8>::MAX,
            rest,
        }))
    }
}

impl AsRef<CStr> for CompactCStr {

    #[inline]
    fn as_ref(&self) -> &CStr {
        self.as_c_str()
    }
}

impl Deref for CompactCStr {

    type Target = CStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_c_str()
    }
}

impl<T> PartialEq<T> for CompactCStr
    where T: AsRef<CStr>,
{

    fn eq(&self, other: &T) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for CompactCStr {}

impl Hash for CompactCStr {

    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_c_str().hash(state);
    }
}

impl Borrow<CStr> for CompactCStr {

    #[inline]
    fn borrow(&self) -> &CStr {
        self.as_c_str()
    }
}

impl Debug for CompactCStr {
    
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_c_str(), f)
    }
}

impl From<&CStr> for CompactCStr {

    #[inline]
    fn from(value: &CStr) -> Self {
        Self::new(value)
    }
}
