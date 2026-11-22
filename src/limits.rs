use core::ops::Range;

use tuhka::vk;

use crate::*;

/// Limits of a GPU.
#[derive(Clone, Copy)]
pub struct GpuLimits {
    pub(crate) vk_10: vk::PhysicalDeviceLimits,
    pub(crate) depth_stencil_resolve_limits: GpuDepthStencilResolveLimits,
    pub(crate) max_buffer_size: DeviceSize,
    pub(crate) max_multiview_count: u32,
    pub(crate) supported_subgroup_operations: SubgroupFeatures,
    pub(crate) max_inline_uniform_total_size: Option<u32>,
}
impl GpuLimits {

    /// The maximum size a [`Buffer`] **can** be.
    #[inline]
    pub fn max_buffer_size(&self) -> DeviceSize {
        self.max_buffer_size
    }

    /// Returns the maximum total size of inline uniform blocks in a shader set.
    ///
    /// If the Vulkan version of the device is less than 1.3, this will return `None`.
    #[inline]
    pub fn max_inline_uniform_total_size(&self) -> Option<u32> {
        self.max_inline_uniform_total_size
    }

    /// This value - 1 specifies the maximum index that the most significant bit of [`RenderLayers::Mask`] **can**
    /// have.
    #[inline]
    pub fn max_multiview_count(&self) -> u32 {
        self.max_multiview_count
    }

    /// Returns [`GpuDepthStencilResolveLimits`] specifying limits on depth and stencil attachment
    /// [resolve modes].
    ///
    /// [resolve modes]: ResolveMode
    #[inline]
    pub fn depth_stencil_resolve_limits(&self) -> &GpuDepthStencilResolveLimits {
        &self.depth_stencil_resolve_limits
    }

    /// Returns a bitmask of [supported subgroup operations].
    ///
    /// [supported subgroup operations]: SubgroupFeatures
    #[inline]
    pub fn supported_subgroup_operations(&self) -> SubgroupFeatures {
        self.supported_subgroup_operations
    }

    /// Largest width guaranteed to be supported by 1D images.
    #[inline]
    pub fn max_image_dimension_1d(&self) -> u32 {
        self.vk_10.max_image_dimension1_d
    }

    /// Largest width or height guaranteed to be supported 2D images.
    #[inline]
    pub fn max_image_dimension_2d(&self) -> u32 {
        self.vk_10.max_image_dimension2_d
    }

    /// Largest width, height or depth guaranteed to be supported 3D images.
    #[inline]
    pub fn max_image_dimension_3d(&self) -> u32 {
        self.vk_10.max_image_dimension3_d
    }

    /// Largest wdith or height guaranteed to be supported by 2D images that are
    /// cube map compatible.
    #[inline]
    pub fn max_image_dimension_cube(&self) -> u32 {
        self.vk_10.max_image_dimension_cube
    }

    /// The maximum number of supported layers for an image.
    #[inline]
    pub fn max_image_array_layers(&self) -> u32 {
        self.vk_10.max_image_array_layers
    }

    /// The maximum number of addressable texels for a buffer, which was created with
    /// [`UNIFORM_TEXEL_BUFFER`][1] or [`STORAGE_TEXEL_BUFFER`][2] usages set.
    ///
    /// [1]: BufferUsages::UNIFORM_TEXEL_BUFFER
    /// [2]: BufferUsages::STORAGE_TEXEL_BUFFER
    #[inline]
    pub fn max_texel_buffer_elements(&self) -> u32 {
        self.vk_10.max_texel_buffer_elements
    }

    /// The maximum value that **can** be specified for the `range` member of
    /// `DescriptorBuffer` when descriptor type is [`uniform buffer`][1].
    ///
    /// [1]: DescriptorType::UniformBuffer
    #[inline]
    pub fn max_uniform_buffer_range(&self) -> u32 {
        self.vk_10.max_uniform_buffer_range
    }

    /// The maximum value that **can** be specified for the `range` member of
    /// `DescriptorBuffer` when descriptor type is [`storage buffer`][1].
    ///
    /// [1]: DescriptorType::StorageBuffer
    #[inline]
    pub fn max_storage_buffer_range(&self) -> u32 {
        self.vk_10.max_storage_buffer_range
    }
   
    /// The maximum size, in bytes, of the pool of push constant memory.
    ///
    /// The offsets + sizes of push constants in shaders **must** be less than or equal to this
    /// limit.
    #[inline]
    pub fn max_push_constants_size(&self) -> u32 {
        self.vk_10.max_push_constants_size
    }
   
    /// This is the maximum number of device memory allocations, which **can**
    /// simultaneously exist on a [`Device`].
    #[inline]
    pub fn max_memory_allocation_count(&self) -> u32 {
        self.vk_10.max_memory_allocation_count
    }
   
    /// This is the maximum number of `samplers`, which **can** simultaneously exist on a
    /// [`Device`].
    #[inline]
    pub fn max_sampler_allocation_count(&self) -> u32 {
        self.vk_10.max_sampler_allocation_count
    }
   
    /// The granularity, in bytes, at which buffer or linear image resources, linear or optimal
    /// tensor resources, and optimal image resources can be bound to adjacent offsets in the same
    /// `DeviceMemory` object without aliasing.
    #[inline]
    pub fn buffer_image_granularity(&self) -> DeviceSize {
        self.vk_10.buffer_image_granularity
    }
   
    /// The total amount of address space available, in bytes, for sparse memory resources.
    #[inline]
    pub fn sparse_address_space_size(&self) -> DeviceSize {
        self.vk_10.sparse_address_space_size
    }
   
    /// The maximum number of descriptor sets that **can** be simultaneously used by a pipeline.
    #[inline]
    pub fn max_bound_descriptor_sets(&self) -> u32 {
        self.vk_10.max_bound_descriptor_sets
    }
   
    /// The maximum number of [samplers] and [combined image samplers] that **can** be accessible to a
    /// single shader stage in a set of shaders.
    ///
    /// [samplers]: DescriptorType::Sampler
    /// [combined image samplers]: DescriptorType::CombinedImageSampler
    #[inline]
    pub fn max_per_stage_descriptor_samplers(&self) -> u32 {
        self.vk_10.max_per_stage_descriptor_samplers
    }
    
    /// The maximum number of uniform buffers that **can** be accessible to a single shader stage in a
    /// set of shaders.
    #[inline]
    pub fn max_per_stage_descriptor_uniform_buffers(&self) -> u32 {
        self.vk_10.max_per_stage_descriptor_uniform_buffers
    }

    /// The maximum number of storage buffers that **can** be accessible to a single shader stage in a
    /// set of shaders.
    #[inline]
    pub fn max_per_stage_descriptor_storage_buffers(&self) -> u32 {
        self.vk_10.max_per_stage_descriptor_storage_buffers
    }

    /// The maximum number of sampled images that **can** be accessible to a single shader stage in a
    /// set of shaders.
    #[inline]
    pub fn max_per_stage_descriptor_sampled_images(&self) -> u32 {
        self.vk_10.max_per_stage_descriptor_sampled_images
    }

    /// The maximum number of storage images that **can** be accessible to a single shader stage in a
    /// set of shaders.
    #[inline]
    pub fn max_per_stage_descriptor_storage_images(&self) -> u32 {
        self.vk_10.max_per_stage_descriptor_storage_images
    }

    /// The maximum number of input attachments that **can** be accessible to a single shader stage in 
    /// a set of shaders.
    #[inline]
    pub fn max_per_stage_descriptor_input_attachments(&self) -> u32 {
        self.vk_10.max_per_stage_descriptor_input_attachments
    }
    
    /// The maximum number of resources that **can** be accessible to a single shader stage in a
    /// set of shaders.
    #[inline]
    pub fn max_per_stage_resources(&self) -> u32 {
        self.vk_10.max_per_stage_resources
    }
   
    /// The maximum number of samplers that **can** be included in a set of shaders.
    #[inline]
    pub fn max_descriptor_set_samplers(&self) -> u32 {
        self.vk_10.max_descriptor_set_samplers
    }

    /// The maximum number of uniform buffers that **can** be included in a set of shaders.
    #[inline]
    pub fn max_descriptor_set_uniform_buffers(&self) -> u32 {
        self.vk_10.max_descriptor_set_uniform_buffers
    }

    /// The maximum number of dynamic uniform buffers that **can** be included in a set of shaders.
    #[inline]
    pub fn max_descriptor_set_uniform_buffers_dynamic(&self) -> u32 {
        self.vk_10.max_descriptor_set_uniform_buffers_dynamic
    }

    /// The maximum number of storage buffers that **can** be included in a set of shaders.
    #[inline]
    pub fn max_descriptor_set_storage_buffers(&self) -> u32 {
        self.vk_10.max_descriptor_set_storage_buffers
    }

    /// The maximum number of dynamic storage buffers that **can** be included in a
    /// set of shaders.
    #[inline]
    pub fn max_descriptor_set_storage_buffers_dynamic(&self) -> u32 {
        self.vk_10.max_descriptor_set_storage_buffers_dynamic
    }

    /// The maximum number of sampled images that **can** be included in a set of shaders.
    #[inline]
    pub fn max_descriptor_set_sampled_images(&self) -> u32 {
        self.vk_10.max_descriptor_set_sampled_images
    }

    /// The maximum number of storage images that **can** be included in a set of shaders.
    #[inline]
    pub fn max_descriptor_set_storage_images(&self) -> u32 {
        self.vk_10.max_descriptor_set_storage_images
    }

    /// The maximum number of input attachments that **can** be included in a set of shaders.
    #[inline]
    pub fn max_descriptor_set_input_attachments(&self) -> u32 {
        self.vk_10.max_descriptor_set_input_attachments
    }
   
    /// The maximum number of vertex input attributes that **can** be contained in a 
    /// set of [vertex input attributes][VertexInputAttributeDescription].
    #[inline]
    pub fn max_vertex_input_attributes(&self) -> u32 {
        self.vk_10.max_vertex_input_attributes
    }
   

    /// The maximum number of vertex input bindings that **can** be contained in a 
    /// set of [vertex input bindings][VertexInputBindingDescription].
    #[inline]
    pub fn max_vertex_input_bindings(&self) -> u32 {
        self.vk_10.max_vertex_input_bindings
    }

    /// The maximum vertex input attribute offset that **can** be added to the vertex input binding
    /// stride.
    ///
    /// The [`offset`][1] of [`VertexInputAttributeDescription`] **must** be less than or equal to this limit.
    ///
    /// [1]: VertexInputAttributeDescription::offset
    #[inline]
    pub fn max_vertex_input_attribute_offset(&self) -> u32 {
        self.vk_10.max_vertex_input_attribute_offset
    }

    /// The maximum vertex input binding stride that **can** be specified in a vertex input binding.
    ///
    /// The [`stride`][1] of a [`VertexInputBindingDescription`] **must** be less than or equal to
    /// this limit.
    ///
    /// [1]: VertexInputBindingDescription::stride   
    #[inline]
    pub fn max_vertex_input_binding_stride(&self) -> u32 {
        self.vk_10.max_vertex_input_binding_stride
    }
   
    /// The maximum number of components of output variables which can be output by a vertex
    /// shader.
    #[inline]
    pub fn max_vertex_output_components(&self) -> u32 {
        self.vk_10.max_vertex_output_components
    }
   
    /// The maximum tessellation generation level supported by the fixed-function tessellation
    /// primitive generator.
    #[inline]
    pub fn max_tessellation_generation_level(&self) -> u32 {
        self.vk_10.max_tessellation_generation_level
    }
   
    /// The maximum patch size, in vertices, of patches that can be processed by the tessellation
    /// control shader and tessellation primitive generator.
    #[inline]
    pub fn max_tessellation_patch_size(&self) -> u32 {
        self.vk_10.max_tessellation_patch_size
    }

    /// The maximum number of components of input variables which can be provided as per-vertex
    /// inputs to the tessellation control shader stage.
    #[inline]
    pub fn max_tessellation_control_per_vertex_input_components(&self) -> u32 {
        self.vk_10.max_tessellation_control_per_vertex_input_components
    }
   
    /// The maximum number of components of per-vertex output variables which can be output from
    /// the tessellation control shader stage.
    #[inline]
    pub fn max_tessellation_control_per_vertex_output_components(&self) -> u32 {
        self.vk_10.max_tessellation_control_per_vertex_output_components
    }
    
    /// The maximum number of components of per-patch output variables which can be output from the
    /// tessellation control shader stage.
    #[inline]
    pub fn max_tessellation_control_per_patch_output_components(&self) -> u32 {
        self.vk_10.max_tessellation_control_per_patch_output_components
    }
   
    /// The maximum total number of components of per-vertex and per-patch output variables which
    /// can be output from the tessellation control shader stage.
    #[inline]
    pub fn max_tessellation_control_total_output_components(&self) -> u32 {
        self.vk_10.max_tessellation_control_total_output_components
    }
   
    /// The maximum number of components of input variables which can be provided as per-vertex
    /// inputs to the tessellation evaluation shader stage.
    #[inline]
    pub fn max_tessellation_evaluation_input_components(&self) -> u32 {
        self.vk_10.max_tessellation_evaluation_input_components
    }
   
    /// The maximum number of components of per-vertex output variables which can be output from the
    /// tessellation evaluation shader stage.
    #[inline]
    pub fn max_tessellation_evaluation_output_components(&self) -> u32 {
        self.vk_10.max_tessellation_evaluation_output_components
    }
   
    /// The maximum number of components of input variables which can be provided as inputs to the
    /// fragment shader stage.
    #[inline]
    pub fn max_fragment_input_components(&self) -> u32 {
        self.vk_10.max_fragment_input_components
    }
   
    /// The maximum number of output attachments which can be written to by the fragment shader stage.
    #[inline]
    pub fn max_fragment_output_attachments(&self) -> u32 {
        self.vk_10.max_fragment_output_attachments
    }
   
    /// The maximum number of output attachments which can be written to by the fragment shader
    /// stage when blending is enabled and one of the dual source blend modes is in use.
    #[inline]
    pub fn max_fragment_dual_src_attachments(&self) -> u32 {
        self.vk_10.max_fragment_dual_src_attachments
    }
   
    /// The total number of storage buffers, storage images, and output Location decorated color
    /// attachments which can be used in the fragment shader stage.
    #[inline]
    pub fn max_fragment_combined_output_resources(&self) -> u32 {
        self.vk_10.max_fragment_combined_output_resources
    }
   
    /// The maximum total storage size, in bytes, available for variables declared with the
    /// Workgroup storage class in shader modules (or with the shared storage qualifier in GLSL) in
    /// the compute shader stage.
    #[inline]
    pub fn max_compute_shared_memory_size(&self) -> u32 {
        self.vk_10.max_compute_shared_memory_size
    }
    
    /// The maximum number of local workgroups that can be dispatched by a single dispatching
    /// command.
    #[inline]
    pub fn max_compute_work_group_count(&self) -> [u32; 3usize] {
        self.vk_10.max_compute_work_group_count
    }
    
    /// The maximum total number of compute shader invocations in a single local workgroup.
    #[inline]
    pub fn max_compute_work_group_invocations(&self) -> u32 {
        self.vk_10.max_compute_work_group_invocations
    }
   
    /// The maximum size of a local compute workgroup, per dimension.
    #[inline]
    pub fn max_compute_work_group_size(&self) -> [u32; 3usize] {
        self.vk_10.max_compute_work_group_size
    }
   
    /// The number of bits of subpixel precision in framebuffer coordinates.
    #[inline]
    pub fn sub_pixel_precision_bits(&self) -> u32 {
        self.vk_10.sub_pixel_precision_bits
    }
   
    /// The number of bits of precision in the division along an axis of an image used for
    /// minification and magnification filter.
    #[inline]
    pub fn sub_texel_precision_bits(&self) -> u32 {
        self.vk_10.sub_texel_precision_bits
    }
   
    /// The number of bits of division that the LOD calculation for mipmap fetching get snapped to
    /// when determining the contribution from each mip level to the mip filtered results.
    #[inline]
    pub fn mipmap_precision_bits(&self) -> u32 {
        self.vk_10.mipmap_precision_bits
    }
   
    /// The maximum index value that can be used for indexed draw calls when using 32-bit indices.
    #[inline]
    pub fn max_draw_indexed_index_value(&self) -> u32 {
        self.vk_10.max_draw_indexed_index_value
    }
   
    /// The maximum absolute [`sampler LOD bias`][1].
    ///
    /// [1]: SamplerAttributes::mip_lod_bias
    #[inline]
    pub fn max_sampler_lod_bias(&self) -> f32 {
        self.vk_10.max_sampler_lod_bias
    }
   
    /// The maximum degree of anisotropic filtering used for an image sampling operation is the
    /// minimum of the [`max_anisotropy`][1] member of the [`SamplerAttributes`] and this limit.
    ///
    /// [1]: SamplerAttributes::max_anisotropy
    #[inline]
    pub fn max_sampler_anisotropy(&self) -> f32 {
        self.vk_10.max_sampler_anisotropy
    }
   
    /// The maximum number of active viewports.
    #[inline]
    pub fn max_viewports(&self) -> u32 {
        self.vk_10.max_viewports
    }
   
    /// The maximum viewport dimensions in the width and height dimensions, respectively.
    #[inline]
    pub fn max_viewport_dimensions(&self) -> [u32; 2usize] {
        self.vk_10.max_viewport_dimensions
    }
   
    /// The minimum..maximum range that the corners of a viewport must be contained in.
    #[inline]
    pub fn viewport_bounds_range(&self) -> [f32; 2usize] {
        self.vk_10.viewport_bounds_range
    }
   
    /// The number of bits of subpixel precision for viewport bounds.
    #[inline]
    pub fn viewport_sub_pixel_bits(&self) -> u32 {
        self.vk_10.viewport_sub_pixel_bits
    }
   
    /// The minimum required alignment, in bytes, of host visible memory allocations within the
    /// host address space.
    #[inline]
    pub fn min_memory_map_alignment(&self) -> usize {
        self.vk_10.min_memory_map_alignment
    }
 
    /// The minimum **required** alignment of the `offset` member of [`BufferViewAttributes`] for
    /// texel buffers.
    #[inline]
    pub fn min_texel_buffer_offset_alignment(&self) -> DeviceSize {
        self.vk_10.min_texel_buffer_offset_alignment
    }
    
    /// The minimum required alignment, in bytes, for the `offset` member of the
    /// `DescriptorBuffer` for [`uniform buffers`][1].
    ///
    /// [1]: DescriptorType::UniformBuffer
    #[inline]
    pub fn min_uniform_buffer_offset_alignment(&self) -> DeviceSize {
        self.vk_10.min_uniform_buffer_offset_alignment
    }

    /// The minimum required alignment, in bytes, for the `offset` member of the
    /// `DescriptorBuffer` for [`storage buffers`][1].
    ///
    /// [1]: DescriptorType::StorageBuffer
    #[inline]
    pub fn min_storage_buffer_offset_alignment(&self) -> DeviceSize {
        self.vk_10.min_storage_buffer_offset_alignment
    }
   
    /// The minimum offset value for the `Offset` or `ConstOffset` image operand of any of the
    /// `OpImageSample*` or `OpImageFetch*` image instructions.
    #[inline]
    pub fn min_texel_offset(&self) -> i32 {
        self.vk_10.min_texel_offset
    }
   
    /// The maximum offset value for the `Offset` or `ConstOffset` image operand of any of the
    /// `OpImageSample*` or `OpImageFetch*` image instructions.
    #[inline]
    pub fn max_texel_offset(&self) -> u32 {
        self.vk_10.max_texel_offset
    }
   
    /// The minimum offset value for the `Offset`, `ConstOffset`, or `ConstOffsets` image operands of any
    /// of the `OpImage*Gather` image instructions.
    #[inline]
    pub fn min_texel_gather_offset(&self) -> i32 {
        self.vk_10.min_texel_gather_offset
    }
   
    /// The maximum offset value for the `Offset`, `ConstOffset`, or `ConstOffsets` image operands of any
    /// of the `OpImage*Gather` image instructions.
    #[inline]
    pub fn max_texel_gather_offset(&self) -> u32 {
        self.vk_10.max_texel_gather_offset
    }
   
    /// The base minimum (inclusive) negative offset value for the Offset operand of the
    /// `InterpolateAtOffset` extended instruction.
    #[inline]
    pub fn min_interpolation_offset(&self) -> f32 {
        self.vk_10.min_interpolation_offset
    }
   
    /// The base maximum (inclusive) positive offset value for the Offset operand of the
    /// `InterpolateAtOffset` extended instruction.
    #[inline]
    pub fn max_interpolation_offset(&self) -> f32 {
        self.vk_10.max_interpolation_offset
    }
   
    /// The number of fractional bits that the x and y offsets to the `InterpolateAtOffset` extended
    /// instruction may be rounded to as fixed-point values.
    #[inline]
    pub fn sub_pixel_interpolation_offset_bits(&self) -> u32 {
        self.vk_10.sub_pixel_interpolation_offset_bits
    }
   
    /// The maximum width for a framebuffer.
    #[inline]
    pub fn max_framebuffer_width(&self) -> u32 {
        self.vk_10.max_framebuffer_width
    }
    
    /// The maximum height for a framebuffer.
    #[inline]
    pub fn max_framebuffer_height(&self) -> u32 {
        self.vk_10.max_framebuffer_height
    }
   
    /// The maximum layer count for a layered framebuffer.
    #[inline]
    pub fn max_framebuffer_layers(&self) -> u32 {
        self.vk_10.max_framebuffer_layers
    }
   
    /// A bitmask of [`sample counts`][1] indicating the color sample counts that are supported
    /// for all framebuffer color attachments with floating- or fixed-point formats.
    ///
    /// [1]: SampleCounts
    #[inline]
    pub fn framebuffer_color_samples(&self) -> SampleCounts {
        SampleCounts::from_bits_truncate(self.vk_10.framebuffer_color_sample_counts.as_raw())
    }

    /// A bitmask of [`sample counts`][1] indicating the supported depth sample counts for all
    /// framebuffer depth/stencil attachments, when the format includes a depth component.
    ///
    /// [1]: SampleCounts
    #[inline]
    pub fn framebuffer_depth_samples(&self) -> SampleCounts {
        SampleCounts::from_bits_truncate(self.vk_10.framebuffer_depth_sample_counts.as_raw())
    }
   
    /// A bitmask of [`sample counts`][1] indicating the supported stencil sample counts for all
    /// framebuffer depth/stencil attachments, when the format includes a stencil component.
    ///
    /// [1]: SampleCounts
    #[inline]
    pub fn framebuffer_stencil_samples(&self) -> SampleCounts {
        SampleCounts::from_bits_truncate(self.vk_10.framebuffer_stencil_sample_counts.as_raw())
    }
  
    /* unused for now
    /// A bitmask of [`sample counts`][1] indicating the supported sample counts for a subpass
    /// which uses no attachments.
    ///
    /// [1]: SampleCounts
    #[inline]
    pub fn framebuffer_no_attachments_sample_counts(&self) -> SampleCounts {
        SampleCounts::from_bits_truncate(self.vk_10.framebuffer_no_attachments_sample_counts.as_raw())
    }
    */
   
    /// The maximum number of color attachments that can be used in a [render pass]. 
    ///
    /// [render pass]: CommandBufferRecorder::render_pass
    #[inline]
    pub fn max_color_attachments(&self) -> u32 {
        self.vk_10.max_color_attachments
    }
   
    /// A bitmask of [`sample counts`][1] indicating the sample counts supported for all 2D images
    /// created with the [`SAMPLED`][2] usage and a [`non-integer`][3] [`color format`][4].
    ///
    /// [1]: SampleCounts
    /// [2]: ImageUsages::SAMPLED
    /// [3]: NumericFormat::is_integer
    /// [4]: Format::numeric_format_color
    #[inline]
    pub fn sampled_image_color_sample_counts(&self) -> SampleCounts {
        SampleCounts::from_bits_truncate(self.vk_10.sampled_image_color_sample_counts.as_raw())
    }
   
    /// a bitmask of [`sample counts`][1] indicating the sample counts supported for all 2D images
    /// created with the [`SAMPLED`][2] usage and an [`integer`][3] [`color format`][4].
    ///
    /// [1]: SampleCounts
    /// [2]: ImageUsages::SAMPLED
    /// [3]: NumericFormat::is_integer
    /// [4]: Format::numeric_format_color
    #[inline]
    pub fn sampled_image_integer_sample_counts(&self) -> SampleCounts {
        SampleCounts::from_bits_truncate(self.vk_10.sampled_image_integer_sample_counts.as_raw())
    }
   
    /// A bitmask of [`sample counts`][1] indicating the sample counts supported for all 2D images
    /// created with the [`SAMPLED`][2] usage and a [`depth format`][3]
    ///
    /// [1]: SampleCounts
    /// [2]: ImageUsages::SAMPLED
    /// [3]: Format::numeric_format_depth
    #[inline]
    pub fn sampled_image_depth_sample_counts(&self) -> SampleCounts {
        SampleCounts::from_bits_truncate(self.vk_10.sampled_image_depth_sample_counts.as_raw())
    }

    /// A bitmask of [`sample counts`][1] indicating the sample counts supported for all 2D images
    /// created with the [`SAMPLED`][2] usage and a [`stencil format`][3]
    ///
    /// [1]: SampleCounts
    /// [2]: ImageUsages::SAMPLED
    /// [3]: Format::numeric_format_stencil
    #[inline]
    pub fn sampled_image_stencil_sample_counts(&self) -> SampleCounts {
        SampleCounts::from_bits_truncate(self.vk_10.sampled_image_stencil_sample_counts.as_raw())
    }

    /// A bitmask of [`sample counts`][1] indicating the sample counts supported for all 2D images
    /// created with the [`STORAGE`][2] usage.
    ///
    /// [1]: SampleCounts
    /// [2]: ImageUsages::STORAGE
    #[inline]
    pub fn storage_image_sample_counts(&self) -> SampleCounts {
        SampleCounts::from_bits_truncate(self.vk_10.storage_image_sample_counts.as_raw())
    }

    /// The maximum number of array elements of a variable decorated with the SampleMask built-in
    /// decoration.
    #[inline]
    pub fn max_sample_mask_words(&self) -> u32 {
        self.vk_10.max_sample_mask_words
    }

    /// Specifies support for timestamps on all graphics and compute queues.
    #[inline]
    pub fn timestamp_compute_and_graphics(&self) -> bool {
        self.vk_10.timestamp_compute_and_graphics != 0
    }

    /// The number of nanoseconds required for a timestamp query to be incremented by 1.
    #[inline]
    pub fn timestamp_period(&self) -> f32 {
        self.vk_10.timestamp_period
    }

    /// The maximum number of clip distances that can be used in a single shader stage.
    #[inline]
    pub fn max_clip_distances(&self) -> u32 {
        self.vk_10.max_clip_distances
    }

    /// The maximum number of cull distances that can be used in a single shader stage.
    #[inline]
    pub fn max_cull_distances(&self) -> u32 {
        self.vk_10.max_cull_distances
    }

    /// The maximum combined number of clip and cull distances that can be used in a single shader
    /// stage.
    #[inline]
    pub fn max_combined_clip_and_cull_distances(&self) -> u32 {
        self.vk_10.max_combined_clip_and_cull_distances
    }

    /// The maximum number of discrete queue priorities in [`DeviceQueueAttributes`].
    #[inline]
    pub fn discrete_queue_priorities(&self) -> u32 {
        self.vk_10.discrete_queue_priorities
    }

    /// The range of supported sizes for points.
    ///
    /// Values written to variables decorated with the `PointSize` built-in decoration are clamped to
    /// this range.
    #[inline]
    pub fn point_size_range(&self) -> Range<f32> {
        self.vk_10.point_size_range[0]..self.vk_10.point_size_range[1]
    }

    /// The range of supported widths for lines.
    ///
    /// Value specified by the `line_width` in [`pipeline_state::LineWidth`] is clamped to this
    /// range.
    #[inline]
    pub fn line_width_range(&self) -> Range<f32> {
        self.vk_10.line_width_range[0]..self.vk_10.line_width_range[1]
    }

    /// The granularity of supported point sizes.
    #[inline]
    pub fn point_size_granularity(&self) -> f32 {
        self.vk_10.point_size_granularity
    }

    /// The granularity of supported line widths.
    #[inline]
    pub fn line_width_granularity(&self) -> f32 {
        self.vk_10.line_width_granularity
    }

    /// Specifies whether lines are rasterized according to the preferred method of rasterization.
    #[inline]
    pub fn strict_lines(&self) -> bool {
        self.vk_10.strict_lines != 0
    }

    /// Specifies whether rasterization uses the standard sample locations.
    #[inline]
    pub fn standard_sample_locations(&self) -> bool {
        self.vk_10.standard_sample_locations != 0
    }

    /// The optimal buffer offset alignment in bytes for `copy_buffet_to_image` and
    /// `copy_image_to_buffer`.
    ///
    #[inline]
    pub fn optimal_buffer_copy_offset_alignment(&self) -> DeviceSize {
        self.vk_10.optimal_buffer_copy_offset_alignment
    }

    /// The optimal buffer row pitch alignment in bytes for `copy_buffer_to_image` and
    /// `copy_image_to_buffer`.
    #[inline]
    pub fn optimal_buffer_copy_row_pitch_alignment(&self) -> DeviceSize {
        self.vk_10.optimal_buffer_copy_row_pitch_alignment
    }

    /// The size and alignment in bytes that bounds concurrent access to host-mapped device memory.
    #[inline]
    pub fn non_coherent_atom_size(&self) -> DeviceSize {
        self.vk_10.non_coherent_atom_size
    }
}
