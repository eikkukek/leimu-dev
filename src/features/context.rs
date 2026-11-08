use core::{
    any::{TypeId, Any},
    hash::{Hash, self},
    ffi::CStr,
    ops::Deref,
};
use std::collections::HashMap;

use tuhka::vk;

use crate::*;

use error::*;

use mem::{vec32, alloc_vec::Vec32, smallbox};

use features::{
    ExtendPhysicalDeviceFeatures2Obj, ExtendsPhysicalDeviceFeatures2Ext,
};

/// A trait for working with flag support dynamically.
pub trait FlagSupport<Flags>: Any + Send + Sync {

    /// Extracts the raw inner value of a flag.
    fn as_raw(&self) -> Flags;

    /// Adds flags to self.
    fn add_flags(&mut self, rhs: Flags);

    /// Returns any `rhs` flags not present in self, if any.
    fn missing_flags(&self, rhs: Flags) -> Option<Flags>;
    
    /// Creates a [`FlagSupportObj`] containing all current flags set in self.
    fn to_obj(&self) -> FlagSupportObj<Flags>;
}

/// An auto trait for checking enum support.
///
/// This is implemented for all types that implement [`bytemuck::CheckedBitPattern`] and
/// [`bytemuck::NoUninit`].
///
/// # Safety
/// This trait should not be implemented manually.
pub unsafe trait EnumSupport: Any + Send + Sync {

    /// Casts a reference to `self` to a reference to [`i32`].
    fn as_raw(&self) -> &i32;

    /// Creates a [`EnumSupportObj`] containing self.
    fn to_obj(&self) -> EnumSupportObj;
}

trait EnumAuto:
    Send + Sync +
    bytemuck::NoUninit +
    bytemuck::CheckedBitPattern<Bits = i32>
{

    fn as_raw(&self) -> &i32 {
        bytemuck::cast_ref(self)
    }
}

impl<T> EnumAuto for T
    where T:
        Send + Sync +
        bytemuck::NoUninit +
        bytemuck::CheckedBitPattern<Bits = i32>
{}

unsafe impl<T: EnumAuto> EnumSupport for T {

    fn as_raw(&self) -> &i32 {
        <T as EnumAuto>::as_raw(self)
    }

    fn to_obj(&self) -> EnumSupportObj {
        EnumSupportObj::new(*self)
    }
}

mod obj {

    use core::{
        ops::{Deref, DerefMut},
        borrow::Borrow,
    };

    use super::*;

    smallbox!(
        struct FlagSupportData<Rhs>: FlagSupport<Rhs>
    );

    unsafe impl<Rhs> Send for FlagSupportData<Rhs, 8> {}
    unsafe impl<Rhs> Sync for FlagSupportData<Rhs, 8> {}

    /// A [`FlagSupport`] trait object.
    pub struct FlagSupportObj<Rhs>(FlagSupportData<Rhs, 8>);

    impl<Rhs> FlagSupportObj<Rhs> {

        /// Creates a new [`FlagSupport`] trait object.
        #[inline]
        pub fn new<T>(x: T) -> Self
            where T: FlagSupport<Rhs>
        {
            Self(FlagSupportData::new(x))
        }
    }

    impl<Rhs: 'static> Clone for FlagSupportObj<Rhs> 
    {

        #[inline]
        fn clone(&self) -> Self {
            self.0.to_obj()
        }
    }

    impl<Rhs> Deref for FlagSupportObj<Rhs> {

        type Target = dyn FlagSupport<Rhs>;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &*self.0
        }
    }

    impl<Rhs> DerefMut for FlagSupportObj<Rhs> {

        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut *self.0
        }
    }

    smallbox!(
        struct EnumSupportData: EnumSupport
    );

    unsafe impl Send for EnumSupportData<4> {}
    unsafe impl Sync for EnumSupportData<4> {}

    /// An [`EnumSupport`] trait object.
    pub struct EnumSupportObj(EnumSupportData<4>);

    impl EnumSupportObj {

        /// Creates a new [`EnumSupport`] trait object.
        #[inline]
        pub fn new<T>(x: T) -> Self
            where T: EnumSupport
        {
            Self(EnumSupportData::new(x))
        }
    }

    impl Deref for EnumSupportObj {

        type Target = dyn EnumSupport;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &*self.0
        }
    }

    impl DerefMut for EnumSupportObj {

        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut *self.0
        }
    }

    impl Clone for EnumSupportObj {

        #[inline]
        fn clone(&self) -> Self {
            self.to_obj()
        }
    }

    impl Hash for EnumSupportObj {

        #[inline]
        fn hash<H: hash::Hasher>(&self, state: &mut H) {
            self.as_raw().hash(state);
        }
    }

    impl PartialEq for EnumSupportObj {

        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.as_raw() == other.as_raw()
        }
    }

    impl Eq for EnumSupportObj {}

    impl Borrow<i32> for EnumSupportObj 
    {
    
        #[inline]
        fn borrow(&self) -> &i32 {
            self.as_raw()
        }
    }
}

pub use obj::{FlagSupportObj, EnumSupportObj};

/// A structure used when selecting suitable [`physical devices`][1].
///
/// This is used for enabling [`Device`] features and registering supported flags and enums.
///
/// [1]: PhysicalDevice
#[derive(Clone)]
pub(crate) struct FeatureQueryContext {
    instance: Instance,
    gpu_info: GpuInfo,
    extension_properties: Vec32<vk::ExtensionProperties>,
    vulkan_10: vk::PhysicalDeviceFeatures,
    vulkan_11: vk::PhysicalDeviceVulkan11Features<'static>,
    vulkan_12: vk::PhysicalDeviceVulkan12Features<'static>,
    pub(crate) required_extensions: HashSet<&'static CStr>,
}

impl FeatureQueryContext {

    pub fn new(
        instance: Instance,
        gpu_info: GpuInfo,
    ) -> Result<Self, Error> {
        let inst_fn = instance.functions();
        let mut properties = vk::PhysicalDeviceProperties2::default();
        unsafe {
            inst_fn.get_physical_device_properties2(
                gpu_info.handle,
                &mut properties
            );
        }
        if gpu_info.api_version < VULKAN_API_VERSION_1_2 {
            return Err(err!(
                "leimu requires at least Vulkan version 1.2 for physical devices, found version is {}",
                gpu_info.api_version,
            ))
        }
        let mut vulkan_11 = vk::PhysicalDeviceVulkan11Features::default();
        let mut vulkan_12 = vk::PhysicalDeviceVulkan12Features::default();
        let mut features = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut vulkan_11)
            .push_next(&mut vulkan_12);
        unsafe {
            inst_fn.get_physical_device_features2(gpu_info.handle, &mut features);
        }
        let extension_properties = unsafe {
            let mut properties = vec32![default(); inst_fn.enumerate_device_extension_properties_len(
                gpu_info.handle, None
            ).describe_err("failed to enumerate physical device extension properties")?.0];
            inst_fn.enumerate_device_extension_properties(gpu_info.handle, None, &mut properties)
                .describe_err("failed to enumerate physical device extension properties")?;
            properties
        };
        Ok(Self {
            gpu_info,
            instance,
            extension_properties,
            vulkan_10: features.features,
            vulkan_11,
            vulkan_12,
            required_extensions: default(),
        })
    }

    /// Returns the Vulkan API version supported by the [`PhysicalDevice`].
    #[inline]
    pub fn api_version(&self) -> Version {
        self.gpu_info.api_version
    }

    /// Gets the current context's [`GpuInfo`].
    #[inline]
    pub fn gpu_info(&self) -> &GpuInfo {
        &self.gpu_info
    }

    /// Gets this context's [`Instance`].
    #[inline]
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// Returns a filled out [`vk::PhysicalDeviceFeatures`] structure indicating which Vulkan 1.0
    /// device features are supported by the [`PhysicalDevice`].
    #[inline]
    pub fn vulkan_10_features(&self) -> &vk::PhysicalDeviceFeatures {
        &self.vulkan_10
    }

    /// Returns a filled out [`vk::PhysicalDeviceFeatures`] structure indicating which Vulkan 1.1
    /// device features are supported by the [`PhysicalDevice`].
    #[inline]
    pub fn vulkan_11_features(&self) -> &vk::PhysicalDeviceVulkan11Features<'static> {
        &self.vulkan_11
    }

    /// Returns a filled out [`vk::PhysicalDeviceFeatures`] structure indicating which Vulkan 1.2
    /// device features are supported by the [`PhysicalDevice`].
    #[inline]
    pub fn vulkan_12_features(&self) -> &vk::PhysicalDeviceVulkan12Features<'static> {
        &self.vulkan_12
    }

    /// Adds a required Vulkan [`Device`] extension.
    ///
    /// If the extension with `name` is not supported by the [`PhysicalDevice`], this returns an
    /// error.
    #[inline]
    pub fn add_required_extension(&mut self, name: &'static CStr) -> Result<(), Error>
    {
        if self.required_extensions.insert(name) &&
            !self.extension_properties
                .iter()
                .any(|prop| prop.extension_name_as_cstr().unwrap_or_default() == name)
        {
            Err(err!(
                "extension {name:?} not present"
            ))?
        }
        Ok(())
    }

    /// Adds a required Vulkan [`Device`] extension.
    ///
    /// If the extension with `name` is not supported by the [`PhysicalDevice`], this returns false.
    #[inline]
    pub fn add_optional_extension(&mut self, name: &'static CStr) -> bool
    {
        if self.required_extensions.insert(name) &&
            !self.extension_properties
                .iter()
                .any(|prop| prop.extension_name_as_cstr().unwrap_or_default() == name)
        {
            return false
        }
        true
    } 

    /// Fills out any structure `E`, which extends [`vk::PhysicalDeviceFeatures2`].
    #[inline]
    pub fn get_physical_device_features<E>(
        &self,
        out: &mut E 
    ) where E: vk::ExtendsPhysicalDeviceFeatures2 {
        let mut features = vk::PhysicalDeviceFeatures2
            ::default()
            .push_next(out);
        unsafe {
            self.instance.functions().get_physical_device_features2(
                self.gpu_info.handle,
                &mut features
            );
        }
    }

    /// Fills out any structure `E`, which extends [`vk::PhysicalDeviceProperties2`].
    #[inline]
    pub fn get_physical_device_properties<E>(
        &self,
        out: &mut E,
    ) where E: vk::ExtendsPhysicalDeviceProperties2 {
        let mut properties = vk::PhysicalDeviceProperties2
            ::default()
            .push_next(out);
        unsafe {
            self.instance.functions().get_physical_device_properties2(
                self.gpu_info.handle,
                &mut properties,
            );
        }
    }
}

#[derive(Clone)]
pub(crate) struct FeatureEnableContext<'a> {
    query_ctx: &'a FeatureQueryContext,
    pub(crate) enable_vulkan_10_features: vk::PhysicalDeviceFeatures,
    pub(crate) supported_enums: HashMap<TypeId, HashSet<EnumSupportObj>>,
    pub(crate) supported_flags32: HashMap<TypeId, FlagSupportObj<Flags32>>,
    pub(crate) supported_flags64: HashMap<TypeId, FlagSupportObj<Flags64>>,
    enable_features: HashMap<TypeId, ExtendPhysicalDeviceFeatures2Obj>,
}

impl<'a> FeatureEnableContext<'a> {

    pub fn from_query_ctx(ctx: &'a FeatureQueryContext) -> Self {
        Self {
            query_ctx: ctx,
            enable_vulkan_10_features: vk::PhysicalDeviceFeatures::default(),
            supported_enums: HashMap::new(),
            supported_flags32: HashMap::new(),
            supported_flags64: HashMap::new(),
            enable_features: HashMap::new(),
        }
    }

    /// Adds supported [`enums`][1].
    ///
    /// # Safety
    /// - The specifies enums **must** be valid enums for this device.
    ///
    /// [1]: EnumSupport
    #[inline]
    pub unsafe fn add_supported_enums<T, I>(
        &mut self,
        enums: I,
    ) where
        I: IntoIterator<Item = T>,
        T: EnumSupport,
    {
        let e = self.supported_enums
            .entry(TypeId::of::<T>())
            .or_default();
        e.extend(enums.into_iter().map(|e| e.to_obj()));
    }

    /// Adds supported 32-bit [`flags`][1].
    ///
    /// # Safety
    /// - The specified flags **must** be valid flags for this device.
    ///
    /// [1]: FlagSupport
    #[inline]
    pub unsafe fn add_supported_flags32<T>(
        &mut self,
        flags: T,
    ) where T: FlagSupport<Flags32> + Copy {
        self.supported_flags32
            .entry(flags.type_id())
            .and_modify(|f| f.add_flags(flags.as_raw()))
            .or_insert_with(|| FlagSupportObj::new(flags));
    }

    /// Adds supported 64-bit [`flags`][1].
    ///
    /// # Safety
    /// - The specified flags **must** be valid flags for this device.
    ///
    /// [1]: FlagSupport
    #[inline]
    pub unsafe fn add_supported_flags64<T>(
        &mut self,
        flags: T,
    ) where T: FlagSupport<Flags64> + Copy {
        self.supported_flags64
            .entry(flags.type_id())
            .and_modify(|f| f.add_flags(flags.as_raw()))
            .or_insert_with(|| FlagSupportObj::new(flags));
    }

    /// This is used to edit/add a structure of type `E` to the p_next chain of
    /// [`vk::DeviceCreateInfo`].
    #[inline]
    pub fn set_features<T, F>(&mut self, f: F)
        where
            T: ExtendsPhysicalDeviceFeatures2Ext + Default + Copy,
            F: FnOnce(T) -> T
    {
        let default = T::default();
        let features = self.enable_features
            .entry(default.type_id())
            .or_insert_with(|| ExtendPhysicalDeviceFeatures2Obj::new(default))
            .as_structure()
            .unwrap();
        *features = f(*features);
    }

    /// Returns an iterator over all structures added by [`set_features`][Self::set_features].
    #[inline]
    pub fn device_features_iter(&mut self) -> impl Iterator<Item = &mut dyn vk::ExtendsPhysicalDeviceFeatures2>
    {
        self.enable_features.values_mut().map(|f| f.as_mut())
    }
}

impl<'a> Deref for FeatureEnableContext<'a> {

    type Target = FeatureQueryContext;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.query_ctx
    }
}
