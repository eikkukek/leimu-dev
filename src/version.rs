use core::fmt::{self, Display};

use tuhka::vk;

/// Represents a semver version with the addition of a `variant`.
///
/// Follows the format of Vulkan versions.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Version(pub(crate) u32);

impl Version {

    /// Creates a new version `variant.major.minor.patch`.
    #[inline]
    pub const fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self(vk::make_api_version(variant, major, minor, patch))
    }

    /// Extracts the `variant` field of this version.
    #[inline]
    pub const fn variant(self) -> u32 {
        vk::api_version_variant(self.0)
    }

    /// Extracts the `major` field of this version.
    #[inline]
    pub const fn major(self) -> u32 {
        vk::api_version_major(self.0)
    }

    /// Extracts the `minor` field of this version.
    #[inline]
    pub const fn minor(self) -> u32 {
        vk::api_version_minor(self.0)
    }

    /// Extracts the `patch` field of this version.
    #[inline]
    pub const fn patch(self) -> u32 {
        vk::api_version_patch(self.0)
    }
    
    /// Extracts the underlying [`u32`] value.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl Default for Version {

    #[inline]
    fn default() -> Self {
        Self::new(0, 0, 1, 0)
    }
}

impl Display for Version {

    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}",
            self.variant(),
            self.major(),
            self.minor(),
            self.patch(),
        )
    }
}
