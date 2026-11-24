use std::alloc::Global;

use core::{
    ptr::{self, NonNull},
    ffi,
    alloc,
    marker::PhantomData,
};

use crate::*;

use tuhka::vk;

use sync::{
    Mutex,
    LazyLock,
    Arc,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct AllocationKey(*const ffi::c_void);

unsafe impl Send for AllocationKey {}

static ALLOCATIONS: LazyLock<Mutex<HashMap<AllocationKey, alloc::Layout>>> = LazyLock::new(|| {
    Mutex::default()
});

unsafe extern "system" fn pfn_allocation(
    _p_user_data: *mut ffi::c_void,
    size: usize,
    alignment: usize,
    _allocation_scope: vk::SystemAllocationScope
) -> *mut ffi::c_void {
    let mut lock = ALLOCATIONS.lock();
    let (ptr, layout) = match alloc::Layout::from_size_align(size, alignment) {
        Ok(layout) => match Global.allocate(layout) {
            Ok(ptr) => (ptr.cast(), layout),
            Err(_) => return ptr::null_mut(),
        },
        Err(_) => return ptr::null_mut(),
    };
    let ptr = ptr.as_ptr();
    lock.insert(AllocationKey(ptr), layout);
    ptr
}

unsafe extern "system" fn pfn_reallocation(
    p_user_data: *mut ffi::c_void,
    p_original: *mut ffi::c_void,
    size: usize,
    alignment: usize,
    allocation_scope: vk::SystemAllocationScope
) -> *mut ffi::c_void {
    if p_original.is_null() {
        return unsafe {
            pfn_allocation(p_user_data, size, alignment, allocation_scope)
        }
    }
    if size == 0 {
        todo!()
    }
    let mut lock = ALLOCATIONS.lock();
    let &old_layout = lock.get(&AllocationKey(p_original))
        .unwrap();
    assert!(old_layout.align() == alignment);
    let (new_ptr, new_layout) = if size > old_layout.size() {
        unsafe {
            match alloc::Layout::from_size_align(size, alignment) {
                Ok(new_layout) => match Global.grow(
                    NonNull::new_unchecked(p_original.cast()),
                    old_layout,
                    new_layout
                ) {
                    Ok(ptr) => (ptr.cast(), new_layout),
                    Err(_) => return ptr::null_mut(),
                },
                Err(_) => return ptr::null_mut(),
            }
        }
    } else {
        unsafe {
            match alloc::Layout::from_size_align(size, alignment) {
                Ok(new_layout) => match Global.shrink(
                    NonNull::new_unchecked(p_original.cast()),
                    old_layout,
                    new_layout
                ) {
                    Ok(ptr) => (ptr.cast(), new_layout),
                    Err(_) => return ptr::null_mut(),
                },
                Err(_) => return ptr::null_mut(),
            }
        }
    };
    let new_ptr = new_ptr.as_ptr();
    lock.remove(&AllocationKey(p_original));
    lock.insert(AllocationKey(new_ptr), new_layout);
    new_ptr
}

unsafe extern "system" fn pfn_free(
    _p_user_data: *mut ffi::c_void,
    p_memory: *mut ffi::c_void
) {
    let mut lock = ALLOCATIONS.lock();
    if p_memory.is_null() {
        return;
    }
    if let Some(layout) = lock.remove(&AllocationKey(p_memory)) {
        unsafe {
            Global.deallocate(
                NonNull::new_unchecked(p_memory.cast()),
                layout
            );
        }
    }
}

unsafe extern "system" fn pfn_internal_allocation(
    _: *mut ffi::c_void,
    _: usize,
    _: vk::InternalAllocationType,
    _: vk::SystemAllocationScope,
) {}

unsafe extern "system" fn pfn_internal_free(
    _: *mut ffi::c_void,
    _: usize,
    _: vk::InternalAllocationType,
    _: vk::SystemAllocationScope,
) {}

pub(super) static ALLOCATION_CALL_BACKS: vk::AllocationCallbacks<'static> = vk::AllocationCallbacks {
    p_user_data: ::core::ptr::null_mut(),
    pfn_allocation,
    pfn_reallocation,
    pfn_free,
    pfn_internal_allocation,
    pfn_internal_free,
    _marker: PhantomData,
};
