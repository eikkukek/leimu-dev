//! A modern Vulkan graphics API for Rust.

#![warn(rustdoc::all)]
#![warn(missing_docs)]

pub mod error;

pub(crate) mod sync;
#[doc(hidden)]
pub mod vec;
pub mod sorted;
pub mod iter;
mod string;
mod format;
mod flags;
mod enums;
mod structs;
mod const_name;
mod version;
mod instance;
mod limits;
mod gpu;
mod features;
mod memory;
mod device;
mod command;
mod pipeline;
mod address;
pub mod attributes;
mod sampler;
mod image;
mod buffer;
mod surface;
mod swapchain;
mod shader;
mod queries;
mod statics;

pub(crate) mod macros;

pub use leimu_mem as mem;
pub use leimu_spirv as spirv;
pub use spirv_tools;
pub use bytemuck;

pub use error::prelude::*;

pub use vec::VecWrapper;
pub use sorted::Sorted;
pub use string::CompactStr;
pub use version::Version;
pub use format::*;
pub use flags::*;
pub use enums::*;
pub use structs::*;
pub use instance::*;
pub use limits::*;
pub use gpu::*;
pub use command::*;
pub use memory::*;
pub use device::*;
pub use pipeline::*;
pub use address::*;
pub use sampler::*;
pub use image::*;
pub use buffer::*;
pub use surface::*;
pub use swapchain::*;
pub use queries::*;
pub use shader::*;

pub use raw_window_handle;

pub(crate) use const_name::ConstName;
pub(crate) use ahash::AHashMap as HashMap;
pub(crate) use ahash::AHashSet as HashSet;
pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;
pub(crate) type IndexSet<T> = indexmap::IndexSet<T, ahash::RandomState>;
pub(crate) use statics::*;

pub(crate) trait Arced: Sized {

    fn arced(self) -> sync::Arc<Self> {
        sync::Arc::new(self)
    }
}

impl<T: Sized> Arced for T {}

pub use features::prelude::*;

mod env {

    use super::*;

    use std::ffi::{CStr, CString};

    /// Gets the version of the application through the `CARGO_PKG_VERSION_*` environment
    /// variables.
    pub fn application_version() -> Version {
        use crate::sync::LazyLock;
        static VERSION: LazyLock<Version> = LazyLock::new(|| {
            let mut major = 0;
            let mut minor = 0;
            let mut patch = 0;
            for (var, value) in std::env::vars() {
                if var == "CARGO_PKG_VERSION_MAJOR" {
                    major = str::parse(&value).unwrap();
                } else if var == "CARGO_PKG_VERSION_MINOR" {
                    minor = str::parse(&value).unwrap();
                } else if var == "CARGO_PKG_VERSION_PATCH" {
                    patch = str::parse(&value).unwrap();
                }
            }
            Version::new(0, major, minor, patch)
        });
        *VERSION
    }

    /// Gets the name of the application through the `CARGO_PKG_NAME` environment
    /// variable.
    ///
    /// Returns the name as both `str` and `CStr` as a `(&str, &CStr)` tuple.
    pub fn application_name() -> (&'static str, &'static CStr) {
        use crate::sync::LazyLock;
        static NAME: LazyLock<(String, CString)> = LazyLock::new(|| {
            let value = std::env::vars()
                .find(|(var, _)| var == "CARGO_PKG_NAME")
                .map(|(_, value)| value)
                .unwrap_or_default();
            let mut bytes: Vec<u8> = value.as_bytes().to_vec();
            bytes.push(0);
            let cvalue = CString
                ::from_vec_with_nul(bytes)
                .unwrap();
            (value, cvalue)
        });
        (&NAME.0, &NAME.1)
    }

    /// Returns the version of `leimu`.
    pub fn engine_version() -> Version {
        Version::new(0, 0, 1, 0)
    }

    /// Returns the name "leimu".
    ///
    /// Returns the name as both `str` and `CStr` as a `(&str, &CStr)` tuple.
    pub fn engine_name() -> (&'static str, &'static CStr) {
        ("leimu", c"leimu")
    }
}

pub use env::*;

/// Vulkan API version 1.0.
pub const VULKAN_API_VERSION_1_0: Version = Version(tuhka::vk::API_VERSION_1_0);
/// Vulkan API version 1.1.
pub const VULKAN_API_VERSION_1_1: Version = Version(tuhka::vk::API_VERSION_1_1);
/// Vulkan API version 1.2.
pub const VULKAN_API_VERSION_1_2: Version = Version(tuhka::vk::API_VERSION_1_2);
/// Vulkan API version 1.3.
pub const VULKAN_API_VERSION_1_3: Version = Version(tuhka::vk::API_VERSION_1_3);
/// Vulkan API version 1.4.
pub const VULKAN_API_VERSION_1_4: Version = Version(tuhka::vk::API_VERSION_1_4);

pub(crate) fn default<T: Default>() -> T {
    T::default()
}

/// Macro for creating generic vector types ([`VecWrapper`]).
///
/// # Example
/// ``` rust
/// use leimu::vec_wrap;
/// use leimu::pipeline_state::ColorBlendEnable;
/// let state: ColorBlendEnable = vec_wrap[true, false, true, true];
/// ```
#[macro_export]
macro_rules! vec_wrap {
    [$v:expr; $n:expr] => {
        $crate::vec::VecWrapper::with_len($v, $n)
    };
    [$($elem:expr),* $(,)?] => {
        $crate::vec::VecWrapper::from_iter([$($elem),*])
    };
}

pub(crate) trait With: Sized {

    #[inline]
    fn with<F: FnOnce(Self) -> Self>(self, f: F) -> Self {
        f(self)
    }
}

impl<T: Sized> With for T {}

pub(crate) fn eq<T: Eq + ?Sized>(a: &T, b: &T) -> bool {
    a == b
}

use core::alloc::Allocator;

/// [`Allocator`] + [`Send`] + [`Sync`]
///
/// [`Allocator`]: core::alloc::Allocator
pub trait AllocatorSendSync: Allocator + Send + Sync {}
impl<T: Allocator + Send + Sync> AllocatorSendSync for T {}

impl Format {

    /// Gets the [`ImageAspects`] of this format.
    #[inline]
    pub fn aspects(self) -> ImageAspects {
        let mut aspects = ImageAspects::empty();
        aspects.set(ImageAspects::COLOR, self.numeric_format_color().is_some() );
        aspects.set(ImageAspects::DEPTH, self.numeric_format_depth().is_some() );
        aspects.set(ImageAspects::STENCIL, self.numeric_format_stencil().is_some());
        aspects
    }
}

/// A wrapper around any allocator implementing [`Allocator`], [`Send`] and [`Sync`].
#[derive(Default, Clone)]
pub enum AnyAllocator {
    /// Uses the [`Global`] allocator.
    ///
    /// [`Global`]: std::alloc::Global
    #[default]
    Global,
    /// Uses any allocator implementing [`Allocator`], [`Send`] and [`Sync`].
    Other(sync::Arc<dyn AllocatorSendSync>),
}

unsafe impl Allocator for AnyAllocator {

    #[inline]
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        match self {
            Self::Global => std::alloc::Global.allocate(layout),
            Self::Other(a) => a.allocate(layout),
        }
    }

    #[inline]
    fn allocate_zeroed(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        match self {
            Self::Global => std::alloc::Global.allocate_zeroed(layout),
            Self::Other(a) => a.allocate_zeroed(layout),
        }
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        unsafe {
            match self {
                Self::Global => std::alloc::Global.deallocate(ptr, layout),
                Self::Other(a) => a.deallocate(ptr, layout),
            }
        }
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: std::ptr::NonNull<u8>,
        old_layout: std::alloc::Layout,
        new_layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError>
    {
        unsafe {
            match self {
                Self::Global => std::alloc::Global.grow(ptr, old_layout, new_layout),
                Self::Other(a) => a.grow(ptr, old_layout, new_layout),
            }
        }
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: std::ptr::NonNull<u8>,
        old_layout: std::alloc::Layout,
        new_layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError>
    {
        unsafe {
            match self {
                Self::Global => std::alloc::Global.grow_zeroed(ptr, old_layout, new_layout),
                Self::Other(a) => a.grow_zeroed(ptr, old_layout, new_layout)
            }
        }
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: std::ptr::NonNull<u8>,
        old_layout: std::alloc::Layout,
        new_layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError>
    {
        unsafe {
            match self {
                Self::Global => std::alloc::Global.shrink(ptr, old_layout, new_layout),
                Self::Other(a) => a.shrink(ptr, old_layout, new_layout),
            }
        }
    }
}

/// Constructs [`AnyAllocator`] from allocator `a`.
pub fn any_allocator<A: AllocatorSendSync + 'static>(a: A) -> AnyAllocator {
    AnyAllocator::Other(sync::Arc::new(a))
}

use mem::iter::CollectIn;

/// Constructs [`Duration`] from seconds in `f32`.
///
/// [`Duration`]: core::time::Duration
#[inline]
pub fn duration_from_secs(secs: f32) -> core::time::Duration {
    core::time::Duration::from_secs_f32(secs)
}
