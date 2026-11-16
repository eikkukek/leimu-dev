use core::{
    any::Any,
    ops::{Deref, DerefMut},
};

use tuhka::vk;

/// An extension trait for types implementing [`vk::ExtendsPhysicalDeviceFeatures2`].
pub trait ExtendsPhysicalDeviceFeatures2Ext: Any {

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
