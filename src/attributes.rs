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
