use core::slice;

use bytemuck::{CheckedBitPattern, checked::cast};

use bitflags::bitflags;

use tuhka::vk;

use crate::*;

use mem::{vec32, alloc_vec::Vec32};

use features::*;

/// Specifies the type of a GPU.
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, CheckedBitPattern)]
pub enum GpuType {
    /// The type of the GPU can not be described by this enumeration, or it is unknown.
    Other = 0,
    /// Indicates a GPU that is an integrated GPU.
    Integrated = 1,
    /// Indicates a GPU that's separated from the CPU.
    Discrete = 2,
    /// Indicates a "virtual" GPU.
    Virtual = 3, 
    /// Indicates that the device running on a CPU instead of GPU.
    Cpu = 4,
}

impl From<vk::PhysicalDeviceType> for GpuType {

    #[inline]
    fn from(value: vk::PhysicalDeviceType) -> Self {
        cast::<vk::PhysicalDeviceType, Self>(value)
    }
}

bitflags! {
    /// A bitmask specifying the capabilities of a queue family.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct QueueFlags: u32 {
        /// Indicates that queues in this queue family supports graphics operations.
        const GRAPHICS = vk::QueueFlags::GRAPHICS.as_raw();
        /// Indicates that queues in this queue family supports compute operations.
        const COMPUTE  = vk::QueueFlags::COMPUTE.as_raw();
    }
}
crate::__bitflags_display!(QueueFlags);

/// Specifies the properties of a queue family.
#[derive(Clone, Copy)]
pub struct QueueFamilyInfo {
    /// The queue family index.
    pub index: u32,
    /// A bitmask specifying the capabilities of queues in this queue family.
    pub flags: QueueFlags,
    /// The number of queues in this queue family.
    pub queue_count: u32,
}

/// Contains information about each queue family of a GPU.
#[derive(Clone)]
pub struct QueueFamilies {
    queue_families: Vec32<QueueFamilyInfo>,
}

impl QueueFamilies {

    /// Returns an iterator over all contained queue families.
    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, QueueFamilyInfo> {
        self.queue_families.iter()
    }
}

impl FromIterator<QueueFamilyInfo> for QueueFamilies {

    fn from_iter<T: IntoIterator<Item = QueueFamilyInfo>>(iter: T) -> Self {
        Self {
            queue_families: iter.into_iter().collect(),
        }
    }
}

/// Contains information about a GPU.
#[derive(Clone)]
pub struct GpuInfo {
    pub(crate) handle: vk::PhysicalDevice,
    /// The index of this GPU in a set of [`suitable GPUs`][1].
    ///
    /// [1]: Instance::find_suitable_gpus
    pub index: u32,
    /// The name of this GPU.
    pub name: String,
    /// The Vulkan api version of this GPU.
    pub api_version: Version,
    /// Specifies what type of GPU this is.
    pub gpu_type: GpuType,
    /// Limits of this GPU.
    pub limits: GpuLimits,
    /// Contains information about each queue family of this GPU.
    pub queue_families: QueueFamilies,
    pub(crate) memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl GpuInfo {

    pub(crate) fn query_support(
        instance: &Instance,
        handle: vk::PhysicalDevice,
        attributes: &DeviceAttributes,
        index: &mut u32,
    ) -> Result<FeatureQueryContext, Error> {
        let mut maintenance4_properties = vk::PhysicalDeviceMaintenance4Properties::default();
        let mut vk_11_properties = vk::PhysicalDeviceVulkan11Properties::default();
        let mut depth_stenci_resolve_properties = vk::PhysicalDeviceDepthStencilResolveProperties::default();
        let mut properties = vk::PhysicalDeviceProperties2
            ::default()
            .push_next(&mut maintenance4_properties)
            .push_next(&mut vk_11_properties)
            .push_next(&mut depth_stenci_resolve_properties);
        unsafe {
            instance.functions()
                .get_physical_device_properties2(
                    handle,
                    &mut properties,
                );
        };
        let properties = properties.properties;
        let api_version = Version(properties.api_version);
        let gpu_type: GpuType = properties.device_type.into();
        let mut queue_family_properties = vec32![];
        unsafe {
            let len = instance
                .functions()
                .get_physical_device_queue_family_properties2_len(handle);
            queue_family_properties.reserve_exact(len);
            queue_family_properties.resize(len, default());
            instance.functions()
                .get_physical_device_queue_family_properties2(
                    handle,
                    &mut queue_family_properties,
                );
        }
        let mut memory_properties2 = default();
        unsafe {
            instance.functions()
                .get_physical_device_memory_properties2(
                    handle,
                    &mut memory_properties2,
                );
        }
        let max_inline_uniform_total_size = (api_version >= VULKAN_API_VERSION_1_3).then(|| {
            let mut properties = vk::PhysicalDeviceVulkan13Properties::default();
            let mut properties2 = vk::PhysicalDeviceProperties2
                ::default()
                .push_next(&mut properties);
            unsafe {
                instance.functions()
                    .get_physical_device_properties2(handle, &mut properties2);
            }
            properties.max_inline_uniform_total_size
        });
        let mut ctx = FeatureQueryContext::new(
            instance.clone(),
            Self { 
                handle,
                memory_properties: memory_properties2.memory_properties,
                index: *index,
                name: properties
                    .device_name_as_cstr()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_owned(),
                api_version,
                gpu_type,
                limits: GpuLimits {
                    vk_10: properties.limits,
                    depth_stencil_resolve_limits: GpuDepthStencilResolveLimits {
                        supported_depth_resolve_modes: ResolveModeFlags::from_bits_truncate(
                            depth_stenci_resolve_properties
                                .supported_depth_resolve_modes.as_raw(),
                        ),
                        supported_stencil_resolve_modes: ResolveModeFlags::from_bits_truncate(
                            depth_stenci_resolve_properties
                                .supported_stencil_resolve_modes.as_raw(),
                        ),
                        independent_resolve_none: depth_stenci_resolve_properties.independent_resolve_none != 0,
                        independent_resolve: depth_stenci_resolve_properties.independent_resolve != 0,
                    },
                    max_buffer_size: maintenance4_properties.max_buffer_size,
                    max_inline_uniform_total_size,
                    max_multiview_count: vk_11_properties.max_multiview_view_count,
                    supported_subgroup_operations: SubgroupFeatures::from_bits_truncate(vk_11_properties
                        .subgroup_supported_operations.as_raw()
                    ),
                },
                queue_families: queue_family_properties
                    .into_iter()
                    .map(|prop| prop.queue_family_properties)
                    .enumerate()
                    .map(|(i, prop)| QueueFamilyInfo {
                        index: i as u32,
                        flags: QueueFlags::from_bits_truncate(prop.queue_flags.as_raw()),
                        queue_count: prop.queue_count,
                    }).collect(),
            }
        )?;
        for feature in &attributes.required_features {
            feature.query_support(&mut ctx)?;
        }
        *index += 1;
        log::info!("GPU `{}` is suitable", ctx.gpu_info().name);
        Ok(ctx)
    }
}
