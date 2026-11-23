use bytemuck::{NoUninit, AnyBitPattern};

use core::{
    slice, 
    ptr::NonNull,
    ops::{RangeBounds, Bound},
    marker::PhantomData,
};

use crate::*;

fn bounds_to_range(size: usize, range: impl RangeBounds<usize>) -> (usize, usize) {
    let begin = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Included(&inc) => inc,
        Bound::Excluded(&ex) => ex,
    };
    let end = match range.end_bound() {
        Bound::Unbounded => size,
        Bound::Included(&inc) => inc + 1,
        Bound::Excluded(&ex) => ex,
    };
    assert!(begin <= end, "invalid range");
    (begin, end)
}

/// Specifies a mapping mode of a [`Buffer`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MapMode {
    /// Specifies that the mapped range is read-only.
    ///
    /// It is invalid to call `MappedBuffer::write_only*` methods when this mode is set.
    ReadOnly,
    /// Specifies that the mapped range is write-only.
    ///
    /// It is invalid to call [`MappedBuffer::read_only`] method when this mode is set.
    WriteOnly,
}

/// A wrapper around write-only data.
///
/// # Examples
/// ```
/// use leimu::WriteOnly;
///
/// let mut array = [0; 3];
/// let mut writeonly = WriteOnly::from_mut(array.as_mut_slice());
/// writeonly.copy_from_slice(&[1, 2, 3]);
/// assert_eq!(array, [1, 2, 3]);
/// ```
#[repr(transparent)]
pub struct WriteOnly<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T: ?Sized> WriteOnly<'a, T> {

    /// Creates a write-only wrapper from a mutable reference.
    #[inline]
    pub fn from_mut(value: &'a mut T) -> Self {
        Self { ptr: NonNull::from_mut(value), _marker: PhantomData }
    }

    /// Creates a write-only wrapper from a [`NonNull<T>`] pointer.
    ///
    /// # Safety
    /// - Writes to the pointer **must** be valid.
    #[inline]
    pub unsafe fn from_non_null(value: NonNull<T>) -> Self {
        Self { ptr: value, _marker: PhantomData }
    }

    /// Extracts the inner raw pointer of `T`.
    ///
    /// # Safety
    /// - You should **not** read from this pointer or create a `&mut T` from it, there is no
    ///   guarantee that the data pointed to is initialized or readable.
    #[inline]
    pub fn as_ptr(&mut self) -> NonNull<T> {
        self.ptr
    }
}

impl<'a, T: Sized> WriteOnly<'a, T> {

    /// Writes `value` to the underlying pointer.
    #[inline]
    pub fn write(&mut self, value: T)
        where T: Copy
    {
        unsafe {
            self.ptr.write(value);
        }
    }

    /// Casts `T` to `U` using an equivalent method to [`bytemuck::try_cast_mut`].
    ///
    /// Returns an error when that function would fail.
    #[inline]
    pub fn try_cast<U>(self) -> Result<WriteOnly<'a, U>, Error>
        where
            T: AnyBitPattern + NoUninit,
            U: AnyBitPattern + NoUninit,
    {
        // extractd from bytemuck::internal::try_cast_mut
        use bytemuck::PodCastError;
        if align_of::<U>() > align_of::<T>() &&
            self.ptr.align_offset(align_of::<U>()) != 0
        {
            Err(PodCastError::TargetAlignmentGreaterAndInputNotAligned)
        } else if size_of::<U>() == size_of::<T>() {
            Ok(unsafe { WriteOnly::from_non_null(self.ptr.cast())  })
        } else {
            Err(PodCastError::SizeMismatch)
        }.describe_err("failed to cast `T` to `U`")
    }

    /// Tries to cast a `WriteOnly<[u8]>` to `Self`.
    ///
    /// Returns an error if `T` would be misaligned or if the length of `bytes` is not equal to
    /// `size_of::<T>()`.
    #[inline]
    pub fn try_from_bytes(mut bytes: WriteOnly<'a, [u8]>) -> Result<Self, Error> {
        if bytes.as_ptr().cast::<u8>().align_offset(align_of::<T>()) != 0 {
            Err(err!(
                "`T` would not be aligned if cast from `bytes`"
            ))
        } else if size_of::<T>() == bytes.len()  {
            unsafe {
                Ok(Self::from_non_null(bytes.as_ptr().cast()))
            }
        } else {
            Err(err!(
                "`bytes.len()` is not equal to the size of `T`"
            ))
        }
    }
}

impl<'a, T> WriteOnly<'a, [T]> { 

    /// Gets the length of the inner slice.
    #[inline]
    pub fn len(&self) -> usize {
        self.ptr.len()
    }

    /// Return whether the length of the inner slice is equal to 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Casts `[T]` to `[U]` using an equivalent method to [`bytemuck::try_cast_slice_mut`].
    ///
    /// Returns an error when that function would fail.
    #[inline]
    pub fn try_cast_slice<U>(self) -> Result<WriteOnly<'a, [U]>, Error>
        where
            T: AnyBitPattern + NoUninit,
            U: AnyBitPattern + NoUninit,
    {
        // extractd from bytemuck::internal::try_cast_slice_mut
        use bytemuck::PodCastError;
        let input_bytes = size_of::<T>() * self.ptr.len();
        if align_of::<U>() > align_of::<T>() &&
            self.ptr.cast::<()>().align_offset(align_of::<U>()) != 0
        {
            Err(PodCastError::TargetAlignmentGreaterAndInputNotAligned)
        } else if size_of::<U>() == size_of::<T>() {
            Ok(NonNull::slice_from_raw_parts(
                self.ptr.cast::<U>(),
                self.ptr.len(),
            ))
        } else if (size_of::<U>() != 0 && input_bytes.is_multiple_of(size_of::<U>()))
            || (size_of::<U>() == 0 && input_bytes == 0)
        {
            let new_len =
                if size_of::<U>() != 0 { input_bytes / size_of::<U>() } else { 0 };
            Ok(NonNull::slice_from_raw_parts(
                self.ptr.cast::<U>(),
                new_len,
            ))
        } else {
            Err(PodCastError::OutputSliceWouldHaveSlop)
        }.map(|ptr| unsafe { WriteOnly::from_non_null(ptr) }).describe_err("failed to cast `[T]` to `[U]`")
        
    }

    /// Functionally equivalent to [`slice::copy_from_slice`].
    ///
    /// Panics if `self.len() != src.len()`.
    #[inline]
    pub fn copy_from_slice(&mut self, src: &[T])
        where T: Copy
    {
        assert_eq!(self.len(), src.len(), "`self.len() != src.len()`");
        unsafe {
            <*const T>::copy_to_nonoverlapping(
                src.as_ptr(),
                self.ptr.cast::<T>().as_ptr(),
                self.ptr.len()
            );
        }
    }

    /// Functionally equivalent to [`slice::fill`].
    #[inline]
    pub fn fill(&mut self, value: T)
        where T: Copy,
    {
        unsafe {
            let ptr = NonNull::with_exposed_provenance(self.ptr.expose_provenance());
            for i in 0..self.len() {
                ptr.add(i).write(value);
            }
        }
    }
   
    /// Functionally equivalent to [`slice::split_at`].
    ///
    /// Panics if `mid` is greater than [`len`][Self::len].
    #[inline]
    pub fn split_at(&mut self, mid: usize) -> (Self, Self) {
        assert!(mid <= self.len(), "`mid > self.len()`");
        let ptr = self.ptr.cast::<T>();
        unsafe {(
            Self::from_non_null(NonNull::slice_from_raw_parts(
                ptr, mid
            )),
            Self::from_non_null(NonNull::slice_from_raw_parts(
                ptr.add(mid),
                self.len() - mid
            ))
        )}
    }

    /// Functionally equivalent to [`slice::as_chuncks_mut`]. Splits the inner slice into a slice of `N`-element
    /// arrays and a remainder slice with a length less than `N`.
    #[inline]
    pub fn into_chunks<const N: usize>(mut self) -> (WriteOnly<'a, [[T; N]]>, Self) {
        let n_chunks = self.len() / N;
        let split = n_chunks * N;
        let (mut left, right) = self.split_at(split);
        unsafe {(
            WriteOnly::from_non_null(NonNull::slice_from_raw_parts(
                left.as_ptr().cast::<[T; N]>(),
                n_chunks,
            )),
            right,
        )}
    }

    /// Gets a subslice of `self`.
    ///
    /// Panics if the range is out of bounds or if the range is invalid.
    #[inline]
    pub fn subslice(&mut self, range: impl RangeBounds<usize>) -> Self {
        let (begin, end) = bounds_to_range(self.len(), range);
        if begin >= self.len() || end > self.len() {
            panic!("`range` is out of bounds");
        }
        unsafe {
            Self::from_non_null(NonNull::slice_from_raw_parts(
                self.ptr.cast::<T>().add(begin),
                end - begin,
            ))
        }
    }
}

/// A mapped [`Buffer`].
pub struct MappedBuffer<'a>
{
    pub(super) buffer: &'a Buffer,
    pub(super) ptr: NonNull<u8>,
    pub(super) range: DeviceSize,
    pub(super) mode: MapMode,
}

impl<'a> MappedBuffer<'a>
{

    /// Returns [`WriteOnly<T>`], where `T` implements [`AnyBitpattern`] and [`NoUninit`].
    ///
    /// # Valid usage
    /// - The mode of this mapping **must** be [`WriteOnly`][MapMode::WriteOnly].
    /// - `size_of::<T>() + offset` **must** be less than or equal to `range`.
    /// - The inner pointer + `offset` **must** be aligned to `T`.
    pub fn write_only<T>(&mut self, offset: DeviceSize) -> Result<WriteOnly<'_, T>, Error>
        where T: AnyBitPattern + NoUninit
    {
        if self.mode != MapMode::WriteOnly {
            Err(err!(
                "mapping is not `MapMode::WriteOnly`, but `write_only` is called"
            ))?
        }
        let end = offset + size_of::<T>() as DeviceSize;
        if self.range < end {
            Err(err!(
                "size of the mapped range `{}` is less than `offset` + size of `T` {end}",
                self.range,
            ))?
        }
        let bytes = NonNull::<[u8]>::slice_from_raw_parts(
            self.ptr,
            size_of::<T>(),
        );
        unsafe {
            WriteOnly::try_from_bytes(WriteOnly::from_non_null(bytes))
        }
    }

    /// Returns a [`WriteOnly`] slice over `range`.
    ///
    /// # Valid usage
    /// - The mode of this mapping **must** be [`WriteOnly`][MapMode::WriteOnly].
    /// - `range` a valid range and it **must** be in bounds.
    pub fn write_only_slice(&mut self, range: impl RangeBounds<usize>) -> Result<WriteOnly<'_, [u8]>, Error> {
        if self.mode != MapMode::WriteOnly {
            Err(err!(
                "mapping is not `MapMode::WriteOnly`, but `write_only_slice` is called"
            ))?
        }
        let (begin, end) = bounds_to_range(self.range as usize, range);
        if begin >= self.range as usize || end > self.range as usize {
            Err(err!(
                "range `{begin}..{end}` is out of bounds of `MappedBuffer`"
            ))?
        }
        unsafe {
            Ok(WriteOnly::from_non_null(NonNull::slice_from_raw_parts(
                self.ptr.add(begin),
                end - begin,
            )))
        }
    }

    /// Returns a read-only `[u8]` slice over `range`.
    ///
    /// # Valid usage
    /// - The mode of this mapping **must** be [`ReadOnly`][MapMode::ReadOnly].
    /// - `range` **must** be in bounds (panics otherwise).
    pub fn read_only(&self, range: impl RangeBounds<usize>) -> Result<&[u8], Error> {
        if self.mode != MapMode::ReadOnly {
            Err(err!(
                "mapping is not `MapMode::ReadOnly`, but `read_only` is called"
            ))?
        }
        let slice = unsafe {
            slice::from_raw_parts_mut(
                self.ptr.as_ptr(),
                self.range as usize,
            )
        };
        let (begin, end) = bounds_to_range(self.range as usize, range);
        Ok(&slice[begin..end])
    }
}
