use super::*;

use core::{
    any::Any,
    ops::Deref,
};
use std::marker::PhantomData;

mod obj {

    use super::*;

    crate::mem::smallbox!(
        pub(super) struct Data: Any + Send + Sync
    );

    unsafe impl Send for Data<128> {}
    unsafe impl Sync for Data<128> {}

    pub(super) struct FeatureDataObj(pub Data<128>);

    impl FeatureDataObj {

        #[inline]
        pub fn new<T>(x: T) -> Self
            where T: 'static + Send + Sync
        {
            Self(Data::new(x))
        }
    }
}

mod inner {
    use super::*;
    crate::mem::smallbox!(
        pub(super) struct Support: Fn[(&mut FeatureQueryContext) -> Result<(), Error>]
    );
    crate::mem::smallbox!(
        pub(super) struct Enable: Fn[(&mut FeatureEnableContext<'_>) -> FeatureData]
    );
}

use obj::FeatureDataObj;

/// The name of a [`Device`] feature.
///
/// Can be used to get information about an enabled feature.
#[derive(Clone, Copy)]
pub struct FeatureName<T: 'static + Send + Sync> {
    pub(crate) name: ConstName,
    _marker: PhantomData<T>,
}

impl<T: 'static + Send + Sync> FeatureName<T> {

    /// Creates a name from a static string.
    #[inline]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: ConstName::new(name),
            _marker: PhantomData,
        }
    }
}

pub(super) enum FeatureEnum {
    Vulkan10(vk_10::Feature),
    Vulkan11(vk_11::Feature),
    Vulkan12(vk_12::Feature),
    Ext(FeatureExt),
}

/// Information about a device feature.
///
/// Used for checking if a device feature is supported by a GPU as well as setting up
/// relevant structures for enabling those features.
pub struct DeviceFeature {
    pub(super) inner: FeatureEnum,
    pub(super) name: ConstName,
}

/// A trait object for any data associated with a [`feature`][features].
///
/// Stores up to 128 bytes inline.
pub(crate) struct FeatureData {
    data: FeatureDataObj,
}

impl FeatureData {

    /// Creates a new feature trait object with any `T` implementing [`Send`], [`Sync`] and [`Any`].
    #[inline]
    pub fn new<T>(data: T) -> Self
        where T: 'static + Send + Sync
    {
        Self {
            data: FeatureDataObj::new(data),
        }
    }

    /// Creates empty trait object.
    pub fn empty() -> Self {
        Self {
            data: FeatureDataObj::new(()),
        }
    }

    /// Tries to retrieve the data used to create this trait object.
    ///
    /// If it was created with a different type of `T`, this returns `None`.
    #[inline]
    pub fn get<T>(&self) -> Option<&T>
        where T: 'static + Send + Sync
    {
        let obj: &dyn Any = self.deref();
        obj.is::<T>().then(|| unsafe {
            &*<*const dyn Any>::cast(obj)
        })
    }
}

impl Deref for FeatureData {

    type Target = dyn Any;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data.0.deref()
    }
}

impl DeviceFeature {

    pub(crate) fn query_support(
        &self,
        ctx: &mut FeatureQueryContext,
    ) -> Result<(), Error> {
        match &self.inner {
            FeatureEnum::Vulkan10(f) => f.query_support(ctx.vulkan_10_features()),
            FeatureEnum::Vulkan11(f) => f.query_support(ctx.vulkan_11_features()),
            FeatureEnum::Vulkan12(f) => f.query_support(ctx.vulkan_12_features()),
            FeatureEnum::Ext(f) => f.query_support(ctx),
        }
    }

    pub(crate) fn enable_features(
        &self,
        ctx: &mut FeatureEnableContext<'_>,
    ) -> (ConstName, FeatureData) {
        (self.name, match &self.inner {
            FeatureEnum::Vulkan10(f) => f.enable(&mut ctx.enable_vulkan_10_features),
            FeatureEnum::Vulkan11(f) => {
                let mut enable = None;
                ctx.set_features(|mut features| { enable = Some(f.enable(&mut features)); features });
                enable.unwrap()
            },
            FeatureEnum::Vulkan12(f) => {
                let mut enable = None;
                ctx.set_features(|mut features| { enable = Some(f.enable(&mut features)); features });
                enable.unwrap()
            },
            FeatureEnum::Ext(f) => f.enable(ctx),
        })
    }
}

pub(super) struct FeatureExt(inner::Support<8>, inner::Enable<8>);

impl FeatureExt {

    #[inline]
    fn query_support(
        &self,
        ctx: &mut FeatureQueryContext, 
    ) -> Result<(), Error> {
        (*self.0)(ctx)
    }

    #[inline]
    fn enable(
        &self,
        ctx: &mut FeatureEnableContext<'_>,
    ) -> FeatureData {
        (*self.1)(ctx)
    }
}

/// Creates a [`DeviceFeature`].
///
/// # Safety
/// Any structure [`added`][1] by custom feature implementations **must** have a valid
/// [`StructureType`][2] for that structure.
///
/// [1]: FeatureEnableContext::set_features
/// [2]: vk::StructureType
pub(crate) unsafe fn feature_ext<S, E, F: 'static + Send + Sync>(
    fn_support: S,
    fn_enable: E,
    name: FeatureName<F>,
) -> DeviceFeature
    where 
        S: Fn(&mut FeatureQueryContext) -> Result<(), Error> + 'static,
        E: Fn(&mut FeatureEnableContext<'_>) -> FeatureData + 'static
{
    DeviceFeature {
        inner: FeatureEnum::Ext(FeatureExt(inner::Support::new(fn_support), inner::Enable::new(fn_enable))),
        name: name.name,
    }
}
