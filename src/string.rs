use core::{
    ops::Deref,
    fmt::{self, Debug, Display},
    hash::{self, Hash},
    borrow::Borrow,
    num::NonZero,
};

use crate::sync::Arc;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Stack {
    bytes: [u8; 23],
    len: NonZero<u8>,
}

impl Stack {

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        let mut len = self.len.get();
        if len == u8::MAX {
            len = 0;
        }
        &self.bytes[0..len as usize]
    }
}

#[derive(Clone)]
pub enum Inner {
    Stack(Stack),
    Heap(Arc<str>),
}

/// Similar to `SmolStr` and `CompactString`, but for an immutable [`str`].
///
/// The size of this string type is 24 bytes, which is the same as [`String`] on
/// 64-bit platforms.
///
/// Strings are stack-allocated if their length is 23 bytes long, otherwise the `str` is stored in a
/// clonable [`Arc`].
///
/// Cloning this string has the worst-case cost of cloning an [`Arc`].
#[derive(Clone)]
pub struct CompactStr(Inner);

impl CompactStr {

    /// Creates a new string.
    #[inline]
    pub fn new(str: impl AsRef<str>) -> Self {
        let str = str.as_ref();
        let n_bytes = str.len();
        if n_bytes > 23 {
            return Self(Inner::Heap(str.into()))
        }
        let mut bytes = [0u8; 23];
        bytes[0..n_bytes]
            .copy_from_slice(
            str.as_bytes()
        );
        Self(Inner::Stack(
            Stack {
                bytes,
                len: NonZero::new(n_bytes as u8)
                    .unwrap_or(NonZero::<u8>::MAX),
            }
        ))
    }

    /// Extracts a [`str`] containing the entire string.
    #[inline]
    pub fn as_str(&self) -> &str {
        match &self.0 {
            Inner::Stack(stack) =>
                str::from_utf8(
                    stack.as_bytes()
                ).unwrap()
            ,
            Inner::Heap(str) => str,
        }
    }
}

impl Default for CompactStr {

    #[inline]
    fn default() -> Self {
        Self(Inner::Stack(Stack {
            bytes: Default::default(),
            len: NonZero::<u8>::MAX,
        }))
    }
}

impl AsRef<str> for CompactStr {

    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for CompactStr {

    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<T> PartialEq<T> for CompactStr
    where T: AsRef<str>,
{

    fn eq(&self, other: &T) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for CompactStr {}

impl Hash for CompactStr {

    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Borrow<str> for CompactStr {

    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Debug for CompactStr {
    
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for CompactStr {

    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl From<&str> for CompactStr {

    #[inline]
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
