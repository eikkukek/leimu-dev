use core::{
    marker::PhantomData,
    ffi::CStr,
    slice,
};

use raw_window_handle::RawDisplayHandle;

use tuhka::{self, vk, window};

use crate::*;

use mem::{
    alloc_vec::Vec32,
    vec32,
};

use sync::Arc;

struct Inner {
    handle: tuhka::Instance,
    library: tuhka::Library,
    display_handle: Option<RawDisplayHandle>,
}

impl Drop for Inner {

    fn drop(&mut self) {
        unsafe {
            self.library.destroy_instance(
                &self.handle,
                None,
            );
        } 
        log::trace!("destroyed `Instance`");
    }
}

/// The entry point for using `leimu`.
///
/// `Instance` can be created with or without a windowing system with [`new_in_display`][1] and
/// [`new_standalone`][2].
///
/// To enable Vulkan validation layers, set the `KHRONOS_VALIDATION` environment variable to `1`.
///
/// [1]: Self::new_in_display
/// [2]: Self::new_standalone
#[derive(Clone)]
pub struct Instance {
    inner: Arc<Inner>,
}

impl Instance {
  
    /// Creates a new instane with `display_handle`.
    pub fn new_in_display(
        display_handle: RawDisplayHandle,
    ) -> Result<Self, Error> {
        let library = unsafe {
            tuhka::Library::load()
        }.expect("Vulkan is not installed on this system");
        let mut found_layers = vec32![];
        let enable_layers = get_layers(&library, &mut found_layers)?;
        let enable_extensions = get_extensions_in_disp(&library, display_handle)?;
        let app_info = vk::ApplicationInfo::default()
            .application_name(application_name().1)
            .application_version(application_version().as_u32())
            .engine_name(engine_name().1)
            .engine_version(engine_version().as_u32())
            .api_version(vk::make_api_version(0, 1, 4, 0));
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(Some(&app_info))
            .p_enabled_layer_names(unsafe { slice::from_raw_parts(
                enable_layers.as_ptr().cast(),
                enable_layers.len() as usize,
            )}).p_enabled_extension_names(&enable_extensions);
        let handle = unsafe {
            library.create_instance(&create_info, None)
        }.describe_err("failed to create Vulkan instance")?;
        log::trace!("created `Instance` in display");
        Ok(Self { inner: Inner {
            handle,
            library,
            display_handle: Some(display_handle),
        }.arced()})
    }

    /// Creates a new instance in standalone mode without a windowing system.
    pub fn new_standalone() -> Result<Self, Error>
    {
        let library = unsafe {
            tuhka::Library::load()
        }.expect("Vulkan is not installed on this system");
        let mut found_layers = vec32![];
        let enable_layers = get_layers(&library, &mut found_layers)?;
        let enable_extensions = vec32![];
        let app_info = vk::ApplicationInfo::default()
            .application_name(application_name().1)
            .application_version(application_version().as_u32())
            .engine_name(engine_name().1)
            .engine_version(engine_version().as_u32())
            .api_version(vk::make_api_version(0, 1, 4, 0));
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(Some(&app_info))
            .p_enabled_layer_names(unsafe { slice::from_raw_parts(
                enable_layers.as_ptr().cast(),
                enable_layers.len() as usize,
            )}).p_enabled_extension_names(&enable_extensions);
        let handle = unsafe {
            library.create_instance(&create_info, None)
        }.describe_err("failed to create Vulkan instance")?;
        log::trace!("created `Instance`");
        Ok(Self { inner: Inner {
            handle,
            library,
            display_handle: None,
        }.arced()})
    }

    /// Returns [`tuhka::Instance`], which contains all core Vulkan 1.4 instance-level functions
    /// pointers.
    pub fn functions(&self) -> &tuhka::Instance {
        &self.inner.handle
    }

    /// Returns whether this instance was created with a [`display handle`][1].
    ///
    /// [1]: RawDisplayHandle
    #[inline]
    pub fn has_display_handle(&self) -> bool {
        self.inner.display_handle.is_some()
    }

    /// Finds all GPUs that match the given `attributes` and returns [`SuitableGpus`], which can be
    /// used to select the best matching GPU and create a [`Device`].
    pub fn find_suitable_gpus(
        &self,
        attributes: DeviceAttributes,
    ) -> Result<SuitableGpus, Error> {
        let mut devices = vec32![];
        unsafe {
            let len = self.functions()
                .enumerate_physical_devices_len()
                .describe_err("failed to enumerate GPUs")?.0;
            devices.reserve_exact(len);
            devices.set_len(len);
            self.functions()
                .enumerate_physical_devices(&mut devices).unwrap();
        }
        let mut suitable = vec32![];
        let mut index = 0;
        for handle in devices {
            match GpuInfo::query_support(
                self,
                handle,
                &attributes,
                &mut index,
            ) {
                Ok(ctx) => suitable.push(ctx),
                Err(err) => log::warn!("{err}"),
            }
        }
        Ok(SuitableGpus { attributes, gpus: suitable, })
    }
}

/// Contains information about a set of suitable GPUs returned by [`find_suitable_gpus`][1].
///
/// [1]: Instance::find_suitable_gpus
pub struct SuitableGpus {
    attributes: DeviceAttributes,
    gpus: Vec32<features::FeatureQueryContext>,
}

crate::__iter_wrapper! {
    /// An iterator over suitable GPUs.
    ///
    /// Yields [`GpuInfo`].
    pub struct SuitableGpuIter<'a>(iter::IterSliceMap<
        'a,
        features::FeatureQueryContext,
        &'a GpuInfo
    >): &'a GpuInfo;
}

impl SuitableGpus {

    /// Returns an iterator over all suitable GPUs.
    #[inline]
    pub fn gpus(
        &self
    ) -> SuitableGpuIter<'_>
    {
        SuitableGpuIter(self.gpus
            .iter()
            .map(|ctx| ctx.gpu_info()))
    }

    /// Returns [`GpuInfo`] at `gpu_index`.
    ///
    /// Panics if `gpu_index` is not the [`index`][1] of one of the gpus queried with
    /// [`Self::gpus`].
    ///
    /// [1]: GpuInfo::index
    #[inline]
    pub fn get_gpu(&self, gpu_index: u32) -> &GpuInfo {
        self.gpus[gpu_index as usize].gpu_info()
    }

    /// Creates a [`Device`] with `gpu_index`.
    ///
    /// Panics if `gpu_index` is not the [`index`][1] of one of the gpus queried with
    /// [`Self::gpus`].
    ///
    /// # Valid usage
    /// - `queue_attributes` **must** contain at least one item.
    /// - If `queue_aliases` contains aliases aliasing other aliases, the other alias **must**
    ///   appear first in the set of aliases.
    /// - Each `alias` in `queue_aliases` **must** be a unique queue name.
    ///
    /// # Parameters
    /// - `gpu_index` the GPU index in [`gpus`][Self::gpus].
    /// - `queue_attributes`: an iterator yielding [`DeviceQueueAttributes`] specifying attributes
    ///   for creating [device queues][DeviceQueue].
    /// - `queue_aliases`: an iterator yielding `(name, alias)` pairs specifying aliases for
    ///   [`queue_labels`][DeviceQueueAttributes::queue_labels].
    ///
    /// [1]: GpuInfo::index
    #[inline]
    pub fn create_device<'a, I, A>(
        &self,
        gpu_index: u32,
        queue_attributes: I,
        queue_aliases: A,
    ) -> Result<Device, Error>
        where
            I: IntoIterator<Item = DeviceQueueAttributes<'a>>,
            A: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let ctx = &self.gpus[gpu_index as usize];
        Device::new(
            &self.attributes,
            ctx,
            queue_attributes,
            queue_aliases,
        )
    }
}

#[repr(transparent)]
struct LtPtr<'a, T>(*const T, PhantomData<&'a ()>);

fn get_layers<'a>(
    library: &tuhka::Library,
    found_layers: &'a mut Vec32<vk::LayerProperties>,
) -> Result<Vec32<LtPtr<'a, i8>>, Error>
{
    let mut enabled_layers = vec32![];
    let enable_validation = std::env
        ::vars()
        .any(|(var, value)|
            var == "KHRONOS_VALIDATION" &&
            value == "1"
        );
    if enable_validation {
        log::trace!("enabling `VK_LAYER_KHRONOS_validation`");
        unsafe {
            let len = library.enumerate_instance_layer_properties_len()
                .describe_err("failed to enumerate instance layers")?.0;
            found_layers.reserve_exact(len);
            found_layers.set_len(len);
            bytemuck::fill_zeroes(found_layers);
            library.enumerate_instance_layer_properties(found_layers).unwrap();
            if let Some(layer) = found_layers.iter().find(|layer| {
                layer.layer_name_as_cstr().unwrap_or_default() == c"VK_LAYER_KHRONOS_validation"
            }) {
                log::trace!("found `VK_LAYER_KHRONOS_validation`");
                enabled_layers.push(LtPtr(layer.layer_name.as_ptr(), PhantomData));
            } else {
                log::warn!("VK_LAYER_KHRONOS_validation not available");
            }
        }
    }
    Ok(enabled_layers)
}

fn get_extensions_in_disp(
    library: &tuhka::Library,
    display_handle: RawDisplayHandle,
) -> Result<Vec32<*const i8>, Error> {
    let mut required = HashSet::<&'static CStr>::new();
    let extensions = window::required_instance_extensions(display_handle)
        .ok_or_else(|| Error::new(None, "unsupported platform"))?;
    required.extend(extensions.iter());
    let mut enable = vec32![];
    unsafe {
        let len = library.enumerate_instance_extension_properties_len(None)
            .describe_err("failed to enumerate instance extensions")?
            .0;
        let mut found_extensions = vec32![];
        found_extensions.reserve_exact(len);
        found_extensions.set_len(len);
        bytemuck::fill_zeroes(&mut found_extensions);
        library.enumerate_instance_extension_properties(None, &mut found_extensions)
            .unwrap();
        for ext in found_extensions {
            let ext = ext.extension_name_as_cstr().unwrap_or_default();
            if let Some(&r) = required.get(&ext) {
                required.remove(ext);
                enable.push(r.as_ptr());
            }
        }
    }
    if !required.is_empty() {
        for r in required {
            log::warn!("failed to find required instance extension {r:?}");
        }
        return Err(Error::new(None, "missing required instance extensions"))
    }
    Ok(enable)
}
