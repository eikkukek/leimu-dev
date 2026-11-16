use bytemuck::{Zeroable, ByteEq, ByteHash, NoUninit};

use arrayvec::ArrayVec;

use tuhka::vk;

use crate::*;

#[derive(Clone, Copy)]
pub(crate) struct MemoryAllocation {
    pub handle: vk::DeviceMemory,
    pub offset: DeviceSize,
    pub size: DeviceSize,
    pub memory_type_index: u32,
}
/// Specifies what type of memory a buffer or an image is backed by.
///
/// The default value is [`DeviceLocal`][Self::DeviceLocal].
#[repr(u32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum MemoryType {
    #[default]
    /// Specifies that the backed memory of this object is local to the GPU, and **may**
    /// not be mapped. Memory of this type is most efficient for device access.
    ///
    /// If you want device local memory that's also to be mappable, you can specify the
    /// [`HostVisiblePreferDeviceLocal`][1] memory type.
    ///
    /// This is the default value.
    ///
    /// [1]: Self::HostVisiblePreferDeviceLocal
    DeviceLocal = 0,
    /// Specifies that the memory is visible to the host and **can** be mapped. This doesn't have
    /// a preference as to whether the memory is also [`DeviceLocal`][1].
    ///
    /// [1]: Self::DeviceLocal
    HostVisible,
    /// This is the same as [`HostVisible`][1], but prefers memory types that are **not** also device
    /// local.
    ///
    /// [1]: Self::HostVisible
    HostVisiblePreferNonDeviceLocal,
    /// This is the same as [`HostVisible`][1], but prefers memory types that **are** also device
    /// local.
    ///
    /// [1]: Self::HostVisible
    HostVisiblePreferDeviceLocal,
}
unsafe impl NoUninit for MemoryType {}
impl MemoryType {

    pub(crate) fn get_memory_type_indices(
        self,
        gpu_info: &GpuInfo
    ) -> ArrayVec<u32, {vk::MAX_MEMORY_TYPES as usize}> {
        let memory_properties = &gpu_info.memory_properties;
        let memory_types = &memory_properties.memory_types[0..memory_properties.memory_type_count as usize];
        match self {
            MemoryType::DeviceLocal => {
                memory_types
                    .iter()
                    .enumerate()
                    .filter_map(|(i, ty)| {
                        ty.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
                            .then_some(i as u32)
                    }).collect()
            },
            MemoryType::HostVisible => {
                memory_types
                    .iter()
                    .enumerate()
                    .filter_map(|(i, ty)| {
                        ty.property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
                            .then_some(i as u32)
                    }).collect()
            },
            MemoryType::HostVisiblePreferNonDeviceLocal => {
                let mut preferred = ArrayVec::<_, {vk::MAX_MEMORY_TYPES as usize}>::new();
                let mut suboptimal= ArrayVec::<_, {vk::MAX_MEMORY_TYPES as usize}>::new();
                for (i, ty) in memory_types
                    .iter()
                    .enumerate()
                    .filter(|(_, ty)| {
                        ty.property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
                    })
                {
                    if !ty.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) {
                        preferred.push(i as u32);
                    } else {
                        suboptimal.push(i as u32);
                    }
                }
                [preferred, suboptimal]
                    .into_iter()
                    .flatten()
                    .collect()
            },
            MemoryType::HostVisiblePreferDeviceLocal => {
                let mut preferred = ArrayVec::<_, {vk::MAX_MEMORY_TYPES as usize}>::new();
                let mut suboptimal= ArrayVec::<_, {vk::MAX_MEMORY_TYPES as usize}>::new();
                for (i, ty) in memory_types
                    .iter()
                    .enumerate()
                    .filter(|(_, ty)| {
                        ty.property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
                    })
                {
                    if ty.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL) {
                        preferred.push(i as u32);
                    } else {
                        suboptimal.push(i as u32);
                    }
                }
                [preferred, suboptimal]
                    .into_iter()
                    .flatten()
                    .collect()
            },
        }
    }
}
