mod allocator;

use core::{
    ffi,
    alloc::Layout,
    ptr::{self, NonNull},
};

use tuhka::vk;

pub(crate) use allocator::Alloc;

#[inline]
pub(crate) unsafe extern "system" fn allocate(
    p_user_data: *mut ffi::c_void,
    size: usize,
    align: usize,
    _: vk::SystemAllocationScope,
) -> *mut ffi::c_void {
    let allocator: &mut Alloc = unsafe {
        p_user_data.cast::<Alloc>()
            .as_mut_unchecked()
    };
    match Layout::from_size_align(size, align) {
        Ok(layout) => {
            allocator.allocate_raw(layout)
                .map_or_else(
                    || {
                        log::warn!("allocation failed");
                        ptr::null_mut()
                    },
                    |ptr| ptr.as_ptr().cast()
                )
        },
        Err(err) => {
            log::warn!("`Layout` error in `allocate`: {err}");
            ptr::null_mut()
        },
    }
}

#[inline]
pub(crate) unsafe extern "system" fn reallocate(
    p_user_data: *mut ffi::c_void,
    p_original: *mut ffi::c_void,
    size: usize,
    align: usize,
    system_allocation_scope: vk::SystemAllocationScope,
) -> *mut ffi::c_void {
    let allocator: &mut Alloc = unsafe {
        p_user_data.cast::<Alloc>()
            .as_mut_unchecked()
    };
    let Some(p_original) = NonNull::new(p_original.cast::<u8>()) else {
        return unsafe {
            allocate(p_user_data, size, align, system_allocation_scope)
        }
    };
    if size == 0 {
        unsafe {
            allocator.deallocate_raw(p_original);
        }
        return ptr::null_mut();
    }
    let Some(original_size) = allocator
        .get_allocation_size(p_original) else {
        log::warn!("invalid pointer {p_original:?} used in `reallocate`");
        return ptr::null_mut();
    };
    let layout = match Layout::from_size_align(size, align) {
        Ok(layout) => layout,
        Err(err) => {
            log::warn!("`Layout` error in `reallocate`: {err}");
            return ptr::null_mut();
        },
    };
    if size <= original_size {
        unsafe {
            allocator.shrink_raw(
                p_original,
                layout,
            )
        }.map_or_else(
            || {
                log::warn!("failed to shrink");
                ptr::null_mut()
            },
            |ptr| ptr.as_ptr().cast()
        )
    } else {
        unsafe {
            allocator.grow(
                p_original,
                layout,
            )
        }.map_or_else(|| {
            log::warn!("failed to grow");
            ptr::null_mut()
        }, |ptr| ptr.as_ptr().cast())
    }
}

#[inline]
pub(crate) unsafe extern "system" fn deallocate(
    p_user_data: *mut ffi::c_void,
    p_memory: *mut ffi::c_void,
) {
    let allocator: &mut Alloc = unsafe {
        p_user_data.cast::<Alloc>()
            .as_mut_unchecked()
    };
    let Some(non_null) = NonNull::new(p_memory) else {
        return;
    };
    unsafe {
        allocator.deallocate_raw(non_null.cast());
    }
}

#[inline]
pub(crate) fn make_callbacks<'a>(alloc: &'a mut Alloc) -> vk::AllocationCallbacks<'a> {
    vk::AllocationCallbacks {
        p_user_data: (alloc as *mut Alloc).cast::<ffi::c_void>(),
        pfn_allocation: allocate,
        pfn_reallocation: reallocate,
        pfn_free: deallocate,
        ..Default::default()
    }
}
