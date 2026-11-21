use bitflags::bitflags;

use bytemuck::{Zeroable, NoUninit, ByteEq, ByteHash};

use tuhka::vk;

use crate::*;

/// 32-bit flags.
pub type Flags32 = u32;
/// 64-bit flags.
pub type Flags64 = u64;

bitflags! {
    /// Bitmask that controls triangle culling.
    #[repr(transparent)]
    #[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash, Debug)]
    pub struct CullMode: Flags32 {
        /// Specifies that no triangles are discarded.
        const NONE = 0x0;
        /// Specifies that the front-facing triangles are discarded.
        const FRONT = 0x1;
        /// Specifies that the back-facing triangles are discarded.
        const BACK = 0x2;
        /// Specifies that the all triangles are discarded.
        ///
        /// Equal to [`Self::all()`].
        const FRONT_AND_BACK = 0x1 | 0x2;
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
    /// If this bitmask contains bits that don't fit in a 32-bit range, the [`maintenance5`][1] feature
    /// **must** be enabled.
    ///
    /// [1]: vk_14::maintenance5
    #[repr(transparent)]
    #[derive(Clone, Copy, ByteEq, ByteHash, Debug)]
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
    #[derive(Clone, Copy, ByteEq, ByteHash, Debug)]
    pub struct ImageUsages: Flags32 {
        /// Specifies that the image **can** be used as the source of a transfer command.
        const TRANSFER_SRC = 0x1;
        /// Specifies that the image **can** be used as the destination of a transfer command.
        const TRANSFER_DST = 0x2;
        /// Specifies that the image **can** be used with desciptor types [`SampledImage`][1] and
        /// [`ImageSampler`][2].
        ///
        /// [1]: DescriptorType::SampledImage
        /// [2]: DescriptorType::ImageSampler
        const SAMPLED = 0x4;
        /// Specifies that the image **can** be used with descriptor type [`StorageImage`][1].
        ///
        /// [1]: DescriptorType::StorageImage
        const STORAGE = 0x8;
        /// Specifies that the image **can** be used as a color attachment.
        const COLOR_ATTACHMENT = 0x10;
        /// Specifies that the image **can** be used as a depth/stencil attachment.
        const DEPTH_STENCIL_ATTACHMENT = 0x20;

        /* unsupported for now
        /// Specifies that the image **may** be backed by lazily allocated memory.
        const TRANSIENT_ATTACHMENT = 0x40;
        */

        /* no supported for now
        /// Specifies that the image **can** be used with descriptor type [`InputAttachment`][1].
        ///
        /// [1]: DescriptorType::InputAttachment
        // const INPUT_ATTACHMENT = vk::ImageUsageFlags::INPUT_ATTACHMENT.as_raw();
        */

        /* not supported for now
        /// Specifies that the image **can** be used with host copy commands and host layout
        /// transitions.
        ///
        /// Requires enabling the [`host_image_copy`][vk_core::host_image_copy] feature.
        const HOST_TRANSFER = vk::ImageUsageFlags::HOST_TRANSFER.as_raw();
        */
    }
}
unsafe impl NoUninit for ImageUsages {}
__bitflags_display!(ImageUsages);
impl From<ImageUsages> for vk::ImageUsageFlags {
    #[inline]
    fn from(value: ImageUsages) -> Self {
        Self::from_raw(value.bits())
    }
}
bitflags! {
    /// Specifies supported image sample counts.
    #[repr(transparent)]
    #[derive(Clone, Copy, ByteEq, ByteHash, Debug)]
    pub struct SampleCounts: Flags32 {
        /// 1 sample per pixel.
        const X1  = 0x1;
        /// 2 samples per pixel.
        const X2  = 0x2;
        /// 4 samples per pixel.
        const X4  = 0x4;
        /// 8 samples per pixel.
        const X8  = 0x8;
        /// 16 samples per pixel.
        const X16 = 0x10;
        /// 32 samples per pixel.
        const X32 = 0x20;
        /// 64 samples per pixel.
        const X64 = 0x40;
    }
}
unsafe impl NoUninit for SampleCounts {}
__bitflags_display!(SampleCounts);
bitflags! {
    /// Specifies supported resolve modes.
    #[repr(transparent)]
    #[derive(Clone, Copy, ByteEq, ByteHash, Debug)]
    pub struct ResolveModeFlags: Flags32 {
        /// Takes the value of the sample at index `0`.
        const SAMPLE_ZERO   = 0x1;
        /// Takes the average of all samples.
        const AVERAGE       = 0x2;
        /// Takes the minimum of all samples.
        const MIN           = 0x4;
        /// Takes the maximum of all samples.
        const MAX           = 0x8;
    }
}
unsafe impl NoUninit for ResolveModeFlags {}
__bitflags_display!(ResolveModeFlags);
bitflags! {
    /// Specifies color components to write to the framebuffer.
    #[repr(transparent)]
    #[derive(Clone, Copy, ByteEq, ByteHash, Debug)]
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
bitflags! {
    /// A bitmask specifying [`Image`] aspects.
    #[repr(transparent)]
    #[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash, Debug)]
    pub struct ImageAspects: Flags32 {
        /// Specifies all available aspects of an image.
        ///
        /// This is equal to [`Self::empty()`] and is the default value.
        const DERIVE = 0x0;
        /// Specifies the color aspect of an image.
        const COLOR   = 0x1;
        /// Specifies the depth aspect of an image.
        const DEPTH   = 0x2;
        /// Specifies the stencil aspect of an image.
        const STENCIL = 0x4;
    }
}
unsafe impl NoUninit for ImageAspects {}
__bitflags_display!(ImageAspects);
bitflags! {
    /// A bitmask of supported alpha composite modes.
    pub struct CompositeAlphaFlags: Flags32 {
        /// Specifies that the alpha component is ignored and the image is treated as if it has a
        /// constantt alpha of `1.0`.
        const OPAQUE = 0x1;
        /// Specifies that non-alpha components of the image are expected to already be multiplied by the alpha
        /// component by the application.
        const PRE_MULTIPLIED = 0x2;
        /// Specifies that non-alpha components of the image are not expected to already be multiplied by the
        /// alpha component by the application; instead, the compositor will multiply the non-alpha
        /// components of the image by the alpha component during compositing.
        const POST_MULTIPLIED = 0x4;
        /// The way in which the presentation engine treats the alpha component in the images is
        /// unknown to the Vulkan API. Instead, the application is responsible for setting the
        /// composite alpha blending mode using native window system commands. If the application
        /// does not set the blending mode using native window system commands, then a
        /// platform-specific default will be use.
        const INHERIT = 0x8;
    }
}
bitflags! {
    /// A bitmask specifying which subgroup operations are supppported in a shader.
    #[repr(transparent)]
    #[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash, Debug)]
    pub struct SubgroupFeatures: Flags32 {
        /// Specifies that the `GroupNonUniform` SPIR-V capability is supported.
        const BASIC = 0x1;
        /// Specifies that the `GroupNonUniformVote` SPIR-V capability is supported.
        const VOTE = 0x2;
        /// Specifies that the `GroupNonUniformArithmetic` SPIR-V capability is supported.
        const ARITHMETIC = 0x4;
        /// Specifies that the `GroupNonUniformBallot` SPIR-V capability is supported.
        const BALLOT = 0x8;
        /// Specifies that the `GroupNonUniformShuffle` SPIR-V capability is supported.
        const SHUFFLE = 0x10;
        /// Specifies that the `GroupNonUniformShuffleRelative` SPIR-V capability is supported.
        const SHUFFLE_RELATIVE = 0x20;
        /// Specifies that the `GroupNonUniformClustered` SPIR-V capability is supported.
        const CLUSTERED = 0x40;
        /// Specifies that the `GroupNonUniformQuad` SPIR-V capability is supported.
        const QUAD = 0x80;
        /// Specifies that the `GroupNonRotateKHR` SPIR-V capability is supported.
        const ROTATE = 0x200;
        /// Specifies that SPIR-V modules **can** use the `ClusterSize` operand to `OpGroupNonUniformRotateKHR`.
        const ROTATE_CLUSTERED = 0x400;
    }
}
unsafe impl NoUninit for SubgroupFeatures {}
__bitflags_display!(SubgroupFeatures);
bitflags! {
    /// A bitmask specifying supported descriptor indexing features.
    #[repr(transparent)]
    #[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash, Debug)]
    pub struct DescriptorIndexingFeatures: Flags32 {
        /// Specifies that [`shader_sampled_image_array_dynamic_indexing`] is supported.
        ///
        /// [`shader_sampled_image_array_dynamic_indexing`]: vk_10::shader_sampled_image_array_dynamic_indexing
        const SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING = 0x0001;
        /// Specifies that [`shader_storage_buffer_array_dynamic_indexing`] is supported.
        ///
        /// [`shader_storage_buffer_array_dynamic_indexing`]: vk_10::shader_storage_buffer_array_dynamic_indexing
        const STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING = 0x0002;
        /// Specifies that [`shader_uniform_buffer_array_dynamic_indexing`] is supported.
        ///
        /// [`shader_uniform_buffer_array_dynamic_indexing`]: vk_10::shader_uniform_buffer_array_dynamic_indexing
        const UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING = 0x0004;
        /// Specifies that [`shader_storage_image_array_dynamic_indexing`] is supported.
        ///
        /// [`shader_storage_image_array_dynamic_indexing`]: vk_10::shader_storage_image_array_dynamic_indexing
        const STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING = 0x0008;
        /// Specifies that [`shader_input_attachment_array_dynamic_indexing`] is supported.
        ///
        /// [`shader_input_attachment_array_dynamic_indexing`]: vk_12::shader_input_attachment_array_dynamic_indexing
        const INPUT_ATTACHMENT_ARRAY_DYNAMIC_INDEXING = 0x0010;
        /// Specifies that [`vk_12::shader_uniform_texel_buffer_array_dynamic_indexing`] is supported.
        ///
        /// [`shader_uniform_texel_buffer_array_dynamic_indexing`]: vk_12::shader_uniform_texel_buffer_array_dynamic_indexing
        const UNIFORM_TEXEL_BUFFER_ARRAY_DYNAMIC_INDEXING = 0x0020;
        /// Specifies that [`shader_storage_texel_buffer_array_dynamic_indexing`] is supported.
        ///
        /// [`shader_storage_texel_buffer_array_dynamic_indexing`]: vk_12::shader_storage_texel_buffer_array_dynamic_indexing
        const STORAGE_TEXEL_BUFFER_ARRAY_DYNAMIC_INDEXING = 0x0040;
        /// Specifies that [`shader_uniform_buffer_array_non_uniform_indexing`] is supported.
        ///
        /// [`shader_uniform_buffer_array_non_uniform_indexing`]: vk_12::shader_uniform_buffer_array_non_uniform_indexing
        const UNIFORM_BUFFER_ARRAY_NON_UNIFORM_INDEXING = 0x0080;
        /// Specifies that [`shader_sampled_image_array_non_uniform_indexing`] is supported.
        ///
        /// [`shader_sampled_image_array_non_uniform_indexing`]: vk_12::shader_sampled_image_array_non_uniform_indexing
        const SAMPLED_IMAGE_ARRAY_NON_UNIFORM_INDEXING = 0x0100;
        /// Specifies that [`shader_storage_buffer_array_non_uniform_indexing`] is supported.
        ///
        /// [`shader_storage_buffer_array_non_uniform_indexing`]: vk_12::shader_storage_buffer_array_non_uniform_indexing
        const STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING = 0x0200;
        /// Specifies that [`shader_storage_image_array_non_uniform_indexing`] is supported.
        ///
        /// [`shader_storage_image_array_non_uniform_indexing`]: vk_12::shader_storage_image_array_non_uniform_indexing
        const STORAGE_IMAGE_ARRAY_NON_UNIFORM_INDEXING = 0x0400;
        /// Specifies that [`shader_input_attachment_array_non_uniform_indexing`] is supported.
        ///
        /// [`shader_input_attachment_array_non_uniform_indexing`]: vk_12::shader_input_attachment_array_non_uniform_indexing
        const INPUT_ATTACHMENT_ARRAY_NON_UNIFORM_INDEXING = 0x0800;
        /// Specifies that [`shader_uniform_texel_buffer_array_non_uniform_indexing`] is supported.
        ///
        /// [`shader_uniform_texel_buffer_array_non_uniform_indexing`]: vk_12::shader_uniform_texel_buffer_array_non_uniform_indexing
        const UNIFORM_TEXEL_BUFFER_ARRAY_NON_UNIFORM_INDEXING = 0x1000;
        /// Specifies that [`shader_storage_texel_buffer_array_non_uniform_indexing`] is supported.
        ///
        /// [`shader_storage_texel_buffer_array_non_uniform_indexing`]: vk_12::shader_storage_texel_buffer_array_non_uniform_indexing
        const STORAGE_TEXEL_BUFFER_ARRAY_NON_UNIFORM_INDEXING = 0x2000;
        /// Specifies that [`runtime_descriptor_array`] is supported.
        ///
        /// [`runtime_descriptor_array`]: vk_12::runtime_descriptor_array
        const RUNTIME_DESCRIPTOR_ARRAY = 0x4000;
    }
}
unsafe impl NoUninit for DescriptorIndexingFeatures {}
__bitflags_display!(DescriptorIndexingFeatures);
impl features::FlagSupport<Flags32> for DescriptorIndexingFeatures {
    
    #[inline]
    fn to_obj(&self) -> features::FlagSupportObj<Flags32> {
        features::FlagSupportObj::new(*self)
    }

    #[inline]
    fn as_raw(&self) -> Flags32 {
        self.bits()
    }

    #[inline]
    fn add_flags(&mut self, rhs: Flags32) {
        Self::from_bits_retain(self.bits() | rhs);
    }

    #[inline]
    fn contains(&self, rhs: Flags32) -> bool {
        self.contains(Self::from_bits_retain(rhs))
    }
}
