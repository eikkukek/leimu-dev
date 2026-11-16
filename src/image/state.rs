use bytemuck::{ByteEq, NoUninit, Zeroable};

use core::ops::{BitOr, BitOrAssign};

use super::*;

use sync::MutexGuard;

#[repr(C)]
#[derive(Clone, Copy, ByteEq, NoUninit, Zeroable)]
pub(crate) struct ImageState {
    pub stage: vk::PipelineStageFlags2,
    pub access: vk::AccessFlags2,
    pub layout: vk::ImageLayout,
    pub queue_family_index: u32,
}

#[inline]
fn bitor_layouts(
    lhs: vk::ImageLayout,
    rhs: vk::ImageLayout,
) -> vk::ImageLayout {
    if lhs == rhs { return lhs }
    if lhs == vk::ImageLayout::UNDEFINED 
    {
        return rhs
    }
    if rhs == vk::ImageLayout::UNDEFINED {
        panic!("invalid `rhs` layout `UNDEFIND`")
    }
    vk::ImageLayout::GENERAL
}

impl BitOr for ImageState {

    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            stage: self.stage | rhs.stage,
            access: self.access | rhs.access,
            layout: bitor_layouts(self.layout, rhs.layout),
            queue_family_index: rhs.queue_family_index,
        }
    }
}

impl BitOrAssign for ImageState {

    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

#[inline]
pub(super) fn memory_barrier(
    handle: vk::Image,
    src: &ImageState,
    dst: &ImageState,
    range: vk::ImageSubresourceRange,
) -> vk::ImageMemoryBarrier2<'static> {
    vk::ImageMemoryBarrier2 {
        src_stage_mask: src.stage,
        src_access_mask: src.access,
        dst_stage_mask: dst.stage,
        dst_access_mask: dst.access,
        old_layout: src.layout,
        new_layout: dst.layout,
        src_queue_family_index: src.queue_family_index,
        dst_queue_family_index: if src.queue_family_index == vk::QUEUE_FAMILY_IGNORED {
            vk::QUEUE_FAMILY_IGNORED
        } else { dst.queue_family_index },
        image: handle,
        subresource_range: range,
        ..default()
    }
}

pub(super) struct ImageRangeState {
    states: NonNull<ImageState>,
    aspects: ImageAspects,
    levels: u32,
    layers: u32,
}

impl ImageRangeState {

    #[inline]
    pub(super) unsafe fn new(
        states: NonNull<ImageState>,
        aspects: ImageAspects,
        levels: u32,
        layers: u32,
    ) -> Self {
        let s = Self { states, aspects, levels, layers };
        let len = s.len();
        unsafe { NonNull::slice_from_raw_parts(
            s.states,
            len
        ).as_mut() }.fill(ImageState {
            stage: vk::PipelineStageFlags2::NONE,
            access: vk::AccessFlags2::NONE,
            layout: vk::ImageLayout::UNDEFINED,
            queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        });
        s
    }

    #[inline]
    pub(super) unsafe fn new_scoped(
        states: NonNull<ImageState>,
        aspects: ImageAspects,
        levels: u32,
        layers: u32,
    ) -> Self {
        let mut s = Self { states, aspects, levels, layers };
        unsafe {
            s.zero_states();
        }
        s
    }

    pub fn calculate_allocation_size(
        aspects: ImageAspects,
        levels: u32,
        layers: u32
    ) -> usize {
        aspects.bits().count_ones() as usize *
        levels as usize *
        layers as usize * size_of::<ImageState>()
    }

    pub fn len(&self) -> usize {
        self.aspects.bits().count_ones() as usize *
        self.levels as usize *
        self.layers as usize
    }

    /// This should **only** be used for the `scoped` state.
    #[inline]
    pub(super) unsafe fn zero_states(&mut self) {
        unsafe {
            bytemuck::fill_zeroes(slice::from_raw_parts_mut(
                self.states.as_ptr(),
                self.len()
            ));
        }
    }

    pub unsafe fn bitor_state(
        &mut self,
        state: ImageState,
        range: vk::ImageSubresourceRange,
    ) {
        let len = self.len();
        let slice = unsafe { NonNull::slice_from_raw_parts(
            self.states,
            len
        ).as_mut() };
        let aspects = ImageAspects
            ::from_bits_retain(range.aspect_mask.as_raw());
        let level_count = self.levels as usize;
        let layer_count = self.layers as usize;
        for aspect in aspects.iter() {
            let aspect_idx = self.aspects
                .iter()
                .enumerate()
                .find_map(|(i, a)| (a == aspect).then_some(i))
                .unwrap();
            for level in range.base_mip_level..range.base_mip_level + range.level_count {
                let level = level as usize;
                for layer in range.base_array_layer..range.base_array_layer + range.layer_count {
                    let layer = layer as usize;
                    let index = aspect_idx * level_count * layer_count + level * layer_count + layer;
                    *unsafe {
                        slice.get_unchecked_mut(index)
                    } |= state;
                }
            }
        }
    }

    unsafe fn produce_barriers(
        &mut self,
        handle: vk::Image,
        scoped: &Self,
        mut collect_barrier: impl FnMut(vk::ImageMemoryBarrier2<'static>),
    ) {
        let len = self.len();
        debug_assert_eq!(len, scoped.len());
        let slice = unsafe { NonNull::slice_from_raw_parts(
            self.states,
            len,
        ).as_mut() };
        let scoped_slice = unsafe { NonNull::slice_from_raw_parts(
            scoped.states,
            len,
        ).as_mut() };
        let aspects = self.aspects;
        let level_count = self.levels as usize;
        let layer_count = self.layers as usize;
        let mut this_memory_barrier = None;
        for (aspect_idx, aspect) in aspects.iter().enumerate() {
            for level in 0..level_count {
                for layer in 0..layer_count {
                    let index = aspect_idx * level_count * layer_count + level * layer_count + layer;
                    unsafe {
                        let this_state = slice.get_unchecked_mut(index);
                        let scoped_state = scoped_slice.get_unchecked(index);
                        if (scoped_state.layout == vk::ImageLayout::UNDEFINED) | (this_state == scoped_state)
                        {
                            if let Some(b) = this_memory_barrier.take() {
                                collect_barrier(b);
                            }
                        } else if let Some(b) = &mut this_memory_barrier
                            && this_state == &(ImageState {
                                stage: b.src_stage_mask,
                                access: b.src_access_mask,
                                layout: b.old_layout,
                                queue_family_index: b.src_queue_family_index,
                            })
                            && b.subresource_range.aspect_mask.as_raw() == aspect.bits()
                        {
                            let range = &mut b.subresource_range;
                            range.level_count =
                                level as u32 - range.base_mip_level + 1;
                            range.layer_count =
                                layer as u32 - range.base_array_layer + 1;
                        } else {
                            if let Some(b) = this_memory_barrier.take() {
                                collect_barrier(b);
                            }
                            this_memory_barrier = Some(memory_barrier(
                                handle,
                                this_state,
                                scoped_state,
                                vk::ImageSubresourceRange {
                                    aspect_mask: vk::ImageAspectFlags::from_raw(aspect.bits()),
                                    base_mip_level: level as u32,
                                    level_count: 1,
                                    base_array_layer: layer as u32,
                                    layer_count: 1,
                                },
                            ));
                        }
                        *this_state = *scoped_state;
                    }
                }
            }
        }
        if let Some(b) = this_memory_barrier.take() {
            collect_barrier(b);
        }
    }
}

pub(crate) struct ImageScoped<'a> {
    pub(super) image: &'a Image,
    pub(super) lock: MutexGuard<'a, States>,
    pub(super) command_buffer: vk::CommandBuffer,
    pub(crate) signal_semaphore: Option<(vk::Semaphore, u64)>,
}

impl<'a> ImageScoped<'a> {

    pub(crate) unsafe fn bitor_state(
        &mut self,
        state: ImageState,
        range: vk::ImageSubresourceRange,
    ) {
        unsafe {
            self.lock.scoped_state
                .bitor_state(state, range);
        }
    }
}

impl<'a> Drop for ImageScoped<'a> {

    fn drop(&mut self) {
        unsafe {
            let lock = &mut *self.lock;
            let device = self.image.device().clone();
            let command_buffer = self.command_buffer;
            lock.state
                .produce_barriers(
                    self.image.handle(),
                    &lock.scoped_state,
                    |b| {
                        device
                            .functions()
                            .cmd_pipeline_barrier2(
                                command_buffer,
                                &vk::DependencyInfo::default()
                                    .image_memory_barriers(&[b])
                            );
                    },
                );
        }
    }
}
