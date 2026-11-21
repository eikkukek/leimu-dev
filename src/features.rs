//! Contains types and functions for enabling [`Device`] features.
//!
//! # Notable user-facing types and functions
//! - [`DeviceFeature`]: An opaque structure containing all relevant data of a feature.
//! - [`FeatureName`]: A special string type, which can be used to retrieve the data associated with
//!   a feature through [`Device::get_feature`].

mod feature;
mod functions;
mod context;
mod device;
mod generated;

use tuhka::vk;

use crate::*;

use error::*;

pub(crate) use feature::*;
pub(crate) use context::*;
pub(crate) use device::*;
pub use functions::*;

/// Core Vulkan 1.0 [`features`][Features].
pub mod vk_10 {
    pub use super::generated::vk_10::*;
}

/// Core Vulkan 1.1 [`features`][Features].
pub mod vk_11 {

    use super::*;

    pub use generated::vk_11::*;
}

/// Core Vulkan 1.2 [`features`][Features].
pub mod vk_12 {

    use super::*;

    pub use generated::vk_12::*;

    #[inline]
    pub(crate) fn timeline_semaphore() -> DeviceFeature {
        const NAME: FeatureName<()> = FeatureName::new("timeline_semaphore");
        feature(
            |ctx| {
                let features = ctx.get_vulkan_12_features();
                if features.timeline_semaphore == 0 {
                    Err(err!(
                        "missing required feature `vk_12::timeline_semaphore`"
                    ))?
                }
                Ok(())
            },
            |ctx| {
                ctx.set_features(|features: vk::PhysicalDeviceVulkan12Features| {
                    features.timeline_semaphore(true)
                });
                Some(())
            },
            NAME,
        )
    }

    #[inline]
    pub(crate) fn separate_depth_stencil_layouts() -> DeviceFeature {
        const NAME: FeatureName<()> = FeatureName::new("separate_depth_stencil_layouts");
        feature(
            |ctx| {
                let features = ctx.get_vulkan_12_features();
                if features.separate_depth_stencil_layouts == 0 {
                    Err(err!(
                        "missing required feature `vk_12::separate_depth_stencil_layouts`"
                    ))?
                }
                Ok(())
            },
            |ctx| {
                ctx.set_features(|features: vk::PhysicalDeviceVulkan12Features| {
                    features.separate_depth_stencil_layouts(true)
                });
                Some(())
            },
            NAME,
        )
    }
}

/// Core Vulkan 1.3 [`features`][Features].
pub mod vk_13 {

    use crate::PipelineStateKind;

    use super::*;

    pub use generated::vk_13::*;

    pub(crate) fn synchronization2() -> DeviceFeature {
        use tuhka::khr;
        static NAME: FeatureName<()> = FeatureName::new("synchronization2");
        feature(
            move |ctx| -> Result<(), Error> {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.add_required_extension(khr::synchronization2::NAME)?;
                }
                let mut features = vk::PhysicalDeviceSynchronization2Features
                    ::default();
                ctx.get_physical_device_features(&mut features);
                if features.synchronization2 == 0 {
                    Err(err!("required feature `vk_13::synchronization2` not supported"))
                } else {
                    Ok(())
                }
            },
            move |ctx| {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.set_features(
                        |features: vk::PhysicalDeviceSynchronization2Features|
                            features.synchronization2(true)
                    );
                } else {
                    ctx.set_features(
                        |features: vk::PhysicalDeviceVulkan13Features|
                            features.synchronization2(true)
                    );
                }
                Some(())
            },
            NAME,
        )
    } 

    pub(crate) fn dynamic_rendering() -> DeviceFeature {
        use tuhka::khr;
        static NAME: FeatureName<()> = FeatureName::new("dynamic_rendering");
        feature(
            move |ctx| -> Result<(), Error> {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.add_required_extension(khr::dynamic_rendering::NAME)?;
                }
                let mut features = vk::PhysicalDeviceDynamicRenderingFeatures
                    ::default();
                ctx.get_physical_device_features(&mut features);
                if features.dynamic_rendering == 0 {
                    Err(err!("required feature `vk_13::dynamic_rendering` not supported"))
                } else {
                    Ok(())
                }
            },
            move |ctx| {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.set_features(
                        |features: vk::PhysicalDeviceDynamicRenderingFeatures|
                            features.dynamic_rendering(true)
                    );
                } else {
                    ctx.set_features(
                        |features: vk::PhysicalDeviceVulkan13Features|
                            features.dynamic_rendering(true)
                    );
                }
                Some(())
            },
            NAME,
        )
    }

    pub(crate) mod maintenance4 {
        //! Adds a collection minor features.
        //!
        //! This is **always** enabled.

        use crate::features::*;
        
        /// The name of this feature.
        static NAME: FeatureName<()> = FeatureName::new("maintenance4");

        pub(crate) fn new() -> DeviceFeature {
            use tuhka::khr;
            feature(
                move |ctx| -> Result<(), Error> {
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        ctx.add_required_extension(khr::maintenance4::NAME)?;
                    }
                    let mut features = vk::PhysicalDeviceMaintenance4Features
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.maintenance4 == 0 {
                        Err(err!("required feature `vk_13::maintenance4` not supported"))
                    } else {
                        Ok(())
                    }
                },
                move |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceMaintenance4Features|
                                features.maintenance4(true)
                        );
                    } else {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceVulkan13Features|
                                features.maintenance4(true)
                        );
                    }
                    let mut properties = vk::PhysicalDeviceMaintenance4Properties
                        ::default();
                    ctx.get_physical_device_properties(&mut properties);
                    Some(())
                },
                NAME,
            )
        }
    }

    pub(crate) fn copy_commands2() -> DeviceFeature {
        use tuhka::khr;
        pub static NAME: FeatureName<()> = FeatureName::new("copy_commands2");
        feature(
            |ctx| {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.add_required_extension(khr::copy_commands2::NAME)?;
                }
                Ok(())
            },
            |_| {
                Some(())
            },
            NAME,
        )
    }

    pub(crate) fn extended_dynamic_state() -> DeviceFeature {
        use tuhka::ext;
        pub static NAME: FeatureName<()> = FeatureName::new("extended_dynamic_state");
        feature(
            |ctx| {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.add_required_extension(ext::extended_dynamic_state::NAME)?;
                    let mut features = vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.extended_dynamic_state == 0 {
                        Err(err!("required feature `vk_13::exteded_dynamic_state` not supported"))
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            },
            |ctx| {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.set_features(
                        |features: vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT|
                            features.extended_dynamic_state(true)
                    );
                }
                ctx.add_supported_enums([
                    // base
                    PipelineStateKind::LineWidth,
                    PipelineStateKind::DepthBias,
                    PipelineStateKind::BlendConstants,
                    PipelineStateKind::DepthBounds,
                    PipelineStateKind::StencilCompareMask,
                    PipelineStateKind::StencilWriteMask,
                    PipelineStateKind::StencilReference,
                    // extended_dynamic_state
                    PipelineStateKind::CullMode,
                    PipelineStateKind::DepthBoundsTestEnable,
                    PipelineStateKind::DepthCompareOp,
                    PipelineStateKind::DepthTestEnable,
                    PipelineStateKind::DepthWriteEnable,
                    PipelineStateKind::FrontFace,
                    PipelineStateKind::PrimitiveTopology,
                    PipelineStateKind::StencilOp,
                    PipelineStateKind::StencilTestEnable,
                    PipelineStateKind::VertexInputBindingStride,
                ]);
                Some(())
            },
            NAME,
        )
    }
}

/// Core Vulkan 1.4 [`features`][Features].
pub mod vk_14 {

    use super::*;

    pub use generated::vk_14::*;

    pub(crate) fn line_rasterization_required() -> DeviceFeature {
        feature(
            |ctx| {
                if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                    ctx.add_required_extension(tuhka::khr::line_rasterization::NAME)?
                }
                Ok(())
            },
            |_| {
                None
            },
            FeatureName::<()>::new("__line_rasterization")
        )
    }
}

/// Core Vulkan [`features`][Features] up to Vulkan 1.4.
pub mod vk_core {

    pub use super::{
        vk_10::*,
        vk_11::*,
        vk_12::*,
        vk_13::*,
        vk_14::*,
    };
}

pub use generated::vk_khr;
pub use generated::vk_ext;

#[inline]
pub(crate) fn core_features() -> impl IntoIterator<Item = DeviceFeature> {
    [
        vk_12::timeline_semaphore(),
        vk_12::separate_depth_stencil_layouts(),
        vk_13::synchronization2(),
        vk_13::dynamic_rendering(),
        vk_13::copy_commands2(),
        vk_13::extended_dynamic_state(),
        vk_13::maintenance4::new(),
        vk_14::line_rasterization_required()
    ]
}

pub(super) mod prelude {

    pub use super::{
        vk_10, vk_11, vk_12, vk_13, vk_14,
        vk_core,
        vk_ext, vk_khr,
        feature::{FeatureName, DeviceFeature},
    };
}
