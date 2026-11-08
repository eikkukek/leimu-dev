use bitflags::bitflags;

use bytemuck::{CheckedBitPattern, NoUninit, ByteEq, ByteHash};

use tuhka::vk;

use crate::*;

/// 32-bit flags.
pub type Flags32 = u32;
/// 64-bit flags.
pub type Flags64 = u64;

bitflags! {
    /// Bitmask that controls triangle culling.
    #[repr(transparent)]
    #[derive(Default, Clone, Copy, CheckedBitPattern, ByteEq, ByteHash, Debug)]
    pub struct CullMode: Flags32 {
        /// Specifies that no triangles are discarded.
        const NONE = vk::CullModeFlags::NONE.as_raw();
        /// Specifies that the front-facing triangles are discarded.
        const FRONT = vk::CullModeFlags::FRONT.as_raw();
        /// Specifies that the back-facing triangles are discarded.
        const BACK = vk::CullModeFlags::BACK.as_raw();
        /// Specifies that the all triangles are discarded.
        ///
        /// Equal to [`Self::all()`].
        const FRONT_AND_BACK = vk::CullModeFlags::FRONT_AND_BACK.as_raw();
    }
}
unsafe impl NoUninit for CullMode {}
__bitflags_display!(CullMode);

impl From<CullMode> for vk::CullModeFlags {

    #[inline]
    fn from(value: CullMode) -> Self {
        Self::from_raw(value.bits())
    }
}

bitflags! {

    /// Bitmask of buffer usages.
    ///
    /// If this bitmask contains bits that don't fit in a 32-bit range, the `maintenance5` feature
    /// **must** be enabled (either explicitly or implicilty).
    #[repr(transparent)]
    #[derive(Clone, Copy, CheckedBitPattern, ByteEq, ByteHash, Debug)]
    pub struct BufferUsages: Flags64 {
        /// Specifies that the buffer **can** be used as the source of a transfer command.
        const TRANSFER_SRC = vk::BufferUsageFlags2::TRANSFER_SRC.as_raw();
        /// Specifies that the buffer **can** be used as the destination of a transfer command.
        const TRANSFER_DST = vk::BufferUsageFlags2::TRANSFER_DST.as_raw();
        /// Specifies that the buffer **can** be used as a uniform texel buffer.
        const UNIFORM_TEXEL_BUFFER = vk::BufferUsageFlags2::UNIFORM_TEXEL_BUFFER.as_raw();
        /// Specifies that the buffer **can** be used as a storage texel buffer.
        const STORAGE_TEXEL_BUFFER = vk::BufferUsageFlags2::STORAGE_TEXEL_BUFFER.as_raw();
        /// Specifies that the buffer **can** be used as a uniform buffer.
        const UNIFORM_BUFFER = vk::BufferUsageFlags2::UNIFORM_BUFFER.as_raw();
        /// Specifies that the buffer **can** be used as a storage buffer.
        const STORAGE_BUFFER = vk::BufferUsageFlags2::STORAGE_BUFFER.as_raw();
        /// Specifies that the buffer **can** be used as an index buffer.
        const INDEX_BUFFER = vk::BufferUsageFlags2::INDEX_BUFFER.as_raw();
        /// Specifies that the buffer **can** be used as a vertex buffer.
        const VERTEX_BUFFER = vk::BufferUsageFlags2::VERTEX_BUFFER.as_raw();
        /// Specifies that the buffer **can** be used as the buffer in indirect drawing commands.
        const INDIRECT_BUFFER = vk::BufferUsageFlags2::INDIRECT_BUFFER.as_raw();
        /// Specifies that the buffer's address **can** be queried and used in shader to access the
        /// buffer's memory through that address.
        ///
        /// The address **can** be stored as a `PhysicalStorageBuffer` storage class pointer in a SPIR-V
        /// module.
        const SHADER_DEVICE_ADDRESS = vk::BufferUsageFlags2::SHADER_DEVICE_ADDRESS.as_raw();
        /// Specifies that the buffer **can** be used as a descriptor heap.
        ///
        /// Requires that the `descriptor_heap` feature is enabled.
        const DESCRIPTOR_HEAP_EXT = vk::BufferUsageFlags2::DESCRIPTOR_HEAP_EXT.as_raw();
    }
}
unsafe impl NoUninit for BufferUsages {}
__bitflags_display!(BufferUsages);

impl BufferUsages {

    /// Gets the bits of self as [`u32`].
    ///
    /// If the set bits don't fit in the 32-bit range, [`None`] is returned.
    #[inline]
    pub fn as_32_bit(&self) -> Option<u32> {
        let bits = self.bits();
        (bits <= u32::MAX as u64).then_some(bits as u32)
    }

    /// Gets the bits of self as [`u64`].
    #[inline]
    pub fn as_64_bit(&self) -> u64 {
        self.bits()
    }
}

bitflags! {

    /// Specifies what an image **can** be used for.
    #[repr(transparent)]
    #[derive(Clone, Copy, CheckedBitPattern, ByteEq, ByteHash, Debug)]
    pub struct ImageUsages: Flags32 {
        /// Specifies that the image **can** be used as the source of a transfer command.
        const TRANSFER_SRC = vk::ImageUsageFlags::TRANSFER_SRC.as_raw();
        /// Specifies that the image **can** be used as the destination of a transfer command.
        const TRANSFER_DST = vk::ImageUsageFlags::TRANSFER_DST.as_raw();
        /// Specifies that the image **can** be used with desciptor types [`SampledImage`][1] and
        /// [`ImageSampler`][2].
        ///
        /// [1]: DescriptorType::SampledImage
        /// [2]: DescriptorType::ImageSampler
        const SAMPLED = vk::ImageUsageFlags::SAMPLED.as_raw();
        /// Specifies that the image **can** be used with descriptor type [`StorageImage`][1].
        ///
        /// [1]: DescriptorType::StorageImage
        const STORAGE = vk::ImageUsageFlags::STORAGE.as_raw();
        /// Specifies that the image **can** be used as a color attachment.
        const COLOR_ATTACHMENT  = vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw();
        /// Specifies that the image **can** be used as a depth/stencil attachment.
        const DEPTH_STENCIL_ATTACHMENT = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT.as_raw();
        /// Specifies that the image **may** be backed by lazily allocated memory.
        const TRANSIENT_ATTACHMENT    = vk::ImageUsageFlags::TRANSIENT_ATTACHMENT.as_raw();
        /// Specifies that the image **can** be used with descriptor type [`InputAttachment`][1].
        ///
        /// [1]: DescriptorType::InputAttachment
        const INPUT_ATTACHMENT = vk::ImageUsageFlags::INPUT_ATTACHMENT.as_raw();
        /// Specifies that the image **can** be used with host copy commands and host layout
        /// transitions.
        ///
        /// Requires enabling the [`host_image_copy`][vk_core::host_image_copy] feature.
        const HOST_TRANSFER = vk::ImageUsageFlags::HOST_TRANSFER.as_raw();
    }
}
unsafe impl NoUninit for ImageUsages {}
__bitflags_display!(ImageUsages);

bitflags! {

    /// Specifies supported image sample counts.
    #[repr(transparent)]
    #[derive(Clone, Copy, CheckedBitPattern, ByteEq, ByteHash, Debug)]
    pub struct SampleCounts: Flags32 {
        /// 1 sample per pixel.
        const BIT_1  = 0x1;
        /// 2 samples per pixel.
        const BIT_2  = 0x2;
        /// 4 samples per pixel.
        const BIT_4  = 0x4;
        /// 8 samples per pixel.
        const BIT_8  = 0x8;
        /// 16 samples per pixel.
        const BIT_16 = 0x10;
        /// 32 samples per pixel.
        const BIT_32 = 0x20;
        /// 64 samples per pixel.
        const BIT_64 = 0x40;
    }
}
unsafe impl NoUninit for SampleCounts {}
__bitflags_display!(SampleCounts);
bitflags! {
    /// Specifies color components to write to the framebuffer.
    #[repr(transparent)]
    #[derive(Clone, Copy, CheckedBitPattern, ByteEq, ByteHash, Debug)]
    pub struct ColorComponents: Flags32 {
        /// Specifies that the `R` component is written to the color attachment.
        const R = 0x1;
        /// Specifies that the `G` component is written to the color attachment.
        const G = 0x2;
        /// Specifies that the `B` component is written to the color attachment.
        const B = 0x4;
        /// Specifies that the `A` component is written to the color attachment.
        const A = 0x8;
    }
}
impl ColorComponents {

    /// Specifies that no components are written to the color attachment.
    pub const NONE: Self = Self::empty();
    /// Specifies that all components are written to the color attachment.
    pub const RGBA: Self = Self::all();
}
unsafe impl NoUninit for ColorComponents {}
__bitflags_display!(ColorComponents);
