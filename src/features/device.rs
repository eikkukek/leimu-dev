use core::{
    any::Any,
    ops::{Deref, DerefMut},
    marker::PhantomData,
    fmt::{self, Display},
};

use tuhka::vk;

use crate::*;

mod obj {

    use super::*;

    crate::mem::smallbox!(
        pub struct Obj: AnyDeviceFn
    );

    pub type Inner = obj::Obj<128>;

    unsafe impl Send for Inner {}
    unsafe impl Sync for Inner {}

    /// A device function trait object.
    ///
    /// Stores up to 128 bytes inline.
    pub struct DeviceFnObj(pub(super) Inner);


    impl Deref for DeviceFnObj {

        type Target = dyn Any;
        
        #[inline]
        fn deref(&self) -> &Self::Target {
            &*self.0
        }
    }

    impl Clone for DeviceFnObj {

        #[inline]
        fn clone(&self) -> Self {
            self.0.to_obj()
        }
    }
}

pub use obj::DeviceFnObj;


/// An auto-trait for cloning [`DeviceFn`] trait objects.
pub trait AnyDeviceFn: Any + Send + Sync {

    /// Clones self to [`DeviceObj`].
    fn to_obj(&self) -> DeviceFnObj;
}

/// A trait for device functions associated with a [`feature`][super].
pub trait DeviceFn: 'static + Clone + Send + Sync {

    /// Loads feature-associated device functions with [`Device`].
    ///
    /// This **should** return an error if the associated feature is [`not enabled`][1].
    ///
    /// [1]: Device::is_feature_enabled
    fn load(device: &Device) -> Result<Self, Error>;
}

impl<T: DeviceFn> AnyDeviceFn for T {

    fn to_obj(&self) -> DeviceFnObj {
        DeviceFnObj(obj::Inner::new(self.clone()))
    }
}

/// The identifier [`DeviceFn`] type.
#[derive(Clone, Copy)]
pub struct DeviceFnName<T: DeviceFn> {
    pub(crate) name: ConstName,
    _marker: PhantomData<T>,
}

impl<T> DeviceFnName<T>
    where T: DeviceFn
{
   
    /// Creates a new [`DeviceFn`] name.
    #[inline]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: ConstName::new(name),
            _marker: PhantomData,
        }
    }
}

impl<T> Display for DeviceFnName<T>
    where T: DeviceFn
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)
    }
}

/// An extension trait for types implementing [`vk::ExtendsPhysicalDeviceFeatures2`].
pub trait ExtendsPhysicalDeviceFeatures2Ext: Any {
    
    /// Extracts the structure type of self.
    fn s_type(&self) -> vk::StructureType;

    /// Gets mutable reference to [`vk::ExtensDeviceCreateInfo`] trait object.
    ///
    /// This just coerces `&mut self` to `&mut dyn vk::ExtendsDevicecreateInfo`.
    fn as_mut(&mut self) -> &mut dyn vk::ExtendsPhysicalDeviceFeatures2;

    /// Creates a [`ExtendsPhysicalDeviceFeatures2Obj`] containing self.
    fn to_obj(&self) -> ExtendPhysicalDeviceFeatures2Obj;
}

impl<T> ExtendsPhysicalDeviceFeatures2Ext for T
    where T:
        vk::ExtendsPhysicalDeviceFeatures2 +
        Clone + Copy + 'static
{
    fn s_type(&self) -> vk::StructureType {
        self.base_in()
            .s_type
    }

    fn as_mut(&mut self) -> &mut dyn vk::ExtendsPhysicalDeviceFeatures2 {
        self
    }

    fn to_obj(&self) -> ExtendPhysicalDeviceFeatures2Obj {
        ExtendPhysicalDeviceFeatures2Obj::new(*self)
    }
}

/// A [`ExtendsPhysicalDeviceFeatures2Ext`] trait object.
pub struct ExtendPhysicalDeviceFeatures2Obj(Box<dyn ExtendsPhysicalDeviceFeatures2Ext>);

impl ExtendPhysicalDeviceFeatures2Obj {

    /// Creates a new [`ExtendsPhysicalDeviceFeatures2Ext`] trait object.
    #[inline]
    pub fn new<T>(x: T) -> Self
        where T: ExtendsPhysicalDeviceFeatures2Ext + 'static
    {
        Self(Box::new(x))
    }

    /// Coerces the inner value to `T`, if the value is of type `T`.
    #[inline]
    pub fn as_structure<T>(&mut self) -> Option<&mut T>
        where T: ExtendsPhysicalDeviceFeatures2Ext
    {
        let s: &mut dyn Any = &mut *self.0;
        s.is::<T>().then(|| unsafe {
            &mut *<*mut dyn Any>::cast::<T>(s)
        })
    }
}

impl Deref for ExtendPhysicalDeviceFeatures2Obj {

    type Target = dyn ExtendsPhysicalDeviceFeatures2Ext;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DerefMut for ExtendPhysicalDeviceFeatures2Obj {

    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl Clone for ExtendPhysicalDeviceFeatures2Obj {

    #[inline]
    fn clone(&self) -> Self {
        self.to_obj()
    }
}
