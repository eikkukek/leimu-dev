//! Contains types and functions for enabling [`Device`] features.
//!
//! # Notable user-facing types and functions
//! - [`DeviceFeature`]: An opaque structure containing all relevant data of a feature.
//! - [`FeatureName`]: A special string type, which can be used to retrieve the data associated with
//!   a feature through [`Device::get_feature`].

mod feature;
mod context;
mod device;

use tuhka::vk;

use crate::*;

use mem::{alloc_vec::Vec32, vec32};

use error::*;

pub(crate) use feature::*;
pub(crate) use context::*;
pub(crate) use device::*;

macro_rules! core_features {
    (
        $(#[doc = $doc:literal])*
        $field:ident $(as $name:ident)?,
        $(
            $(#[doc = $rest_doc:literal])*
            $rest_field:ident $(as $rest_name:ident)?
        ),* $(,)?
    ) => {
        core_features!(
            $(#[doc = $doc])*
            $field $(as $name)?
        );
        core_features!($(
            $(#[doc = $rest_doc])*
            $rest_field $(as $rest_name)?
        ),*);
    };
    () => {};
    (
        $(#[doc = $doc:literal])*
        $field:ident
    ) => {

        $(#[doc = $doc])*
        pub mod $field {
            use crate::features::*;
            use super::Inner;
            /// The name of this feature.
            pub const NAME: FeatureName<()> = FeatureName::new(stringify!($field));

            /// Creates the [`DeviceFeature`]
            pub fn new() -> DeviceFeature {
                super::feature(
                    Inner::new(move |query_support, features| {
                        if query_support {
                            if features.$field == 0 {
                                Err(err!("{}", concat!(
                                    "device feature `", stringify!($field),
                                    "` not supported"
                                )))?
                            } else {
                                Ok(None)
                            }
                        } else {
                            *features = features.$field(true);
                            Ok(Some(FeatureData::empty()))
                        }
                    }),
                    NAME.name,
                )
            }
        }
    };
    (
        $(#[doc = $doc:literal])*
        $field:ident as $name:ident
    ) => {

        $(#[doc = $doc])*
        pub mod $name {
            use crate::features::*;
            use super::Inner;
            /// The name of this feature.
            pub const NAME: FeatureName<()> = FeatureName::new(stringify!($name));

            /// Creates the [`DeviceFeature`]
            pub fn new() -> DeviceFeature {
                super::feature(
                    Inner::new(move |query_support, features| {
                        if query_support {
                            if features.$field == 0 {
                                Err(err!("{}", concat!(
                                    "device feature `", stringify!($field),
                                    "` not supported",
                                )))?
                            } else {
                                Ok(None)
                            }
                        } else {
                            *features = features.$field(true);
                            Ok(Some(FeatureData::empty()))
                        }
                    }),
                    NAME.name,
                )
            }
        }
    };
}

/// Core Vulkan 1.0 [`features`][Features].
pub mod vk_10 {

    use super::*;

    crate::mem::smallbox!(
        struct Inner: Fn[(bool, &mut vk::PhysicalDeviceFeatures) -> Result<Option<FeatureData>, Error>]
    );

    pub(super) struct Feature(Inner<8>);

    impl Feature {

        #[inline]
        pub fn query_support(
            &self,
            features: &vk::PhysicalDeviceFeatures,
        ) -> Result<(), Error> {
            (*self.0)(true, &mut features.clone())
                .map(|_| ())
        }

        pub fn enable(
            &self,
            features: &mut vk::PhysicalDeviceFeatures,
        ) -> FeatureData {
            (*self.0)(false, features)
                .unwrap()
                .unwrap()
        }
    }

    #[inline]
    fn feature(
        inner: Inner<8>,
        name: ConstName,
    ) -> DeviceFeature {
        DeviceFeature {
            inner: FeatureEnum::Vulkan10(Feature(inner)),
            name,
        }
    } 

    core_features!(
        /// Enables [robust buffer access][1] guarantees for shader buffer accesses.
        ///
        /// [1]: https://docs.vulkan.org/spec/latest/chapters/shaders.html#shaders-robust-buffer-access
        robust_buffer_access,
        /// Specifies the full 32-bit range of indices is supported for indexed draw calls when using
        /// a [`IndexType::U32`].
        full_draw_index_uint32,
        /// Specifies whether image views **can** be created with the [`Cube`] and
        /// [`CubeArray`] view types.
        ///
        /// [`Cube`]: ImageViewDim::Cube
        /// [`CubeArray`]: ImageViewDim::CubeArray
        image_cube_array,
        /// Specifies whether the [color blend] settings are controlled independently per-attachment.
        ///
        /// []
        independent_blend,
        /// Specifies whether geometry shaders are supported.
        geometry_shader,
        /// Specifies whether tessellation shaders are supported.
        tessellation_shader,
        /// Specifies whether samples shading and multisample interpolation are supported.
        sample_rate_shading,
        /// Specifies whether blend operations which take two sources are supported.
        dual_src_blend,
        /// Specifies whether [logic operations] are supported. 
        ///
        /// [logic operations]: pipeline_state::LogicOpEnable
        logic_op,
        /// Specifies whether multiple draw indirect is supported.
        multi_draw_indirect,
        /// Specifies whether indirect drawing calls support the `first_instance` parameter.
        draw_indirect_first_instance,
        /// Specifies whether [depth clamping] is supported.
        ///
        /// [depth clamping]: pipeline_state::DepthClampEnable
        depth_clamp,
        /// Specifies whether [depth bias clamping] is supported.
        ///
        /// [depth bias clamping]: pipeline_state::DepthBias::clamp
        depth_bias_clamp,
        /// Specifies whether [`point`][1] and [`line`][2] fill modes are supported.
        ///
        /// [1]: PolygonMode::Point
        /// [2]: PolygonMode::Line
        fill_mode_non_solid,
        /// Specifies whether depth bounds tests are supported.
        depth_bounds,
        /// Specifies whether lines with width other than 1.0 are supported.
        wide_lines,
        /// Specifies whether points with size greater than 1.0 are supported.
        large_points,
        /// Specifies whether replacing the alpha value of the fragment shader color output in the
        /// multisample coverage fragment operation.
        ///
        /// If this feature is not enabled, then [`AlphaToOneEnable`] **must** be `false`.
        ///
        /// [`AlphaToOneEnable`]: pipeline_state::AlphaToOneEnable
        alpha_to_one,
        /// Specifies whether more than one viewport is supported.
        multi_viewport,
        /// Specifies whether anisotropic filtering is supported.
        sampler_anisotropy,
        /// Specifies whether all of the ETC2 and EAC compressed texture [`formats`][1] are supported.
        ///
        /// [1]: Format
        texture_compression_etc2,
        /// Specifies whether all of the ASTC LDR compressed texture [`formats`][1] are supported.
        ///
        /// [1]: Format
        texture_compression_astc_ldr,
        /// Specifies whether all of the BC compressed texture [`formats`][1] are supported.
        ///
        /// [1]: Format
        texture_compression_bc,
        /// Specifies whether occlusion queries returning actual sample counts are supported.
        occlusion_query_precise,
        /// Specifies whether the pipeline statistics queries are supported.
        pipeline_statistics_query,
        /// Specifies whether storage buffers and images support stores and atomic operations in the
        /// vertex, tessellation, and geometry shader stages.
        vertex_pipeline_stores_and_atomics,
        /// Specifies whether storage buffers and images support stores and atomic operations in the
        /// fragment shader stage.
        fragment_stores_and_atomics,
        /// Specifies whether the PointSize built-in decoration is available in the tessellation
        /// control, tessellation evaluation, and geometry shader stages.
        shader_tessellation_and_geometry_point_size,
        /// Specifies whether the extended set of image gather instructions are available in shader
        /// code.
        shader_image_gather_extended,
        /// Specifies whether all the “storage image extended formats” are supported.
        ///
        /// # Storage image extended formats
        /// - [`Format::R16g16_Sfloat`]
        /// - [`Format::B10g11r11_Ufloat_Pack32`]
        /// - [`Format::R16_Sfloat`]
        /// - [`Format::R16g16b16a16_Unorm`]
        /// - [`Format::A2b10g10r10_Unorm_Pack32`]
        /// - [`Format::R16g16_Unorm`]
        /// - [`Format::R8g8_Unorm`]
        /// - [`Format::R16_Unorm`]
        /// - [`Format::R8_Unorm`]
        /// - [`Format::R16g16b16a16_Snorm`]
        /// - [`Format::R16g16_Snorm`]
        /// - [`Format::R8g8_Snorm`]
        /// - [`Format::R16_Snorm`]
        /// - [`Format::R8_Snorm`]
        /// - [`Format::R16g16_Sint`]
        /// - [`Format::R8g8_Sint`]
        /// - [`Format::R16_Sint`]
        /// - [`Format::R8_Sint`]
        /// - [`Format::A2b10g10r10_Uint_Pack32`]
        /// - [`Format::R16g16_Uint`]
        /// - [`Format::R8g8_Uint`]
        /// - [`Format::R16_Uint`]
        /// - [`Format::R8_Uint`]
        shader_storage_image_extended_formats,
        /// Specifies whether multisampled storage images are supported.
        shader_storage_image_multisample,
        /// Specifies whether storage images and storage texel buffers require a format qualifier to be
        /// specified when reading.
        shader_storage_image_read_without_format,
        /// Specifies whether storage images and storage texel buffers require a format qualifier to be
        /// specified when writing.
        shader_storage_image_write_without_format,
        /// Specifies whether arrays of uniform buffers **can** be indexed by integer expressions that are
        /// dynamically uniform within either the subgroup or the invocation group in shader code..
        shader_uniform_buffer_array_dynamic_indexing,
        /// Specifies whether arrays of samplers or sampled images **can** be indexed by integer
        /// expressions that are dynamically uniform within either the subgroup or the invocation group
        /// in shader code.
        shader_sampled_image_array_dynamic_indexing,
        /// Specifies whether arrays of storage buffers **can** be indexed by integer expressions that are
        /// dynamically uniform within either the subgroup or the invocation group in shader code.
        shader_storage_buffer_array_dynamic_indexing,
        /// Specifies whether arrays of storage images **can** be indexed by integer expressions that are
        /// dynamically uniform within either the subgroup or the invocation group in shader code.
        shader_storage_image_array_dynamic_indexing,
        /// Specifies whether clip distances are supported in shader code.
        shader_clip_distance,
        /// Specifies whether cull distances are supported in shader code.
        shader_cull_distance,
        /// Specifies whether 64-bit floats are supported in shader code.
        shader_float64,
        /// Specifies whether 64-bit integers (signed and unsigned) are supported in shader code.
        shader_int64,
        /// Specifies whether 16-bit integers (signed and unsigned) are supported in shader code.
        shader_int16,
        /// Specifies whether image operations that return resource residency information are supported
        /// in shader code.
        shader_resource_residency,
        /// Specifies whether image operations specifying the minimum resource LOD are supported in
        /// shader code.
        shader_resource_min_lod,
        /// Specifies whether resource memory **can** be managed at opaque sparse block level instead of at
        /// the object level.
        sparse_binding,
        /// Specifies whether the device **can** access partially resident buffers.
        sparse_residency_buffer,
        /// Specifies whether the device **can** access partially resident 2D images with 1 sample per
        /// pixel.
        sparse_residency_image2_d as sparse_residency_image_2d,
        /// Specifies whether the device **can** access partially resident 3D images.
        sparse_residency_image3_d as sparse_residency_image_3d,
        /// Specifies whether the physical device **can** access partially resident 2D images with 2
        /// samples per pixel.
        sparse_residency2_samples as sparse_residency_2_samples,
        /// Specifies whether the physical device **can** access partially resident 2D images with 4
        /// samples per pixel.
        sparse_residency4_samples as sparse_residency_4_samples,
        /// Specifies whether the physical device **can** access partially resident 2D images with 8
        /// samples per pixel.
        sparse_residency8_samples as sparse_residency_8_samples,
        /// Specifies whether the physical device **can** access partially resident 2D images with 16
        /// samples per pixel.
        sparse_residency16_samples as sparse_residency_16_samples,
        /// Specifies whether the physical device **can** correctly access data aliased into multiple
        /// locations.
        sparse_residency_aliased,
        /// Specifies whether pipelines in a subpass with no attachments **must** have the same
        /// [`samples`][SampleCountBit].
        variable_multisample_rate,
        /// Specifies whether a secondary command buffer **may** be executed while a query is active.
        inherited_queries,
    ); 
}

/// Core Vulkan 1.1 [`features`][Features].
pub mod vk_11 {

    use super::*;

    crate::mem::smallbox!(
        struct Inner: Fn[(bool, &mut vk::PhysicalDeviceVulkan11Features) -> Result<Option<FeatureData>, Error>]
    );

    pub(super) struct Feature(Inner<8>);

    impl Feature {

        #[inline]
        pub fn query_support(
            &self,
            features: &vk::PhysicalDeviceVulkan11Features,
        ) -> Result<(), Error> {
            (*self.0)(true, &mut features.clone())
                .map(|_| ())
        }

        pub fn enable(
            &self,
            features: &mut vk::PhysicalDeviceVulkan11Features,
        ) -> FeatureData {
            (*self.0)(false, features)
                .unwrap()
                .unwrap()
        }
    }

    #[inline]
    fn feature(
        inner: Inner<8>,
        name: ConstName,
    ) -> DeviceFeature {
        DeviceFeature {
            inner: FeatureEnum::Vulkan11(Feature(inner)),
            name,
        }
    } 

    core_features!(
        /// Specifies whether SPIR-V objects in the `StorageBuffer`, `ShaderRecordBufferKHR`, or
        /// `PhysicalStorageBuffer` storage class with the `Block` decoration **can** have 16-bit
        /// integer and floating-point members.
        storage_buffer16_bit_access as storage_buffer_16_bit_access,
        /// Specifies whether SPIR-V objects in the `Uniform` storage class with the `Block` decoration
        /// **can** have 16-bit integer and floating-point members.
        uniform_and_storage_buffer16_bit_access as uniform_and_storage_buffer_16_bit_access,
        /// Spcifies whether SPIR-V objects in the `PushConstant` storage class **can** have 16-bit
        /// integer and floating-point members.
        storage_push_constant16 as storage_push_constant_16,
        /// Specifies whether SPIR-V objects in the `Input` and `Output` storage classes **can** have
        /// 16-bit integer and floating-point members.
        storage_input_output16 as storage_input_output_16,
        /// Specifies whether multiview rendering is supported. If this feature is not enabled, the
        /// `view_mask` **must** be zero.
        multiview,
        /// Specifies whether multiview rendering is supported with geometry shaders.
        multiview_geometry_shader,
        /// Specifies whether multiview rendering is supported with tessellation shaders.
        multiview_tessellation_shader,
        /// Specifies whether SPIR-V `VariablePointersStorageBuffer` capability is supported.
        variable_pointers_storage_buffer,
        /// Specifies whether SPIR-V `VariablePointers` capability is supported.
        variable_pointers,
        /// Specifies whether protected memory is supported.
        protected_memory,
        /// Specifies whether SPIR-V `DrawParameters` capability is supported.
        shader_draw_parameters,
    );

    /* not supported for now
    /// Specifies whether sampler YCbCr conversion is supported.
    pub mod sampler_ycbcr_conversion {

        use crate::features::*;

        /// The name of this feature.
        pub const NAME: FeatureName<()> = FeatureName::new(stringify!($field));

        /// Creates the [`DeviceFeature`]
        pub fn new() -> DeviceFeature {
            /*super::feature(
                Inner::new(move |query_support, features| {
                    if query_support {
                        if features.sampler_ycbcr_conversion == 0 {

                        } else {
                            Ok(None)
                        }
                    } else {
                        *features = features.sampler_ycbcr_conversion(enable);
                        Ok(Some(FeatureData::new(enable)))
                    }
                }),
                NAME.name,
            )*/
            feature_ext(
                |ctx| {
                    if ctx.vulkan_11_features().sampler_ycbcr_conversion == 0 {
                        Err(err!("device feature `sampler_ycbcr_conversion` not supported"))
                    } else {
                        Ok(())
                    }
                }, 
                move |ctx| {
                    ctx.set_features(|features: vk::PhysicalDeviceVulkan11Features|
                        features.sampler_ycbcr_conversion(true)
                    );
                    ctx.add_supported_enums([
                        Format::G8b8g8r8_422_Unorm,
                        Format::B8g8r8g8_422_Unorm,
                        Format::G8_B8_R8_3plane_420_Unorm,
                        Format::G8_B8r8_2plane_420_Unorm,
                        Format::G8_B8_R8_3plane_422_Unorm,
                        Format::G8_B8r8_2plane_422_Unorm,
                        Format::G8_B8_R8_3plane_444_Unorm,
                        Format::R10x6_Unorm_Pack16,
                        Format::R10x6g10x6_Unorm_2pack16,
                        Format::R10x6g10x6b10x6a10x6_Unorm_4pack16,
                        Format::G10x6b10x6g10x6r10x6_422_Unorm_4pack16,
                        Format::B10x6g10x6r10x6g10x6_422_Unorm_4pack16,
                        Format::G10x6_B10x6_R10x6_3plane_420_Unorm_3pack16,
                        Format::G10x6_B10x6r10x6_2plane_420_Unorm_3pack16,
                        Format::G10x6_B10x6_R10x6_3plane_422_Unorm_3pack16,
                        Format::G10x6_B10x6r10x6_2plane_422_Unorm_3pack16,
                        Format::G10x6_B10x6_R10x6_3plane_444_Unorm_3pack16,
                        Format::R12x4_Unorm_Pack16,
                        Format::R12x4g12x4_Unorm_2pack16,
                        Format::R12x4g12x4b12x4a12x4_Unorm_4pack16,
                        Format::G12x4b12x4g12x4r12x4_422_Unorm_4pack16,
                        Format::B12x4g12x4r12x4g12x4_422_Unorm_4pack16,
                        Format::G12x4_B12x4_R12x4_3plane_420_Unorm_3pack16,
                        Format::G12x4_B12x4r12x4_2plane_420_Unorm_3pack16,
                        Format::G12x4_B12x4_R12x4_3plane_422_Unorm_3pack16,
                        Format::G12x4_B12x4r12x4_2plane_422_Unorm_3pack16,
                        Format::G12x4_B12x4_R12x4_3plane_444_Unorm_3pack16,
                        Format::G16b16g16r16_422_Unorm,
                        Format::B16g16r16g16_422_Unorm,
                        Format::G16_B16_R16_3plane_420_Unorm,
                        Format::G16_B16r16_2plane_420_Unorm,
                        Format::G16_B16_R16_3plane_422_Unorm,
                        Format::G16_B16r16_2plane_422_Unorm,
                        Format::G16_B16_R16_3plane_444_Unorm,
                    ]);
                    FeatureData::empty()
                },
                NAME,
            ) 
        }
    }*/
}

/// Core Vulkan 1.2 [`features`][Features].
pub mod vk_12 {

    use super::*;
    
    crate::mem::smallbox!(
        struct Inner: Fn[(bool, &mut vk::PhysicalDeviceVulkan12Features) -> Result<Option<FeatureData>, Error>]
    );

    pub(super) struct Feature(Inner<8>);

    impl Feature {

        #[inline]
        pub fn query_support(
            &self,
            features: &vk::PhysicalDeviceVulkan12Features,
        ) -> Result<(), Error> {
            (*self.0)(true, &mut features.clone())
                .map(|_| ())
        }

        pub fn enable(
            &self,
            features: &mut vk::PhysicalDeviceVulkan12Features,
        ) -> FeatureData {
            (*self.0)(false, features)
                .unwrap()
                .unwrap()
        }
    }

    #[inline]
    fn feature(
        inner: Inner<8>,
        name: ConstName,
    ) -> DeviceFeature {
        DeviceFeature {
            inner: FeatureEnum::Vulkan12(Feature(inner)),
            name,
        }
    }

    core_features!(
        /// Allows the use of the [`SamplerWrappingMode::MirrorClampToEdge`].
        sampler_mirror_clamp_to_edge,
        /// Allows the use of `vkCmdDrawIndirectCount` and `vkCmdDrawIndexedIndirectCount`.
        draw_indirect_count,
        /// Specifies whether SPIR-V objects in the `StorageBuffer` `ShaderRecordBufferKHR`, or
        /// `PhysicalStorageBuffer` storage class with the `Block` decoration **can** have 8-bit
        /// integer members.
        storage_buffer8_bit_access as storage_buffer_8_bit_access,
        /// Specifies whether SPIR-V objects in the Uniform storage class with the `Block`
        /// decoration **can** have 8-bit integer members.
        uniform_and_storage_buffer8_bit_access as uniform_and_storage_buffer_8_bit_access,
        /// Specifies whether objects in the `PushConstant` storage class **can** have 8-bit integer
        /// members.
        storage_push_constant8 as storage_push_constant_8,
        /// Specifies whether shaders **can** perform 64-bit unsigned and signed integer atomic
        /// operations on buffers.
        shader_buffer_int64_atomics,
        /// Specifies whether shaders **can** perform 64-bit unsigned and signed integer atomic
        /// operations on shared and payload memory.
        shader_shared_int64_atomics,
        /// Specifies whether 16-bit floats are supported in shader code.
        shader_float16,
        /// Specifies whether 8-bit unsigned and signed integers are supported in shader code.
        shader_int8,
        /// Specifies whether the the minimum set of descriptor indexing features are supported.
        ///
        /// The minimum set includes:
        /// - [`shader_sampled_image_array_dynamic_indexing`][1]
        /// - [`shader_storage_buffer_array_dynamic_indexing`][2]
        /// - [`shader_uniform_texel_buffer_array_dynamic_indexing`]
        /// - [`shader_storage_texel_buffer_array_dynamic_indexing`]
        /// - [`shader_sampled_image_array_non_uniform_indexing`]
        /// - [`shader_storage_buffer_array_non_uniform_indexing`]
        /// - [`shader_uniform_texel_buffer_array_non_uniform_indexing`]
        /// - [`descriptor_binding_sampled_image_update_after_bind`]
        /// - [`descriptor_binding_storage_image_update_after_bind`]
        /// - [`descriptor_binding_storage_buffer_update_after_bind`]
        /// - [`descriptor_binding_uniform_texel_buffer_update_after_bind`]
        /// - [`descriptor_binding_storage_texel_buffer_update_after_bind`]
        /// - [`descriptor_binding_update_unused_while_pending`]
        /// - [`descriptor_binding_partially_bound`]
        /// - [`runtime_descriptor_array`]
        ///
        /// [1]: vk_10::shader_sampled_image_array_dynamic_indexing
        /// [2]: vk_10::shader_storage_buffer_array_dynamic_indexing
        descriptor_indexing,
        /// Specifies whether arrays of input attachments **can** be indexed by integer expressions
        /// that are dynamically uniform within either the subgroup or the invocation group in
        /// shader code.
        shader_input_attachment_array_dynamic_indexing,
        /// Specifies whether arrays of uniform texel buffers **can** be indexed by integer expressions
        /// that are dynamically uniform within either the subgroup or the invocation group in
        /// shader code.
        shader_uniform_texel_buffer_array_dynamic_indexing,
        /// Specifies whether arrays of storage texel buffers **can** be indexed by integer expressions
        /// that are dynamically uniform within either the subgroup or the invocation group in
        /// shader code.
        shader_storage_texel_buffer_array_dynamic_indexing,
        /// Specifies whether arrays of uniform buffers **can** be indexed by
        /// non-uniform integer expressions in shader code.
        shader_uniform_buffer_array_non_uniform_indexing,
        /// Specifies whether arrays of samplers or sampled images **can** be indexed by
        /// non-uniform integer expressions in shader code.
        shader_sampled_image_array_non_uniform_indexing,
        /// Specifies whether arrays of storage buffers **can** be indexed by non-uniform
        /// integer expressions in shader code.
        shader_storage_buffer_array_non_uniform_indexing,
        /// Specifies whether arrays of storage images **can** be indexed by non-uniform integer
        /// expressions in shaader code.
        shader_storage_image_array_non_uniform_indexing,
        /// Specifies whether arrays of input attachments **can** be indexed by non-uniform integer
        /// expressions in shader code.
        shader_input_attachment_array_non_uniform_indexing,
        /// Specifies whether arrays of uniform texel buffers can be indexed by non-uniform integer
        /// expressions in shader code.
        shader_uniform_texel_buffer_array_non_uniform_indexing,
        /// Specifies whether arrays of storage texel buffers can be indexed by non-uniform integer
        /// expressions in shader code.
        shader_storage_texel_buffer_array_non_uniform_indexing,
        /// Specifies whether the uniform buffer descriptors **can** be updated after a set is bound.
        descriptor_binding_uniform_buffer_update_after_bind,
        /// Specifies whether the sampled image descriptors **can** be updated after a set is bound.
        descriptor_binding_sampled_image_update_after_bind,
        /// Specifies whether the storage image descriptors **can** be updated after a set is bound.
        descriptor_binding_storage_image_update_after_bind,
        /// Specifies whether the storage buffer descriptors **can** be updated after a set is bound.
        descriptor_binding_storage_buffer_update_after_bind,
        /// Specifies whether the uniform texel buffer descriptors **can** be updated after a set is bound.
        descriptor_binding_uniform_texel_buffer_update_after_bind,
        /// Specifies whether the storage texel buffer descriptors **can** be updated after a set is bound.
        descriptor_binding_storage_texel_buffer_update_after_bind,
        /// Specifies whether descriptors can be updated while the set is in use.
        descriptor_binding_update_unused_while_pending,
        /// Specifies whether statically using a descriptor set binding in which some descriptors
        /// are not valid is supported.
        descriptor_binding_partially_bound,
        /// Specifies whether descriptor sets with a variable-sized last binding is supported.
        descriptor_binding_variable_descriptor_count,
        /// Specifies whether SPIR-V `RunTimeDescriptorArray` capability is supported.
        runtime_descriptor_array,
        /// Specifies whether a minimum set of required formats supporting min/max filtering is
        /// supported.
        ///
        /// The minimum set includes:
        /// - [`Format::R8_Unorm`]
        /// - [`Format::R8_Snorm`]
        /// - [`Format::R16_Unorm`]
        /// - [`Format::R16_Snorm`]
        /// - [`Format::R16_Sfloat`]
        /// - [`Format::R32_Sfloat`]
        /// - [`Format::D16_Unorm`]
        /// - [`Format::X8_D24_Unorm_Pack32`]
        /// - [`Format::D32_Sfloat`]
        /// - [`Format::D16_Unorm_S8_Uint`]
        /// - [`Format::D24_Unorm_S8_Uint`]
        /// - [`Format::D32_Sfloat_S8_Uint`]
        sampler_filter_minmax,
        /// Specifies whether layout of resource blocks in shaders using scalar alginment is
        /// supported.
        scalar_block_layout,
        /// Specifies whether uniform buffers support the same layouts as storage and other kinds of
        /// buffers.
        uniform_buffer_standard_layout,
        /// Specifies whether subgroup operations **can** use 8-bit integer, 16-bit integer, 64-bit
        /// integer, 16-bit floating-point, and vectors of these types in group operations with
        /// subgroup scope, if the implementation supports the types.
        shader_subgroup_extended_types,
        /// Specifies whether resetting queries from the host with `vkResetQueryPool` is supported.
        host_query_reset,
        /// Specifies a [`ShaderDeviceAddress`] **can** be queried with
        /// [`Buffer::shader_device_address`].
        buffer_device_address,
        /// Specifies whether shader modules **can** declare the `VulkanMemoryModel` capability.
        vulkan_memory_model,
        /// Specifies whether `VulkanMemoryModel` can use `Device` scope synchronization.
        vulkan_memory_model_device_scope,
        /// Specifies whether the Vulkan Memory Model can use availability and visibility chains with
        /// more than one element.
        vulkan_memory_model_availability_visibility_chains,
        /// Specifies whether the `ShaderViewportIndex` SPIR-V capability is supported, enabling
        /// variables decorated with the `ViewportIndex` built-in to be exported from mesh, vertex
        /// or tessellation evaluation shaders.
        shader_output_viewport_index,
        /// Specifies whether the `ShaderLayer` SPIR-V capability is supported, enabling variables
        /// decorated with the `Layer` built-in to be exported from mesh, vertex or tessellation
        /// evaluation shaders.
        shader_output_layer,
        /// Specifies whether the "Id" operand of `OpGroupNonUniformBroadcast` **can** be
        /// dynamically uniform within a subgroup, and the "Index" operand of
        /// `OpGroupNonUniformQuadBroadcast` **can** be dynamically uniform within the derivative
        /// group.
        subgroup_broadcast_dynamic_id,
    );

    #[inline]
    pub(crate) fn timeline_semaphore() -> DeviceFeature {
        feature(
            Inner::new(|query_support, features| if query_support {
                if features.timeline_semaphore == 0 {
                    Err(err!(
                        "missing required feature timeline_semaphore"
                    ))
                } else {
                    Ok(None)
                }
            } else {
                *features = features.timeline_semaphore(true);
                Ok(Some(FeatureData::empty()))
            }),
            ConstName::new("timeline_semaphore"),
        )
    }

    #[inline]
    pub(crate) fn separate_depth_stencil_layouts() -> DeviceFeature {
        feature(
            Inner::new(|query_support, features| if query_support {
                if features.separate_depth_stencil_layouts == 0 {
                    Err(err!(
                        "missing required feature separate_depth_stencil_layouts"
                    ))
                } else {
                    Ok(None)
                }
            } else {
                *features = features.separate_depth_stencil_layouts(true);
                Ok(Some(FeatureData::empty()))
            }),
            ConstName::new("separate_depth_stencil_layouts"),
        )
    }
}

/// Core Vulkan 1.3 [`features`][Features].
pub mod vk_13 {

    use crate::PipelineStateKind;

    use super::*;

    pub mod robust_image_access {
        //! Specifies support for Robust Image Access guarantees for shader image accesses.
        //!
        //! See [`new`] for enabling this feature.

        use crate::features::*;

        /// The name of this feature.
        ///
        /// The data associated is a boolean value specifying whether this feature is enabled for
        /// all pipelines.
        pub const NAME: FeatureName<bool> = FeatureName::new("robust_image_access");

        /// Creates the [`DeviceFeature`]..
        ///
        /// # Parameters
        /// `enable`: whether to enable this feature for all pipelines.
        pub fn new(enable: bool) -> DeviceFeature {
            use tuhka::ext;
            feature_ext(
                move |ctx| -> Result<(), Error> {
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        ctx.add_required_extension(ext::image_robustness::NAME)?;
                    }
                    let mut features = vk::PhysicalDeviceImageRobustnessFeatures
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.robust_image_access == 0 {
                        Err(err!("`robust_image_access` not supported"))
                    } else {
                        Ok(())
                    }
                },
                move |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceImageRobustnessFeatures|
                                features.robust_image_access(enable)
                        );
                    } else {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceVulkan13Features|
                                features.robust_image_access(enable)
                        );
                    }
                    FeatureData::new(enable)
                },
                NAME,
            )
        }
    }

    pub mod inline_uniform_block {
        //! Specifies that inline uniform block descriptors **may** be used.
        //!
        //! See [`Builder`] for enabling `inline_uniform_block` features.

        use crate::features::*;

        /// [`inline_uniform_block`][self] properties.
        #[derive(Clone, Copy)]
        pub struct Properties {
            /// The maximum size in bytes of a single inline uniform block binding.
            pub max_inline_uniform_block_size: u32,
            /// The maximum number of inline uniform blocks that **can** be accessible to a single
            /// shader stage.
            pub max_per_stage_descriptor_inline_uniform_blocks: u32,
            /// The maximum number of inline uniform blocks that **can** be accessible to a single
            /// shader stage with or without the `UPDATE_AFTER_BIND_POOL` bit set.
            pub max_per_stage_descriptor_update_after_bind_inline_uniform_blocks: u32,
            /// The maximum number of inline uniform block bindings that **can** be included in
            /// descriptor bindings across all pipeline shader stages and
            /// descriptor set numbers.
            pub max_descriptor_set_inline_uniform_blocks: u32,
            /// The maximum number of inline uniform block bindings that **can** be included in
            /// descriptor bindings across all pipeline shader stages and
            /// descriptor set numbers with or without the `UPDATE_AFTER_BIND_POOL` bit set.
            pub max_descriptor_set_update_after_bind_inline_uniform_blocks: u32,
            /// The maximum total size in bytes of all inline uniform block bindings across all
            /// pipeline shader stages and descriptor set numbers that **can** be included in a
            /// set of shaders.
            pub max_inline_uniform_total_size: u32,
        }

        /// Enabled [`inline_uniform_block`][self] features.
        #[derive(Clone, Copy)]
        pub struct Features {
            /// Specifies that inline uniform block descriptors **can** be updated after a set is
            /// bound.
            pub update_after_bind: bool,
        }

        /// Data associated with this feature.
        #[derive(Clone, Copy)]
        pub struct Data(Features, Properties);

        impl Data {

            /// Enabled [`inline_uniform_block`][self] features.
            pub fn enabled_features(&self) -> &Features {
                &self.0
            }

            /// [`inline_uniform_block`][self] properties.
            pub fn properties(&self) -> &Properties {
                &self.1
            }
        }

        /// The name of this feature.
        pub const NAME: FeatureName<Data> = FeatureName::new("inline_uniform_block"); 

        /// A builder for enabling [`inline_uniform_block`][self] features.
        #[derive(Default, Clone, Copy)]
        pub struct Builder {
            update_after_bind: bool,
        }

        /// Creates [`Builder`].
        #[inline]
        pub fn builder() -> Builder {
            Builder::default()
        }

        impl Builder {
        
            /// Creates the builder.
            #[inline]
            pub fn new() -> Self {
                builder()
            }

            /// Specifies whether inline uniform block descriptors **can** be updated after a set is
            /// bound.
            #[inline] 
            pub fn update_after_bind(mut self, update_after_bind: bool) -> Self {
                self.update_after_bind = update_after_bind;
                self
            }

            /// Creates the [`DeviceFeature`].
            pub fn build(self) -> DeviceFeature {
                use tuhka::ext;
                feature_ext(
                    move |ctx| -> Result<(), Error> {
                        if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                            ctx.add_required_extension(ext::inline_uniform_block::NAME)?;
                        }
                        let mut features = vk::PhysicalDeviceInlineUniformBlockFeatures
                            ::default();
                        ctx.get_physical_device_features(&mut features);
                        if features.inline_uniform_block == 0 {
                            Err(err!("inline_uniform_block not supported"))
                        } else if self.update_after_bind && 
                            features.descriptor_binding_inline_uniform_block_update_after_bind == 0
                        {
                            Err(err!("inline_uniform_block `update_after_bind` not supported"))
                        } else {
                            Ok(())
                        }
                    },
                    move |ctx| {
                        if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                            ctx.set_features(
                                |features: vk::PhysicalDeviceInlineUniformBlockFeatures|
                                    features
                                    .inline_uniform_block(true)
                                    .descriptor_binding_inline_uniform_block_update_after_bind(self.update_after_bind)
                            );
                            let mut properties = vk::PhysicalDeviceInlineUniformBlockProperties
                                ::default();
                            ctx.get_physical_device_properties(&mut properties);
                            FeatureData::new(Data(Features { update_after_bind: self.update_after_bind }, Properties {
                                max_inline_uniform_block_size:
                                    properties.max_inline_uniform_block_size,
                                max_per_stage_descriptor_inline_uniform_blocks:
                                    properties.max_per_stage_descriptor_inline_uniform_blocks,
                                max_per_stage_descriptor_update_after_bind_inline_uniform_blocks:
                                    properties.max_per_stage_descriptor_update_after_bind_inline_uniform_blocks,
                                max_descriptor_set_inline_uniform_blocks:
                                    properties.max_descriptor_set_inline_uniform_blocks,
                                max_descriptor_set_update_after_bind_inline_uniform_blocks:
                                    properties.max_descriptor_set_update_after_bind_inline_uniform_blocks,
                                max_inline_uniform_total_size: u32::MAX,
                            }))
                        } else {
                            ctx.set_features(
                                |features: vk::PhysicalDeviceVulkan13Features|
                                    features
                                    .inline_uniform_block(true)
                                    .descriptor_binding_inline_uniform_block_update_after_bind(self.update_after_bind)
                            );
                            let mut properties = vk::PhysicalDeviceVulkan13Properties
                                ::default();
                            ctx.get_physical_device_properties(&mut properties);
                            FeatureData::new(Data(Features { update_after_bind: self.update_after_bind }, Properties {
                                max_inline_uniform_block_size:
                                    properties.max_inline_uniform_block_size,
                                max_per_stage_descriptor_inline_uniform_blocks:
                                    properties.max_per_stage_descriptor_inline_uniform_blocks,
                                max_per_stage_descriptor_update_after_bind_inline_uniform_blocks:
                                    properties.max_per_stage_descriptor_update_after_bind_inline_uniform_blocks,
                                max_descriptor_set_inline_uniform_blocks:
                                    properties.max_descriptor_set_inline_uniform_blocks,
                                max_descriptor_set_update_after_bind_inline_uniform_blocks:
                                    properties.max_descriptor_set_update_after_bind_inline_uniform_blocks,
                                max_inline_uniform_total_size:
                                    properties.max_inline_uniform_total_size,
                            }))
                        }
                    },
                    NAME,
                )
            }
        }
    }

    pub mod shader_terminate_invocation {
        //! Specifies that the SPIR-V extension `SPV_KHR_terminate_invocation` is supported.
        //!
        //! Provides a new SPIR-V instruction `OpTerminateInvocation`, which **should** be used instead of
        //! `OpKill`.
        //!
        //! If this feature is supported, it is always enabled.

        use crate::features::*;

        /// The name of this feature.
        pub const NAME: FeatureName<bool> = FeatureName::new("shader_terminate_invocation");

        /// Creates the [`DeviceFeature`].
        pub(crate) fn new() -> DeviceFeature {
            use crate::sync::{Arc, atomic::{self, AtomicBool}};
            use tuhka::khr;
            let check = Arc::new(AtomicBool::new(false));
            let enable = check.clone();
            feature_ext(
                move |ctx| -> Result<(), Error> {
                    let mut supported = false;
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        supported = ctx.add_optional_extension(khr::shader_terminate_invocation::NAME);
                    }
                    let mut features = vk::PhysicalDeviceShaderTerminateInvocationFeatures
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.shader_terminate_invocation == 0 {
                        supported = false;
                    }
                    check.store(supported, std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                },
                move |ctx| {
                    if enable.load(atomic::Ordering::Relaxed) {
                        if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                            ctx.set_features(
                                |features: vk::PhysicalDeviceShaderTerminateInvocationFeatures|
                                    features.shader_terminate_invocation(true)
                            );
                        } else {
                            ctx.set_features(
                                |features: vk::PhysicalDeviceVulkan13Features|
                                    features.shader_terminate_invocation(true)
                            );
                        }
                        FeatureData::new(true)
                    } else {
                        FeatureData::new(false)
                    }
                },
                NAME,
            )
        }
    }

    pub(crate) fn synchronization2() -> DeviceFeature {
        use tuhka::khr;
        const NAME: FeatureName<()> = FeatureName::new("synchronization2");
        feature_ext(
            move |ctx| -> Result<(), Error> {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.add_required_extension(khr::synchronization2::NAME)?;
                }
                let mut features = vk::PhysicalDeviceSynchronization2Features
                    ::default();
                ctx.get_physical_device_features(&mut features);
                if features.synchronization2 == 0 {
                    Err(err!("required feature synchronization2 not supported"))
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
                FeatureData::empty()
            },
            NAME,
        )
    } 

    pub mod shader_zero_initialize_workgroup_memory {
        //! Specifies whether zero-initializing a variable in `Workgroup` storage class is supported
        //! (with `OpConstantNull`).
        //!
        //! See [`new`] for enabling this feature.
        //!
        //! If this feature is not enabled and any shader initializes a `Workgroup` storage class
        //! variable with `OpConstantNull`, that shader will be invalid and compilation will return
        //! an error.

        use crate::features::*;

        /// The name of this feature.
        pub const NAME: FeatureName<()> = FeatureName::new("shader_zero_initialize_workgroup_memory");

        /// Creates the [`DeviceFeature`].
        pub fn new() -> DeviceFeature {
            use tuhka::khr;
            feature_ext(
                move |ctx| -> Result<(), Error> {
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        ctx.add_required_extension(khr::zero_initialize_workgroup_memory::NAME)?;
                    }
                    let mut features = vk::PhysicalDeviceZeroInitializeWorkgroupMemoryFeatures
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.shader_zero_initialize_workgroup_memory == 0 {
                        Err(err!("shader_zero_initialize_workgroup_memory not supported"))
                    } else {
                        Ok(())
                    }
                },
                move |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceZeroInitializeWorkgroupMemoryFeatures|
                                features.shader_zero_initialize_workgroup_memory(true)
                        );
                    } else {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceVulkan13Features|
                                features.shader_zero_initialize_workgroup_memory(true)
                        );
                    }
                    FeatureData::empty()
                },
                NAME,
            )
    }
    } 

    pub(crate) fn dynamic_rendering() -> DeviceFeature {
        use tuhka::khr;
        const NAME: FeatureName<()> = FeatureName::new("dynamic_rendering");
        feature_ext(
            move |ctx| -> Result<(), Error> {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.add_required_extension(khr::dynamic_rendering::NAME)?;
                }
                let mut features = vk::PhysicalDeviceDynamicRenderingFeatures
                    ::default();
                ctx.get_physical_device_features(&mut features);
                if features.dynamic_rendering == 0 {
                    Err(err!("required feature dynamic_rendering not supported"))
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
                FeatureData::empty()
            },
            NAME,
        )
    }

    pub mod shader_integer_dot_product {
        //! Specifies whether the `DotProductInputAll`, `DotProductInput4x8Bit`,
        //! `DotProductInput4x8BitPacked` and `DotProduct` capabilities **can** be used in shaders.
        //!
        //! See [`new`] for enabling this feature.
        //!
        //! These capabilities allow the use of `OpSDot*` and `OpUDot*` instructions.
        //!
        //! If this feature is not enabled and any of these capabilities are included in a shader, that
        //! shader will be invalid and will return an error.

        use crate::features::*;

        /// The name of this feature.
        pub const NAME: FeatureName<()> = FeatureName::new("shader_integer_dot_product");

        /// Creates the [`DeviceFeature`].
        pub fn new() -> DeviceFeature {
            use tuhka::khr;
            feature_ext(
                move |ctx| -> Result<(), Error> {
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        ctx.add_required_extension(khr::shader_integer_dot_product::NAME)?;
                    }
                    let mut features = vk::PhysicalDeviceShaderIntegerDotProductFeatures
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.shader_integer_dot_product == 0 {
                        Err(err!("shader_integer_dot_product not supported"))
                    } else {
                        Ok(())
                    }
                },
                move |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceShaderIntegerDotProductFeatures|
                                features.shader_integer_dot_product(true)
                        );
                    } else {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceVulkan13Features|
                                features.shader_integer_dot_product(true)
                        );
                    }
                    FeatureData::empty()
                },
                NAME,
            )
        }
    }

    pub mod maintenance4 {
        //! Adds a collection minor features.
        //!
        //! This is **always** enabled.

        use crate::features::*;
        
        /// The name of this feature.
        const NAME: FeatureName<()> = FeatureName::new("maintenance4");

        pub(crate) fn new() -> DeviceFeature {
            use tuhka::khr;
            feature_ext(
                move |ctx| -> Result<(), Error> {
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        ctx.add_required_extension(khr::maintenance4::NAME)?;
                    }
                    let mut features = vk::PhysicalDeviceMaintenance4Features
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.maintenance4 == 0 {
                        Err(err!("required feature maintenance4 not supported"))
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
                    FeatureData::empty()
                },
                NAME,
            )
        }
    }

    pub(crate) fn copy_commands2() -> DeviceFeature {
        use tuhka::khr;
        pub const NAME: FeatureName<()> = FeatureName::new("copy_commands2");
        feature_ext(
            |ctx| {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.add_required_extension(khr::copy_commands2::NAME)?;
                }
                Ok(())
            },
            |_| {
                FeatureData::empty()
            },
            NAME,
        )
    }

    pub(crate) fn extended_dynamic_state() -> DeviceFeature {
        use tuhka::ext;
        pub const NAME: FeatureName<()> = FeatureName::new("extended_dynamic_state");
        feature_ext(
            |ctx| {
                if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                    ctx.add_required_extension(ext::extended_dynamic_state::NAME)?;
                    let mut features = vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.extended_dynamic_state == 0 {
                        Err(err!("required feature exteded_dynamic_state not supported"))
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
                FeatureData::empty()
            },
            NAME,
        )
    }

    pub mod extended_dynamic_state2 {
        //! Extends the available states that can be dynamic for a pipeline.
        //!
        //! See [`Builder`] for enabling empty `extended_dynamic_state2` features.
        //!
        //! If the [`extended_dynamic_state2`][1] feature is enabled, the following
        //! [`pipeline_state`] **can** be dynamic:
        //! - [`DepthBiasEnable`][pipeline_state::DepthBiasEnable]
        //! - [`PrimitiveRestartEnable`][pipeline_state::PrimitiveRestartEnable]
        //! - [`RasterizerDiscardEnable`][pipeline_state::RasterizerDiscardEnable]
        //!
        //! If the [`logic_op`][2] feature is enabled, the [`LogicOp`] [pipeline state]
        //! **can** be dynamic.
        //!
        //! If the [`patch_control_points`][3] feature is enabled, the [`PatchControlPoints`][4] [pipeline state]
        //! **can** be dynamic.
        //!
        //! [1]: self::Features::extended_dynamic_state2
        //! [2]: self::Features::logic_op
        //! [3]: self::Features::patch_control_points
        //! [4]: pipeline_state::PatchControlPoints
        //! [pipeline state]: pipeline_state

        use tuhka::ext::{self, extended_dynamic_state2::Device};

        use crate::sync::OnceLock;

        use crate::features::*;

        /// Enabled [`extended_dynamic_state2`][self] features.
        #[derive(Default, Clone, Copy)]
        pub struct Features {
            /// Allows the following [pipeline states][pipeline_state] to be dynamic:
            /// - [`DepthBiasEnable`][pipeline_state::DepthBiasEnable]
            /// - [`PrimitiveRestartEnable`][pipeline_state::PrimitiveRestartEnable]
            /// - [`RasterizerDiscardEnable`][pipeline_state::RasterizerDiscardEnable]
            pub extended_dynamic_state2: bool,
            /// Allows the [`LogicOp`] [pipeline state][pipeline_state] to be dynamic.
            pub logic_op: bool,
            /// Allows the [`PatchControlPoints`][1] [pipeline state][2] to be dynamic.
            ///
            /// [1]: pipeline_state::PatchControlPoints
            /// [2]: pipeline_state
            pub patch_control_points: bool,
        }

        /// Data associated with this feature.
        #[derive(Clone)]
        pub struct Data(Features, OnceLock<Device>);

        impl Data {

            /// Returns enabled [`extended_dynamic_state2`][self] features.
            ///
            /// It **may** be more efficient to query supported dynamic state directly with
            /// [`is_pipeline_state_dynamic_allowed`][1].
            ///
            /// [1]: crate::Device::is_pipeline_state_dynamic_allowed
            #[inline]
            pub fn enabled_features(&self) -> &Features {
                &self.0
            }

            /// [`extended_dynamic_state2`][self] [`device`][Device].
            pub(crate) fn functions(&self, device: &crate::Device) -> &Device {
                self.1.get_or_init(|| Device::new(device.functions()))
            }
        }

        /// The name of this feature.
        pub const NAME: FeatureName<Data> = FeatureName::new("extended_dynamic_state2");

        /// Creates [`Builder`].
        pub fn builder() -> Builder {
            Builder::default()
        }

        /// A builder for enabling [`extended_dynamic_state2`][self] features.
        #[derive(Default, Clone, Copy)]
        pub struct Builder {
            extended_dynamic_state2: bool,
            logic_op: bool,
            patch_control_points: bool,
        }

        impl Builder {

            /// Creates the builder.
            #[inline]
            pub fn new() -> Self {
                builder()
            }

            /// Allows the following [pipeline states][pipeline_state] to be dynamic:
            /// - [`DepthBiasEnable`][pipeline_state::DepthBiasEnable]
            /// - [`PrimitiveRestartEnable`][pipeline_state::PrimitiveRestartEnable]
            /// - [`RasterizerDiscardEnable`][pipeline_state::RasterizerDiscardEnable]
            pub fn extended_dynamic_state2(mut self, enable: bool) -> Self {
                self.extended_dynamic_state2 = enable;
                self
            }

            /// Allows the [`LogicOp`] [pipeline state][pipeline_state] to be dynamic.
            pub fn logic_op(mut self, enable: bool) -> Self {
                self.logic_op = enable;
                self
            }

            /// Allows the [`PatchControlPoints`][pipeline_state::PatchControlPoints] [pipeline state][pipeline_state] to be dynamic.
            pub fn patch_control_points(mut self, enable: bool) -> Self {
                self.patch_control_points = enable;
                self
            }
        
            /// Creates the [`DeviceFeature`].
            pub fn build(self) -> DeviceFeature {
                feature_ext(
                    move |ctx| {
                        if ctx.api_version() < VULKAN_API_VERSION_1_3 ||
                            self.logic_op ||
                            self.patch_control_points
                        {
                            ctx.add_required_extension(ext::extended_dynamic_state2::NAME)?;
                            let vk_features = vk::PhysicalDeviceExtendedDynamicState2FeaturesEXT
                                ::default();
                            if self.extended_dynamic_state2 && vk_features.extended_dynamic_state2 == 0 {
                                Err(err!(
                                    "`extended_dynamic_state2` feature not supported"
                                ))?
                            }
                            if self.logic_op &&
                                    vk_features.extended_dynamic_state2_logic_op == 0 {
                                Err(err!(
                                    "`extended_dynamic_state2_logic_op` feature not supported"
                                ))?
                            }
                            if self.patch_control_points &&
                                vk_features.extended_dynamic_state2_patch_control_points == 0
                            {
                                Err(err!(
                                    "`extended_dynamic_state2_patch_control_points` feature not supported"
                                ))?
                            }
                        }
                        Ok(())
                    },
                    move |ctx| {
                        if self.logic_op ||
                            self.patch_control_points
                        {
                            ctx.set_features(
                                |f: vk::PhysicalDeviceExtendedDynamicState2FeaturesEXT|
                                    f.extended_dynamic_state2(self.extended_dynamic_state2)
                                        .extended_dynamic_state2_logic_op(self.logic_op)
                                        .extended_dynamic_state2_patch_control_points(
                                            self.patch_control_points
                                        )
                            );
                        }
                        if self.extended_dynamic_state2 {
                            ctx.add_supported_enums([
                                PipelineStateKind::DepthBiasEnable,
                                PipelineStateKind::PrimitiveRestartEnable,
                                PipelineStateKind::RasterizerDiscardEnable,
                            ]);
                        }
                        if self.logic_op {
                            ctx.add_supported_enums([PipelineStateKind::LogicOp]);
                        }
                        if self.patch_control_points {
                            ctx.add_supported_enums([PipelineStateKind::PatchControlPoints]);
                        }
                        FeatureData::new(Data(Features {
                            extended_dynamic_state2: self.extended_dynamic_state2,
                            logic_op: self.logic_op,
                            patch_control_points: self.patch_control_points,
                        }, OnceLock::new()))
                    },
                    NAME,
                )
            }
        }
    }

    pub mod _4444_formats {
        //! Allows the use of the [`A4r4g4b4_Unorm_Pack16`][1] and [`A4b4g4r4_Unorm_Pack16`][2]
        //! formats.
        //!
        //! [1]: Format::A4r4g4b4_Unorm_Pack16
        //! [2]: Format::A4b4g4r4_Unorm_Pack16

        use tuhka::ext;

        use crate::features::*;
    
        /// The name of this feature.
        pub const NAME: FeatureName<()> = FeatureName::new("extended_dynamic_state");

        /// Creates the [`DeviceFeature`].
        pub fn new() -> DeviceFeature {
            feature_ext(
                |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_3 {
                        ctx.add_required_extension(ext::_4444_formats::NAME)
                    } else {
                        Ok(())
                    }
                },
                |ctx| {
                    ctx.add_supported_enums([
                        Format::A4r4g4b4_Unorm_Pack16,
                        Format::A4b4g4r4_Unorm_Pack16,
                    ]);
                    FeatureData::new(())
                },
                NAME,
            )
        }
    }
}

/// Core Vulkan 1.4 [`features`][Features].
pub mod vk_14 {   

    pub mod line_rasterization {
        //! Contains the [`rectangular_lines`][1], [`bresenham_lines`][2], [`smooth_lines`][3],
        //! [`stippled_rectangular_lines`][4], [`stippled_bresenham_lines`][5], and
        //! [`stippled_smooth_lines`][6] features.
        //!
        //! See [`Builder`] for enabling `line_rasterization` features.
        //!
        //! [1]: Features::rectangular_lines
        //! [2]: Features::bresenham_lines
        //! [3]: Features::smooth_lines
        //! [4]: Features::stippled_rectangular_lines
        //! [5]: Features::stippled_bresenham_lines
        //! [6]: Features::stippled_smooth_lines

        use crate::features::*;

        /// [`line_rasterization`][self] properties.
        #[derive(Clone, Copy)]
        pub struct Properties {
            /// The number of bits of subpixel precision in framebuffer coordinates when rasterizing
            /// line segments.
            pub line_sub_pixel_precision_bits: u32,
        }

        /// Enabled [`line_rasterization`][self] features.
        #[derive(Clone, Copy)]
        pub struct Features {
            /// Allows the use of rectangular line rasterization.
            pub rectangular_lines: bool,
            /// Allows the use of bresenham line rasterization.
            pub bresenham_lines: bool,
            /// Allows the use of smooth line rasterization.
            pub smooth_lines: bool,
            /// Allows rectangular line rasterization with stippled line rasterization
            /// enabled.
            pub stippled_rectangular_lines: bool,
            /// Allows bresenham line rasterization with stippled line rasterization
            /// enabled.
            pub stippled_bresenham_lines: bool,
            /// Allows smooth line rasterization with stippled line rasterization
            /// enabled.
            pub stippled_smooth_lines: bool,
        }

        /// Data associated with this feature.
        pub struct Data(Features, Properties);

        impl Data {

            /// Enabled [`line_rasterization`][self] features.
            pub fn enabled_features(&self) -> &Features {
                &self.0
            }

            /// [`line_rasterization`][self] properties.
            pub fn properties(&self) -> &Properties {
                &self.1
            }
        }

        /// A builder for enabling [`line_rasterization`][self] features.
        #[derive(Default, Clone, Copy)]
        pub struct Builder {
            rectangular_lines: bool,
            bresenham_lines: bool,
            smooth_lines: bool,
            stippled_rectangular_lines: bool,
            stippled_bresenham_lines: bool,
            stippled_smooth_lines: bool,
        }

        /// The name of this feature.
        pub const NAME: FeatureName<Data> = FeatureName::new("line_rasterization");

        /// Creates [`Builder`].
        #[inline]
        pub fn builder() -> Builder {
            default()
        }

        impl Builder {

            /// Creates the builder.
            #[inline]
            pub fn new() -> Self {
                builder()
            }

            /// Allows the use of rectangular line rasterization.
            #[inline]
            pub fn rectangular_lines(mut self, enable: bool) -> Self {
                self.rectangular_lines = enable;
                self
            }

            /// Allows the use of bresenham line rasterization.
            #[inline]
            pub fn bresenham_lines(mut self, enable: bool) -> Self {
                self.bresenham_lines = enable;
                self
            }

            /// Allows the use of smooth line rasterization.
            #[inline]
            pub fn smooth_lines(mut self, enable: bool) -> Self {
                self.smooth_lines = enable;
                self
            }

            /// Allows rectangular line rasterization with stippled line rasterization
            /// enabled.
            #[inline]
            pub fn stippled_rectangular_lines(mut self, enable: bool) -> Self {
                self.stippled_rectangular_lines = enable;
                self
            }

            /// Allows bresenham line rasterization with stippled line rasterization
            /// enabled.
            #[inline]
            pub fn stippled_bresenham_lines(mut self, enable: bool) -> Self {
                self.stippled_bresenham_lines = enable;
                self
            }

            /// Allows smooth line rasterization with stippled line rasterization
            /// enabled.
            #[inline]
            pub fn stippled_smooth_lines(mut self, enable: bool) -> Self {
                self.stippled_smooth_lines = enable;
                self
            }

            /// Creates the [`DeviceFeature`].
            pub fn build(self) -> DeviceFeature {
                use tuhka::khr;
                feature_ext(
                    move |ctx| {
                        if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                            ctx.add_required_extension(khr::line_rasterization::NAME)?;
                        }
                        let mut supported = vk::PhysicalDeviceLineRasterizationFeatures
                            ::default();
                        ctx.get_physical_device_features(&mut supported);
                        if self.rectangular_lines &&
                            supported.rectangular_lines == 0
                        {
                            Err(err!(
                                "rectangular_lines not supported"
                            ))?
                        }
                        if self.bresenham_lines &&
                            supported.bresenham_lines == 0
                        {
                            Err(err!(
                                "bresenham_lines not supported"
                            ))?
                        }
                        if self.smooth_lines &&
                            supported.smooth_lines == 0
                        {
                            Err(err!(
                                "smooth_lines not supported"
                            ))?
                        }
                        if self.stippled_rectangular_lines &&
                            supported.stippled_rectangular_lines == 0
                        {
                            Err(err!(
                                "stippled_rectangular_lines not supported"
                            ))?
                        }
                        if self.stippled_bresenham_lines &&
                            supported.stippled_bresenham_lines == 0
                        {
                            Err(err!(
                                "stippled_bresenham_lines not supported"
                            ))?
                        }
                        if self.stippled_smooth_lines &&
                            supported.stippled_smooth_lines == 0
                        {
                            Err(err!(
                                "stippled_smooth_lines not supported"
                            ))?
                        }
                        Ok(())
                    },
                    move |ctx| {
                        if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                            ctx.set_features(
                                |enable: vk::PhysicalDeviceVulkan14Features|
                                    enable.rectangular_lines(self.rectangular_lines)
                                        .bresenham_lines(self.bresenham_lines)
                                        .smooth_lines(self.smooth_lines)
                                        .stippled_rectangular_lines(self.stippled_rectangular_lines)
                                        .stippled_bresenham_lines(self.stippled_bresenham_lines)
                                        .stippled_smooth_lines(self.stippled_smooth_lines)
                            );
                        } else {
                            ctx.set_features(
                                |enable: vk::PhysicalDeviceLineRasterizationFeatures|
                                    enable.rectangular_lines(self.rectangular_lines)
                                        .bresenham_lines(self.bresenham_lines)
                                        .smooth_lines(self.smooth_lines)
                                        .stippled_rectangular_lines(self.stippled_rectangular_lines)
                                        .stippled_bresenham_lines(self.stippled_bresenham_lines)
                                        .stippled_smooth_lines(self.stippled_smooth_lines)
                            );
                        }
                        let mut properties = vk::PhysicalDeviceLineRasterizationProperties
                            ::default();
                        ctx.get_physical_device_properties(&mut properties);
                        FeatureData::new(Data(
                            Features {
                                rectangular_lines: self.rectangular_lines,
                                bresenham_lines: self.bresenham_lines,
                                smooth_lines: self.smooth_lines,
                                stippled_rectangular_lines: self.stippled_rectangular_lines,
                                stippled_bresenham_lines: self.stippled_bresenham_lines,
                                stippled_smooth_lines: self.stippled_smooth_lines
                            },
                            Properties {
                                line_sub_pixel_precision_bits: properties.line_sub_pixel_precision_bits
                            },
                        ))
                    },
                    NAME,
                )
            }
        }
    }

    pub mod vertex_attribute_instance_rate_divisor {
        //! Specifies whether vertex attribute fetching may be repeated in instanced rendering.
        //!
        //! See [`Builder`] for enabling `vertex_attribute_instance_rate_divisor` features.
        //!
        //! If this feature is enabled and `vertex_binding_divisor` is not 1, the attributes of an
        //! instance is repeated multiple times before fetching the next attributes. 
        //!
        //! If the specified value is zero, every instance has the attributes of the first instance.

        use crate::features::*;
 
        /// [`vertex_attribute_instance_rate_divisor`][self] properties
        #[derive(Clone, Copy)]
        pub struct Properties {
            /// The maximum value of the number of instances that **can** repeat the same vertex
            /// attribute data.
            pub max_vertex_attrib_divisor: u32,
            /// Specifies whether non-zero value for `first_instance` is supported when `divisor` is
            /// not 1.
            pub supports_non_zero_first_instance: bool,
        }

        /// Enabled [`vertex_attribute_instance_rate_divisor`][self] features.
        #[derive(Clone, Copy)]
        pub struct Features {
            /// Allows `divisor` to be zero.
            pub zero_divisor: bool,
        } 

        /// Data associated with this feature.
        #[derive(Clone, Copy)]
        pub struct Data(Features, Properties);

        impl Data {

            /// Enabled [`vertex_attribute_instance_rate_divisor`][self] features.
            pub fn enabled_features(&self) -> &Features {
                &self.0
            }

            /// [`vertex_attribute_instance_rate_divisor`][self] properties.
            pub fn properties(&self) -> &Properties {
                &self.1
            }
        }

        /// The name of the feature.
        pub const NAME: FeatureName<Data> = FeatureName::new("vertex_attribute_instance_rate_divisor");

        /// A builder for enabling [`vertex_attribute_instance_rate_divisor`][self] features.
        #[derive(Default, Clone, Copy)]
        pub struct Builder {
            zero_divisor: bool,
        }

        /// Creates [`Builder`].
        pub fn builder() -> Builder {
            Builder::default()
        }

        impl Builder {

            /// Creates the builder.
            #[inline]
            pub fn new() -> Self {
                builder()
            }

            /// Allows `divisor` to be zero.
            pub fn zero_divisor(mut self, enable: bool) -> Self {
                self.zero_divisor = enable;
                self
            }

            /// Creates the [`DeviceFeature`].
            pub fn build(self) -> DeviceFeature {
                use tuhka::khr;
                feature_ext(
                    move |ctx| {
                        if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                            ctx.add_required_extension(khr::vertex_attribute_divisor::NAME)?;
                        }
                        let mut features = vk::PhysicalDeviceVertexAttributeDivisorFeatures
                            ::default();
                        ctx.get_physical_device_features(&mut features);
                        if features.vertex_attribute_instance_rate_divisor == 0 {
                            Err(err!("vertex_attribute_instance_rate_divisor not supported"))
                        } else if self.zero_divisor && features.vertex_attribute_instance_rate_zero_divisor == 0 {
                            Err(err!("vertex_attribute_instance_rate_divisor_zero_divisor not supported"))
                        } else {
                            Ok(())
                        }
                    },
                    move |ctx| {
                        if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                            ctx.set_features(
                                |features: vk::PhysicalDeviceVertexAttributeDivisorFeatures|
                                    features
                                        .vertex_attribute_instance_rate_divisor(true)
                                        .vertex_attribute_instance_rate_zero_divisor(self.zero_divisor)
                            );
                        } else {
                            ctx.set_features(
                                |features: vk::PhysicalDeviceVulkan14Features|
                                    features
                                        .vertex_attribute_instance_rate_divisor(true)
                                        .vertex_attribute_instance_rate_zero_divisor(self.zero_divisor)
                            );
                        }
                        let mut properties = vk::PhysicalDeviceVertexAttributeDivisorProperties
                            ::default();
                        ctx.get_physical_device_properties(&mut properties);
                        FeatureData::new(Data(
                            Features {
                                zero_divisor: self.zero_divisor
                            },
                            Properties {
                                max_vertex_attrib_divisor: properties.max_vertex_attrib_divisor,
                                supports_non_zero_first_instance:
                                    properties.supports_non_zero_first_instance != 0,
                            },
                        ))
                    },
                    NAME,
                )
            }
        }
    }

    pub mod index_type_uint8 {
        //! Allows the use [`IndexType::U8`].
        //!
        //! See [`new`] for enabling this feature.

        use crate::features::*;

        /// The name of this feature.
        pub const NAME: FeatureName<()> = FeatureName::new("index_type_uint8");

        /// Creates the [`DeviceFeature`].
        pub fn new() -> DeviceFeature {
            use tuhka::khr;
            feature_ext(
                |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                        ctx.add_required_extension(khr::index_type_uint8::NAME)?;
                    }
                    let mut features = vk::PhysicalDeviceIndexTypeUint8Features
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.index_type_uint8 == 0 {
                        Err(err!("index_type_uint8 not supported"))
                    } else {
                        Ok(())
                    }
                },
                move |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceIndexTypeUint8Features|
                                features.index_type_uint8(true)
                        );
                    } else {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceVulkan14Features|
                                features.index_type_uint8(true)
                        );
                    }
                    FeatureData::empty()
                },
                NAME,
            )
        }
    }
    
    /* unsupported for now
    pub mod dynamic_rendering_local_read {
        //! Allows the reads from attachments written by previous fragment shaders within a dynamic
        //! render pass.
        //!
        //! See [`new`] for enabling this feature.

        use crate::features::*;

        /// [`dynamic_rendering_local_read`][self] properties.
        #[derive(Clone, Copy)]
        pub struct Properties {
            /// Specifies whether [depth/stencil attachments] **can** be input attachments.
            ///
            /// [depth/stencil attachments]: DepthStencilAttachment
            pub depth_stencil_attachments: bool,
            /// Specifies whether multisampled attachments **can** be input attachments.
            pub multisampled_attachments: bool,
        }
 
        /// The name of this feature.
        pub const NAME: FeatureName<Properties> = FeatureName::new("dynamic_rendering_local_read");

        /// Creates the [`DeviceFeature`]
        pub fn new() -> DeviceFeature {
            use tuhka::khr;
            feature_ext(
                |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                        ctx.add_required_extension(khr::dynamic_rendering_local_read::NAME)?;
                    }
                    let mut features = vk::PhysicalDeviceDynamicRenderingLocalReadFeatures
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.dynamic_rendering_local_read == 0 {
                        Err(err!("dynamic_rendering_local_read not supported"))
                    } else {
                        Ok(())
                    }
                },
                move |ctx| {
                    let mut properties = Properties {
                        depth_stencil_attachments: true,
                        multisampled_attachments: true,
                    };
                    if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceDynamicRenderingLocalReadFeatures|
                                features.dynamic_rendering_local_read(true)
                        );
                    } else {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceVulkan14Features|
                                features.dynamic_rendering_local_read(true)
                        );
                        let mut vk_14_properties = vk::PhysicalDeviceVulkan14Properties::default();
                        ctx.get_physical_device_properties(&mut vk_14_properties);
                        properties.depth_stencil_attachments = vk_14_properties
                            .dynamic_rendering_local_read_depth_stencil_attachments != 0;
                        properties.multisampled_attachments = vk_14_properties
                            .dynamic_rendering_local_read_multisampled_attachments != 0;
                    }
                    FeatureData::new(properties)
                },
                NAME
            )
        }
    } */

    /* not supported for now
    pub mod host_image_copy {
        //! Allows the use of `copy_memory_to_image`, `copy_image_to_memory`and
        //! `copy_image_to_image`.
        //!
        //! See [`new`] for enabling this feature.
        //!
        //! These commands allow copying data between host memory and images without staging the data
        //! through a GPU-accessible buffer.

        use crate::features::*;

        /// [`host_image_copy`][self] properties.
        #[derive(Clone)]
        pub struct Properties {
            /// Specifies, which image layouts are supported for the source image of a host image
            /// copy.
            pub copy_src_layouts: Box<[vk::ImageLayout]>,
            /// Specifies, which image layouts are supported for the destination image of a host image
            /// copy.
            pub copy_dst_layouts: Box<[vk::ImageLayout]>,
            /// An array of [`vk::UUID_SIZE`] uint8_t values representing a universally unique identifier
            /// for the implementation’s swizzling layout of images created with [`vk::ImageTiling::OPTIMAL`].
            pub optimal_tiling_layout_uuid: [u8; vk::UUID_SIZE as usize],
            /// Indicates that creating an image with [`ImageUsages::HOST_TRANSFER`] flag set
            /// does not affect the memory type requirements of the image.
            pub identical_memory_type_requirements: bool,
        }

        /// Contains data associated with this feature.
        #[derive(Clone)]
        pub struct Data(Properties);

        impl Data {

            /// `host_image_copy` properties.
            pub fn properties(&self) -> &Properties {
                &self.0
            }
        }

        /// The name of this feature.
        pub const NAME: FeatureName<Data> = FeatureName::new("host_image_copy");

        /// Creates the [`DeviceFeature`].
        pub fn new() -> DeviceFeature {
            use tuhka::ext;
            feature_ext(
                |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                        ctx.add_required_extension(ext::host_image_copy::NAME)?;
                    }
                    let mut features = vk::PhysicalDeviceHostImageCopyFeatures
                        ::default();
                    ctx.get_physical_device_features(&mut features);
                    if features.host_image_copy == 0 {
                        Err(err!("host_image_copy not supported"))
                    } else {
                        Ok(())
                    }
                },
                move |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceHostImageCopyFeatures|
                                features.host_image_copy(true)
                        );
                    } else {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceVulkan14Features|
                                features.host_image_copy(true)
                        );
                    }
                    let mut properties = vk::PhysicalDeviceHostImageCopyProperties
                        ::default();
                    ctx.get_physical_device_properties(&mut properties);
                    let mut copy_src_layouts: Box<[_]> = (0..properties.copy_src_layout_count)
                        .map(|_| vk::ImageLayout::UNDEFINED)
                        .collect();
                    let mut copy_dst_layouts: Box<[_]> = (0..properties.copy_dst_layout_count)
                        .map(|_| vk::ImageLayout::UNDEFINED)
                        .collect();
                    properties.p_copy_src_layouts = copy_src_layouts.as_mut_ptr();
                    properties.p_copy_dst_layouts = copy_dst_layouts.as_mut_ptr();
                    ctx.get_physical_device_properties(&mut properties);
                    FeatureData::new(Data(Properties {
                        copy_src_layouts,
                        copy_dst_layouts,
                        optimal_tiling_layout_uuid:
                            properties.optimal_tiling_layout_uuid,
                        identical_memory_type_requirements:
                        properties.identical_memory_type_requirements != 0
                    }))
                },
                NAME
            )
        }
    } */

    pub mod push_descriptor {
        //! Allows the use of push descriptors.
        //!
        //! See [`new`] for enabling this feature.

        use tuhka::khr;

        use crate::features::*; 

        /// [`push_descriptor`][self] properties.
        #[derive(Clone, Copy)]
        pub struct Properties {
            /// The maximum number of descriptors that **can** be used in a descriptor set layout
            /// created with the `PUSH_DESCRIPTOR` flag set.
            pub max_push_descriptors: u32,
        }

        /// Contains data associated with this feature.
        #[derive(Clone, Copy)]
        pub struct Data(Properties);

        impl Data {

            /// `push_descriptor` properties.
            pub fn properties(&self) -> &Properties {
                &self.0
            }
        }

        /// The name of this feature.
        pub const NAME: FeatureName<Data> = FeatureName::new("push_descriptor");

        /// Creates the [`DeviceFeature`].
        pub fn new() -> DeviceFeature {
            feature_ext(
                |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                        ctx.add_required_extension(khr::push_descriptor::NAME)?;
                        Ok(())
                    } else {
                        let mut features = vk::PhysicalDeviceVulkan14Features
                            ::default();
                        ctx.get_physical_device_features(&mut features);
                        if features.push_descriptor == 0 {
                            Err(err!("`push_descriptor` feature not supported"))
                        } else {
                            Ok(())
                        }
                    }
                },
                move |ctx| {
                    if ctx.api_version() >= VULKAN_API_VERSION_1_4 {
                        ctx.set_features(
                            |features: vk::PhysicalDeviceVulkan14Features|
                                features.push_descriptor(true)
                        );
                    }
                    let mut properties = vk::PhysicalDevicePushDescriptorProperties
                        ::default();
                    ctx.get_physical_device_properties(&mut properties);
                    FeatureData::new(Data(Properties {
                        max_push_descriptors: properties.max_push_descriptors,
                    }))
                },
                NAME
            ) 
        }
    }

    pub mod maintenance5 {
        //! Adds a collection of minor features.
        //!
        //! New features:
        //! - Allows the use of the [`A1b5g5r5_Unorm_Pack16`][1] and [`A8_Unorm`][2] formats.
        //! - Allows the use of [`BufferUsages`] beyond the 32-bit limit, which is required for some
        //!   features.
        //! - Internal improvements.
        //!
        //! [1]: Format::A1b5g5r5_Unorm_Pack16
        //! [2]: Format::A8_Unorm
        
        use tuhka::khr;

        use crate::features::*;
    
        /// The name of this feature.
        pub const NAME: FeatureName<()> = FeatureName::new("maintenance5");

        /// Creates the [`DeviceFeature`]
        pub fn new() -> DeviceFeature {
            feature_ext(
                |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                        ctx.add_required_extension(khr::maintenance5::NAME)?;
                        let mut features = vk::PhysicalDeviceMaintenance5Features::default();
                        ctx.get_physical_device_features(&mut features);
                        if features.maintenance5 == 0 {
                            Err(err!(
                                "`maintenance5` feature not supported"
                            ))
                        } else {
                            Ok(())
                        }
                    } else {
                        let mut features = vk::PhysicalDeviceVulkan14Features::default();
                        ctx.get_physical_device_features(&mut features);
                        if features.maintenance5 == 0 {
                            Err(err!(
                                "`maintenance5` feature not supported"
                            ))
                        } else {
                            Ok(())
                        }
                    }
                },
                |ctx| {
                    if ctx.api_version() < VULKAN_API_VERSION_1_4 {
                        ctx.set_features(|features: vk::PhysicalDeviceMaintenance5Features|
                            features.maintenance5(true)
                        );
                    } else {
                        ctx.set_features(|features: vk::PhysicalDeviceVulkan14Features|
                            features.maintenance5(true)
                        );
                    }
                    ctx.add_supported_enums([
                        Format::A1b5g5r5_Unorm_Pack16,
                        Format::A8_Unorm,
                    ]);
                    FeatureData::empty()
                },
                NAME
            )
        }
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

/// [`Features`] prefixed with KHR.
pub mod vk_khr {

    use super::*;

    pub mod robustness2 {
        //! Adds stricter requirements for how out of bounds reads and writes are handled.
        //!
        //! See [`Builder`] for enabling `robustness2` features.

        use tuhka::khr;

        use crate::features::*;

        /// [`robustness2`][self] properties.
        #[derive(Clone, Copy)]
        pub struct Properties {
            /// The number of bytes that the range of a storage buffer descriptor is rounded up to
            /// when used for bounds-checking when the [`robust_buffer_access2`][1] feature is enabled.
            ///
            /// [1]: Features::robust_buffer_access2
            pub robust_storage_buffer_access_size_alignment: DeviceSize,
            /// The number of bytes that the range of a uniform buffer descriptor is rounded up
            /// to when used for bounds-checking when the [`robust_buffer_access2`][1] feature is enabled.
            ///
            /// [1]: Features::robust_buffer_access2
            pub robust_uniform_buffer_access_size_alignment: DeviceSize,
        }

        /// [`robustness2`][self] features.
        #[derive(Default, Clone, Copy)]
        pub struct Features {
            /// Specifies that robust buffer access 2 guarantees for shader buffer accesses are
            /// supported.
            pub robust_buffer_access2: bool,
            /// Specifies robust image access 2 guarantees for shader buffer accesses are supported.
            pub robust_image_access2: bool,
        }

        /// Contains data about this feature.
        #[derive(Clone, Copy)]
        pub struct Data(Features, Features, Properties);

        impl Data {

            /// Enabled [`robustness2`][self] features.
            pub fn enabled_features(&self) -> &Features {
                &self.0
            }

            /// Supported [`robustness2`][self] features.
            pub fn supported_features(&self) -> &Features {
                &self.1
            }

            /// [`robustness2`][self] properties.
            #[inline]
            pub fn properties(&self) -> &Properties {
                &self.2
            }
        }
        
        /// The name of this feature.
        pub const NAME: FeatureName<Data> = FeatureName::new("robustness2");

        /// Creates [`Builder`].
        pub fn builder() -> Builder {
            Builder::default()
        }

        /// A builder for enabling [`robustness2`][self] features.
        #[derive(Default, Clone, Copy)]
        pub struct Builder {
            robust_buffer_access2: bool,
            robust_image_access2: bool,
        }

        impl Builder {

            /// Creates the builder.
            #[inline]
            pub fn new() -> Self {
                builder()
            }

            /// Enables robust buffer access 2 guarantees for shader buffer accesses.
            pub fn robust_buffer_access2(mut self, enable: bool) -> Self {
                self.robust_buffer_access2 = enable;
                self
            }

            /// Enables robust image access 2 guarantees for shader buffer accesses.
            pub fn robust_image_access2(mut self, enable: bool) -> Self {
                self.robust_image_access2 = enable;
                self
            }

            /// Creates the [`DeviceFeature`].
            pub fn build(self) -> DeviceFeature {
                feature_ext(
                    move |ctx| {
                        ctx.add_required_extension(khr::robustness2::NAME)?;
                        let mut vk_features = vk::PhysicalDeviceRobustness2FeaturesKHR
                            ::default();
                        ctx.get_physical_device_features(&mut vk_features);
                        if self.robust_buffer_access2 &&
                            vk_features.robust_buffer_access2 == 0
                        {
                            Err(err!(
                                "`robust_buffer_access2` feature not supported"
                            ))?
                        }
                        if self.robust_image_access2 &&
                            vk_features.robust_image_access2 == 0
                        {
                            Err(err!(
                                "`robust_image_access2` feature not supported"
                            ))?
                        }
                        Ok(())
                    },
                    move |ctx| {
                        ctx.set_features(
                            |f: vk::PhysicalDeviceRobustness2FeaturesKHR|
                                f.robust_buffer_access2(self.robust_buffer_access2)
                                    .robust_image_access2(self.robust_image_access2)
                        );
                        let mut supported = vk::PhysicalDeviceRobustness2FeaturesKHR
                            ::default();
                        ctx.get_physical_device_features(&mut supported);
                        let mut properties = vk::PhysicalDeviceRobustness2PropertiesKHR
                            ::default();
                        ctx.get_physical_device_properties(&mut properties);
                        FeatureData::new(Data(
                            Features {
                                robust_buffer_access2: self.robust_buffer_access2,
                                robust_image_access2: self.robust_image_access2,
                            },
                            Features {
                                robust_buffer_access2: supported.robust_buffer_access2 != 0,
                                robust_image_access2: supported.robust_image_access2 != 0,
                            },
                            Properties {
                                robust_storage_buffer_access_size_alignment:
                                    properties.robust_storage_buffer_access_size_alignment,
                                robust_uniform_buffer_access_size_alignment:
                                    properties.robust_uniform_buffer_access_size_alignment,
                            },
                        ))
                    },
                    NAME,
                )
            }
        }
    }

    pub mod swapchain {
        //! Allows the use of [`Swapchain`].
            
        use tuhka::khr;

        use crate::features::*;

        use sync::OnceLock;

        use khr::swapchain::Device;

        pub struct Data(OnceLock<Device>);

        /// The name of this future.
        pub const NAME: FeatureName<Data> = FeatureName::new("swapchain");

        impl Data {

            #[inline]
            pub(crate) fn functions(&self, device: &crate::Device) -> &khr::swapchain::Device {
                self.0.get_or_init(|| Device::new(device.functions()))
            }
        }

        /// Creates the [`DeviceFeature`].
        pub fn new() -> DeviceFeature {
            feature_ext(
                |ctx| {
                    if ctx.instance().has_display_handle() {
                        ctx.add_required_extension(khr::swapchain::NAME)?;
                        Ok(())
                    } else {
                        Ok(())
                    }
                },
                |_| {
                    FeatureData::new(Data(OnceLock::new()))
                },
                NAME,
            )
        }
    }
}

/// [`Features`] prefixed with EXT.
pub mod vk_ext {

    pub mod mesh_shader {
        //! Allows the use of mesh shaders.
        //!
        //! Replaces the traditional primitive shading pipeline.
        //!
        //! If the [`task_shader`][1] feature is enabled, shaders with the `TaskEXT`
        //! shader stage **can** be created.
        //!
        //! [1]: Features::task_shader

        use tuhka::ext;

        use crate::sync::OnceLock;

        use crate::features::*;

        /// [`mesh_shader`][self] properties.
        #[derive(Clone, Copy, Debug)]
        pub struct Properties {
            /// The maximum amount of local workgroups that **can** be launched by a
            /// mesh task drawing command (`group_count_x * group_count_y * group_count_z`).
            pub max_task_work_group_total_count: u32,
            /// The maximum amount of local workgroups that **can** be launched for by a
            /// mesh task drawing command in each x/y/z dimension.
            pub max_task_work_group_count: [u32; 3usize],
            /// The maximum total number of task shader invocations in a single workgroup in a mesh
            /// shader.
            pub max_task_work_group_invocations: u32,
            /// The maximum size of a local workgroup for a task shader in each dimension. 
            pub max_task_work_group_size: [u32; 3usize],
            /// The maximum total storage size, in bytes, available for variables declared with the
            /// `TaskPayloadWorkgroupEXT` storage class in shader modules in the task shader stage.
            pub max_task_payload_size: u32,
            /// The maximum total storage size, in bytes, available for variables that are declared
            /// with the `TaskPayloadWorkgroupEXT` or `Workgroup` storage class, in shader modules in
            /// the task shader stage.
            pub max_task_shared_memory_size: u32,
            /// The maximum number of local workgroups that **can** be launched for a mesh shader, either
            /// directly by a mesh tasks drawing command, or emitted by a single task shader
            /// workgroup.
            pub max_task_payload_and_shared_memory_size: u32,
            /// The maximum number of local workgroups that **can** be launched for a mesh shader, either
            /// directly by a mesh tasks drawing command, or emitted by a single task shader
            /// workgroup.
            pub max_mesh_work_group_total_count: u32,
            /// The maximum number of local workgroups that **can** be launched for a mesh shader in each
            /// dimension, either directly by a mesh tasks drawing command, or emitted by a single
            /// task shader workgroup.
            pub max_mesh_work_group_count: [u32; 3usize],
            /// The maximum total number of mesh shader invocations in a single local workgroup.
            pub max_mesh_work_group_invocations: u32,
            /// The maximum size of a local workgroup for a mesh shader in each dimension.
            pub max_mesh_work_group_size: [u32; 3usize],
            /// The maximum total storage size, in bytes, available for variables declared with the
            /// `Workgroup` storage class in shader modules in the mesh shader stage.
            pub max_mesh_shared_memory_size: u32,
            /// The maximum total storage size, in bytes, available for variables that are declared
            /// with the `TaskPayloadWorkgroupEXT` or `Workgroup` storage class in shader modules in the
            /// mesh shader stage.
            pub max_mesh_payload_and_shared_memory_size: u32,
            /// The maximum total storage size, in bytes, available for output variables in shader
            /// modules in the mesh shader stage.
            pub max_mesh_output_memory_size: u32,
            /// The maximum total storage size, in bytes, available for variables that are declared
            /// with the `TaskPayloadWorkgroupEXT` storage class, or output variables in shader
            /// modules in the mesh shader stag.
            pub max_mesh_payload_and_output_memory_size: u32,
            /// The maximum number of components of output variables which **can** be output from the
            /// mesh shader stage.
            pub max_mesh_output_components: u32,
            /// The maximum number of vertices which **can** be emitted by a single mesh shader
            /// workgroup.
            pub max_mesh_output_vertices: u32,
            /// The maximum number of primitives which **can** be emitted by a single mesh shader
            /// workgroup.
            pub max_mesh_output_primitives: u32,
            /// One greater than the maximum layer index that **can** be output from the mesh shader
            /// stage.
            pub max_mesh_output_layers: u32,
            /// One greater than the maximum view index that **can** be used by any mesh shader.
            pub max_mesh_multiview_view_count: u32,
            /// The granularity of vertex allocation. The number of output vertices allocated for
            /// the mesh shader stage is padded to a multiple of this number.
            pub mesh_output_per_vertex_granularity: u32,
            /// The granularity of primitive allocation. The number of output primitives allocated
            /// for the mesh shader stage is padded to a multiple of this number.
            pub mesh_output_per_primitive_granularity: u32,
            /// The maximum number of task shader invocations in a single workgroup that is
            /// preferred by the implementation for optimal performance.
            pub max_preferred_task_work_group_invocations: u32,
            /// The maximum number of mesh shader invocations in a single workgroup that is
            /// preferred by the implementation for optimal performance.
            pub max_preferred_mesh_work_group_invocations: u32,
            /// Specifies whether writes to the vertex output array in a mesh shader yield best
            /// performance when the array index matches `LocalInvocationIndex`.
            pub prefers_local_invocation_vertex_output: bool,
            /// Specifies whether writes to the primitive output array in a mesh shader yield best
            /// performance when the array index matches `LocalInvocationIndex`.
            pub prefers_local_invocation_primitive_output: bool,
            /// Specifies whether output vertices should be compacted after custom culling in the
            /// mesh shader for best performance, otherwise keeping the vertices at their original
            /// location may be better.
            pub prefers_compact_vertex_output: bool,
            /// Specifies whether output primitives should be compacted after custom culling in the
            /// mesh shader for best performance, otherwise the use of `CullPrimitiveEXT` may be
            /// better.
            pub prefers_compact_primitive_output: bool,
        }

        /// Enabled [`mesh_shader`][self] features.
        #[derive(Default, Clone, Copy)]
        pub struct Features {
            /// Indicates that task shaders **can** be used.
            pub task_shader: bool,
            /// Indicates that mesh shaders **can** use multi-view rendering.
            pub multiview_mesh_shader: bool,
            /// Indicates that primitive fragment shading rate is allowed mesh shaders.
            pub primitive_fragment_shading_rate_mesh_shader: bool,
        }

        /// A builder for enabling [`mseh_shader`][self] features.
        #[derive(Default, Clone, Copy)]
        pub struct Builder {
            task_shader: bool,
            multiview_mesh_shader: bool,
            primitive_fragment_shading_rate_mesh_shader: bool,
        }

        /// Creates [`Builder`].
        pub fn builder() -> Builder {
            Builder::default()
        }

        /// Device-level functions in this extension.
        pub type Device = ext::mesh_shader::Device;

        /// Contains data associated with this feature.
        #[derive(Clone)]
        pub struct Data(Properties, Features, OnceLock<Device>);

        
        impl Data {

            /// [`mesh_shader`][self] properties.
            #[inline]
            pub fn properties(&self) -> &Properties {
                &self.0
            }

            /// Enabled [`mesh_shader`][self] features.
            #[inline]
            pub fn enabled_features(&self) -> &Features {
                &self.1
            }

            /// [`mesh_shader`][self] [`device`][Device].
            #[inline]
            pub(crate) fn functions(&self, device: &crate::Device) -> &Device {
                self.2.get_or_init(|| Device::new(device.functions()))
            }
        }
    
        /// The name of this feature.
        pub const NAME: FeatureName<Data> = FeatureName::new("mesh_shader");

        impl Builder {

            /// Creates the builder.
            #[inline]
            pub fn new() -> Self {
                builder()
            }

            /// Allows the use of task shaders.
            pub fn task_shader(mut self, enable: bool) -> Self {
                self.task_shader = enable;
                self
            }

            /// Allows the use of multi-view rendering with mesh shaders
            pub fn multiview_mesh_shader(mut self, enable: bool) -> Self {
                self.multiview_mesh_shader = enable;
                self
            }

            /// Allows the use of primitive fragment shading rate in mesh shaders.
            pub fn primitive_fragment_shading_rate_mesh_shader(mut self, enable: bool) -> Self {
                self.primitive_fragment_shading_rate_mesh_shader = enable;
                self
            }

            /// Creates the [`DeviceFeature`].
            pub fn build(self) -> DeviceFeature {
                feature_ext(
                    move |ctx| {
                        ctx.add_required_extension(ext::mesh_shader::NAME)?;
                        let mut vk_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default();
                        ctx.get_physical_device_features(&mut vk_features);
                        if self.task_shader && vk_features.task_shader == 0 {
                            Err(err!("`task_shader` feature not supported"))
                        } else if vk_features.mesh_shader == 0 {
                            Err(err!("`mesh_shader` feature not supported"))
                        } else if self.multiview_mesh_shader && vk_features.multiview_mesh_shader == 0 {
                            Err(err!("`multiview_mesh_shader` feature not supported"))
                        } else if self.primitive_fragment_shading_rate_mesh_shader &&
                            vk_features.primitive_fragment_shading_rate_mesh_shader == 0
                        {
                            Err(err!("`primitive_fragment_shading_rate_mesh_shader` feature not supported"))
                        } else {
                            Ok(())
                        }
                    },
                    move |ctx| {
                        let mut properties = vk::PhysicalDeviceMeshShaderPropertiesEXT
                            ::default();
                        ctx.get_physical_device_properties(&mut properties);
                        macro_rules! prop {
                            ($prop:ident, $($name:ident $(!= $b:literal)?),+ $(,)?) => {Properties {$(
                                $name: $prop.$name $(!= $b)?,
                            )+}};
                        }
                        let properties = prop!(properties,
                            max_task_work_group_total_count,
                            max_task_work_group_count,
                            max_task_work_group_invocations,
                            max_task_work_group_size,
                            max_task_payload_size,
                            max_task_shared_memory_size,
                            max_task_payload_and_shared_memory_size,
                            max_mesh_work_group_total_count,
                            max_mesh_work_group_count,
                            max_mesh_work_group_invocations,
                            max_mesh_work_group_size,
                            max_mesh_shared_memory_size,
                            max_mesh_payload_and_shared_memory_size,
                            max_mesh_output_memory_size,
                            max_mesh_payload_and_output_memory_size,
                            max_mesh_output_components,
                            max_mesh_output_vertices,
                            max_mesh_output_primitives,
                            max_mesh_output_layers,
                            max_mesh_multiview_view_count,
                            mesh_output_per_vertex_granularity,
                            mesh_output_per_primitive_granularity,
                            max_preferred_task_work_group_invocations,
                            max_preferred_mesh_work_group_invocations,
                            prefers_local_invocation_vertex_output != 0,
                            prefers_local_invocation_primitive_output != 0,
                            prefers_compact_vertex_output != 0,
                            prefers_compact_primitive_output != 0,
                        );
                        ctx.set_features(
                            |f: vk::PhysicalDeviceMeshShaderFeaturesEXT| {
                                f.task_shader(self.task_shader)
                                    .mesh_shader(true)
                                    .multiview_mesh_shader(self.multiview_mesh_shader)
                                    .primitive_fragment_shading_rate_mesh_shader(
                                        self.primitive_fragment_shading_rate_mesh_shader
                                    )
                            }
                        );
                        FeatureData::new(Data(
                            properties,
                            Features {
                                task_shader: self.task_shader,
                                multiview_mesh_shader: self.multiview_mesh_shader,
                                primitive_fragment_shading_rate_mesh_shader:
                                    self.primitive_fragment_shading_rate_mesh_shader,
                            },
                            OnceLock::new()))
                    },
                    NAME,
                )
            }
        }
    }

    pub mod extended_dynamic_state3 {
        //! Extends the dynamic state for pipelines.
        //!
        //! See [`Builder`] for `extended_dynamic_state3` features.

        use tuhka::ext::{self, extended_dynamic_state3::Device};

        use crate::sync::OnceLock;

        use crate::features::*;

        /// Enabled [`extended_dynamic_state3`][self] features.
        #[derive(Default, Clone, Copy)]
        pub struct Features {
            /// Specifies that [`TessellationDomainOrigin`] **can** be dynamic.
            pub tessellation_domain_origin: bool,
            /// Specifies that [`DepthClampEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::DepthClampEnable
            pub depth_clamp_enable: bool,
            /// Specifies that [`PolygonMode`] **can** be dynamic.
            pub polygon_mode: bool,
            /// Specifies that [`RasterizationSamples`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::RasterizationSamples
            pub rasterization_samples: bool,
            /// Specifies that [`SampleMask`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::SampleMask
            pub sample_mask: bool,
            /// Specifies that [`AlphaToCoverageEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::AlphaToCoverageEnable
            pub alpha_to_coverage_enable: bool,
            /// Specifies that [`AlphaToOneEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::AlphaToOneEnable
            pub alpha_to_one_enable: bool,
            /// Specifies that [`LogicOpEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::LogicOpEnable
            pub logic_op_enable: bool,
            /// Specifies that [`ColorBlendEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::ColorBlendEnable
            pub color_blend_enable: bool,
            /// Specifies that [`ColorBlendEquations`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::ColorBlendEquations
            pub color_blend_equation: bool,
            /// Specifies that [`ColorWriteMask`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::ColorWriteMask
            pub color_write_mask: bool,
            /// Specifies that [`LineRasterizationMode`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::LineRasterizationMode
            pub line_rasterization_mode: bool,
            /// Specifies that [`LineStippleEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::LineStippleEnable
            pub line_stipple_enable: bool,
        }

        /// Contains data associated with this feature.
        #[derive(Clone)]
        pub struct Data(Features, OnceLock<Device>);

        impl Data {

            /// Returns enabled [`extended_dynamic_state3`][self] features.
            ///
            /// It **may** be more efficient to query supported dynamic state directly with
            /// [`is_pipeline_state_dynamic_allowed`][1].
            ///
            /// [1]: crate::Device::is_pipeline_state_dynamic_allowed
            #[inline]
            pub fn enabled_features(&self) -> &Features {
                &self.0
            }

            /// Returns [`Device`], which contains all relevant Vulkan function pointers.
            pub(crate) fn functions(&self, device: &crate::Device) -> &Device {
                self.1.get_or_init(|| Device::new(device.functions()))
            }
        }

        /// The name of this feature.
        pub const NAME: FeatureName<Data> = FeatureName::new("extended_dynamic_state3");

        /// A builder for enabling features.
        #[derive(Default, Clone, Copy)]
        pub struct Builder {
            extended_dynamic_state3_tessellation_domain_origin: bool,
            extended_dynamic_state3_depth_clamp_enable: bool,
            extended_dynamic_state3_polygon_mode: bool,
            extended_dynamic_state3_rasterization_samples: bool,
            extended_dynamic_state3_sample_mask: bool,
            extended_dynamic_state3_alpha_to_coverage_enable: bool,
            extended_dynamic_state3_alpha_to_one_enable: bool,
            extended_dynamic_state3_logic_op_enable: bool,
            extended_dynamic_state3_color_blend_enable: bool,
            extended_dynamic_state3_color_blend_equation: bool,
            extended_dynamic_state3_color_write_mask: bool,
            extended_dynamic_state3_line_rasterization_mode: bool,
            extended_dynamic_state3_line_stipple_enable: bool,
        }

        impl Builder {

            /// Creates the builder.
            #[inline]
            pub fn new() -> Self {
                builder()
            }

            /// Specifies that [`TessellationDomainOrigin`] **can** be dynamic.
            #[inline]
            pub fn tessellation_domain_origin(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_tessellation_domain_origin = enable;
                self
            }
            /// Specifies that [`DepthClampEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::DepthClampEnable
            #[inline]
            pub fn depth_clamp_enable(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_depth_clamp_enable = enable;
                self
            }
            /// Specifies that [`PolygonMode`] **can** be dynamic.
            #[inline]
            pub fn polygon_mode(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_polygon_mode = enable;
                self
            }
            /// Specifies that [`RasterizationSamples`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::RasterizationSamples
            #[inline]
            pub fn rasterization_samples(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_rasterization_samples = enable;
                self
            }
            /// Specifies that [`SampleMask`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::SampleMask
            #[inline]
            pub fn sample_mask(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_sample_mask = enable;
                self
            }
            /// Specifies that [`AlphaToCoverageEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::AlphaToCoverageEnable
            #[inline]
            pub fn alpha_to_coverage_enable(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_alpha_to_coverage_enable = enable;
                self
            }
            /// Specifies that [`AlphaToOneEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::AlphaToOneEnable
            #[inline]
            pub fn alpha_to_one_enable(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_alpha_to_one_enable = enable;
                self
            }
            /// Specifies that [`LogicOpEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::LogicOpEnable
            #[inline]
            pub fn logic_op_enable(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_logic_op_enable = enable;
                self
            }
            /// Specifies that [`ColorBlendEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::ColorBlendEnable
            #[inline]
            pub fn color_blend_enable(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_color_blend_enable = enable;
                self
            }
            /// Specifies that [`ColorBlendEquations`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::ColorBlendEquations
            #[inline]
            pub fn color_blend_equation(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_color_blend_equation = enable;
                self
            }
            /// Specifies that [`ColorWriteMask`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::ColorWriteMask
            #[inline]
            pub fn color_write_mask(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_color_write_mask = enable;
                self
            }
            /// Specifies that [`LineRasterizationMode`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::LineRasterizationMode
            #[inline]
            pub fn line_rasterization_mode(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_line_rasterization_mode = enable;
                self
            }
            /// Specifies that [`LineStippleEnable`][1] **can** be dynamic.
            ///
            /// [1]: pipeline_state::LineStippleEnable
            #[inline]
            pub fn line_stipple_enable(mut self, enable: bool) -> Self {
                self.extended_dynamic_state3_line_stipple_enable = enable;
                self
            }
    
            /// Creates the [`DeviceFeature`].
            pub fn build(self) -> DeviceFeature {
                feature_ext(
                    move |ctx| {
                        ctx.add_required_extension(ext::extended_dynamic_state3::NAME)?;
                        let mut vk_features = vk::PhysicalDeviceExtendedDynamicState3FeaturesEXT
                            ::default();
                        ctx.get_physical_device_features(&mut vk_features);
                        macro_rules! check_features {
                            ($($name:ident),+ $(,)?) => {$(
                                if self.$name &&
                                    vk_features.$name == 0
                                {
                                    Err(err!("{}", concat!(
                                        "`", stringify!($name), "` feature not supported",
                                    )))?
                                }
                            )+};
                        }
                        check_features!(
                            extended_dynamic_state3_tessellation_domain_origin,
                            extended_dynamic_state3_depth_clamp_enable,
                            extended_dynamic_state3_polygon_mode,
                            extended_dynamic_state3_rasterization_samples,
                            extended_dynamic_state3_sample_mask,
                            extended_dynamic_state3_alpha_to_coverage_enable,
                            extended_dynamic_state3_alpha_to_one_enable,
                            extended_dynamic_state3_logic_op_enable,
                            extended_dynamic_state3_color_blend_enable,
                            extended_dynamic_state3_color_blend_equation,
                            extended_dynamic_state3_color_write_mask,
                            extended_dynamic_state3_line_rasterization_mode,
                            extended_dynamic_state3_line_stipple_enable,
                        );
                        Ok(())
                    },
                    move |ctx| {
                        ctx.set_features(
                            |f: vk::PhysicalDeviceExtendedDynamicState3FeaturesEXT| {
                                f.extended_dynamic_state3_tessellation_domain_origin(
                                    self.extended_dynamic_state3_tessellation_domain_origin
                                ).extended_dynamic_state3_depth_clamp_enable(
                                    self.extended_dynamic_state3_depth_clamp_enable
                                ).extended_dynamic_state3_polygon_mode(
                                    self.extended_dynamic_state3_polygon_mode
                                ).extended_dynamic_state3_rasterization_samples(
                                    self.extended_dynamic_state3_rasterization_samples
                                ).extended_dynamic_state3_sample_mask(
                                    self.extended_dynamic_state3_sample_mask
                                ).extended_dynamic_state3_alpha_to_coverage_enable(
                                    self.extended_dynamic_state3_alpha_to_coverage_enable
                                ).extended_dynamic_state3_alpha_to_one_enable(
                                    self.extended_dynamic_state3_alpha_to_one_enable
                                ).extended_dynamic_state3_logic_op_enable(
                                    self.extended_dynamic_state3_logic_op_enable
                                ).extended_dynamic_state3_color_blend_enable(
                                    self.extended_dynamic_state3_color_blend_enable
                                ).extended_dynamic_state3_color_blend_equation(
                                    self.extended_dynamic_state3_color_blend_equation
                                ).extended_dynamic_state3_color_write_mask(
                                    self.extended_dynamic_state3_color_write_mask
                                ).extended_dynamic_state3_line_rasterization_mode(
                                    self.extended_dynamic_state3_line_rasterization_mode
                                ).extended_dynamic_state3_line_stipple_enable(
                                    self.extended_dynamic_state3_line_stipple_enable
                                )
                            }
                        );
                        if self.extended_dynamic_state3_tessellation_domain_origin {
                            ctx.add_supported_enums([
                                PipelineStateKind::TessellationDomainOrigin
                            ]);
                        }
                        if self.extended_dynamic_state3_depth_clamp_enable {
                            ctx.add_supported_enums([
                                PipelineStateKind::DepthClampEnable
                            ]);
                        }
                        if self.extended_dynamic_state3_polygon_mode {
                            ctx.add_supported_enums([
                                PipelineStateKind::PolygonMode
                            ]);
                        }
                        if self.extended_dynamic_state3_rasterization_samples {
                            ctx.add_supported_enums([
                                PipelineStateKind::RasterizationSamples
                            ]);
                        }
                        if self.extended_dynamic_state3_sample_mask {
                            ctx.add_supported_enums([
                                PipelineStateKind::SampleMask
                            ]);
                        }
                        if self.extended_dynamic_state3_alpha_to_coverage_enable {
                            ctx.add_supported_enums([
                                PipelineStateKind::AlphaToCoverageEnable
                            ]);
                        }
                        if self.extended_dynamic_state3_alpha_to_one_enable {
                            ctx.add_supported_enums([
                                PipelineStateKind::AlphaToOneEnable
                            ]);
                        }
                        if self.extended_dynamic_state3_logic_op_enable {
                            ctx.add_supported_enums([
                                PipelineStateKind::LogicOpEnable
                            ]);
                        }
                        if self.extended_dynamic_state3_color_blend_enable {
                            ctx.add_supported_enums([
                                PipelineStateKind::ColorBlendEnable
                            ]);
                        }
                        if self.extended_dynamic_state3_color_blend_equation {
                            ctx.add_supported_enums([
                                PipelineStateKind::ColorBlendEquation
                            ]);
                        }
                        if self.extended_dynamic_state3_color_write_mask {
                            ctx.add_supported_enums([
                                PipelineStateKind::ColorWriteMask
                            ]);
                        }
                        if self.extended_dynamic_state3_line_rasterization_mode {
                            ctx.add_supported_enums([
                                PipelineStateKind::LineRasterizationMode
                            ]);
                        }
                        if self.extended_dynamic_state3_line_stipple_enable {
                            ctx.add_supported_enums([
                                PipelineStateKind::LineStippleEnable
                            ]);
                        }
                        FeatureData::new(Data(
                            Features {
                                tessellation_domain_origin: self.extended_dynamic_state3_tessellation_domain_origin,
                                depth_clamp_enable: self.extended_dynamic_state3_depth_clamp_enable,
                                polygon_mode: self.extended_dynamic_state3_polygon_mode,
                                rasterization_samples: self.extended_dynamic_state3_rasterization_samples,
                                sample_mask: self.extended_dynamic_state3_sample_mask,
                                alpha_to_coverage_enable: self.extended_dynamic_state3_alpha_to_coverage_enable,
                                alpha_to_one_enable: self.extended_dynamic_state3_alpha_to_one_enable,
                                logic_op_enable: self.extended_dynamic_state3_logic_op_enable,
                                color_blend_enable: self.extended_dynamic_state3_color_blend_enable,
                                color_blend_equation: self.extended_dynamic_state3_color_blend_equation,
                                color_write_mask: self.extended_dynamic_state3_color_write_mask,
                                line_rasterization_mode: self.extended_dynamic_state3_line_rasterization_mode,
                                line_stipple_enable: self.extended_dynamic_state3_line_stipple_enable 
                            },
                            OnceLock::new(),
                        ))
                    },
                    NAME,
                )
                }
        }
   
        /// Creates [`Builder`] with all features disabled.
        #[inline]
        pub fn builder() -> Builder {
            Builder::default()
        }
    }

    pub mod vertex_input_dynamic_state {
        //! Allows [`VertexInput`][1] [pipeline state][crate::pipeline_state] to be dynamic.
        //!
        //! [1]: crate::pipeline_state::VertexInput
        //! [pipeline state]: pipeline_state

        use tuhka::ext::{self, vertex_input_dynamic_state::Device};

        use crate::sync::OnceLock;

        use crate::features::*;

        /// Contains data associated with this feature.
        #[derive(Clone)]
        pub struct Data(OnceLock<Device>);

        impl Data {

            /// [`mesh_shader`][self] [`device`][Device].
            #[inline]
            pub(crate) fn functions(&self, device: &crate::Device) -> &Device {
                self.0.get_or_init(|| Device::new(device.functions()))
            }
        }

        /// The name of this feature.
        pub const NAME: FeatureName<Data> = FeatureName::new("vertex_input_dynamic_state");

        /// Creates the [`DeviceFeature`]
        pub fn new() -> DeviceFeature {
            feature_ext(
                move |ctx| {
                    ctx.add_required_extension(ext::vertex_input_dynamic_state::NAME)?;
                    let mut vk_features = vk::PhysicalDeviceVertexInputDynamicStateFeaturesEXT
                        ::default();
                    ctx.get_physical_device_features(&mut vk_features);
                    if vk_features.vertex_input_dynamic_state == 0 {
                        Err(err!(
                            "`vertex_input_dynamic_state` not supported"
                        ))?
                    }
                    Ok(())
                },
                move |ctx| {
                    ctx.set_features(|f: vk::PhysicalDeviceVertexInputDynamicStateFeaturesEXT|
                        f.vertex_input_dynamic_state(true)
                    );
                    FeatureData::new(Data(OnceLock::new()))
                },
                NAME,
            )
        }
    }

    // TODO: VK_EXT_descriptor_heap
    //
    // Requires: maintenance5 and buffer_device_addresses
}

#[inline]
pub(crate) fn core_features() -> [DeviceFeature; 8] {
    [
        vk_12::timeline_semaphore(),
        vk_12::separate_depth_stencil_layouts(),
        vk_13::synchronization2(),
        vk_13::dynamic_rendering(),
        vk_13::copy_commands2(),
        vk_13::extended_dynamic_state(),
        vk_13::maintenance4::new(),
        vk_13::shader_terminate_invocation::new(),
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
