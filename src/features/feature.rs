use super::*;

use core::{
    any::Any,
    ops::Deref,
    fmt::{self, Display}
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
        pub(super) struct Enable: Fn[(&mut FeatureEnableContext<'_>) -> Option<FeatureData>]
    );
}

use obj::FeatureDataObj;

/// The name of a [`Device`] feature.
///
/// Can be used to get information about an enabled feature.
#[derive(Debug)]
pub struct FeatureName<T: Send + Sync + 'static> {
    pub(crate) name: ConstName,
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> FeatureName<T> {

    /// Creates a name from a static string.
    #[inline]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: ConstName::new(name),
            _marker: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> Display for FeatureName<T> {
    
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.name, f)
    }
}

impl<T: Send + Sync + 'static> Clone for FeatureName<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { name: self.name, _marker: PhantomData }
    }
}
impl<T: Send + Sync + 'static> Copy for FeatureName<T> {}

/// Information about a device feature.
///
/// Used for checking if a device feature is supported by a GPU as well as setting up
/// relevant structures for enabling those features.
pub struct DeviceFeature {
    pub(super) inner: Feature,
    pub(crate) name: ConstName,
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
        self.inner.query_support(ctx)
    }

    pub(crate) fn enable_features(
        &self,
        ctx: &mut FeatureEnableContext<'_>,
    ) -> Option<(ConstName, FeatureData)> {
        Some((self.name, self.inner.enable(ctx)?))
    }
}

pub(super) struct Feature(inner::Support<8>, inner::Enable<8>);

impl Feature {

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
    ) -> Option<FeatureData> {
        (*self.1)(ctx)
    }
}

/// Creates a [`DeviceFeature`].
///
/// [1]: FeatureEnableContext::set_features
/// [2]: vk::StructureType
pub(super) fn feature<S, E, F: 'static + Send + Sync>(
    fn_support: S,
    fn_enable: E,
    name: FeatureName<F>,
) -> DeviceFeature
    where 
        S: Fn(&mut FeatureQueryContext) -> Result<(), Error> + 'static,
        E: Fn(&mut FeatureEnableContext<'_>) -> Option<F> + 'static
{
    DeviceFeature {
        inner: Feature(
            inner::Support::new(fn_support),
            inner::Enable::new(move |ctx| fn_enable(ctx).map(FeatureData::new))
        ),
        name: name.name,
    }
}
