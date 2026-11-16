use bytemuck::{ByteEq, ByteHash, NoUninit, Zeroable};
use bitflags::bitflags;
use tuhka::vk;

use crate::*;

use sync::Arc;

bitflags! {
    /// A bitmask specifying sampler attributes.
    #[repr(transparent)]
    #[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
    pub struct SamplerFlags: u32 {
        /// Whether to enable anisotropic filtering.
        ///
        /// # Valid usage
        /// - If the [`sample_anisotropy`] feature is not enabled, this bit **must** not be set.
        ///
        /// [`sample_anisotropy`]: vk_10::sampler_anisotropy
        const ANISOTROPY_ENABLE = 0x1;
        /// Whether to apply comparison against a reference value during lookups.
        const COMPARE_ENABLE = 0x2;
        /// Whether to use unnormalized (`0.0..image_extent`) or normalized (`0.0..1.0`)
        /// texel coordinates to address the texels of an image.
        ///
        /// # Valid usage
        /// When this bit is set:
        /// - the values of [`mag_filter`] and [`min_filter`] **must** be equal,
        /// - [`mipmap_mode`] **must** be [`SamplerMipmapMode::Nearest`],
        /// - [`min_lod`] and [`max_lod`] **must** be `0`,
        /// - [`wrapping_mode`] `u` and `v` elements **must** be either [`ClampToEdge`] or
        ///   [`ClampToBorder`], and
        /// - the [`ANISOTROPY_ENABLE`] and [`COMPARE_ENABLE`] bits **must** not be set,
        const UNNORMALIZED_COORDINATES = 0x4;
    }
}
unsafe impl NoUninit for SamplerFlags {}

/// Parameters of a newly created [`Sampler`].
///
/// This **can** be used to create a [`Sampler`] with [`Device::create_sampler`].
///
/// # Valid usage
/// The valid parameters of this structure is described in each field.
pub type SamplerAttributes<'a> = attributes::SamplerAttributes<&'a str>;

impl PartialEq for SamplerAttributes<'_> {

    #[inline]
    fn eq(&self, other: &Self) -> bool {
        eq(&self.label, &other.label) &&
        eq(&self.flags, &other.flags) &&
        eq(&self.mag_filter, &other.mag_filter) &&
        eq(&self.min_filter, &other.min_filter) &&
        eq(&self.mipmap_mode, &other.mipmap_mode) &&
        eq(&self.wrapping_mode, &other.wrapping_mode) &&
        eq(bytemuck::bytes_of(&self.mip_lod_bias), bytemuck::bytes_of(&other.mip_lod_bias)) &&
        eq(bytemuck::bytes_of(&self.max_anisotropy), bytemuck::bytes_of(&other.max_anisotropy)) &&
        eq(&self.compare_op, &other.compare_op) &&
        eq(bytemuck::bytes_of(&self.min_lod), bytemuck::bytes_of(&other.min_lod)) &&
        eq(bytemuck::bytes_of(&self.max_lod), bytemuck::bytes_of(&other.max_lod)) &&
        eq(&self.border_color, &other.border_color)
    }
}

impl Default for SamplerAttributes<'_> {

    fn default() -> Self {
        Self {
            label: None,
            flags: default(),
            mag_filter: default(),
            min_filter: default(),
            mipmap_mode: default(),
            wrapping_mode: default(),
            mip_lod_bias: 0.0,
            max_anisotropy: 1.0,
            compare_op: default(),
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: default(),
        }
    }
}

struct Inner {
    device: Device,
    handle: vk::Sampler,
    attributes: attributes::SamplerAttributes<CompactStr>,
}

impl Drop for Inner {

    fn drop(&mut self) {
        unsafe { self.device
            .functions()
            .destroy_sampler(self.handle, None);
        }
        if let Some(label) = &self.attributes.label {
            log::info!("destroyed `Sampler` `{label}`");
        }
    }
}

/// A sampler object.
///
/// To create a sampler, use [`Device::create_sampler`].
#[must_use]
#[derive(Clone)]
pub struct Sampler {
    inner: Arc<Inner>,
}

impl Sampler {

    /// Retrieves the assigned label of this sampler.
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.attributes().label
            .as_deref()
    }

    /// Retrieves the attributes used to create this sampler.
    #[inline]
    pub fn attributes(&self) -> &attributes::SamplerAttributes<CompactStr> {
        &self.inner.attributes
    }

    /// Creates the sampler.
    pub(crate) fn new(
        device: Device,
        attributes: SamplerAttributes,
    ) -> Result<Self, Error> {
        let unnormalized_coordinates = attributes.flags
            .contains(SamplerFlags::UNNORMALIZED_COORDINATES);
        let anisotropy_enable = attributes.flags
            .contains(SamplerFlags::ANISOTROPY_ENABLE);
        let compare_enable = attributes.flags
            .contains(SamplerFlags::COMPARE_ENABLE);
        if unnormalized_coordinates {
            if attributes.min_filter != attributes.mag_filter {
                Err(err!(
                    "`UNNORMALIZED_COORDINATES` bit is set, but `min_filter` `{:?}` and `mag_filter` `{:?}` are not equal",
                    attributes.min_filter, attributes.mag_filter
                ))?
            }
            if attributes.mipmap_mode != SamplerMipmapMode::Nearest {
                Err(err!(
                    "`UNNORMALIZED_COORDINATES` bit is set, but `mipmap_mode` is not `SamplerMipMapMode::Nearest`",
                ))?
            }
            if attributes.min_lod != 0.0 {
                Err(err!(
                    "`UNNORMALIZED_COORDINATES` bit is set, but `min_lod` is not `0.0`",
                ))?
            }
            if attributes.max_lod != 0.0 {
                Err(err!(
                    "`UNNORMALIZED_COORDINATES` bit is set, but `max_lod` is not `0.0`",
                ))?
            }
            if !matches!(attributes.wrapping_mode[0], SamplerWrappingMode::ClampToEdge | SamplerWrappingMode::ClampToBorder) {
                Err(err!(
                    "`UNNORMALIZED_COORDINATES` bit is set, but `wrapping_mode[0]` is not `ClampToEdge` or `ClampToBorder`",
                ))?
            }
            if !matches!(attributes.wrapping_mode[1], SamplerWrappingMode::ClampToEdge | SamplerWrappingMode::ClampToBorder) {
                Err(err!(
                    "`UNNORMALIZED_COORDINATES` bit is set, but `wrapping_mode[1]` is not `ClampToEdge` or `ClampToBorder`",
                ))?
            }
            if anisotropy_enable {
                Err(err!(
                    "`UNNORMALIZED_COORDINATES` and `ANISOTROPY_ENABLE` are both set",
                ))?
            }
            if compare_enable {
                Err(err!(
                    "`UNNORMALIZED_COORDINATES` and `COMPARE_ENABLE` are both set",
                ))?
            }
        }
        if anisotropy_enable &&
            device.get_feature(vk_10::sampler_anisotropy::NAME)
                .is_none()
        {
            Err(err!(
                "`ANISOTROPY_ENABLE` bit is set but the `sampler_anisotropy` feature is not enabled",
            ))?
        }
        if attributes.min_lod > attributes.max_lod {
            Err(err!(
                "`min_lod` `{}` is greter than `max_lod` `{}`",
                attributes.min_lod, attributes.max_lod,
            ))?
        }
        if attributes.mip_lod_bias > device.gpu_info().limits.max_sampler_lod_bias() {
            Err(err!(
                "`mip_lod_bias` `{}` is greater tan `max_sampler_lod_bias` `{}`",
                attributes.mip_lod_bias, device.gpu_info().limits.max_sampler_lod_bias(),
            ))?
        }
        if anisotropy_enable &&
            attributes.max_anisotropy < 1.0 ||
            attributes.max_anisotropy > device.gpu_info().limits.max_sampler_anisotropy()
        {
            Err(err!(
                "`max_anisotropy` `{}` is not in the range `1.0..={}` (`1.0..=max_sampler_anisotropy`)",
                attributes.max_anisotropy,
                device.gpu_info().limits.max_sampler_anisotropy(),
            ))?
        }
        if attributes.wrapping_mode.contains(&SamplerWrappingMode::MirrorClampToEdge) &&
            device.get_feature(vk_12::sampler_mirror_clamp_to_edge::NAME)
                .is_none()
        {
            Err(err!(
                "`SamplerWrappingMode::MirrorClampToEdge` is used but the `sampler_mirror_clamp_to_edge` feature is not enabled",
            ))?
        }
        let create_info = vk::SamplerCreateInfo::default()
            .mag_filter(attributes.mag_filter.into())
            .min_filter(attributes.min_filter.into())
            .mipmap_mode(attributes.mipmap_mode.into())
            .address_mode_u(attributes.wrapping_mode[0].into())
            .address_mode_v(attributes.wrapping_mode[1].into())
            .address_mode_w(attributes.wrapping_mode[2].into())
            .mip_lod_bias(attributes.mip_lod_bias)
            .anisotropy_enable(anisotropy_enable)
            .max_anisotropy(attributes.max_anisotropy)
            .compare_enable(compare_enable)
            .compare_op(attributes.compare_op.into())
            .min_lod(attributes.min_lod)
            .max_lod(attributes.max_lod)
            .border_color(attributes.border_color.into())
            .unnormalized_coordinates(unnormalized_coordinates);
        let handle = unsafe { device
            .functions()
            .create_sampler(&create_info, None)
        }.describe_err("Failed to create `VkSampler`")?;
        if let Some(label) = attributes.label {
            log::info!("created `Sampler` `{label}`");
        }
        Ok(Self {
            inner: Inner {
                device,
                handle,
                attributes: attributes.map_label(|label| label.map(Into::into)),
            }.arced(),
        })
    }
}
