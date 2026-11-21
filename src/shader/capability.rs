use super::*;

use op::Capability;

macro_rules! cap {
    ($cap:expr, $feature:path) => {
        ($cap, |device, _| {
            use $feature::{NAME};
            if !device
                .is_feature_enabled(NAME)
            {
                Err(format!(
                    "the `{}` feature is not enabled",
                    stringify!($feature)
                ))
            } else {
                Ok(())
            }
        })
    };
}

pub(super) static CAPABILITY_QUERIES: statics::StaticMap<
    Capability,
    FpQuery,
> = static_map(
    &[
        (Capability::MATRIX, |_, _| Ok(())),
        (Capability::SHADER, |_, _| Ok(())),
        (Capability::INPUT_ATTACHMENT, |_, _| Ok(())),
        (Capability::SAMPLED_1D, |_, _| Ok(())),
        (Capability::IMAGE_1D, |_, _| Ok(())),
        (Capability::SAMPLED_BUFFER, |_, _| Ok(())),
        (Capability::IMAGE_BUFFER, |_, _| Ok(())),
        (Capability::IMAGE_QUERY, |_, _| Ok(())),
        (Capability::DERIVATIVE_CONTROL, |_, _| Ok(())),
        cap!(Capability::GEOMETRY, vk_10::geometry_shader),
        cap!(Capability::TESSELLATION, vk_10::tessellation_shader),
        cap!(Capability::FLOAT_64, vk_10::shader_float64),
        cap!(Capability::INT_64, vk_10::shader_int64),
        (Capability::INT_64_ATOMICS, |device, args| {
            for &(_, type_id) in &args.atomic_inst_ptr_types {
                let mut result_type = args.module
                    .get_result(type_id)
                    .unwrap();
                result_type.advance(1).ok();
                let storage_class = op::StorageClass::parse_one(&mut result_type).unwrap();
                let mut ty = args.module
                    .get_result(op::IdRef::parse_one(&mut result_type).unwrap())
                    .unwrap();
                if ty.code() == op::Code::TYPE_INT {
                    ty.advance(1).ok();
                    let width = ty.read().unwrap();
                    if width == 64 {
                        if storage_class == op::StorageClass::WORKGROUP {
                            if !device.is_feature_enabled(vk_12::shader_shared_int64_atomics::NAME)
                            {
                                Err(concat!(
                                    "64-bit integer atomics used on `Workgroup` storage class memory, ",
                                    "but the `vk_12::shared_shader_int64_atomics` feature is not enabled"
                                ))?
                            }
                        } else if storage_class == op::StorageClass::IMAGE {
                            Err(
                                "64-bit integer atomics used of `Image` storage class memory, which is not currently supported"
                            )?
                        } else if !device.is_feature_enabled(vk_12::shader_buffer_int64_atomics::NAME)
                        {
                            Err(
                                "64-bit integer atomics used on buffer memory, but the `vk_12::shader_buffer_int64_atomics` feature is not enabled"
                            )?
                        }
                    }
                }
            }
            Ok(())
        }),
        (Capability::ATOMIC_FLOAT_32_ADD_EXT, |device, args| {
            for type_id in args.atomic_inst_ptr_types
                .iter().filter_map(|&(code, type_id)| (code == op::Code::ATOMIC_FADD_EXT).then_some(type_id))
            {
                let mut result_type = args.module
                    .get_result(type_id)
                    .unwrap();
                result_type.advance(1).ok();
                let storage_class = op::StorageClass::parse_one(&mut result_type).unwrap();
                match storage_class {
                    op::StorageClass::WORKGROUP => {
                        if !device
                            .is_feature_enabled(vk_ext::shader_shared_float32_atomic_add::NAME)
                        {
                            Err(
                                concat!(
                                    "`OpAtomicFAdd` used on `Workgroup` storage class memory, ",
                                    "but the `vk_ext::shader_shared_float32_atomic_add` feature is not enabled",
                                )
                            )?
                        }
                    },
                    op::StorageClass::IMAGE => {
                        if !device
                            .is_feature_enabled(vk_ext::shader_image_float32_atomic_add::NAME)
                        {
                            Err(
                                "`OpAtomicFAdd` used on `Image` storage class memory, but the `vk_ext::shader_atomic_float::image_f32_atomic_add` feature is not enabled"
                            )?
                        }
                    },
                    _ => if !device
                            .is_feature_enabled(vk_ext::shader_buffer_float32_atomic_add::NAME)
                    {
                        Err(
                            "`OpAtomicFAdd` used on buffer memory, but the `vk_ext::shader_atomic_float::buffer_f32_atomic_add` feature is not enabled"
                        )?
                    }
                }
            }
            Ok(())
        }),
        (Capability::ATOMIC_FLOAT_64_ADD_EXT, |device, args| {
            for type_id in args.atomic_inst_ptr_types
                .iter().filter_map(|&(code, type_id)| (code == op::Code::ATOMIC_FADD_EXT).then_some(type_id))
            {
                let mut result_type = args.module
                    .get_result(type_id)
                    .unwrap();
                result_type.advance(1).ok();
                let storage_class = op::StorageClass::parse_one(&mut result_type).unwrap();
                match storage_class {
                    op::StorageClass::WORKGROUP => {
                        if !device
                            .is_feature_enabled(vk_ext::shader_shared_float64_atomic_add::NAME)
                        {
                            Err(
                                "`OpAtomicFAdd` used on `Workgroup` storage class memory, but the `vk_ext::shader_atomic_float::shared_f64_atomic_add` feature is not enabled"
                            )?
                        }
                    },
                    op::StorageClass::IMAGE => {
                        Err(
                            "`OpAtomicFAdd` with 64-bit floats used on `Image` storage class memory"
                        )?
                    },
                    _ => if !device
                            .is_feature_enabled(vk_ext::shader_buffer_float64_atomic_add::NAME)
                    {
                        Err(
                            "`OpAtomicFAdd` used on buffer memory, but the `vk_ext::shader_atomic_float::buffer_f64_atomic_add` feature is not enabled"
                        )?
                    }
                }
            }
            Ok(())
        }),
        cap!(Capability::INT_16, vk_10::shader_int16),
        cap!(Capability::TESSELLATION_POINT_SIZE, vk_10::shader_tessellation_and_geometry_point_size),
        cap!(Capability::GEOMETRY_POINT_SIZE, vk_10::shader_tessellation_and_geometry_point_size),
        cap!(Capability::IMAGE_GATHER_EXTENDED, vk_10::shader_image_gather_extended),
        cap!(Capability::STORAGE_IMAGE_MULTISAMPLE, vk_10::shader_storage_image_multisample),
        (Capability::UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::UNIFORM_BUFFER_ARRAY_DYNAMIC_INDEXING)
            {
                Err(
                    "the `vk_10::shader_uniform_buffer_array_dynamic_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::SAMPLED_IMAGE_ARRAY_DYNAMIC_INDEXING)
            {
                Err(
                    "the `vk_10::shader_sampled_image_array_dynamic_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::STORAGE_BUFFER_ARRAY_DYNAMIC_INDEXING)
            {
                Err(
                    "the `vk_10::shader_storage_buffer_array_dynamic_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::STORAGE_IMAGE_ARRAY_DYNAMIC_INDEXING)
            {
                Err(
                    "the `vk_10::shader_storage_image_array_dynamic_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        cap!(Capability::CLIP_DISTANCE, vk_10::shader_clip_distance),
        cap!(Capability::CULL_DISTANCE, vk_10::shader_cull_distance),
        cap!(Capability::IMAGE_CUBE_ARRAY, vk_10::image_cube_array),
        cap!(Capability::SAMPLE_RATE_SHADING, vk_10::sample_rate_shading),
        cap!(Capability::SPARSE_RESIDENCY, vk_10::shader_resource_residency),
        cap!(Capability::MIN_LOD, vk_10::shader_resource_min_lod),
        cap!(Capability::SAMPLED_CUBE_ARRAY, vk_10::image_cube_array),
        cap!(Capability::IMAGE_MSARRAY, vk_10::shader_storage_image_multisample),
        (Capability::STORAGE_IMAGE_EXTENDED_FORMATS, |_, _| { Ok(()) }),
        cap!(Capability::INTERPOLATION_FUNCTION, vk_10::sample_rate_shading),
        cap!(Capability::STORAGE_IMAGE_READ_WITHOUT_FORMAT, vk_10::shader_storage_image_read_without_format),
        cap!(Capability::STORAGE_IMAGE_WRITE_WITHOUT_FORMAT, vk_10::shader_storage_image_write_without_format),
        cap!(Capability::MULTI_VIEWPORT, vk_10::multi_viewport),
        cap!(Capability::DRAW_PARAMETERS, vk_11::shader_draw_parameters),
        cap!(Capability::MULTI_VIEW, vk_11::multiview),
        cap!(Capability::VARIABLE_POINTERS_STORAGE_BUFFER, vk_11::variable_pointers_storage_buffer),
        // always supported on Vulkan 1.1
        (Capability::SUBGROUP_BALLOT_KHR, |_, _| Ok(()) ),
        (Capability::SUBGROUP_VOTE_KHR, |_, _| Ok(()) ),
        cap!(Capability::SHADER_VIEWPORT_INDEX, vk_12::shader_output_viewport_index),
        cap!(Capability::SHADER_LAYER, vk_12::shader_output_layer),
        cap!(Capability::STORAGE_BUFFER_16_BIT_ACCESS, vk_11::storage_buffer_16_bit_access),
        cap!(Capability::UNIFORM_AND_STORAGE_BUFFER_16_BIT_ACCESS, vk_11::uniform_and_storage_buffer_16_bit_access),
        cap!(Capability::STORAGE_PUSH_CONSTANT_16, vk_11::storage_push_constant_16),
        cap!(Capability::STORAGE_INPUT_OUTPUT_16, vk_11::storage_input_output_16),
        (Capability::GROUP_NON_UNIFORM, |device, _| {
            if !device
                .gpu_info()
                .limits
                .supported_subgroup_operations()
                .contains(SubgroupFeatures::BASIC)
            {
                Err(
                    "`SubgroupFeatures::BASIC` feature is not supported"
                )?
            }
            Ok(())
        }),
        (Capability::GROUP_NON_UNIFORM_VOTE, |device, _| {
            if !device
                .gpu_info()
                .limits
                .supported_subgroup_operations()
                .contains(SubgroupFeatures::VOTE)
            {
                Err(
                    "`SubgroupFeatures::VOTE` feature is not supported"
                )?
            }
            Ok(())
        }),
        (Capability::GROUP_NON_UNIFORM_ARITHMETIC, |device, _| {
            if !device
                .gpu_info()
                .limits
                .supported_subgroup_operations()
                .contains(SubgroupFeatures::ARITHMETIC)
            {
                Err(
                    "`SubgroupFeatures::ARITHMETIC` feature is not supported"
                )?
            }
            Ok(())
        }),
        (Capability::GROUP_NON_UNIFORM_BALLOT, |device, _| {
            if !device
                .gpu_info()
                .limits
                .supported_subgroup_operations()
                .contains(SubgroupFeatures::BALLOT)
            {
                Err(
                    "`SubgroupFeatures::BALLOT` feature is not supported"
                )?
            }
            Ok(())
        }),
        (Capability::GROUP_NON_UNIFORM_SHUFFLE, |device, _| {
            if !device
                .gpu_info()
                .limits
                .supported_subgroup_operations()
                .contains(SubgroupFeatures::SHUFFLE)
            {
                Err(
                    "`SubgroupFeatures::SHUFFLE` feature is not supported"
                )?
            }
            Ok(())
        }),
        (Capability::GROUP_NON_UNIFORM_SHUFFLE_RELATIVE, |device, _| {
            if !device
                .gpu_info()
                .limits
                .supported_subgroup_operations()
                .contains(SubgroupFeatures::SHUFFLE_RELATIVE)
            {
                Err(
                    "`SubgroupFeatures::SHUFFLE_RELATIVE` feature is not supported"
                )?
            }
            Ok(())
        }),
        (Capability::GROUP_NON_UNIFORM_CLUSTERED, |device, _| {
            if !device
                .gpu_info()
                .limits
                .supported_subgroup_operations()
                .contains(SubgroupFeatures::CLUSTERED)
            {
                Err(
                    "`SubgroupFeatures::CLUSTERED` feature is not supported"
                )?
            }
            Ok(())
        }),
        (Capability::GROUP_NON_UNIFORM_QUAD, |device, _| {
            if !device
                .gpu_info()
                .limits
                .supported_subgroup_operations()
                .contains(SubgroupFeatures::QUAD)
            {
                Err(
                    "`SubgroupFeatures::QUAD` feature is not supported"
                )?
            }
            Ok(())
        }),
        cap!(Capability::GROUP_NON_UNIFORM_ROTATE_KHR, vk_14::shader_subgroup_rotate),
        cap!(Capability::SHADER_NON_UNIFORM, vk_14::shader_subgroup_rotate),
        (Capability::SHADER_NON_UNIFORM, |_, _| Ok(())),
        (Capability::RUNTIME_DESCRIPTOR_ARRAY, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::RUNTIME_DESCRIPTOR_ARRAY) {
                Err(
                    "the `vk_12::runtime_descriptor_array` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::INPUT_ATTACHMENT_ARRAY_DYNAMIC_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::INPUT_ATTACHMENT_ARRAY_DYNAMIC_INDEXING) {
                Err(
                    "the `vk_12::shader_input_attachment_array_dynamic_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::UNIFORM_TEXEL_BUFFER_ARRAY_DYNAMIC_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::UNIFORM_TEXEL_BUFFER_ARRAY_DYNAMIC_INDEXING) {
                Err(
                    "the `vk_12::shader_uniform_texel_buffer_array_dynamic_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::STORAGE_TEXEL_BUFFER_ARRAY_DYNAMIC_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::STORAGE_TEXEL_BUFFER_ARRAY_DYNAMIC_INDEXING) {
                Err(
                    "the `vk_12::shader_storage_texel_buffer_array_dynamic_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::UNIFORM_BUFFER_ARRAY_NON_UNIFORM_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::UNIFORM_BUFFER_ARRAY_NON_UNIFORM_INDEXING) {
                Err(
                    "the `vk_12::shader_uniform_buffer_array_non_uniform_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::SAMPLED_IMAGE_ARRAY_NON_UNIFORM_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::SAMPLED_IMAGE_ARRAY_NON_UNIFORM_INDEXING) {
                Err(
                    "the `vk_12::shader_sampled_image_array_non_uniform_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING) {
                Err(
                    "the `vk_12::shader_storage_buffer_array_non_uniform_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::STORAGE_IMAGE_ARRAY_NON_UNIFORM_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::STORAGE_IMAGE_ARRAY_NON_UNIFORM_INDEXING) {
                Err(
                    "the `vk_12::shader_storage_image_array_non_uniform_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::INPUT_ATTACHMENT_ARRAY_NON_UNIFORM_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::INPUT_ATTACHMENT_ARRAY_NON_UNIFORM_INDEXING) {
                Err(
                    "the `vk_12::shader_input_attachment_array_non_uniform_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::UNIFORM_TEXEL_BUFFER_ARRAY_NON_UNIFORM_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::UNIFORM_TEXEL_BUFFER_ARRAY_NON_UNIFORM_INDEXING) {
                Err(
                    "the `vk_12::shader_uniform_texel_buffer_array_non_uniform_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        (Capability::STORAGE_TEXEL_BUFFER_ARRAY_NON_UNIFORM_INDEXING, |device, _| {
            if !device.are_flags32_supported(DescriptorIndexingFeatures::STORAGE_TEXEL_BUFFER_ARRAY_NON_UNIFORM_INDEXING) {
                Err(
                    "the `vk_12::shader_storage_texel_buffer_array_non_uniform_indexing` feature is not enabled"
                )?
            }
            Ok(())
        }),
        cap!(Capability::FLOAT_16, vk_12::shader_float16),
        cap!(Capability::INT_8, vk_12::shader_int8),
        (Capability::STORAGE_BUFFER_8_BIT_ACCESS, |device, _| {
            if !device.is_feature_enabled(vk_12::storage_buffer_8_bit_access::NAME)
                && !device.is_feature_enabled(vk_12::uniform_and_storage_buffer_8_bit_access::NAME)
            {
                Err(
                    "the `vk_12::storage_buffer_8_bit_access` feature is not enabled"
                )?
            }
            Ok(())
        }),
        cap!(Capability::UNIFORM_AND_STORAGE_BUFFER_8_BIT_ACCESS, vk_12::uniform_and_storage_buffer_8_bit_access),
        cap!(Capability::STORAGE_PUSH_CONSTANT_8, vk_12::storage_push_constant_8),
        cap!(Capability::VULKAN_MEMORY_MODEL, vk_12::vulkan_memory_model),
        cap!(Capability::VULKAN_MEMORY_MODEL_DEVICE_SCOPE, vk_12::vulkan_memory_model_device_scope),
        cap!(Capability::PHYSICAL_STORAGE_BUFFER_ADDRESSES, vk_12::buffer_device_address),
        cap!(Capability::DEMOTE_TO_HELPER_INVOCATION, vk_13::shader_demote_to_helper_invocation),
        cap!(Capability::DEMOTE_TO_HELPER_INVOCATION, vk_13::shader_demote_to_helper_invocation),
        cap!(Capability::DOT_PRODUCT_INPUT_ALL, vk_13::shader_integer_dot_product),
        cap!(Capability::DOT_PRODUCT_INPUT_4X_8_BIT, vk_13::shader_integer_dot_product),
        cap!(Capability::DOT_PRODUCT_INPUT_4X_8_BIT_PACKED, vk_13::shader_integer_dot_product),
        cap!(Capability::DOT_PRODUCT, vk_13::shader_integer_dot_product),
        cap!(Capability::MESH_SHADING_EXT, vk_ext::mesh_shader),
        cap!(Capability::EXPECT_ASSUME_KHR, vk_14::shader_expect_assume),
        cap!(Capability::FLOAT_CONTROLS_2, vk_14::shader_float_controls2),
        (Capability::DESCRIPTOR_HEAP_EXT, |_, _| {
            Err(
                "descriptor heaps are not yet supported",
            )?
        }),
    ]
);
