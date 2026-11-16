#![doc = r" This file is auto-generated, do not modify manually."]
use bytemuck::{ByteEq, ByteHash, CheckedBitPattern, NoUninit, Zeroable};
#[doc = r" An enumeration of numeric format types."]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NumericFormat {
    #[doc = r" Components are unsigned normalized values in the range `0.0..=1.0`."]
    Unorm,
    #[doc = r" Components are signed normalized values in the range `-1.0..=1.0`."]
    Snorm,
    #[doc = r" Components are unsigned integer values that get converted to"]
    #[doc = r" floating-point in the range `0..2^BIT_WIDTH`."]
    Uscaled,
    #[doc = r" Components are signed intger values that get converted to"]
    #[doc = r" floating-point in the range `-2^(BIT_WIDTH-1)..2^(BIT_WIDTH-1)`."]
    Sscaled,
    #[doc = r" Components are unsigned integer valus in the range `0..2^BIT_WIDTH`."]
    Uint,
    #[doc = r" Components are signed integer values in the range"]
    #[doc = r" `-2^(BIT_WIDTH-1)..2^(BIT_WIDTH-1)`"]
    Sint,
    #[doc = r" Components are unsigned floating-point numbers."]
    Ufloat,
    #[doc = r" Components are signed floating-point numbers."]
    Sfloat,
    #[doc = r" The `R`, `G`, and `B` components are unsigned normalized values that represent values"]
    #[doc = r" using `sRGB` nonlinear encoding, while the `A` component (if one exists) is a regular"]
    #[doc = r" unsigned normalized value."]
    Srgb,
    #[doc = r" Components are signed fractional integer values that get converted to"]
    #[doc = r" floating-point in the range `-1024.0..=1023.96875`."]
    Sfixed5,
    #[doc = r" Components are booleans."]
    Bool,
}
impl NumericFormat {
    #[doc = r" Returns whether the format is a floating point format."]
    #[inline]
    pub fn is_floating_point(self) -> bool {
        matches!(
            self,
            Self::Unorm
                | Self::Snorm
                | Self::Uscaled
                | Self::Sscaled
                | Self::Ufloat
                | Self::Sfloat
                | Self::Srgb
        )
    }
    #[doc = r" Returns whether the format is an unsigned integer format."]
    #[inline]
    pub fn is_unsigned_integer(self) -> bool {
        matches!(self, Self::Uint)
    }
    #[doc = r" Returns whether the format is a signed integer format."]
    #[inline]
    pub fn is_signed_integer(self) -> bool {
        matches!(self, Self::Sint)
    }
    #[doc = r" Returns whether the format is a scaled signed integer format."]
    #[inline]
    pub fn is_scaled_signed_integer(self) -> bool {
        matches!(self, Self::Sfixed5)
    }
    #[doc = r" Returns whether the format is any integer format."]
    #[inline]
    pub fn is_integer(self) -> bool {
        matches!(self, Self::Uint | Self::Sint | Self::Sfixed5)
    }
    #[doc = r" Returns whether the format is a boolean format."]
    #[inline]
    pub fn is_boolean(self) -> bool {
        matches!(self, Self::Bool)
    }
}
impl ::core::fmt::Display for NumericFormat {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::Unorm => write!(f, "unorm"),
            Self::Snorm => write!(f, "snorm"),
            Self::Uscaled => write!(f, "uscaled"),
            Self::Sscaled => write!(f, "sscaled"),
            Self::Uint => write!(f, "uint"),
            Self::Sint => write!(f, "sint"),
            Self::Ufloat => write!(f, "ufloat"),
            Self::Sfloat => write!(f, "sfloat"),
            Self::Srgb => write!(f, "srgb"),
            Self::Sfixed5 => write!(f, "sfixed5"),
            Self::Bool => write!(f, "bool"),
        }
    }
}
#[doc = r" Describes the compatibility class of a [`Format`]."]
#[derive(Clone, Copy, Debug)]
pub struct FormatCompatibility {
    name: &'static str,
    num: u32,
    texel_block_size: u64,
    texel_block_extent: (u32, u32, u32),
    texels_per_block: u8,
    is_alpha_format: bool,
}
impl FormatCompatibility {
    #[doc = r" Returns the size of a single texel block. "]
    #[inline]
    pub const fn texel_block_size(&self) -> u64 {
        self.texel_block_size
    }
    #[doc = r" Returns the 3D extent of a single texel block."]
    #[inline]
    pub fn texel_block_extent<T>(&self) -> T
    where
        T: From<(u32, u32, u32)>,
    {
        self.texel_block_extent.into()
    }
    #[doc = r" Returns how many texels there are in a single texel block."]
    #[inline]
    pub const fn texels_per_block(&self) -> u8 {
        self.texels_per_block
    }
    #[doc = r" Returns whether `self` is size-compatible with `other`."]
    #[doc = r""]
    #[doc = r" Returns `true` is neither or both are alpha formats and if the sizes of their texel"]
    #[doc = r" blocks are equal."]
    #[inline]
    pub const fn is_size_compatible_with(&self, other: &Self) -> bool {
        self.texel_block_size == other.texel_block_size
            && !(self.is_alpha_format ^ other.is_alpha_format)
    }
}
impl PartialEq for FormatCompatibility {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num
    }
}
impl Eq for FormatCompatibility {}
impl ::core::hash::Hash for FormatCompatibility {
    #[inline]
    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.num);
    }
}
impl ::core::fmt::Display for FormatCompatibility {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{}", self.name)
    }
}
#[derive(Clone, Copy)]
pub(super) struct FormatInfo {
    pub name: &'static str,
    pub compatibility: &'static FormatCompatibility,
    pub numeric_color: Option<NumericFormat>,
    pub numeric_depth: Option<NumericFormat>,
    pub numeric_stencil: Option<NumericFormat>,
    pub is_compressed: bool,
}
static COMPATIBILITY: [FormatCompatibility; 97usize] = [
    FormatCompatibility {
        name: "undefined",
        num: 0u32,
        texel_block_size: 0u64,
        texel_block_extent: (0u32, 0u32, 0u32),
        texels_per_block: 0u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "8-bit",
        num: 1u32,
        texel_block_size: 1u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "16-bit",
        num: 2u32,
        texel_block_size: 2u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "8-bit alpha",
        num: 3u32,
        texel_block_size: 1u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: true,
    },
    FormatCompatibility {
        name: "24-bit",
        num: 4u32,
        texel_block_size: 3u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "32-bit",
        num: 5u32,
        texel_block_size: 4u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "48-bit",
        num: 6u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "64-bit",
        num: 7u32,
        texel_block_size: 8u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "96-bit",
        num: 8u32,
        texel_block_size: 12u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "128-bit",
        num: 9u32,
        texel_block_size: 16u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "192-bit",
        num: 10u32,
        texel_block_size: 24u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "256-bit",
        num: 11u32,
        texel_block_size: 32u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "D16",
        num: 12u32,
        texel_block_size: 2u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "D24",
        num: 13u32,
        texel_block_size: 4u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "D32",
        num: 14u32,
        texel_block_size: 4u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "S8",
        num: 15u32,
        texel_block_size: 1u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "D16S8",
        num: 16u32,
        texel_block_size: 3u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "D24S8",
        num: 17u32,
        texel_block_size: 4u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "D32S8",
        num: 18u32,
        texel_block_size: 5u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "BC1_RGB",
        num: 19u32,
        texel_block_size: 8u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "BC1_RGBA",
        num: 20u32,
        texel_block_size: 8u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "BC2",
        num: 21u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "BC3",
        num: 22u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "BC4",
        num: 23u32,
        texel_block_size: 8u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "BC5",
        num: 24u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "BC6H",
        num: 25u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "BC7",
        num: 26u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ETC2_RGB",
        num: 27u32,
        texel_block_size: 8u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ETC2_RGBA",
        num: 28u32,
        texel_block_size: 8u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ETC2_EAC_RGBA",
        num: 29u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "EAC_R",
        num: 30u32,
        texel_block_size: 8u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "EAC_RG",
        num: 31u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_4x4",
        num: 32u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 16u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_5x4",
        num: 33u32,
        texel_block_size: 16u64,
        texel_block_extent: (5u32, 4u32, 1u32),
        texels_per_block: 20u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_5x5",
        num: 34u32,
        texel_block_size: 16u64,
        texel_block_extent: (5u32, 5u32, 1u32),
        texels_per_block: 25u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_6x5",
        num: 35u32,
        texel_block_size: 16u64,
        texel_block_extent: (6u32, 5u32, 1u32),
        texels_per_block: 30u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_6x6",
        num: 36u32,
        texel_block_size: 16u64,
        texel_block_extent: (6u32, 6u32, 1u32),
        texels_per_block: 36u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_8x5",
        num: 37u32,
        texel_block_size: 16u64,
        texel_block_extent: (8u32, 5u32, 1u32),
        texels_per_block: 40u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_8x6",
        num: 38u32,
        texel_block_size: 16u64,
        texel_block_extent: (8u32, 6u32, 1u32),
        texels_per_block: 48u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_8x8",
        num: 39u32,
        texel_block_size: 16u64,
        texel_block_extent: (8u32, 8u32, 1u32),
        texels_per_block: 64u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_10x5",
        num: 40u32,
        texel_block_size: 16u64,
        texel_block_extent: (10u32, 5u32, 1u32),
        texels_per_block: 50u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_10x6",
        num: 41u32,
        texel_block_size: 16u64,
        texel_block_extent: (10u32, 6u32, 1u32),
        texels_per_block: 60u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_10x8",
        num: 42u32,
        texel_block_size: 16u64,
        texel_block_extent: (10u32, 8u32, 1u32),
        texels_per_block: 80u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_10x10",
        num: 43u32,
        texel_block_size: 16u64,
        texel_block_extent: (10u32, 10u32, 1u32),
        texels_per_block: 100u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_12x10",
        num: 44u32,
        texel_block_size: 16u64,
        texel_block_extent: (12u32, 10u32, 1u32),
        texels_per_block: 120u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_12x12",
        num: 45u32,
        texel_block_size: 16u64,
        texel_block_extent: (12u32, 12u32, 1u32),
        texels_per_block: 144u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "32-bit G8B8G8R8",
        num: 46u32,
        texel_block_size: 4u64,
        texel_block_extent: (2u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "32-bit B8G8R8G8",
        num: 47u32,
        texel_block_size: 4u64,
        texel_block_extent: (2u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "8-bit 3-plane 420",
        num: 48u32,
        texel_block_size: 3u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "8-bit 2-plane 420",
        num: 49u32,
        texel_block_size: 3u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "8-bit 3-plane 422",
        num: 50u32,
        texel_block_size: 3u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "8-bit 2-plane 422",
        num: 51u32,
        texel_block_size: 3u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "8-bit 3-plane 444",
        num: 52u32,
        texel_block_size: 3u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "64-bit R10G10B10A10",
        num: 53u32,
        texel_block_size: 8u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "64-bit G10B10G10R10",
        num: 54u32,
        texel_block_size: 8u64,
        texel_block_extent: (2u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "64-bit B10G10R10G10",
        num: 55u32,
        texel_block_size: 8u64,
        texel_block_extent: (2u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "10-bit 3-plane 420",
        num: 56u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "10-bit 2-plane 420",
        num: 57u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "10-bit 3-plane 422",
        num: 58u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "10-bit 2-plane 422",
        num: 59u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "10-bit 3-plane 444",
        num: 60u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "64-bit R12G12B12A12",
        num: 61u32,
        texel_block_size: 8u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "64-bit G12B12G12R12",
        num: 62u32,
        texel_block_size: 8u64,
        texel_block_extent: (2u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "64-bit B12G12R12G12",
        num: 63u32,
        texel_block_size: 8u64,
        texel_block_extent: (2u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "12-bit 3-plane 420",
        num: 64u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "12-bit 2-plane 420",
        num: 65u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "12-bit 3-plane 422",
        num: 66u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "12-bit 2-plane 422",
        num: 67u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "12-bit 3-plane 444",
        num: 68u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "64-bit G16B16G16R16",
        num: 69u32,
        texel_block_size: 8u64,
        texel_block_extent: (2u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "64-bit B16G16R16G16",
        num: 70u32,
        texel_block_size: 8u64,
        texel_block_extent: (2u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "16-bit 3-plane 420",
        num: 71u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "16-bit 2-plane 420",
        num: 72u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "16-bit 3-plane 422",
        num: 73u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "16-bit 2-plane 422",
        num: 74u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "16-bit 3-plane 444",
        num: 75u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "PVRTC1_2BPP",
        num: 76u32,
        texel_block_size: 8u64,
        texel_block_extent: (8u32, 4u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "PVRTC1_4BPP",
        num: 77u32,
        texel_block_size: 8u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "PVRTC2_2BPP",
        num: 78u32,
        texel_block_size: 8u64,
        texel_block_extent: (8u32, 4u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "PVRTC2_4BPP",
        num: 79u32,
        texel_block_size: 8u64,
        texel_block_extent: (4u32, 4u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_3x3x3",
        num: 80u32,
        texel_block_size: 16u64,
        texel_block_extent: (3u32, 3u32, 3u32),
        texels_per_block: 27u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_4x3x3",
        num: 81u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 3u32, 3u32),
        texels_per_block: 36u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_4x4x3",
        num: 82u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 4u32, 3u32),
        texels_per_block: 48u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_4x4x4",
        num: 83u32,
        texel_block_size: 16u64,
        texel_block_extent: (4u32, 4u32, 4u32),
        texels_per_block: 64u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_5x4x4",
        num: 84u32,
        texel_block_size: 16u64,
        texel_block_extent: (5u32, 4u32, 4u32),
        texels_per_block: 80u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_5x5x4",
        num: 85u32,
        texel_block_size: 16u64,
        texel_block_extent: (5u32, 5u32, 4u32),
        texels_per_block: 100u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_5x5x5",
        num: 86u32,
        texel_block_size: 16u64,
        texel_block_extent: (5u32, 5u32, 5u32),
        texels_per_block: 125u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_6x5x5",
        num: 87u32,
        texel_block_size: 16u64,
        texel_block_extent: (6u32, 5u32, 5u32),
        texels_per_block: 150u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_6x6x5",
        num: 88u32,
        texel_block_size: 16u64,
        texel_block_extent: (6u32, 6u32, 5u32),
        texels_per_block: 180u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "ASTC_6x6x6",
        num: 89u32,
        texel_block_size: 16u64,
        texel_block_extent: (6u32, 6u32, 6u32),
        texels_per_block: 216u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "8-bit 2-plane 444",
        num: 90u32,
        texel_block_size: 3u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "10-bit 2-plane 444",
        num: 91u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "12-bit 2-plane 444",
        num: 92u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "16-bit 2-plane 444",
        num: 93u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "64-bit R14G14B14A14",
        num: 94u32,
        texel_block_size: 8u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "14-bit 2-plane 420",
        num: 95u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
    FormatCompatibility {
        name: "14-bit 2-plane 422",
        num: 96u32,
        texel_block_size: 6u64,
        texel_block_extent: (1u32, 1u32, 1u32),
        texels_per_block: 1u8,
        is_alpha_format: false,
    },
];
#[doc = r" An enumeration of all supported Vulkan formats."]
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, CheckedBitPattern, ByteHash, ByteEq, Debug)]
#[allow(non_camel_case_types)]
pub enum Format {
    #[default]
    #[doc = "Specifies that the format is not specified."]
    #[doc = "# Format information"]
    #[doc = "- Class: `undefined` (uncompressed)"]
    #[doc = "- Aspects: ``NONE``"]
    #[doc = "- Texel block size `0`"]
    #[doc = "- Texel block extent `(0, 0, 0)`"]
    #[doc = "- Texels per block `0`"]
    Undefined = 0i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `8-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `1`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R4g4_Unorm_Pack8 = 1i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R4g4b4a4_Unorm_Pack16 = 2i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B4g4r4a4_Unorm_Pack16 = 3i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R5g6b5_Unorm_Pack16 = 4i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B5g6r5_Unorm_Pack16 = 5i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R5g5b5a1_Unorm_Pack16 = 6i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B5g5r5a1_Unorm_Pack16 = 7i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A1r5g5b5_Unorm_Pack16 = 8i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `8-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `1`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8_Unorm = 9i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `8-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `1`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8_Snorm = 10i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `8-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `1`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8_Uscaled = 11i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `8-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `1`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8_Sscaled = 12i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `8-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `1`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8_Uint = 13i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `8-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `1`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8_Sint = 14i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `8-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `1`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8_Srgb = 15i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8_Unorm = 16i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8_Snorm = 17i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8_Uscaled = 18i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8_Sscaled = 19i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8_Uint = 20i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8_Sint = 21i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8_Srgb = 22i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8_Unorm = 23i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8_Snorm = 24i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8_Uscaled = 25i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8_Sscaled = 26i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8_Uint = 27i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8_Sint = 28i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8_Srgb = 29i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8_Unorm = 30i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8_Snorm = 31i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8_Uscaled = 32i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8_Sscaled = 33i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8_Uint = 34i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8_Sint = 35i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `24-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8_Srgb = 36i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8a8_Unorm = 37i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8a8_Snorm = 38i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8a8_Uscaled = 39i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8a8_Sscaled = 40i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8a8_Uint = 41i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8a8_Sint = 42i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R8g8b8a8_Srgb = 43i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8a8_Unorm = 44i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8a8_Snorm = 45i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8a8_Uscaled = 46i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8a8_Sscaled = 47i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8a8_Uint = 48i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8a8_Sint = 49i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B8g8r8a8_Srgb = 50i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A8b8g8r8_Unorm_Pack32 = 51i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A8b8g8r8_Snorm_Pack32 = 52i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A8b8g8r8_Uscaled_Pack32 = 53i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A8b8g8r8_Sscaled_Pack32 = 54i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A8b8g8r8_Uint_Pack32 = 55i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A8b8g8r8_Sint_Pack32 = 56i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A8b8g8r8_Srgb_Pack32 = 57i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2r10g10b10_Unorm_Pack32 = 58i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2r10g10b10_Snorm_Pack32 = 59i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2r10g10b10_Uscaled_Pack32 = 60i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2r10g10b10_Sscaled_Pack32 = 61i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2r10g10b10_Uint_Pack32 = 62i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2r10g10b10_Sint_Pack32 = 63i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2b10g10r10_Unorm_Pack32 = 64i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2b10g10r10_Snorm_Pack32 = 65i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2b10g10r10_Uscaled_Pack32 = 66i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2b10g10r10_Sscaled_Pack32 = 67i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2b10g10r10_Uint_Pack32 = 68i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A2b10g10r10_Sint_Pack32 = 69i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16_Unorm = 70i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16_Snorm = 71i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16_Uscaled = 72i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16_Sscaled = 73i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16_Uint = 74i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16_Sint = 75i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16_Sfloat = 76i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16_Unorm = 77i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16_Snorm = 78i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16_Uscaled = 79i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16_Sscaled = 80i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16_Uint = 81i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16_Sint = 82i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16_Sfloat = 83i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `48-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `6`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16_Unorm = 84i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `48-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `6`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16_Snorm = 85i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `48-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `6`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16_Uscaled = 86i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `48-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `6`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16_Sscaled = 87i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `48-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `6`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16_Uint = 88i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `48-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `6`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16_Sint = 89i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `48-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `6`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16_Sfloat = 90i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16a16_Unorm = 91i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16a16_Snorm = 92i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16a16_Uscaled = 93i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16a16_Sscaled = 94i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16a16_Uint = 95i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16a16_Sint = 96i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R16g16b16a16_Sfloat = 97i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32_Uint = 98i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32_Sint = 99i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32_Sfloat = 100i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32g32_Uint = 101i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32g32_Sint = 102i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32g32_Sfloat = 103i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `96-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `12`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32g32b32_Uint = 104i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `96-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `12`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32g32b32_Sint = 105i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `96-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `12`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32g32b32_Sfloat = 106i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `128-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32g32b32a32_Uint = 107i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `128-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32g32b32a32_Sint = 108i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `128-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R32g32b32a32_Sfloat = 109i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64_Uint = 110i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64_Sint = 111i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `64-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64_Sfloat = 112i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `128-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64g64_Uint = 113i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `128-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64g64_Sint = 114i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `128-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64g64_Sfloat = 115i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `192-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `24`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64g64b64_Uint = 116i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `192-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `24`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64g64b64_Sint = 117i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `192-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `24`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64g64b64_Sfloat = 118i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `256-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `32`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64g64b64a64_Uint = 119i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `256-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `32`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64g64b64a64_Sint = 120i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `256-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `32`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    R64g64b64a64_Sfloat = 121i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    B10g11r11_Ufloat_Pack32 = 122i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `32-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    E5b9g9r9_Ufloat_Pack32 = 123i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `D16` (uncompressed)"]
    #[doc = "- Aspects: `DEPTH`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    D16_Unorm = 124i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `D24` (uncompressed)"]
    #[doc = "- Aspects: `DEPTH`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    X8_D24_Unorm_Pack32 = 125i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `D32` (uncompressed)"]
    #[doc = "- Aspects: `DEPTH`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    D32_Sfloat = 126i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `S8` (uncompressed)"]
    #[doc = "- Aspects: `STENCIL`"]
    #[doc = "- Texel block size `1`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    S8_Uint = 127i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `D16S8` (uncompressed)"]
    #[doc = "- Aspects: `DEPTH | STENCIL`"]
    #[doc = "- Texel block size `3`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    D16_Unorm_S8_Uint = 128i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `D24S8` (uncompressed)"]
    #[doc = "- Aspects: `DEPTH | STENCIL`"]
    #[doc = "- Texel block size `4`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    D24_Unorm_S8_Uint = 129i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `D32S8` (uncompressed)"]
    #[doc = "- Aspects: `STENCIL`"]
    #[doc = "- Texel block size `5`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    D32_Sfloat_S8_Uint = 130i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC1_RGB` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc1_Rgb_Unorm_Block = 131i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC1_RGB` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc1_Rgb_Srgb_Block = 132i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC1_RGBA` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc1_Rgba_Unorm_Block = 133i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC1_RGBA` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc1_Rgba_Srgb_Block = 134i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC2` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc2_Unorm_Block = 135i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC2` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc2_Srgb_Block = 136i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC3` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc3_Unorm_Block = 137i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC3` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc3_Srgb_Block = 138i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC4` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc4_Unorm_Block = 139i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC4` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc4_Snorm_Block = 140i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC5` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc5_Unorm_Block = 141i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC5` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc5_Snorm_Block = 142i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC6H` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc6h_Ufloat_Block = 143i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC6H` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc6h_Sfloat_Block = 144i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC7` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc7_Unorm_Block = 145i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `BC7` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Bc7_Srgb_Block = 146i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ETC2_RGB` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Etc2_R8g8b8_Unorm_Block = 147i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ETC2_RGB` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Etc2_R8g8b8_Srgb_Block = 148i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ETC2_RGBA` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Etc2_R8g8b8a1_Unorm_Block = 149i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ETC2_RGBA` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Etc2_R8g8b8a1_Srgb_Block = 150i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ETC2_EAC_RGBA` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Etc2_R8g8b8a8_Unorm_Block = 151i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ETC2_EAC_RGBA` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Etc2_R8g8b8a8_Srgb_Block = 152i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `EAC_R` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Eac_R11_Unorm_Block = 153i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `EAC_R` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `8`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Eac_R11_Snorm_Block = 154i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `EAC_RG` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Eac_R11g11_Unorm_Block = 155i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `EAC_RG` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Eac_R11g11_Snorm_Block = 156i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_4x4` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Astc_4x4_Unorm_Block = 157i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_4x4` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(4, 4, 1)`"]
    #[doc = "- Texels per block `16`"]
    Astc_4x4_Srgb_Block = 158i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_5x4` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(5, 4, 1)`"]
    #[doc = "- Texels per block `20`"]
    Astc_5x4_Unorm_Block = 159i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_5x4` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(5, 4, 1)`"]
    #[doc = "- Texels per block `20`"]
    Astc_5x4_Srgb_Block = 160i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_5x5` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(5, 5, 1)`"]
    #[doc = "- Texels per block `25`"]
    Astc_5x5_Unorm_Block = 161i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_5x5` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(5, 5, 1)`"]
    #[doc = "- Texels per block `25`"]
    Astc_5x5_Srgb_Block = 162i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_6x5` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(6, 5, 1)`"]
    #[doc = "- Texels per block `30`"]
    Astc_6x5_Unorm_Block = 163i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_6x5` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(6, 5, 1)`"]
    #[doc = "- Texels per block `30`"]
    Astc_6x5_Srgb_Block = 164i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_6x5` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(6, 5, 1)`"]
    #[doc = "- Texels per block `30`"]
    Astc_6x6_Unorm_Block = 165i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_6x5` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(6, 5, 1)`"]
    #[doc = "- Texels per block `30`"]
    Astc_6x6_Srgb_Block = 166i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_8x5` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(8, 5, 1)`"]
    #[doc = "- Texels per block `40`"]
    Astc_8x5_Unorm_Block = 167i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_8x5` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(8, 5, 1)`"]
    #[doc = "- Texels per block `40`"]
    Astc_8x5_Srgb_Block = 168i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_8x6` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(8, 6, 1)`"]
    #[doc = "- Texels per block `48`"]
    Astc_8x6_Unorm_Block = 169i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_8x6` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(8, 6, 1)`"]
    #[doc = "- Texels per block `48`"]
    Astc_8x6_Srgb_Block = 170i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_8x8` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(8, 8, 1)`"]
    #[doc = "- Texels per block `64`"]
    Astc_8x8_Unorm_Block = 171i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_8x8` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(8, 8, 1)`"]
    #[doc = "- Texels per block `64`"]
    Astc_8x8_Srgb_Block = 172i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_10x5` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(10, 5, 1)`"]
    #[doc = "- Texels per block `50`"]
    Astc_10x5_Unorm_Block = 173i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_10x5` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(10, 5, 1)`"]
    #[doc = "- Texels per block `50`"]
    Astc_10x5_Srgb_Block = 174i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_10x6` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(10, 6, 1)`"]
    #[doc = "- Texels per block `60`"]
    Astc_10x6_Unorm_Block = 175i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_10x6` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(10, 6, 1)`"]
    #[doc = "- Texels per block `60`"]
    Astc_10x6_Srgb_Block = 176i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_10x8` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(10, 8, 1)`"]
    #[doc = "- Texels per block `80`"]
    Astc_10x8_Unorm_Block = 177i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_10x8` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(10, 8, 1)`"]
    #[doc = "- Texels per block `80`"]
    Astc_10x8_Srgb_Block = 178i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_10x10` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(10, 10, 1)`"]
    #[doc = "- Texels per block `100`"]
    Astc_10x10_Unorm_Block = 179i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_10x10` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(10, 10, 1)`"]
    #[doc = "- Texels per block `100`"]
    Astc_10x10_Srgb_Block = 180i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_12x10` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(12, 10, 1)`"]
    #[doc = "- Texels per block `120`"]
    Astc_12x10_Unorm_Block = 181i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_12x10` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(12, 10, 1)`"]
    #[doc = "- Texels per block `120`"]
    Astc_12x10_Srgb_Block = 182i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_12x12` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(12, 12, 1)`"]
    #[doc = "- Texels per block `144`"]
    Astc_12x12_Unorm_Block = 183i32,
    #[doc = "# Format information"]
    #[doc = "- Class: `ASTC_12x12` (compressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `16`"]
    #[doc = "- Texel block extent `(12, 12, 1)`"]
    #[doc = "- Texels per block `144`"]
    Astc_12x12_Srgb_Block = 184i32,
    #[doc = "Requires enabling the [`_4444_formats`][1] feature."]
    #[doc = ""]
    #[doc = "[1]: crate::vk_13::_4444_formats"]
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A4r4g4b4_Unorm_Pack16 = 1000340000i32,
    #[doc = "Requires enabling the [`_4444_formats`][1] feature."]
    #[doc = ""]
    #[doc = "[1]: crate::vk_13::_4444_formats"]
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A4b4g4r4_Unorm_Pack16 = 1000340001i32,
    #[doc = "Requires enabling the [`maintenance5`][1] feature."]
    #[doc = ""]
    #[doc = "[1]: vk_14::maintenance5"]
    #[doc = "# Format information"]
    #[doc = "- Class: `16-bit` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `2`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A1b5g5r5_Unorm_Pack16 = 1000470000i32,
    #[doc = "Requires enabling the [`maintenance5`][1] feature."]
    #[doc = ""]
    #[doc = "[1]: vk_14::maintenance5"]
    #[doc = "# Format information"]
    #[doc = "- Class: `8-bit alpha` (uncompressed)"]
    #[doc = "- Aspects: `COLOR`"]
    #[doc = "- Texel block size `1`"]
    #[doc = "- Texel block extent `(1, 1, 1)`"]
    #[doc = "- Texels per block `1`"]
    A8_Unorm = 1000470001i32,
}
unsafe impl NoUninit for Format {}
impl From<Format> for tuhka::vk::Format {
    #[inline]
    fn from(value: Format) -> Self {
        Self::from_raw(value as i32)
    }
}
impl Format {
    pub(super) const fn info(self) -> &'static FormatInfo {
        let raw = self as usize;
        if raw >= 185usize {
            let mut offset = raw % 10;
            let div = raw / 10;
            offset += div % 10 * 10 + (div == 10) as usize * 10;
            let mut ext = (raw / 1000) % 10;
            ext += (raw / 10000) % 10 * 10;
            ext += (raw / 100000) % 10 * 100;
            let index = offset + ext;
            EXT_INFOS[index].as_ref().unwrap()
        } else {
            &CORE_INFOS[raw]
        }
    }
    #[doc = r" Returns whether using this format doesn't require enabling any"]
    #[doc = r" features."]
    #[inline]
    pub const fn is_builtin(self) -> bool {
        (self as usize) < 185usize
    }
    #[doc = r" Returns [`FormatCompatibility`] of this format."]
    #[inline]
    pub const fn compatibility(self) -> &'static FormatCompatibility {
        self.info().compatibility
    }
    #[doc = r" Returns whether `self` is compatible with `other`."]
    #[doc = r""]
    #[doc = r" Returns `true` if their [compatibility] values are equal."]
    #[doc = r""]
    #[doc = r" [compatibility]: Self::compatibility"]
    #[inline]
    pub const fn is_compatible_with(self, other: Self) -> bool {
        self.compatibility().num == other.compatibility().num
    }
    #[doc = r" Returns whether `self` is size-compatible with `other`."]
    #[doc = r""]
    #[doc = r" Returns `true` is neither or both are alpha formats and if the sizes of their texel"]
    #[doc = r" blocks are equal."]
    #[inline]
    pub const fn is_size_compatible_with(self, other: Self) -> bool {
        self.compatibility()
            .is_size_compatible_with(other.compatibility())
    }
    #[doc = r" Converts `self` to `str`."]
    #[inline]
    pub const fn to_str(self) -> &'static str {
        self.info().name
    }
    #[doc = r" Returns the [`NumericFormat`] of the color aspect of this format, which is [`None`]"]
    #[doc = r" if the format doesn't have a color component."]
    #[inline]
    pub const fn numeric_format_color(self) -> Option<NumericFormat> {
        self.info().numeric_color
    }
    #[doc = r" Returns the [`NumericFormat`] of the depth aspect of this format, which is [`None`]"]
    #[doc = r" if the format doesn't have a depth component."]
    #[inline]
    pub const fn numeric_format_depth(self) -> Option<NumericFormat> {
        self.info().numeric_depth
    }
    #[doc = r" Returns the [`NumericFormat`] of the stencil aspect of this format, which is"]
    #[doc = r" [`None`] if the format doesn't have a stencil component."]
    #[inline]
    pub const fn numeric_format_stencil(self) -> Option<NumericFormat> {
        self.info().numeric_stencil
    }
    #[doc = r" Returns whether the format is a compressed format."]
    #[inline]
    pub const fn is_compressed(&self) -> bool {
        self.info().is_compressed
    }
    #[doc = r" Returns whether the format is a depth/stencil format."]
    #[inline]
    pub const fn is_depth_stencil(self) -> bool {
        let info = self.info();
        info.numeric_depth.is_some() || info.numeric_stencil.is_some()
    }
}
impl ::core::fmt::Display for Format {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
static CORE_INFOS: [FormatInfo; 185usize] = [
    FormatInfo {
        name: "Undefined",
        compatibility: &COMPATIBILITY[0usize],
        numeric_color: None,
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R4g4_Unorm_Pack8",
        compatibility: &COMPATIBILITY[1usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R4g4b4a4_Unorm_Pack16",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B4g4r4a4_Unorm_Pack16",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R5g6b5_Unorm_Pack16",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B5g6r5_Unorm_Pack16",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R5g5b5a1_Unorm_Pack16",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B5g5r5a1_Unorm_Pack16",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A1r5g5b5_Unorm_Pack16",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8_Unorm",
        compatibility: &COMPATIBILITY[1usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8_Snorm",
        compatibility: &COMPATIBILITY[1usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8_Uscaled",
        compatibility: &COMPATIBILITY[1usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8_Sscaled",
        compatibility: &COMPATIBILITY[1usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8_Uint",
        compatibility: &COMPATIBILITY[1usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8_Sint",
        compatibility: &COMPATIBILITY[1usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8_Srgb",
        compatibility: &COMPATIBILITY[1usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8_Unorm",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8_Snorm",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8_Uscaled",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8_Sscaled",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8_Uint",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8_Sint",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8_Srgb",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8_Unorm",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8_Snorm",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8_Uscaled",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8_Sscaled",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8_Uint",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8_Sint",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8_Srgb",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8_Unorm",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8_Snorm",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8_Uscaled",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8_Sscaled",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8_Uint",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8_Sint",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8_Srgb",
        compatibility: &COMPATIBILITY[4usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8a8_Unorm",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8a8_Snorm",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8a8_Uscaled",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8a8_Sscaled",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8a8_Uint",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8a8_Sint",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R8g8b8a8_Srgb",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8a8_Unorm",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8a8_Snorm",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8a8_Uscaled",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8a8_Sscaled",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8a8_Uint",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8a8_Sint",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B8g8r8a8_Srgb",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A8b8g8r8_Unorm_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A8b8g8r8_Snorm_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A8b8g8r8_Uscaled_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A8b8g8r8_Sscaled_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A8b8g8r8_Uint_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A8b8g8r8_Sint_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A8b8g8r8_Srgb_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2r10g10b10_Unorm_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2r10g10b10_Snorm_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2r10g10b10_Uscaled_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2r10g10b10_Sscaled_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2r10g10b10_Uint_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2r10g10b10_Sint_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2b10g10r10_Unorm_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2b10g10r10_Snorm_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2b10g10r10_Uscaled_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2b10g10r10_Sscaled_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2b10g10r10_Uint_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "A2b10g10r10_Sint_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16_Unorm",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16_Snorm",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16_Uscaled",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16_Sscaled",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16_Uint",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16_Sint",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16_Sfloat",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16_Unorm",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16_Snorm",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16_Uscaled",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16_Sscaled",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16_Uint",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16_Sint",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16_Sfloat",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16_Unorm",
        compatibility: &COMPATIBILITY[6usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16_Snorm",
        compatibility: &COMPATIBILITY[6usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16_Uscaled",
        compatibility: &COMPATIBILITY[6usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16_Sscaled",
        compatibility: &COMPATIBILITY[6usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16_Uint",
        compatibility: &COMPATIBILITY[6usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16_Sint",
        compatibility: &COMPATIBILITY[6usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16_Sfloat",
        compatibility: &COMPATIBILITY[6usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16a16_Unorm",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16a16_Snorm",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16a16_Uscaled",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Uscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16a16_Sscaled",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Sscaled),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16a16_Uint",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16a16_Sint",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R16g16b16a16_Sfloat",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32_Uint",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32_Sint",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32_Sfloat",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32g32_Uint",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32g32_Sint",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32g32_Sfloat",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32g32b32_Uint",
        compatibility: &COMPATIBILITY[8usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32g32b32_Sint",
        compatibility: &COMPATIBILITY[8usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32g32b32_Sfloat",
        compatibility: &COMPATIBILITY[8usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32g32b32a32_Uint",
        compatibility: &COMPATIBILITY[9usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32g32b32a32_Sint",
        compatibility: &COMPATIBILITY[9usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R32g32b32a32_Sfloat",
        compatibility: &COMPATIBILITY[9usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64_Uint",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64_Sint",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64_Sfloat",
        compatibility: &COMPATIBILITY[7usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64g64_Uint",
        compatibility: &COMPATIBILITY[9usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64g64_Sint",
        compatibility: &COMPATIBILITY[9usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64g64_Sfloat",
        compatibility: &COMPATIBILITY[9usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64g64b64_Uint",
        compatibility: &COMPATIBILITY[10usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64g64b64_Sint",
        compatibility: &COMPATIBILITY[10usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64g64b64_Sfloat",
        compatibility: &COMPATIBILITY[10usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64g64b64a64_Uint",
        compatibility: &COMPATIBILITY[11usize],
        numeric_color: Some(NumericFormat::Uint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64g64b64a64_Sint",
        compatibility: &COMPATIBILITY[11usize],
        numeric_color: Some(NumericFormat::Sint),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "R64g64b64a64_Sfloat",
        compatibility: &COMPATIBILITY[11usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "B10g11r11_Ufloat_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Ufloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "E5b9g9r9_Ufloat_Pack32",
        compatibility: &COMPATIBILITY[5usize],
        numeric_color: Some(NumericFormat::Ufloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "D16_Unorm",
        compatibility: &COMPATIBILITY[12usize],
        numeric_color: None,
        numeric_depth: Some(NumericFormat::Unorm),
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "X8_D24_Unorm_Pack32",
        compatibility: &COMPATIBILITY[13usize],
        numeric_color: None,
        numeric_depth: Some(NumericFormat::Unorm),
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "D32_Sfloat",
        compatibility: &COMPATIBILITY[14usize],
        numeric_color: None,
        numeric_depth: Some(NumericFormat::Sfloat),
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "S8_Uint",
        compatibility: &COMPATIBILITY[15usize],
        numeric_color: None,
        numeric_depth: None,
        numeric_stencil: Some(NumericFormat::Uint),
        is_compressed: false,
    },
    FormatInfo {
        name: "D16_Unorm_S8_Uint",
        compatibility: &COMPATIBILITY[16usize],
        numeric_color: None,
        numeric_depth: Some(NumericFormat::Unorm),
        numeric_stencil: Some(NumericFormat::Uint),
        is_compressed: false,
    },
    FormatInfo {
        name: "D24_Unorm_S8_Uint",
        compatibility: &COMPATIBILITY[17usize],
        numeric_color: None,
        numeric_depth: Some(NumericFormat::Unorm),
        numeric_stencil: Some(NumericFormat::Uint),
        is_compressed: false,
    },
    FormatInfo {
        name: "D32_Sfloat_S8_Uint",
        compatibility: &COMPATIBILITY[18usize],
        numeric_color: None,
        numeric_depth: None,
        numeric_stencil: Some(NumericFormat::Uint),
        is_compressed: false,
    },
    FormatInfo {
        name: "Bc1_Rgb_Unorm_Block",
        compatibility: &COMPATIBILITY[19usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc1_Rgb_Srgb_Block",
        compatibility: &COMPATIBILITY[19usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc1_Rgba_Unorm_Block",
        compatibility: &COMPATIBILITY[20usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc1_Rgba_Srgb_Block",
        compatibility: &COMPATIBILITY[20usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc2_Unorm_Block",
        compatibility: &COMPATIBILITY[21usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc2_Srgb_Block",
        compatibility: &COMPATIBILITY[21usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc3_Unorm_Block",
        compatibility: &COMPATIBILITY[22usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc3_Srgb_Block",
        compatibility: &COMPATIBILITY[22usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc4_Unorm_Block",
        compatibility: &COMPATIBILITY[23usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc4_Snorm_Block",
        compatibility: &COMPATIBILITY[23usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc5_Unorm_Block",
        compatibility: &COMPATIBILITY[24usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "Bc5_Snorm_Block",
        compatibility: &COMPATIBILITY[24usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    },
    FormatInfo {
        name: "Bc6h_Ufloat_Block",
        compatibility: &COMPATIBILITY[25usize],
        numeric_color: Some(NumericFormat::Ufloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc6h_Sfloat_Block",
        compatibility: &COMPATIBILITY[25usize],
        numeric_color: Some(NumericFormat::Sfloat),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc7_Unorm_Block",
        compatibility: &COMPATIBILITY[26usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Bc7_Srgb_Block",
        compatibility: &COMPATIBILITY[26usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Etc2_R8g8b8_Unorm_Block",
        compatibility: &COMPATIBILITY[27usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Etc2_R8g8b8_Srgb_Block",
        compatibility: &COMPATIBILITY[27usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Etc2_R8g8b8a1_Unorm_Block",
        compatibility: &COMPATIBILITY[28usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Etc2_R8g8b8a1_Srgb_Block",
        compatibility: &COMPATIBILITY[28usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Etc2_R8g8b8a8_Unorm_Block",
        compatibility: &COMPATIBILITY[29usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Etc2_R8g8b8a8_Srgb_Block",
        compatibility: &COMPATIBILITY[29usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Eac_R11_Unorm_Block",
        compatibility: &COMPATIBILITY[30usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Eac_R11_Snorm_Block",
        compatibility: &COMPATIBILITY[30usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Eac_R11g11_Unorm_Block",
        compatibility: &COMPATIBILITY[31usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Eac_R11g11_Snorm_Block",
        compatibility: &COMPATIBILITY[31usize],
        numeric_color: Some(NumericFormat::Snorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_4x4_Unorm_Block",
        compatibility: &COMPATIBILITY[32usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_4x4_Srgb_Block",
        compatibility: &COMPATIBILITY[32usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_5x4_Unorm_Block",
        compatibility: &COMPATIBILITY[33usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_5x4_Srgb_Block",
        compatibility: &COMPATIBILITY[33usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_5x5_Unorm_Block",
        compatibility: &COMPATIBILITY[34usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_5x5_Srgb_Block",
        compatibility: &COMPATIBILITY[34usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_6x5_Unorm_Block",
        compatibility: &COMPATIBILITY[35usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_6x5_Srgb_Block",
        compatibility: &COMPATIBILITY[35usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_6x6_Unorm_Block",
        compatibility: &COMPATIBILITY[35usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_6x6_Srgb_Block",
        compatibility: &COMPATIBILITY[35usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_8x5_Unorm_Block",
        compatibility: &COMPATIBILITY[37usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_8x5_Srgb_Block",
        compatibility: &COMPATIBILITY[37usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_8x6_Unorm_Block",
        compatibility: &COMPATIBILITY[38usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_8x6_Srgb_Block",
        compatibility: &COMPATIBILITY[38usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_8x8_Unorm_Block",
        compatibility: &COMPATIBILITY[39usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_8x8_Srgb_Block",
        compatibility: &COMPATIBILITY[39usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_10x5_Unorm_Block",
        compatibility: &COMPATIBILITY[40usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_10x5_Srgb_Block",
        compatibility: &COMPATIBILITY[40usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_10x6_Unorm_Block",
        compatibility: &COMPATIBILITY[41usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_10x6_Srgb_Block",
        compatibility: &COMPATIBILITY[41usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_10x8_Unorm_Block",
        compatibility: &COMPATIBILITY[42usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_10x8_Srgb_Block",
        compatibility: &COMPATIBILITY[42usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_10x10_Unorm_Block",
        compatibility: &COMPATIBILITY[43usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_10x10_Srgb_Block",
        compatibility: &COMPATIBILITY[43usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_12x10_Unorm_Block",
        compatibility: &COMPATIBILITY[44usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_12x10_Srgb_Block",
        compatibility: &COMPATIBILITY[44usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_12x12_Unorm_Block",
        compatibility: &COMPATIBILITY[45usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
    FormatInfo {
        name: "Astc_12x12_Srgb_Block",
        compatibility: &COMPATIBILITY[45usize],
        numeric_color: Some(NumericFormat::Srgb),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: true,
    },
];
static EXT_INFOS: [Option<FormatInfo>; 472usize] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(FormatInfo {
        name: "A4r4g4b4_Unorm_Pack16",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    }),
    Some(FormatInfo {
        name: "A4b4g4r4_Unorm_Pack16",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(FormatInfo {
        name: "A1b5g5r5_Unorm_Pack16",
        compatibility: &COMPATIBILITY[2usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    }),
    Some(FormatInfo {
        name: "A8_Unorm",
        compatibility: &COMPATIBILITY[3usize],
        numeric_color: Some(NumericFormat::Unorm),
        numeric_depth: None,
        numeric_stencil: None,
        is_compressed: false,
    }),
];
