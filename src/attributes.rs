//! Contains various attribute structures.

use crate::*;

/// Parameters of a newly created [`Sampler`].
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SamplerAttributes<L> {
    /// An optional debug label of the sampler.
    pub label: Option<L>,
    /// A bitmask of sampler attributes.
    ///
    /// The default value is no flags set.
    pub flags: SamplerFlags,
    /// Specifies the magnification filter to apply to lookups.
    ///
    /// The default value is [`SamplerFilter::Nearest`].
    pub mag_filter: SamplerFilter,
    /// Specifies the minification filter to apply to lookups.
    ///
    /// The default value is [`SamplerFilter::Nearest`].
    pub min_filter: SamplerFilter,
    /// Specifies the mipmap filter to apply to lookups.
    ///
    /// The default value is [`SamplerMipmapMode::Nearest`].
    pub mipmap_mode: SamplerMipmapMode,
    /// Specifies a wrapping operation for `[u, v, w]` to use when sampling an image out of bounds.
    ///
    /// The default value is [`SamplerWrappingMode::Repeat`] for all coordinates.
    ///
    /// # Valid usage
    /// If the [`sampler_mirror_clamp_to_edge`] feature is not enabled, the
    /// [`SamplerWrappingMode::MirrorClampToEdge`] **must** mot be used.
    /// 
    /// [`sampler_mirror_clamp_to_edge`]: vk_12::sampler_mirror_clamp_to_edge
    pub wrapping_mode: [SamplerWrappingMode; 3],
    /// The bias added to mipmap LOD calculations and bias provided by image sampling operations in
    /// `SPIR-V`.
    ///
    /// This is described in detail in the [LOD Operation] section of the Vulkan docs.
    ///
    /// The default value is `0.0`.
    ///
    /// # Valid usage
    /// This **must** be less than or equal to [`max_sampler_lod_bias`].
    ///
    /// [LOD operation]: https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-level-of-detail-operation
    /// [`max_sampler_lod_bias`]: GpuLimits::max_sampler_lod_bias
    pub mip_lod_bias: f32,
    /// Specifies the anisotropy value clamp used by the sampler when the [`ANISOTROPY_ENABLE`] bit
    /// of [`flags`][Self::flags] is set.
    ///
    /// This **must** be in the range 1.0..=[max_sampler_anisotropy].
    ///
    /// The default value is `1.0`
    ///
    /// [`ANISOTROPY_ENABLE`]: SamplerFlags::ANISOTROPY_ENABLE
    /// [max_sampler_anisotropy]: GpuLimits::max_sampler_anisotropy
    pub max_anisotropy: f32,
    /// Specifies comparison operator to apply to fetched data before filtering when the
    /// [`COMPARE_ENABLE`] bit of [`flags`][Self::flags] is set.
    ///
    /// This is described in detail in the [Depth Compare Operation] section of the Vulkan docs.
    ///
    /// The default value is [`CompareOp::Never`].
    ///
    /// [`COMPARE_ENABLE`]: SamplerFlags::COMPARE_ENABLE
    /// [Depth Compare Operation]: https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-depth-compare-operation
    pub compare_op: CompareOp,
    /// Used to clamp the minimum of the computed LOD.
    ///
    /// The default value is `0.0`.
    ///
    /// # Valid usage
    /// This **must** be less than or equal to [`max_lod`][Self::max_lod].
    pub min_lod: f32,
    /// Used to clamp the maximum of the computed LOD.
    ///
    /// The default value is `0.0`.
    ///
    /// # Valid usage
    /// This **must** be greater than or equal to [`min_lod`][Self::min_lod].
    pub max_lod: f32,
    /// Specifies the border color to use.
    ///
    /// The default value is [`SamplerBorderColor::FloatTransparentBlack`].
    pub border_color: SamplerBorderColor,
}
impl<L> SamplerAttributes<L> {

    /// Maps the label type.
    #[inline]
    pub fn map_label<U>(self, f: impl FnOnce(Option<L>) -> Option<U>) -> SamplerAttributes<U> {
        SamplerAttributes {
            label: f(self.label),
            flags: self.flags,
            mag_filter: self.mag_filter,
            min_filter: self.min_filter,
            mipmap_mode: self.mipmap_mode,
            wrapping_mode: self.wrapping_mode,
            mip_lod_bias: self.mip_lod_bias,
            max_anisotropy: self.max_anisotropy,
            compare_op: self.compare_op,
            min_lod: self.min_lod,
            max_lod: self.max_lod,
            border_color: self.border_color,
        }
    }
}
/// Parameters of a newly created [`Buffer`].
///
/// This **can** be used to create a [`Buffer`] with [`Device::create_buffer`].
///
/// # Valid usage
/// The valid parameters of this structure is described in each field.
#[derive(Clone, Copy)]
pub struct BufferAttributes<L> {
    /// An optional debug label of the buffer.
    pub label: Option<L>,
    /// The size of the buffer.
    ///
    /// # Valid usage
    /// This **must** be in the range to 1..[`max_buffer_size`][1].
    ///
    /// [1]: GpuLimits::max_buffer_size
    pub size: DeviceSize,
    /// What the buffer is used for.
    ///
    /// At least one usage **must** be specified.
    pub usage: BufferUsages,
    /// Specifies what type of memory this buffer is backed by.
    pub memory_type: MemoryType,
}
impl<L> BufferAttributes<L> {

    /// Maps the label type.
    #[inline]
    pub fn map_label<U>(self, f: impl FnOnce(Option<L>) -> Option<U>) -> BufferAttributes<U> {
        BufferAttributes {
            label: f(self.label),
            size: self.size,
            usage: self.usage,
            memory_type: self.memory_type,
        }
    }
}

/// Parameters of a newly created [`Image`].
#[derive(Clone, Copy)]
pub struct ImageAttributes<L, V> {
    /// The debug label of the image.
    pub label: Option<L>,
    /// Specifies the format and type of the texel blocks contained in the image.
    ///
    /// The default value is [`Undefined`][1].
    ///
    /// # Valid usage
    /// It is invalid for the format to be [`Undefined`][1] and you **should** specify a different format
    /// for the image before creating it,
    ///
    /// [1]: Format::Undefined
    pub format: Format,
    /// Specifies the number of levels of detail in the image.
    ///
    /// The default value is `1`.
    ///
    /// # Valid usage
    /// This **must** be greater than or equal to `1`.
    pub mip_levels: u32,
    /// Specifies how many array layers the image has.
    ///
    /// The default value is `1`.
    ///
    /// # Valid usage
    /// This **must** be greater than or equal to `1`.
    pub array_layers: u32,
    /// Specifies the number of samples per texel.
    //
    /// The default value is [`SampleCountBit::X1`] (one sample).
    pub samples: SampleCountBit,
    /// A bitmask specifying what the image **can** be used for.
    ///
    /// The default value is an empty bitmask.
    ///
    /// # Valid usage
    /// - At least one usage **must** be specified.
    /// - If this bitmask contains any of [`COLOR_ATTACHMENT`][1], [`DEPTH_STENCIL_ATTACHMENT`][2] or
    ///   [`INPUT_ATTACHMENT`][3], then [`extent`][4] width **must** be less than or equal to
    ///   [`max_framebuffer_width`][5] and height **must** be less than or equal to
    ///   [`max_framebuffer_height`][6].
    /// - If the bitmask contains the [`STORAGE`][7] and if the
    ///   [`shader_storage_image_multisample`][8] feature is not enabled then [`samples`][9]
    ///   **must** be [`SampleCountBit::X1`].
    /// 
    /// Additional validation rules are specified in [`Device::query_image_creation_limits`].
    ///
    /// [1]: ImageUsages::COLOR_ATTACHMENT
    /// [2]: ImageUsages::DEPTH_STENCIL_ATTACHMENT
    /// [3]: ImageUsages::INPUT_ATTACHMENT
    /// [4]: Self::extent
    /// [5]: GpuLimits::max_framebuffer_width
    /// [6]: GpuLimits::max_framebuffer_height
    /// [7]: ImageUsages::STORAGE
    /// [8]: vk_10::shader_storage_image_multisample
    /// [9]: Self::samples
    pub usage: ImageUsages,
    /// An optional bitmask specifying separate usages for the stencil aspect of
    /// the image,
    ///
    /// # Valid usage
    /// - If this bitmask is `Some`, at least one usage **must** be specified.
    /// - If this bitmask is `Some` and
    ///     - if the bitmask contains any of [`COLOR_ATTACHMENT`][1],
    ///       [`DEPTH_STENCIL_ATTACHMENT`][2] or [`INPUT_ATTACHMENT`][3] bits then [`extent`][4] width
    ///       **must** be less than or equal to [`max_framebuffer_width`][5] and height **must** be
    ///       less than or equal to [`max_framebuffer_height`][6] or,
    ///     - if the bitmask contains the [`STORAGE`][7] and if the
    ///       [`shader_storage_image_multisample`][8] feature is not enabled then [`samples`][9]
    ///       **must** be [`SampleCountBit::X1`].
    /// 
    /// Additional validation rules are specified in [`Device::query_image_creation_limits`].
    ///
    /// [1]: ImageUsages::COLOR_ATTACHMENT
    /// [2]: ImageUsages::DEPTH_STENCIL_ATTACHMENT
    /// [3]: ImageUsages::INPUT_ATTACHMENT
    /// [4]: Self::extent
    /// [5]: GpuLimits::max_framebuffer_width
    /// [6]: GpuLimits::max_framebuffer_height
    /// [7]: ImageUsages::STORAGE
    /// [8]: vk_10::shader_storage_image_multisample
    /// [9]: Self::samples
    pub stencil_usage: Option<ImageUsages>,
    /// The dimension of the image.
    ///
    /// The default value is [`ImageDim::Dim2`].
    pub dimensions: ImageDim,
    /// A bitmask specifying what kinds of image views **can** be created of this image.
    pub view_compatibility: ImageViewCompatibility,
    /// Specifies formats that **can** be used as the format of image views created of this image.
    ///
    /// The image's [`format`][1] can always be used as the format of image views.
    ///
    /// Each format in this list **must** be [`compatible`][2] with [`format`][1].
    ///
    /// [1]: Self::format
    /// [2]: Format::is_compatible_with
    pub view_formats: V,
    /// Specifies the extent of this image.
    ///
    /// The default value is `1` for each dimension.
    ///
    /// # Valid usage
    /// Each of `width`, `height` and `depth` **must** be greater than or equal to `1`.
    pub extent: ImageExtent,
    /// Specifies what type of memory this image is backed by.
    ///
    /// The default value is [`MemoryType::DeviceLocal`].
    pub memory_type: MemoryType,
}
impl<L, M> Default for ImageAttributes<L, M>
    where M: Default
{

    #[inline]
    fn default() -> Self {
        Self {
            label: None,
            format: Format::Undefined,
            mip_levels: 1,
            array_layers: 1,
            samples: SampleCountBit::X1,
            usage: ImageUsages::empty(),
            stencil_usage: None,
            dimensions: ImageDim::Dim2,
            view_compatibility: ImageViewCompatibility::empty(),
            view_formats: default(),
            extent: ImageExtent { width: 1, height: 1, depth: 1 },
            memory_type: default(),
        }
    }
}
impl<L, M> ImageAttributes<L, M> {

    /// Maps the label type.
    #[inline]
    pub fn map_label<U>(self, f: impl FnOnce(Option<L>) -> Option<U>) -> ImageAttributes<U, M> {
        ImageAttributes {
            label: f(self.label),
            format: self.format,
            mip_levels: self.mip_levels,
            array_layers: self.array_layers,
            samples: self.samples,
            stencil_usage: None,
            usage: self.usage,
            dimensions: self.dimensions,
            view_compatibility: self.view_compatibility,
            view_formats: self.view_formats,
            extent: self.extent,
            memory_type: self.memory_type,
        }
    }

    /// Maps the label type.
    #[inline]
    pub fn map_view_formats<U>(self, f: impl FnOnce(M) -> U) -> ImageAttributes<L, U> {
        ImageAttributes {
            label: self.label,
            format: self.format,
            mip_levels: self.mip_levels,
            array_layers: self.array_layers,
            samples: self.samples,
            usage: self.usage,
            stencil_usage: self.stencil_usage,
            dimensions: self.dimensions,
            view_compatibility: self.view_compatibility,
            view_formats: f(self.view_formats),
            extent: self.extent,
            memory_type: self.memory_type,
        }
    }
}
/// Parameters of a newly created [`ImageView`].
#[derive(Clone, Copy)]
pub struct ImageViewAttributes<L> {
    /// The debug label of the image view.
    pub label: Option<L>,
    /// Specifies the format of this image view.
    ///
    /// A `None` value specifies the same format as the [`Image`].
    ///
    /// # Valid usage
    /// - If this is `Some`, the format **must** be either the same as the image's format or be
    ///   included in the [`ImageAttributes::view_formats`] list specified when creating the image.
    pub format: Option<Format>,
    /// The dimension of the image.
    ///
    /// # Valid usage
    /// - If the image's [samples] is not [`SampleCountBit::X1`] this **must** be either
    ///   [`Dim2`] or [`Dim2Array`].
    /// - If [`view_compatibility`] is empty or [`REINTERPRET_TEXEL_BLOCKS`] and:
    ///     - if the image is [`Dim1`][1] this **must** be [`Dim1`] or [`Dim1Array`], or
    ///     - if the image is [`Dim2`][2] this **must** be [`Dim2`] or [`Dim2Array`], or
    ///     - if the image is [`Dim3`][3] this **must** be [`Dim3`].
    /// - If [`view_compatibility`] contains [`CUBE`][4] this **must** be either [`Dim2`],
    ///   [`Dim2Array`], [`Cube`] or [`CubeArray`].
    /// - If [`view_compatibility`] doesn't contain [`CUBE`][4] this **must** not be [`Cube`] or
    ///   [`CubeArray`].
    /// - If [`view_compatibility`] contains [`DIM3_SLICE_AS_ARRAY`] this **must** be either 
    ///   [`Dim2`], [`Dim2Array`] or [`Dim3`].
    ///
    /// [samples]: ImageAttributes::samples
    /// [`view_compatibility`]: ImageAttributes::view_compatibility
    /// [1]: ImageDim::Dim1
    /// [`Dim1`]: ImageViewDim::Dim1
    /// [`Dim1Array`]: ImageViewDim::Dim1Array
    /// [2]: ImageDim::Dim2
    /// [`Dim2`]: ImageViewDim::Dim2
    /// [`Dim2Array`]: ImageViewDim::Dim2Array
    /// [3]: ImageDim::Dim3
    /// [`Dim3`]: ImageViewDim::Dim3
    /// [4]: ImageViewCompatibility::CUBE
    /// [`Cube`]: ImageViewDim::Cube
    /// [`CubeArray`]: ImageViewDim::CubeArray
    /// [`REINTERPRET_TEXEL_BLOCKS`]: ImageViewCompatibility::REINTERPRET_TEXEL_BLOCKS
    pub dimensions: ImageViewDim,
    /// Specifies the range of image aspects/levels/layers this view spans.
    ///
    /// # Valid usage
    /// In addition to the valid usages documented in [`ImageSubresourceRange`]: 
    /// - [`aspects`] **must** only contain bits that are valid for [`format`].
    /// - If the image is [`Dim3`][1] and [`dimensions`] is [`Dim2`] or [`Dim2Array`],
    ///   `level_count` **must** be `1`.
    /// - If the image was created with the [`REINTERPRET_TEXEL_BLOCKS`] and [`format`] is an
    ///   [uncompressed] format, [`level_count`] and [`layer_count`] **must** both be `1`.
    /// - If [`dimensions`] is [`Dim1`], [`Dim2`] or [`Dim3`], [`layer_count`] **must** be `1`.
    /// - If [`dimensions`] is [`Cube`], [`layer_count`] **must** be `6` or if [`layer_count`] is
    ///   `None` the remaining array layers **must** be `6`.
    /// - If [`dimensions`] is [`CubeArray`], [`layer_count`] **must** be a multiple `6` or if
    ///   [`layer_count`] is `None` the remaining array layers **must** be a multiple of `6`.
    ///
    /// [`aspects`]: ImageSubresourceRange::aspects
    /// [`view_compatibility`]: ImageAttributes::view_compatibility
    /// [`dimensions`]: Self::dimensions
    /// [1]: ImageDim::Dim3
    /// [`Dim1`]: ImageViewDim::Dim1
    /// [`Dim2`]: ImageViewDim::Dim2
    /// [`Dim3`]: ImageViewDim::Dim3
    /// [`Dim2Array`]: ImageViewDim::Dim2Array
    /// [`Cube`]: ImageViewDim::Cube
    /// [`CubeArray`]: ImageViewDim::CubeArray
    /// [`REINTERPRET_TEXEL_BLOCKS`]: ImageViewCompatibility::REINTERPRET_TEXEL_BLOCKS
    /// [`format`]: Self::format
    /// [uncompressed]: Format::is_compressed
    /// [`level_count`]: ImageSubresourceRange::level_count
    /// [`layer_count`]: ImageSubresourceRange::layer_count
    pub range: ImageSubresourceRange,
    /// Specifies how the image view maps the components of the image:
    pub component_mapping: ComponentMapping,
}
