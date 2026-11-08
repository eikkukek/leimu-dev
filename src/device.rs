use core::{
    ffi::CStr,
    hash::{Hash, self},
    borrow::Borrow,
};

use tuhka::vk;

use crate::{
    sync::{Arc, atomic::{self, AtomicU64}},
    mem::{alloc::LocalAlloc, alloc_vec::Vec32, vec32},
    features::{FeatureName, DeviceFeature, self},
    HashMap,
    sorted::SortByKey,
    *,
};

use features::FeatureData;

/// A collection of [`DeviceFeature`] objects.
///
/// [`DeviceFeature`] objects can be created with modules prefixed with `vk_`, like [`vk_core`]
/// which includes all supported Vulkan features up to Vulkan 1.4.
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
}

impl Features {

    /// Creates an empty set of features.
    pub fn new() -> Self {
        Self::from_iter(features::core_features())
    }

    /// Appends device features to this structure.
    pub fn with_features<I>(mut self, features: I) -> Self
        where I: IntoIterator<Item = DeviceFeature>
    {
        self.features.extend(features);
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
        Self { features: iter.into_iter().collect(), }
    }
}

impl<'a> IntoIterator for &'a Features {

    type Item = &'a DeviceFeature;
    type IntoIter = ::core::slice::Iter<'a, DeviceFeature>;

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

/// A unique [`Device`] identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DeviceId(u64);

/// Next [`DeviceId`].
static NEXT_DEVICE_ID: AtomicU64 = AtomicU64::new(0);

/// Contains the handle of and metadata about a device queue.
///
/// It is invalid (but not `unsafe`) to use this with a [`Device`] other than the one this queue
/// originates from.
///
/// To get a device queue, use the [`Device::get_queue`] method.
#[derive(Clone)]
pub struct DeviceQueue {
    device_id: DeviceId,
    label: CompactStr,
    handle: vk::Queue,
}

impl DeviceQueue {

    /// Gets the name of this queue as specified in [`DeviceQueueAttributes::queue_labels`].
    #[inline]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Extracts the [`DeviceId`] of the [`Device`] this queue originates from.
    #[inline]
    pub fn device_id(&self) -> DeviceId {
        self.device_id
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

struct Inner {
    device_id: DeviceId,
    instance: Instance,
    functions: tuhka::Device,
    gpu_info: GpuInfo,
    features: HashMap<ConstName, FeatureData>,
    enabled_extensions: HashSet<&'static CStr>,
    queues: HashSet<DeviceQueue>,
}

impl Drop for Inner {

    fn drop(&mut self) {
        unsafe { self.instance.functions()
            .destroy_device(
                &self.functions,
                None
            );
        }
        log::trace!("destroyed `Device`");
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
        let device_id = DeviceId(NEXT_DEVICE_ID.fetch_add(1, atomic::Ordering::AcqRel));
        let mut queues = HashSet::new();
        for (family_index, queue_index, name) in queue_indices_and_names {
            let handle = unsafe {
                guard.1.get_device_queue(family_index, queue_index)
            };
            let queue = DeviceQueue {
                device_id,
                label: name.into(),
                handle,
            };
            if !queues.insert(queue) {
                Err(err!(
                    "queue name `{name}` repeated"
                ))?
            }
        }
        for (name, alias) in queue_aliases {
            if let Some(queue) = queues.get(name).cloned() {
                if !queues.insert(DeviceQueue { device_id, label: alias.into(), handle: queue.handle, }) {
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
        log::trace!("created `Device`");
        Ok(Self { inner: Inner {
            device_id,
            gpu_info,
            functions: guard.into_inner(),
            instance,
            features,
            enabled_extensions,
            queues,
        }.arced()})
    }

    /// Returns [`tuhka::Device`], which contains all core Vulkan 1.4 device-level function
    /// pointers.
    #[inline]
    pub(crate) fn functions(&self) -> &tuhka::Device {
        &self.inner.functions
    }

    /// Gets the unique [`DeviceId`] identifier of this device.
    pub fn id(&self) -> DeviceId {
        self.inner.device_id
    }

    /// Gets information about the GPU.
    pub fn gpu_info(&self) -> &GpuInfo {
        &self.inner.gpu_info
    }

    /// Returns whether [`pipeline_state`] `T` **can** be dynamic.
    pub fn is_pipeline_state_dynamic_allowed<T: PipelineStateObject>(&self) -> bool {
        todo!()
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
    /// `allocator` is **not** a GPU memory allocator, but a regular allocator implementing
    /// [`LocalAlloc`]. It is used for allocating needed data for of the [`Buffer`] structure.
    /// If the allocator fails to allocate, this function will panic. The total
    /// allocation size can be queried with [`BufferAttributes::calculate_allocation_size`].
    ///
    /// To use the [`GlobalAlloc`][1], you can just pass the unit type `()` for `allocator`.
    ///
    /// # Valid usage
    /// The valid parameters for creating the sampler are described in the fields of
    /// [`BufferAttributes`].
    ///
    /// [1]: std::alloc::GlobalAlloc
    #[inline]
    pub fn create_buffer<A>(
        &self,
        attributes: BufferAttributes<'_>,
        allocator: A,
    ) -> Result<Buffer<A>, Error>
        where A: LocalAlloc + Send + Sync
    {
        Buffer::new(self.clone(), attributes, allocator)
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
