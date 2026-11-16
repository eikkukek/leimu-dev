use core::{
    ffi::CStr,
    hash::{Hash, self},
    borrow::Borrow,
    any::TypeId,
    ptr::NonNull,
    slice,
};

use tuhka::vk;

use crate::{
    sync::{Arc, Mutex, MutexGuard},
    mem::{alloc_vec::Vec32, vec32},
    features::{
        FeatureName,
        DeviceFeature,
        EnumSupport,
        EnumSupportObj,
        self,
    },
    HashMap,
    sorted::SortByKey,
    *,
};

use mem::alloc_vec::FixedVec32;

use features::FeatureData;

/// A collection of [`DeviceFeature`] objects.
///
/// [`DeviceFeature`] objects can be created with modules prefixed with `vk_`, like [`vk_core`]
/// which includes all supported Vulkan features up to Vulkan 1.4.
///
/// # Panics
/// - If duplicate features with the same name are found.
///
/// # Examples
/// ```
/// use leimu::{Features, vk_core};
///
/// let features = Features::from_iter([
///     vk_core::push_descriptor::new(),
///     vk_core::shader_zero_initialize_workgroup_memory::new(),
///     vk_core::extended_dynamic_state2::builder()
///         .extended_dynamic_state2(true)
///         .build()
/// ]);
/// ```
pub struct Features {
    features: Vec32<DeviceFeature>,
    names: HashSet<ConstName>,
}

impl Features {

    /// Creates an empty set of features.
    pub fn new() -> Self {
        Self::from_iter(features::core_features())
    }

    /// Whether a set of features with `name` is contained in `self`.
    #[inline]
    pub fn contains_feature<T>(&self, name: FeatureName<T>) -> bool
        where T: Send + Sync + 'static,
    {
        self.names.contains(&name.name)
    }

    /// Appends device features to this structure.
    pub fn with_features<I>(mut self, features: I) -> Self
        where I: IntoIterator<Item = DeviceFeature>
    {
        self.features.extend(features.into_iter().inspect(|f| {
            assert!(self.names.insert(f.name), "duplicate feature `{}`", f.name)
        }));
        self
    }
}

impl Default for Features {

    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<DeviceFeature> for Features {

    fn from_iter<T: IntoIterator<Item = DeviceFeature>>(iter: T) -> Self {
        let mut features: Vec32<_> = features::core_features().into_iter().collect();
        features.extend(iter);
        let mut names = HashSet::new();
        for f in &features {
            assert!(names.insert(f.name), "duplicate feature `{}`", f.name);
        }
        Self { features, names }
    }
}

impl<'a> IntoIterator for &'a Features {

    type Item = &'a DeviceFeature;
    type IntoIter = slice::Iter<'a, DeviceFeature>;

    fn into_iter(self) -> Self::IntoIter {
        self.features.iter()
    }
}

/// Structure describing required device attributes when enumerating GPUs.
#[derive(Default)]
pub struct DeviceAttributes {
    /// Specifies a set of features to enable.
    pub required_features: Features,
}

/// A structure specifying [`Device`] queue creation attributes.
///
/// # Valid usage
/// - The number of elements in [`queue_priorities`] and [`queue_labels`] **must** be equal.
///
/// [`queue_priorities`]: Self::queue_priorities
/// [`queue_labels`]: Self::queue_labels
#[repr(C)]
#[derive(Default, Clone)]
pub struct DeviceQueueAttributes<'a> {
    /// The index of the queue family used to create the queues.
    ///
    /// # Valid usage
    /// The index **must** be unique within the set of `DeviceQueueAttributes` used to create a
    /// [`Device`], and there **must** exist a queue family with that index in [`GpuInfo`].
    pub queue_family_index: u32,
    /// A slice specifying queue priorities within the queue family.
    ///
    /// A higher value means higher priority.
    ///
    /// # Valid usage
    /// - Each value in this slice **must** be in the range `0.0..=1.0` and the number of elements
    ///   **must** be less than or equal to the queried number of queues within the queue family.
    /// - The number of unique priority values **must** be less than or equal to
    ///   [`discrete_queue_priorities`].
    ///
    /// [`discrete_queue_priorities`]: GpuLimits::discrete_queue_priorities
    pub queue_priorities: Vec<f32>,
    /// A slice specifying the names of queues withing the queue family.
    ///
    /// The queues can be queried with [`Device::get_queue`].
    ///
    /// # Valid usage
    /// - The names **must** be unique within all queues across all [`DeviceQueueAttributes`].
    pub queue_labels: Vec<&'a str>,
}
impl SortByKey for DeviceQueueAttributes<'_> {
    type Key = u32;
    #[inline]
    fn key(&self) -> &Self::Key {
        &self.queue_family_index
    }
}

crate::__id!(
    /// A unique [`Device`] identifier.
    DeviceId
);

static DEVICE_LOCK: Mutex<DeviceId> = Mutex::new(DeviceId(0));

/// Represents a single [`Device`] queue.
///
/// To get a device queue, use the [`Device::get_queue`] method.
///
/// # Valid usage
/// It is invalid (but not `unsafe`) to use this with a [`Device`] other than the one this queue
/// originates from.
///
/// [command buffers]: CommandBuffer
#[derive(Clone)]
pub struct DeviceQueue {
    device_id: DeviceId,
    label: CompactStr,
    handle: vk::Queue,
    family_index: u32,
}

impl DeviceQueue {

    /// Gets the name of this queue as specified in [`DeviceQueueAttributes::queue_labels`].
    #[inline]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Gets the [`DeviceId`] of the [`Device`] this queue originates from.
    #[inline]
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }

    /// Gets the queue family index of this queue.
    #[inline]
    pub fn family_index(&self) -> u32 {
        self.family_index
    }

    #[inline]
    pub(crate) fn handle(&self) -> vk::Queue {
        self.handle
    }
}

impl PartialEq for DeviceQueue {

    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Eq for DeviceQueue {}

impl Hash for DeviceQueue {

    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}

impl Borrow<str> for DeviceQueue {

    #[inline]
    fn borrow(&self) -> &str {
        self.label()
    }
}

#[derive(Clone, Copy)]
struct SemaphoreSubmitInfo {
    semaphore: vk::Semaphore,
    value: u64,
    stage_mask: vk::PipelineStageFlags2,
}

struct SubmitCache {
    queue: DeviceQueue,
    command_buffer: vk::CommandBuffer,
    wait_semaphores: (NonNull<SemaphoreSubmitInfo>, u32, u32),
    signal_semaphores: (NonNull<SemaphoreSubmitInfo>, u32, u32),
}

unsafe impl Send for SubmitCache {}
unsafe impl Sync for SubmitCache {}

impl SubmitCache { 
   
    /// Returns `(&mut command_buffer, &mut wait_semaphores, &mut signal_semaphores)`
    ///
    /// The semaphores are the raw parts of a vector.
    #[inline]
    fn get(&mut self) -> (
        &mut vk::CommandBuffer,
        &mut (NonNull<SemaphoreSubmitInfo>, u32, u32),
        &mut (NonNull<SemaphoreSubmitInfo>, u32, u32),
    )
    {
        (
            &mut self.command_buffer,
            &mut self.wait_semaphores,
            &mut self.signal_semaphores,
        )
    }
}

#[derive(Default)]
struct PollState {
    id: u64,
    submit_cache: Vec32<SubmitCache>,
}

impl PollState {
 

    /// Returns `(&mut command_buffer, &mut wait_semaphores, &mut signal_semaphores)`
    ///
    /// The semaphores are the raw parts of a vector.
    #[inline]
    pub fn get_submit_cache(&mut self, queue: DeviceQueue) -> (
        &mut vk::CommandBuffer,
        &mut (NonNull<SemaphoreSubmitInfo>, u32, u32),
        &mut (NonNull<SemaphoreSubmitInfo>, u32, u32),
    ) {
        self.submit_cache.push_mut(SubmitCache {
            queue,
            command_buffer: vk::CommandBuffer::null(),
            wait_semaphores: (NonNull::dangling(), 0, 0),
            signal_semaphores: (NonNull::dangling(), 0, 0),
        }).get()
    }

    #[inline]
    unsafe fn clear_cache(&mut self, alloc: &impl Allocator) {
        for c in &self.submit_cache {
            unsafe {
                let _wait = FixedVec32::from_parts_in(
                    c.wait_semaphores.0,
                    0,
                    c.wait_semaphores.2,
                    alloc
                );
                let _signal = FixedVec32::from_parts_in(
                    c.signal_semaphores.0,
                    0,
                    c.signal_semaphores.2,
                    alloc
                );
            }
        }
        self.submit_cache.clear();
    }
}

struct DeviceLock(MutexGuard<'static, DeviceId>);

unsafe impl Send for DeviceLock {}
unsafe impl Sync for DeviceLock {}

struct Inner {
    device_id: DeviceId,
    _lock: DeviceLock,
    instance: Instance,
    functions: tuhka::Device,
    gpu_info: GpuInfo,
    features: HashMap<ConstName, FeatureData>,
    enabled_extensions: HashSet<&'static CStr>,
    queues: HashSet<DeviceQueue>,
    supported_enums: HashMap<TypeId, HashSet<EnumSupportObj>>,
    poll_state: Mutex<PollState>,
    //supported_flags32: HashMap<TypeId, FlagSupportObj<Flags32>>,
    //supported_flags64: HashMap<TypeId, FlagSupportObj<Flags64>>,
}

impl Drop for Inner {

    fn drop(&mut self) {
        unsafe { self.instance.functions()
            .destroy_device(
                &self.functions,
                None
            );
        }
        log::info!("destroyed `Device`");
    }
}

/// Represents a Vulkan device.
///
/// This is the main interface for interacting with the GPU via `leimu`.
#[must_use]
#[derive(Clone)]
pub struct Device {
    inner: Arc<Inner>,
}

static_assertions::assert_impl_all!(Device: Send, Sync);

/// The result of a successful [`Device::poll`] call.
pub struct Poll {
}

impl Device {

    /// Creates the device from `ctx`.
    /// 
    /// Assumes that required queries have been made.
    pub(crate) fn new<'a>(
        attributes: &DeviceAttributes,
        ctx: &features::FeatureQueryContext,
        queue_attributes: impl IntoIterator<Item = DeviceQueueAttributes<'a>>,
        queue_aliases: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, Error> {
        let mut lock = DEVICE_LOCK.lock();
        use std::collections::hash_map::Entry;
        let gpu_info = ctx.gpu_info().clone();
        let instance = ctx.instance().clone();
        let mut features = HashMap::new();
        let mut ctx = features::FeatureEnableContext::from_query_ctx(ctx);
        for feature in &attributes.required_features {
            let (name, data) = feature.enable_features(&mut ctx);
            match features.entry(name) {
                Entry::Vacant(vac) => { vac.insert(data); },
                Entry::Occupied(_) => {
                    log::warn!("feature `{name}` repeated")
                },
            }
        }
        let queues: Sorted<DeviceQueueAttributes> = queue_attributes
            .into_iter()
            .collect();
        let mut queue_create_infos = vec32![];
        let mut queue_indices_and_names = vec32![];
        let mut queues = queues.iter();
        let mut last_q = queues.next().ok_or_else(|| err!(
            "`queue_attributes` is empty"
        ))?;
        let mut discrete_priorities = Vec32::new();
        for next_q in queues {
            if let Some(q) = gpu_info.queue_families.iter().find(|q| q.index == last_q.queue_family_index) {
                if last_q.queue_priorities.len() as u32 > q.queue_count {
                    Err(err!(
                        "queue count of the queue family at index `{}` is `{}`, which is less than `queue_priorities.len()` `{}`",
                        q.index, q.queue_count, last_q.queue_priorities.len(),
                    ))?
                }
            } else {
                Err(err!(
                    "no queue family with index `{}` exists in the GPU",
                    last_q.queue_family_index,
                ))?
            }
            if last_q.queue_labels.len() != last_q.queue_priorities.len() {
                Err(err!(
                    "`queue_labels.len()` `{}` is not equal to `queue_priorities.len()` `{}`",
                    last_q.queue_labels.len(), last_q.queue_priorities.len(),
                ))?
            }
            if last_q.queue_family_index == next_q.queue_family_index {
                Err(err!(
                    "queue family with index `{}` is repeated in `queue_attributes`",
                    last_q.queue_family_index,
                ))?
            }
            for (i, (&name, priority)) in last_q.queue_labels.iter().zip(&last_q.queue_priorities).enumerate() {
                queue_indices_and_names.push((last_q.queue_family_index, i as u32, name));
                if !discrete_priorities.contains(&priority) {
                    discrete_priorities.push(priority);
                }
            }
            if discrete_priorities.len() > gpu_info.limits.discrete_queue_priorities() {
                Err(err!(
                    "the number of discrete priorities (`{}`) in `queue_priorites` is greater than the maximum `{}`",
                    discrete_priorities.len(), gpu_info.limits.discrete_queue_priorities(),
                ))?
            }
            discrete_priorities.clear();
            last_q = next_q;
            queue_create_infos.push(vk::DeviceQueueCreateInfo::default()
                .queue_family_index(last_q.queue_family_index)
                .queue_priorities(&last_q.queue_priorities)
            );
        }
        if let Some(q) = gpu_info.queue_families.iter().find(|q| q.index == last_q.queue_family_index) {
            if last_q.queue_priorities.len() as u32 > q.queue_count {
                Err(err!(
                    "queue count of the queue family at index `{}` is `{}`, which is less than `queue_priorities.len()` `{}`",
                    q.index, q.queue_count, last_q.queue_priorities.len(),
                ))?
            }
        } else {
            Err(err!(
                "no queue family with index `{}` exists in the GPU",
                last_q.queue_family_index,
            ))?
        }
        if last_q.queue_labels.len() != last_q.queue_priorities.len() {
            Err(err!(
                "`queue_labels.len()` `{}` is not equal to `queue_priorities.len()` `{}`",
                last_q.queue_labels.len(), last_q.queue_priorities.len(),
            ))?
        }
        for (i, (&name, priority)) in last_q.queue_labels.iter().zip(&last_q.queue_priorities).enumerate() {
            queue_indices_and_names.push((last_q.queue_family_index, i as u32, name));
            if !discrete_priorities.contains(&priority) {
                discrete_priorities.push(priority);
            }
        }
        if discrete_priorities.len() > gpu_info.limits.discrete_queue_priorities() {
            Err(err!(
                "the number of discrete priorities (`{}`) in `queue_priorites` is greater than the maximum `{}`",
                discrete_priorities.len(), gpu_info.limits.discrete_queue_priorities(),
            ))?
        }
        queue_create_infos.push(vk::DeviceQueueCreateInfo::default()
            .queue_family_index(last_q.queue_family_index)
            .queue_priorities(&last_q.queue_priorities)
        );
        let enabled_extensions = ctx.required_extensions.clone();
        let p_enabled_extension_names: Vec32<*const i8> = enabled_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect();
        let mut vk_features = vk::PhysicalDeviceFeatures2
            ::default()
            .features(ctx.enable_vulkan_10_features)
            .with(|mut features| {
                for next in ctx.device_features_iter() {
                    features = features.push_next(next)
                }
                features
            });
        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .p_enabled_extension_names(&p_enabled_extension_names)
            .push_next(&mut vk_features);
        let functions = unsafe {
            instance.functions().create_device(
                gpu_info.handle,
                &create_info,
                None
            ).describe_err("failed to create `VkDevice`")?
        };
        struct Guard<'a>(&'a Instance, tuhka::Device);
        impl Drop for Guard<'_> {

            fn drop(&mut self) {
                unsafe {
                    self.0.functions()
                        .destroy_device(&self.1, None);
                }
            }
        }
        impl Guard<'_> {

            #[inline]
            fn into_inner(self) -> tuhka::Device {
                use core::mem::MaybeUninit;
                let s = MaybeUninit::new(self);
                unsafe {
                    (&s.assume_init_ref().1 as *const tuhka::Device)
                    .read()
                }
            }
        }
        let guard = Guard(&instance, functions);
        let device_id = DeviceId::next();
        let mut queues = HashSet::new();
        for (family_index, queue_index, name) in queue_indices_and_names {
            let handle = unsafe {
                guard.1.get_device_queue(family_index, queue_index)
            };
            let queue = DeviceQueue {
                device_id,
                label: name.into(),
                handle,
                family_index,
            };
            if !queues.insert(queue) {
                Err(err!(
                    "queue name `{name}` repeated"
                ))?
            }
        }
        for (name, alias) in queue_aliases {
            if let Some(queue) = queues.get(name).cloned() {
                if !queues.insert(DeviceQueue {
                    device_id,
                    label: alias.into(),
                    handle: queue.handle,
                    family_index: queue.family_index,
                }) {
                    Err(err!(
                        "queue name `{name}` repeated"
                    ))?
                }
            } else {
                Err(err!(
                    "cannot alias queue name `{name}` (no such queue)",
                ))?
            }
        }
        log::info!("created `Device` from GPU `{}`", gpu_info.name);
        *lock = device_id;
        Ok(Self { inner: Inner {
            _lock: DeviceLock(lock),
            device_id,
            gpu_info,
            functions: guard.into_inner(),
            instance,
            features,
            enabled_extensions,
            queues,
            supported_enums: ctx.supported_enums,
            poll_state: Mutex::default(),
        }.arced()})
    }

    /// Returns [`tuhka::Device`], which contains all core Vulkan 1.4 device-level function
    /// pointers.
    #[inline]
    pub(crate) fn functions(&self) -> &tuhka::Device {
        &self.inner.functions
    }
   
    /// Returns the [`Instance`] used to create this device.
    pub fn instance(&self) -> &Instance {
        &self.inner.instance
    }

    /// Gets the unique [`DeviceId`] identifier of this device.
    pub fn id(&self) -> DeviceId {
        self.inner.device_id
    }

    /// Gets information about the GPU.
    pub fn gpu_info(&self) -> &GpuInfo {
        &self.inner.gpu_info
    }

    /// Returns whether an enum `e` is supported.
    fn is_enum_supported<E: EnumSupport>(&self, e: E) -> bool {
        self.inner.supported_enums
            .get(&TypeId::of::<E>())
            .is_some_and(|supported| supported.contains(e.as_raw()))
    }

    /// Returns whether `format` is supported by this device.
    #[inline]
    pub fn is_format_supported(&self, format: Format) -> bool {
        format.is_builtin() ||
        self.is_enum_supported(format)
    }

    /// Gets data associated with a feature.
    pub fn get_feature<T: Send + Sync>(&self, name: FeatureName<T>) -> Option<&T> {
        self.inner.features
            .get(&name.name)
            .map(|f| f.get().unwrap())
    }

    /// Returns whether a Vulkan extension with the exact `Name string` is enabled.
    #[inline]
    pub fn is_extension_enbled(&self, name: &CStr) -> bool {
        self.inner.enabled_extensions.contains(name)
    }

    /// Gets a queue with `name`.
    #[inline]
    pub fn get_queue(&self, name: &str) -> Option<&DeviceQueue> {
        self.inner.queues.get(name)
    }

    /// Creates a [`Sampler`] with `attributes`.
    ///
    /// # Valid usage
    /// The valid parameters for creating the sampler are described in the fields of
    /// [`SamplerAttributes`].
    #[inline]
    pub fn create_sampler(&self, attributes: SamplerAttributes) -> Result<Sampler, Error> {
        Sampler::new(self.clone(), attributes)
    }

    /// Creates a [`Buffer`] with `attributes` and `allocator`.
    ///
    /// # Parameters
    /// - `attributes`: the parameters used to create the buffer.
    /// - `allocator`: this is **not** a GPU memory allocator, but a regular allocator implementing
    ///   [`Allocator`]. It is used for allocating needed data for of the [`Buffer`] structure.
    ///   If the allocator fails to allocate, this function will panic. The total
    ///   allocation size can be queried with [`ImageViewAttributes::calculate_allocation_size`].
    ///
    /// # Valid usage
    /// The valid parameters for creating the buffer are described in the fields of
    /// [`BufferAttributes`].
    ///
    /// [`Allocator`]: core::alloc::Allocator
    #[inline]
    pub fn create_buffer(
        &self,
        attributes: BufferAttributes<'_>,
        allocator: AnyAllocator,
    ) -> Result<Buffer, Error>
    {
        Buffer::new(self.clone(), attributes, allocator)
    }

    /// Gets image creation limits for a set of parameters for a to-be created [`Image`] which are:
    /// - `format`: the [`Format`] of the image,
    /// - `dimensions`: the [`ImageDim`] of the image,
    /// - `usage`: a bitmask specifying what the image **can** be used for,
    /// - `stencil_usage`: an optional bitmask specifying separate usages for the stencil aspect of
    ///   the image,
    /// - `view_compatibility`: a bitmask specifying what kinds of image views **can** be created of
    ///   this image, and
    /// - `view_formats`: formats that **can** be used as the format of image views created of
    ///   this image.
    ///
    /// # Valid usage
    /// - `usage` **must** be a valid combination of [`ImageUsages`] bits and it **must** not be
    ///   empty.
    /// - Each format in `format` and `view_formats` **must** be [supported] by this device.
    /// - If `view_compatibility` doesn't contain [`REINTERPRET_TEXEL_BLOCKS`][1] bit then each format in
    ///   `view_formats` **must** be [compatible] with it, or
    /// - if `format` is a [compressed format] and [`REINTERPRET_TEXEL_BLOCKS`][1] is set then each format in
    ///   `view_formats` **must** be [size compatible] with it.
    /// - If `stencil_usage` is `Some`:
    ///     - it **must** be a valid combination of [`ImageUsages`] bits and it **must** not be
    ///       empty, and
    ///     - if `usage` includes the [`ImageUsages::DEPTH_STENCIL_ATTACHMENT`] bit, `stencil_usage`
    ///       **must** also include that bit, and
    ///     - if `usage` **does not**  include [`ImageUsages::DEPTH_STENCIL_ATTACHMENT`] bit,
    ///       `stencil_usage` **must** also  not include that bit.
    /// 
    /// Additional valid usage of `view_compatibility` is specified in the bits of
    /// [`ImageViewCompatibility`].
    ///
    /// [supported]: Self::is_format_supported
    /// [uncompressed format]: Format::is_compressed
    /// [compatible]: Format::is_compatible_with
    /// [compressed format]: Format::is_compressed
    /// [size compatible]: Format::is_size_compatible_with
    /// [1]: ImageViewCompatibility::REINTERPRET_TEXEL_BLOCKS
    pub fn query_image_creation_limits(
        &self,
        format: Format,
        dimensions: ImageDim,
        usage: ImageUsages,
        stencil_usage: Option<ImageUsages>,
        view_compatibility: ImageViewCompatibility,
        view_formats: &[Format],
    ) -> Result<ImageCreationLimits, FormatQueryError>
    {
        let unknown_usage = usage.difference(ImageUsages::all());
        if usage.is_empty() {
            Err(FormatQueryError::InvalidInput("`usage` is empty".to_string()))?
        } else if !unknown_usage.is_empty() {
            Err(FormatQueryError::InvalidInput(format!(
                "unknown image usage bits `{unknown_usage:b}` detected"
            )))?
        }
        let mut stencil_usage_info = None;
        if let Some(stencil_usage) = stencil_usage {
            let unknown_usage = stencil_usage.difference(ImageUsages::all());
            if stencil_usage.is_empty() {
                Err(FormatQueryError::InvalidInput("`stencil_usage` is empty".to_string()))?
            } else if !unknown_usage.is_empty() {
                Err(FormatQueryError::InvalidInput(format!(
                    "unknown stencil usage bits `{unknown_usage:b}` detected"
                )))?
            }
            if usage.contains(ImageUsages::DEPTH_STENCIL_ATTACHMENT)
                ^ stencil_usage.contains(ImageUsages::DEPTH_STENCIL_ATTACHMENT)
            {
                if usage.contains(ImageUsages::DEPTH_STENCIL_ATTACHMENT) {
                    Err(FormatQueryError::InvalidInput(
                        "the `DEPTH_STENCIL_ATTACHMENT` bit is set in `usage`, but it is not set in `stencil_usage`"
                            .to_string(),
                    ))?
                }
                if stencil_usage.contains(ImageUsages::DEPTH_STENCIL_ATTACHMENT) {
                    Err(FormatQueryError::InvalidInput(
                        "the `DEPTH_STENCIL_ATTACHMENT` bit is set in `stencil_usage`, but it is not set in `usage`"
                         .to_string(),
                    ))?
                }
            }
            stencil_usage_info = Some(vk::ImageStencilUsageCreateInfo::default().stencil_usage(stencil_usage.into()));
        }
        if !self.is_format_supported(format)
        {
            Err(FormatQueryError::FormatNotSupported(format))?
        }
        let mut create_flags = vk::ImageCreateFlags::empty();
        let mut min_array_layers = 1;
        let mut require_cube_extent = false;
        let unknown_compat = view_compatibility.difference(ImageViewCompatibility::all());
        if !unknown_compat.is_empty() {
            Err(FormatQueryError::InvalidInput(format!(
                "unknown view compatibility bits `{unknown_compat:b}` detected"
            )))?
        }
        if view_compatibility.contains(ImageViewCompatibility::CUBE) {
            if dimensions != ImageDim::Dim2 {
                Err(FormatQueryError::InvalidInput(
                    "`ImageViewCompatibility::CUBE` is set, but dimensions is not `Dim2`".to_string(),
                ))?
            }
            min_array_layers = 6;
            require_cube_extent = true;
            create_flags |= vk::ImageCreateFlags::CUBE_COMPATIBLE;
        }
        if view_compatibility.contains(ImageViewCompatibility::DIM3_SLICE_AS_ARRAY)
        {
            if dimensions != ImageDim::Dim3 {
                Err(FormatQueryError::InvalidInput(
                    "`ImageViewCompatibility::DIM3_SLICE_AS_ARRAY` is set, but `dimensions` is not `Dim3`".to_string(),
                ))?
            }
            create_flags |= vk::ImageCreateFlags::TYPE_2D_ARRAY_COMPATIBLE;
        }
        let mut format_list = None;
        let mut info = vk::PhysicalDeviceImageFormatInfo2::default()
            .format(format.into())
            .ty(dimensions.into())
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage.into())
            .flags(create_flags)
            .with(|info|
                if let Some(stencil) = &mut stencil_usage_info {
                    info.push_next(stencil)
                } else {
                    info
                }
            );
        if view_compatibility.contains(ImageViewCompatibility::REINTERPRET_TEXEL_BLOCKS) {
            create_flags |= vk::ImageCreateFlags::MUTABLE_FORMAT | vk::ImageCreateFlags::BLOCK_TEXEL_VIEW_COMPATIBLE;
            if !format.is_compressed() {
                Err(FormatQueryError::InvalidInput(
                    "`ImageViewCompatibility::DIM3_SLICE_AS_ARRAY` is set, but `dimensions` is not `Dim3`".to_string(),
                ))?
            }
            for f in view_formats {
                if !self.is_format_supported(format) {
                    Err(FormatQueryError::InvalidInput(format!(
                        "image format `{f}` is not supported",
                    )))?
                }
                if !f.is_size_compatible_with(format) {
                    Err(FormatQueryError::InvalidInput(format!(
                        "format `{}` is not size-compatible with format `{}`",
                        f, format,
                    )))?
                }
            }
            let list = format_list.insert(vk::ImageFormatListCreateInfo
                ::default()
                .view_formats(bytemuck::cast_slice(view_formats))
            );
            info = info.push_next(list);
        } else if !view_formats.is_empty() {
            create_flags |= vk::ImageCreateFlags::MUTABLE_FORMAT;
            for f in view_formats {
                if !self.is_format_supported(format) {
                    Err(FormatQueryError::InvalidInput(format!(
                        "image format `{f}` is not supported",
                    )))?
                }
                if !f.is_compatible_with(format) {
                    Err(FormatQueryError::InvalidInput(format!(
                        "format `{}` is not compatible with format `{}`",
                        f, format,
                    )))?
                }
            } 
            let list = format_list.insert(vk::ImageFormatListCreateInfo
                ::default()
                .view_formats(bytemuck::cast_slice(view_formats))
            );
            info = info.push_next(list);
        }
        let mut format_properties2 = vk::ImageFormatProperties2::default();
        unsafe { self 
            .instance()
            .functions()
            .get_physical_device_image_format_properties2(
                self.gpu_info().handle,
                &info,
                &mut format_properties2,
            ).map_err(|err| {
                match err {
                    vk::Result::ERROR_FORMAT_NOT_SUPPORTED => {
                        FormatQueryError::FormatNotSupported(format)
                    },
                    vk::Result::ERROR_IMAGE_USAGE_NOT_SUPPORTED_KHR => {
                        FormatQueryError::UsageNotSupported(usage, stencil_usage)
                    },
                    vk::Result::ERROR_VALIDATION_FAILED => {
                        FormatQueryError::InvalidInput("invalid input for unknown reasons (Vulkan validation error)".to_string())
                    },
                    err => FormatQueryError::InternalError(Error::new(Some(err), "internal Vulkan error")),
                }
            })?;
        };
        let format_properties = format_properties2.image_format_properties;
        Ok(ImageCreationLimits {
            max_mip_levels: format_properties.max_mip_levels,
            max_array_layers: format_properties.max_array_layers,
            min_array_layers,
            max_extent: format_properties.max_extent.into(),
            require_cube_extent,
            sample_counts: SampleCounts::from_bits_retain(format_properties.sample_counts.as_raw()),
            create_flags,
        })
    }

    /// Creates an [`Image`] with `attributes` and `allocator`.
    ///
    /// Limits for image creation **can** be queried before creating the image with
    /// [`Self::query_image_creation_limits`]. The limits include things like maximum component-wise
    /// [extent], maximum [mip levels], maximum [array layers], and valid [sample counts].
    ///
    /// # Parameters
    /// - `attributes`: the parameters used to create the image.
    /// - `allocator`: this is **not** a GPU memory allocator, but a regular allocator implementing
    ///   [`Allocator`]. It is used for allocating needed data for of the [`Image`] structure.
    ///   If the allocator fails to allocate, this function will panic. The total
    ///   allocation size can be queried with [`ImageViewAttributes::calculate_allocation_size`].
    ///
    /// # Valid usage
    /// The valid parameters for creating the image are described in the fields of
    /// [`ImageAttributes`] and in the documentation of [`Self::query_image_creation_limits`].
    ///
    /// [extent]: ImageAttributes::extent
    /// [mip levels]: ImageAttributes::mip_levels
    /// [array layers]: ImageAttributes::array_layers
    /// [sample counts]: ImageAttributes::samples
    /// [`Allocator`]: core::alloc:Allocator
    #[inline]
    pub fn create_image(
        &self,
        attributes: ImageAttributes<'_>,
        allocator: AnyAllocator,
    ) -> Result<Image, Error>
    {
        Image::new(self.clone(), &attributes, allocator)
    }

    /// Crates an [`ImageView`] with `image`, `attributes` and `allocator`.
    ///
    /// # Parameters
    /// - `image`: the [`Image`] this view covers.
    /// - `attributes`: the parameters used to create the view.
    /// - `allocator`: this is **not** a GPU memory allocator, but a regular allocator implementing
    ///   [`Allocator`]. It is used for allocating needed data for of the [`ImageView`] structure.
    ///   If the allocator fails to allocate, this function will panic. The total
    ///   allocation size can be queried with [`ImageViewAttributes::calculate_allocation_size`].
    ///
    /// # Valid usage
    /// The valid parameters for creating the image are described in [`ImageViewAttributes`].
    ///
    /// [`Allocator`]: core::alloc::Allocator
    #[inline]
    pub fn create_image_view(
        &self,
        image: Image,
        attributes: ImageViewAttributes<'_>,
        allocator: AnyAllocator,
    ) -> Result<ImageView, Error>
    {
        ImageView::new(image, attributes, allocator)
    }

    /// Creates a [`CommandPool`].
    ///
    /// # Parameters
    /// - `label`: an optional debug label assigned to this command pool.
    /// - `queue`: the [`DeviceQueue`] that [command buffers] allocated from this pool are
    ///   submitted to.
    ///
    /// # Panics
    /// if [`DeviceId`] of `queue` is different from the id of `self`.
    ///
    /// [command buffers]: CommandBuffer
    #[inline]
    pub fn create_command_pool( 
        &self,
        label: Option<&str>,
        queue: DeviceQueue,
    ) -> Result<CommandPool, Error> {
        CommandPool::new(self.clone(), label, queue)
    }

    /// Creates a [`Swapchain`].
    ///
    /// # valid usage
    /// - The [`swapchain`][3] feature **must** be enabled.
    /// - `present_queue` **must** be capable of presentation.
    /// - The [instance][1] of `surface` **must** be the same as the [`instance`][2] of `self` (will
    ///   panic otherwise).
    /// - The [`DeviceId`] of `present_queue` **must** be the same as the device id of `self`.
    ///
    /// [1]: Surface::instance
    /// [2]: Self::instance
    /// [3]: vk_khr::swapchain
    #[inline]
    pub fn create_swapchain<'a>(
        &self,
        label: Option<&str>,
        surface: Surface<'a>,
        present_queue: DeviceQueue,
    ) -> Result<Swapchain<'a>, Error> {
        Swapchain::new(
            label,
            surface,
            self.clone(),
            present_queue,
        )
    }

    /// Advances the runtime once.
    pub fn poll<'a, C, A>(
        &self,
        command_buffers: C,
        alloc: A,
    ) -> Result<Poll, Error>
        where
            C: IntoIterator<Item = &'a mut CommandBuffer>,
            A: Allocator + Clone,
    {
        let mut state = self.inner.poll_state.lock();
        let mut poll_id = state.id + 1;
        for (i, b) in command_buffers.into_iter().enumerate() {
            let queue = b.pool().queue().clone();
            let CommandBufferPollResult {
                command_buffer,
                wait_scope,
                signal_scope,
                wait_semaphores,
                signal_semaphores,
            } = b.poll(
                poll_id,
                &alloc
            ).describe_err(|| format!("failed to poll `CommandBuffer` at index `{i}`"))?;
            let (cb, c_wait_semaphores, c_signal_semaphores) = state.get_submit_cache(queue);
            *cb = command_buffer;
            *c_wait_semaphores = wait_semaphores
                .into_iter()
                .map(|(h, v)| SemaphoreSubmitInfo { semaphore: h, value: v, stage_mask: wait_scope, })
                .collect_in::<FixedVec32<_, _>>(alloc.clone()).describe_err("allocation with `alloc` failed")?
                .into_parts();
            *c_signal_semaphores = signal_semaphores
                .into_iter()
                .map(|(h, v)| SemaphoreSubmitInfo { semaphore: h, value: v, stage_mask: signal_scope, })
                .collect_in::<FixedVec32<_, _>>(alloc.clone()).describe_err("allocation with `alloc` failed")?
                .into_parts();
            poll_id += 1;
        }
        state.id = poll_id;
        for SubmitCache { queue, command_buffer, wait_semaphores, signal_semaphores }
            in &state.submit_cache
        {
            let command_buffer_infos = [vk::CommandBufferSubmitInfo {
                command_buffer: *command_buffer,
                ..Default::default()
            }];
            let wait_semaphore_infos: FixedVec32<_, _> = unsafe { slice::from_raw_parts(
                    wait_semaphores.0.as_ptr(),
                    wait_semaphores.1 as usize,
                )}.iter()
                .copied()
                .map(|SemaphoreSubmitInfo { semaphore, value, stage_mask }| vk::SemaphoreSubmitInfo {
                    semaphore,
                    value,
                    stage_mask,
                    ..Default::default()
                }).collect_in(&alloc).describe_err("allocation with `alloc` failed")?;
            let signal_semaphore_infos: FixedVec32<_, _> = unsafe { slice::from_raw_parts(
                    signal_semaphores.0.as_ptr(),
                    signal_semaphores.1 as usize,
                )}.iter()
                .copied()
                .map(|SemaphoreSubmitInfo { semaphore, value, stage_mask }| vk::SemaphoreSubmitInfo {
                    semaphore,
                    value,
                    stage_mask,
                    ..Default::default()
                }).collect_in(&alloc).describe_err("allocation with `alloc` failed")?;
            let submits = vk::SubmitInfo2::default()
                .wait_semaphore_infos(&wait_semaphore_infos)
                .command_buffer_infos(&command_buffer_infos)
                .signal_semaphore_infos(&signal_semaphore_infos);
            unsafe {
                self.functions()
                    .queue_submit2(queue
                        .handle(),
                        &[submits],
                        vk::Fence::null(),
                    ).describe_err("`vkQueueSumit2` failed")?;
            }
        }
        unsafe {
            state.clear_cache(&alloc);
        }
        Ok(Poll { })
    }

    #[allow(unused_variables)]
    pub(crate) fn allocate_memory(
        &self,
        size: DeviceSize,
        align: DeviceSize,
        memory_type_indices: &[u32],
    ) -> Result<MemoryAllocation, Error> {
        let selected_index = memory_type_indices[0];
        Ok(MemoryAllocation {
            handle: unsafe { self.functions().allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(size)
                    .memory_type_index(selected_index),
                None
            ) }.describe_err("failed to allocate `VkDeviceMemory`")?,
            offset: 0,
            size,
            memory_type_index: selected_index,
        })
    }

    pub(crate) unsafe fn deallocate_memory(
        &self,
        allocation: MemoryAllocation,
    ) {
        unsafe {
            self.functions()
                .free_memory(allocation.handle, None);
        }
    }
}
