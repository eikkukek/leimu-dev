use std::alloc::{self, Layout, Allocator};
use core::{
    ptr::NonNull,
    num::NonZeroUsize,
};

pub(crate) struct Blocks {
    pub ptr: NonNull<[u8]>,
    allocated_blocks: Vec<(NonNull<u8>, usize)>,
    largest_free: (NonNull<u8>, usize),
}

impl Blocks {

    pub fn new(
        ptr: NonNull<[u8]>,
        layout: Layout,
    ) -> Self {
        Self {
            ptr,
            allocated_blocks: vec![],
            largest_free: (ptr.cast(), layout.size()),
        }
    }
}

pub(crate) struct Inner {
    pub layout: Layout,
    pub blocks: Vec<Blocks>,
}

impl Inner {

    pub fn new(
        layout: Layout,
    ) -> Self {
        let ptr = alloc::Global
            .allocate(layout)
            .expect("`std::alloc::Global::allocate` failed");
        let block = Blocks::new(ptr, layout);
        Self {
            layout,
            blocks: vec![block],
        }
    }
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

pub struct Alloc {
    pub(crate) inner: Inner,
}

impl Drop for Alloc {

    #[inline]
    fn drop(&mut self) {
        let blocks = &mut self.inner.blocks;
        for b in blocks.iter() {
            unsafe {
                alloc::Global
                    .deallocate(b.ptr.cast(), self.inner.layout);
            }
        }
        blocks.clear();
    }
}

impl Alloc {

    pub fn new(
        max_allocation_size: usize,
        max_alignment: usize,
    ) -> Result<Self, alloc::LayoutError>{
        let layout = Layout::from_size_align(
            max_allocation_size,
            max_alignment,
        )?;
        Ok(Self {
            inner: Inner::new(layout),
        })
    }

    fn find_pointer_blocks(blocks: &mut [Blocks], ptr: NonNull<u8>) -> Option<&mut Blocks> {
        blocks.iter_mut()
            .find(|blocks| {
                ptr.addr() >= blocks.ptr.addr() &&
                ptr.addr().get() < blocks.ptr.addr().get() + blocks.ptr.len()
            })
    }

    fn allocate_raw_internal(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        let all_blocks = &mut self.inner.blocks;
        let block_layout = self.inner.layout;
        if layout.size() > block_layout.size() ||
            layout.align() > block_layout.align()
        {
            None?
        }
        let mut prev_end: Option<NonZeroUsize> = None;
        let mut last_end: Option<NonZeroUsize> = None;
        let mut added_offset = 0;
        let mut block_and_idx = None;
        for blocks in all_blocks
            .iter_mut()
            .filter(|b| b.largest_free.1 >= layout.size())
        {
            if let Some(idx) = blocks
                .allocated_blocks
                .iter()
                .copied()
                .chain([(unsafe { blocks.ptr.cast().add(blocks.ptr.len()) }, 0)])
                .enumerate()
                .find_map(|(i, (ptr, size))| {
                    let free_size = if let Some(prev) = prev_end {
                        last_end = prev_end;
                        NonZeroUsize::new(ptr.addr().get() - prev.get())
                    } else {
                        NonZeroUsize::new(ptr.addr().get()
                            - blocks.ptr.addr().get()
                        )
                    };
                    let Some(free_size) = free_size else {
                        last_end = Some(unsafe { ptr.add(size).addr() });
                        prev_end = NonZeroUsize::new(ptr.addr().get() + size);
                        return (layout.size() == 0).then_some(i);
                    };
                    added_offset = NonNull::<u8>::without_provenance(prev_end.unwrap_or(
                        NonZeroUsize::new(layout.align()).unwrap()
                    )).align_offset(layout.align());
                    prev_end = NonZeroUsize::new(ptr.addr().get() + size);
                    if free_size.get() < added_offset {
                        None?
                    }
                    (free_size.get() - added_offset >= layout.size())
                        .then_some(i)
                })
            {
                block_and_idx = Some((blocks, idx));
                break;
            } else if blocks.allocated_blocks.is_empty() {
                block_and_idx = Some((blocks, 0));
                break;
            }
        }
        let (blocks, idx) = match block_and_idx {
            Some(b) => b,
            None => {
                let ptr = alloc::Global
                    .allocate(block_layout)
                    .expect("`std::alloc::Global::allocate` failed");
                all_blocks.push(Blocks::new(ptr, block_layout));
                unsafe {
                    (
                        all_blocks.last_mut().unwrap_unchecked(),
                        0,
                    )
                }
            },
        };
        let unaligned_ptr = blocks.ptr.cast().with_addr(last_end
            .unwrap_or_else(|| { blocks.ptr.addr()} )
        );
        let aligned_pointer = unsafe { unaligned_ptr.add(added_offset) };
        blocks.allocated_blocks
            .insert(
                idx,
                (aligned_pointer, layout.size()),
            );
        if unaligned_ptr.addr() == blocks.largest_free.0.addr() {
            let mut iter = blocks
                .allocated_blocks
                .iter()
                .copied()
                .chain(unsafe { [(blocks.ptr.cast().add(blocks.ptr.len()), 0)] });
            let mut prev = iter.next().unwrap();
            let largest_free = iter
                .map(|ptr_size| {
                    let ptr = unsafe {
                        prev.0.add(prev.1)
                    };
                    let free = ptr_size.0.addr().get() - ptr.addr().get();
                    prev = ptr_size;
                    (ptr, free)
                }).max_by_key(|&(_, size)| size)
                .unwrap();
            blocks.largest_free = largest_free;
        }
        Some(NonNull::slice_from_raw_parts(
            aligned_pointer,
            layout.size(),
        ))
    }

    #[inline]
    pub(crate) fn get_allocation_size(&mut self, ptr: NonNull<u8>) -> Option<usize> {
        let all_blocks = &mut self.inner.blocks;
        let blocks = Self::find_pointer_blocks(all_blocks, ptr)?;
        let part = blocks.allocated_blocks
            .partition_point(|&x| x.0.addr() < ptr.addr());
        Some(blocks.allocated_blocks.get(part)?.1)
    }

    #[inline]
    pub(crate) fn allocate_raw(
        &mut self,
        layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        self.allocate_raw_internal(layout)
    }

    unsafe fn deallocate_raw_internal(
        &mut self,
        ptr: NonNull<u8>,
    ) {
        let all_blocks = &mut self.inner.blocks;
        let blocks = Self::find_pointer_blocks(
            all_blocks,
            ptr
        ).unwrap();
        let part = blocks.allocated_blocks
            .partition_point(|&x| x.0.addr() < ptr.addr());
        let block = blocks.allocated_blocks.remove(part);
        let left = part.checked_sub(1).map(|i| blocks.allocated_blocks[i])
            .unwrap_or((blocks.ptr.cast(), 0));
        let right = blocks.allocated_blocks
            .get(part)
            .map(|&(ptr, _)| ptr)
            .unwrap_or(unsafe { blocks.ptr.cast().add(blocks.ptr.len()) });
        let left_end = unsafe { left.0.add(left.1) };
        let freed_left = block.0.addr().get() - left_end.addr().get();
        let freed_right = right.addr().get() - block.0.addr().get();
        let freed_total = freed_right + freed_left;
        if freed_total > blocks.largest_free.1
        {
            blocks.largest_free = (left_end, freed_total);
        }
    }

    #[inline]
    pub(crate) unsafe fn deallocate_raw(
        &mut self,
        ptr: NonNull<u8>,
    ) {
        unsafe {
            self.deallocate_raw_internal(ptr);
        }
    }

    pub(crate) unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        new_layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        let blocks = Self
            ::find_pointer_blocks(&mut self.inner.blocks, ptr)
            .unwrap();
        let part = blocks.allocated_blocks
            .partition_point(|&x| x.0.addr() < ptr.addr());
        let (ptr, size) = blocks.allocated_blocks[part];
        assert!(new_layout.size() >= size, "`new_layout.size() < old_layout.size()`");
        assert!((size as *const u8).align_offset(new_layout.align()) == 0,
            "`new_layout.align() != old_layout.align()`",
        );
        let new_ptr = self.allocate_raw_internal(new_layout)?;
        unsafe {
            ptr.copy_to_nonoverlapping(
                new_ptr.cast(),
                size,
            );
        }
        unsafe {
            self.deallocate_raw_internal(ptr);
        }
        Some(new_ptr)
    }

    pub(crate) unsafe fn shrink_raw(
        &mut self,
        ptr: NonNull<u8>,
        new_layout: Layout,
    ) -> Option<NonNull<[u8]>> {
        let all_blocks = &mut self.inner.blocks;
        if new_layout.size() == 0 {
            unsafe {
                self.deallocate_raw_internal(ptr.cast());
            }
            return None
        }
        let blocks = Self
            ::find_pointer_blocks(all_blocks, ptr)
            .unwrap();
        let part = blocks.allocated_blocks
            .partition_point(|&x| x.0.addr() < ptr.addr());
        let (ptr, size) = &mut blocks.allocated_blocks[part];
        assert!(new_layout.size() <= *size, "`new_layout.size() > old_layout.size()`");
        assert!((*size as *const u8).align_offset(new_layout.align()) == 0,
            "`new_layout.align() != old_layout.align()`",
        );
        *size = new_layout.size();
        let (ptr, size) = (*ptr, *size);
        if let Some((start, size)) = blocks.allocated_blocks.get(part + 1)
            .map(|next_block| {
                let end = NonZeroUsize::new(ptr.addr().get() + size).unwrap();
                (blocks.ptr.cast::<u8>().with_addr(end), next_block.0.addr().get() - end.get())
            })
            && size > blocks.largest_free.1
        {
            blocks.largest_free = (start, size);
        }
        Some(NonNull::slice_from_raw_parts(
            ptr,
            new_layout.size(),
        ))
    }

    pub fn log_stats(&self) {
        #[derive(Debug)]
        struct Stats {
            used_blocks: usize,
            used_space: usize,
        }
        let stats = Stats {
            used_blocks: self.inner.blocks.len(),
            used_space: self.inner.blocks
                .iter()
                .map(|b| self.inner.layout.size() - b.largest_free.1)
                .sum(),
        };
        log::info!("allocator stats: {stats:?}")
    }
}

#[cfg(test)]
#[test]
fn test() {
    let mut alloc = Alloc::new(
        64,
        8
    ).unwrap();
    {
        let ptr_1 = alloc.allocate_raw(Layout::new::<u8>()).unwrap();
        let ptr_2 = alloc.allocate_raw(Layout::new::<String>()).unwrap();
        assert!(ptr_1.addr() < ptr_2.addr());
        unsafe {
            alloc.deallocate_raw(ptr_2.cast());
        }
        assert_eq!(alloc.inner.blocks[0].largest_free.1, 63);
        unsafe {
            alloc.deallocate_raw(ptr_1.cast());
        }
    }
    assert_eq!(alloc.inner.blocks[0].largest_free.1, 64);
    {
        let ptr1 = alloc
            .allocate_raw(Layout::new::<u64>())
            .unwrap();
        assert_eq!(alloc.inner.blocks[0].largest_free.1, 56);
        let ptr2 = alloc
            .allocate_raw(Layout::new::<u64>())
            .unwrap();
        assert_eq!(alloc.inner.blocks[0].largest_free.1, 48);
        unsafe {
            alloc.deallocate_raw(ptr1.cast());
            assert_eq!(alloc.inner.blocks[0].largest_free.1, 48);
            alloc.deallocate_raw(ptr2.cast());
            assert_eq!(alloc.inner.blocks[0].largest_free.1, 64);
        }
    }
    assert_eq!(alloc.inner.blocks[0].largest_free.1, 64);
    let ptr_1 = alloc.allocate_raw(Layout::array::<u8>(64).unwrap())
        .unwrap();
    let ptr_2 = alloc.allocate_raw(Layout::array::<u8>(64).unwrap())
        .unwrap();
    assert_eq!(alloc.inner.blocks[0].largest_free.1, 0);
    assert_eq!(alloc.inner.blocks[1].largest_free.1, 0);
    unsafe {
        alloc.deallocate_raw(ptr_1.cast());
    }
    let ptr_1 = alloc.allocate_raw(Layout::array::<u8>(32).unwrap()).unwrap();
    assert_eq!(alloc.inner.blocks[0].largest_free.1, 32);
    unsafe {
        alloc.deallocate_raw(ptr_2.cast());
    }
    assert_eq!(alloc.inner.blocks[1].largest_free.1, 64);
    assert!(alloc.allocate_raw(Layout::new::<u128>()).is_none());
    unsafe {
        alloc.deallocate_raw(ptr_1.cast());
    }
    let mut ptr = alloc.allocate_raw(Layout::array::<u64>(2).unwrap()).unwrap();
    unsafe {
        ptr = alloc.shrink_raw(ptr.cast(), Layout::array::<u64>(1).unwrap()).unwrap();
        ptr = alloc.grow(ptr.cast(), Layout::array::<u64>(2).unwrap()).unwrap();
    }
    unsafe {
        alloc.deallocate_raw(ptr.cast());
    }
    assert_eq!(alloc.inner.blocks[0].largest_free.1, 64);
}
