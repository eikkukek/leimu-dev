use core::{
    ops::{RangeBounds, Bound},
    num::NonZeroU32,
};

use bytemuck::{ByteEq, ByteHash, NoUninit, Zeroable};

use tuhka::vk;

use crate::{
    sorted::SortByKey,
    *,
};

/// Specifies the stencil operations state for a face.
///
/// # Default values
/// Each field has the default value of it's type.
#[repr(C)]
#[derive(Default, Clone, Copy, NoUninit, ByteHash, ByteEq)]
pub struct StencilOpState {
    /// Stencil operation performed when the stencil test fails for a sample.
    pub fail_op: StencilOp,
    /// Stencil operation performed when both the stencil and depth tests pass for a sample.
    pub pass_op: StencilOp,
    /// Stencil operation performed for samples that pass the stencil test but fail the depth
    /// test.
    pub depth_fail_op: StencilOp,
    /// The comparision operator used in the stencil test.
    pub compare_op: CompareOp,
}
/// Specifies vertex input binding description.
///
/// # Default values
/// - [`binding`][Self::binding]: `0`
/// - [`stride`][Self::stride]: `0`
/// - [`input_rate`][Self::input_rate]: [`VertexInputRate::Vertex`]
/// - [`divisor`][Self::divisor]: `1`.
#[repr(C)]
#[derive(Clone, Copy, NoUninit, ByteHash, ByteEq, Zeroable)]
pub struct VertexInputBindingDescription {
    /// The binding number this structure describes.
    pub binding: u32,
    /// The bytes stride between consecutive elements within the buffer.
    pub stride: u32,
    /// The rate at which vertex attributes are pulled from the buffer.
    pub input_rate: VertexInputRate,
    /// The number of successive instances that will use the same value of the vertex attribute when
    /// instanced rendering is enabled.
    ///
    /// This **must** be `1` (the default value) if the [`vertex_attribute_instance_rate_divisor`][1] feature
    /// is not enabled.
    ///
    /// [1]: vk_14::vertex_attribute_instance_rate_divisor
    pub divisor: u32,
}
impl Default for VertexInputBindingDescription {

    #[inline]
    fn default() -> Self {
        Self {
            binding: 0,
            stride: 0,
            input_rate: VertexInputRate::Vertex,
            divisor: 1,
        }
    }
}
impl SortByKey for VertexInputBindingDescription {

    type Key = u32;

    #[inline]
    fn key(&self) -> &Self::Key {
        &self.binding
    }
}
/// Specifies vertex input attribute description.
#[repr(C)]
#[derive(Default, Clone, Copy, NoUninit, ByteHash, ByteEq, Zeroable)]
pub struct VertexInputAttributeDescription {
    /// The shader input location of this attribute.
    pub location: u32,
    /// The binding number which this attribute takes its data from.
    pub binding: u32,
    /// The [`Format`] of the vertex attribute data.
    pub format: Format,
    /// The byte offset of this attribute relative to the start of the vertex input binding.
    pub offset: u32,
}
impl SortByKey for VertexInputAttributeDescription {

    type Key = u32;

    #[inline]
    fn key(&self) -> &Self::Key {
        &self.location
    }
}
/// Specifies limits on depth and stencil attachment [resolve modes].
///
/// [resolve modes]: ResolveMode
#[derive(Clone, Copy)]
pub struct GpuDepthStencilResolveLimits {
    /// A bitmask of supported depth resolve modes.
    pub supported_depth_resolve_modes: ResolveModeFlags,
    /// A bitmask of supported stencil resolve modes.
    pub supported_stencil_resolve_modes: ResolveModeFlags,
    /// Specifies whether specifying a resolve [`ImageView`] for only one of depth or stencil attachments
    /// is allowed.
    ///
    /// This is guaranteed to be supported if [`independent_resolve`] is supported.
    ///
    /// [`independent_resolve`]: Self::independent_resolve
    pub independent_resolve_none: bool,
    /// Specifies whether depth and stencil attachments **can** have different resolve modes when a
    /// resolve [`ImageView`] is specified for both.
    pub independent_resolve: bool,
}
/// Specifies the color blend equation for an attachment.
#[repr(C)]
#[derive(Default, Clone, Copy, NoUninit, ByteHash, ByteEq)]
pub struct ColorBlendEquation {
    /// Determines which blend factor is used for `src_blend_factor.color`.
    pub src_color_blend_factor: BlendFactor,
    /// Determines which blend factor is used for `dst_blend_factor.color`.
    pub dst_color_blend_factor: BlendFactor,
    /// Specifies which blend operation is used to calculate the `RGB` values to write to the color
    /// attachment.
    pub color_blend_op: BlendOp,
    /// Determines which blend factor is used for `src_blend_factor.alpha`.
    pub src_alpha_blend_factor: BlendFactor,
    /// Determines which blend factor is used for `dst_blend_factor.alpha`.
    pub dst_alpha_blend_factor: BlendFactor,
    /// Specifies which blend operation is used to calculate the alpha values to write to the color
    /// attachment.
    pub alpha_blend_op: BlendOp,
}
impl ColorBlendEquation {
    /// Returns whether any [`BlendFactor`] in this equation uses the second source components.
    ///
    /// If this returns `true`, the [`dual_src_blend`][1] feature **must** be enabled and the
    /// maximum number of color attachments in the pipeline **must** be less than or equal to
    /// [`max_fragment_dual_src_attachments`][2].
    ///
    /// [1]: vk_10::dual_src_blend
    /// [2]: GpuLimits::max_fragment_dual_src_attachments
    #[inline]
    pub fn uses_second_source(self) -> bool {
        self.src_color_blend_factor.uses_second_source() ||
        self.dst_color_blend_factor.uses_second_source() ||
        self.src_alpha_blend_factor.uses_second_source() ||
        self.dst_alpha_blend_factor.uses_second_source()
    }
}
/// Specifies a range of image aspects/levels/layers.
///
/// The default value of this structure specifies a range that spans the entire image.
#[repr(C)]
#[derive(Default, Clone, Copy, NoUninit, Zeroable, ByteEq, ByteHash)]
pub struct ImageSubresourceRange {
    /// A bitmask of image aspects.
    ///
    /// The default value is [`ImageAspects::DERIVE`], which derives all the aspects of an image for
    /// this range.
    pub aspects: ImageAspects,
    /// Specifies that first mip level of this range.
    ///
    /// # Valid usage
    /// - This **must** be less than the [number of levels] in the image.
    ///
    /// [number of levels]: ImageAttributes::mip_levels
    pub base_mip_level: u32,
    /// Specifies that number of mip levels from [`base_mip_level`][1] included in this range.
    ///
    /// A `None` value specifies the rest of the available mip levels.
    ///
    /// # Valid usage
    /// - If this is `Some`, `base_mip_level + level_count` **must** be less than or equal to the
    ///   [number of levels] in the image.
    ///
    /// [1]: Self::base_mip_level
    /// [number of levels]: ImageAttributes::mip_levels
    pub level_count: Option<NonZeroU32>,
    /// Specifies the first array layer of this range.
    ///
    /// # Valid usage
    /// - If the image's wasn't created with the [`ImageViewCompatibility::DIM3_SLICE_AS_ARRAY`] and
    ///   if the range does not specify the range of a [`Dim2`] or [`Dim2Array`] image view, this
    ///   **must** be less than the [number of layers] in the image, or otherwise
    /// - this **must** be less than the computed depth of the image's [extent] at
    ///   [`base_mip_level`].
    ///
    /// [number of layers]: ImageAttributes::array_layers
    /// [extent]: ImageAttributes::extent
    /// [`base_mip_level`]: Self::base_mip_level
    pub base_array_layer: u32,
    /// Specifies that number of array layers from [`base_array_layer`][1] included in this range.
    ///
    /// A `None` value specifies the rest of the available layers (or depth).
    ///
    /// # Valid usage
    /// - If this is `Some` and:
    ///   - if the image's wasn't created with the [`ImageViewCompatibility::DIM3_SLICE_AS_ARRAY`] and
    ///     if the range does not specify the range of a [`Dim2`] or [`Dim2Array`] image view,
    ///     `base_array_layer + layer_count` **must** be less than or equal to the
    ///     [number of layers] in the image, or otherwise
    ///   - `base_array_layer + layer_count` **must** be less than or equal to the computed depth of
    ///     the image's [extent] at [`base_mip_level`].
    ///
    /// [number of layers]: ImageAttributes::array_layers
    /// [extent]: ImageAttributes::extent
    /// [`base_mip_level`]: Self::base_mip_level
    pub layer_count: Option<NonZeroU32>,
}
impl ImageSubresourceRange {

    /// Copies `self` and sets `aspects`.
    #[inline]
    pub fn aspects(mut self, aspects: ImageAspects) -> Self {
        self.aspects = aspects;
        self
    }
    /// Copies `self` and sets `base_mip_level` and `level_count` based on a range.
    #[inline]
    pub fn levels(mut self, levels: impl RangeBounds<u32>) -> Self {
        let start = match levels.start_bound()
        {
            Bound::Included(&inc) => inc,
            Bound::Excluded(&ex) => ex,
            Bound::Unbounded => 0,
        };
        let count = match levels.end_bound() {
            Bound::Included(&inc) => NonZeroU32::new(inc + 1 - start),
            Bound::Excluded(&ex) => NonZeroU32::new(ex - start),
            Bound::Unbounded => None,
        };
        self.base_mip_level = start;
        self.level_count = count;
        self
    }
    /// Copies `self` and sets `base_array_layer` and `layer_count` based on a range.
    #[inline]
    pub fn layers(mut self, layers: impl RangeBounds<u32>) -> Self {
        let start = match layers.start_bound()
        {
            Bound::Included(&inc) => inc,
            Bound::Excluded(&ex) => ex,
            Bound::Unbounded => 0,
        };
        let count = match layers.end_bound() {
            Bound::Included(&inc) => NonZeroU32::new(inc + 1 - start),
            Bound::Excluded(&ex) => NonZeroU32::new(ex - start),
            Bound::Unbounded => None,
        };
        self.base_array_layer = start;
        self.layer_count = count;
        self
    }
    /// Copies `self` and sets `layer_count`.
    ///
    /// `0` indicates remaining layers from `base_array_layers`.
    #[inline]
    pub fn layer_count(mut self, count: u32) -> Self {
        self.layer_count = count.try_into().ok();
        self
    }
}
/// Specifies the component mapping of an [`ImageView`].
///
/// The default value is [`Identity`] for all fields.
///
/// [`Identity`]: ComponentSwizzle::Identity
#[repr(C)]
#[derive(Default, Clone, Copy, NoUninit, Zeroable, ByteEq, ByteHash)]
pub struct ComponentMapping {
    /// Specifies the component value placed in the `R` component.
    pub r: ComponentSwizzle,
    /// Specifies the component value placed in the `G` component.
    pub g: ComponentSwizzle,
    /// Specifies the component value placed in the `B` component.
    pub b: ComponentSwizzle,
    /// Specifies the component value placed in the `A` component.
    pub a: ComponentSwizzle,
}
impl From<ComponentMapping> for vk::ComponentMapping {
    #[inline]
    fn from(value: ComponentMapping) -> Self {
        bytemuck::cast(value)
    }
}
