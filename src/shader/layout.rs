use core::{
    str,
    ptr::NonNull,
    mem::{DropGuard, forget},
    num::NonZeroU32,
    alloc,
};
use tuhka::vk;
use arrayvec::ArrayVec;

use crate::*;
use sync::{
    atomic::{self, AtomicU64},
    Mutex,
};
use spirv::reflect;
use mem::{
    pack::{pack_alloc, pack_block},
    alloc_vec::FixedVec32,
};

#[derive(Clone)]
enum PipelineLayoutTypeInner
{
    /// Specifies a simple compute shader.
    Compute {
        compute: Shader,
    },
    /// Specifies a vertex shading layout.
    Vertex {
        vertex: Shader,
        fragment: Option<Shader>,
    },
    /// Specifies a mesh shading layout.
    Mesh {
        task: Option<Shader>,
        mesh: Shader,
        fragment: Option<Shader>,
    }
}

/// Specifies the type of a [`PipelineLayout`].
#[derive(Clone)]
pub struct PipelineLayoutType {
    inner: PipelineLayoutTypeInner,
}

impl PipelineLayoutType
{

    /// Specifies a compute pipeline layout.
    ///
    /// # Valid usage
    /// - If `compute` is a shader:
    ///     - it **must** be in the [`Compute`] shader stage.
    /// - If `compute` is a `(shader_module, name)` tuple:
    ///     - the [`ShaderModule`] **must** have an entry point with `name`, which is in the
    ///       [`Compute`] stage.
    ///
    /// [`Compute`]: ShaderStage::Compute
    #[inline]
    pub fn compute<'a, S>(compute: S) -> Result<Self, ShaderReflectError>
        where S: Into<ShaderReflectArgs<'a>>
    {
        let args: ShaderReflectArgs = compute.into();
        Ok(Self {
            inner: PipelineLayoutTypeInner::Compute {
                compute: args.reflect(ShaderStage::Compute)?,
            }
        })
    }

    /// Specifies a traditional vertex shading pipeline layout.
    ///
    /// # Valid usage
    /// - If `vertex` is a shader:
    ///     - it **must** be in the [`Vertex`] shader stage.
    /// - If `vertex` is a `(shader_module, name)` tuple:
    ///     - the [`ShaderModule`] **must** have an entry point with `name`, which is in the
    ///       [`Vertex`] stage.
    /// - If `fragment` is `Some`, and:
    ///     - if it is a shader:
    ///         - it **must** be in the [`Fragment`] shader stage, or
    ///     - if it is a `(shader_module, name)` tuple:
    ///         - the [`ShaderModule`] **must** have an entry point with `name`, which is in the
    ///           [`Fragment`] stage.
    ///
    /// [`Vertex`]: ShaderStage::Vertex
    /// [`Fragment`]: ShaderStage::Fragment
    #[inline]
    pub fn vertex<'a, S>(
        vertex: S,
        fragment: Option<S>,
    ) -> Result<Self, ShaderReflectError>
        where S: Into<ShaderReflectArgs<'a>>
    {
        let args: ShaderReflectArgs = vertex.into();
        let vertex = args.reflect(ShaderStage::Vertex)?;
        let args: Option<ShaderReflectArgs> = fragment
            .map(Into::into);
        let fragment = args
            .into_iter()
            .try_fold(
                None,
                |_, args| {
                    Ok(Some(args.reflect(ShaderStage::Fragment)?))
                }
            )?;
        Ok(Self {
            inner: PipelineLayoutTypeInner::Vertex {
                vertex,
                fragment,
            }
        })
    }

    /// Specifies a mesh shading pipeline layout.
    ///
    /// # Valid usage
    /// - The [`vk_ext::mesh_shader`] feature **must** be enabled.
    /// - If `task` is `Some`:
    ///     - the [`vk_ext::task_shader`] feature **must** be enabled, and
    ///     - if it is a shader:
    ///         - it **must** be in the [`TaskEXT`] shader stage, or
    ///     - if it is a `(shader_module, name)` tuple:
    ///         - the [`ShaderModule`] **must** have an entry point with `name`, which is in the
    ///           [`TaskEXT`] stage.
    /// - If `mesh` is a shader:
    ///     - it **must** be in the [`MeshEXT`] shader stage.
    /// - If `mesh` is a `(shader_module, name)` tuple:
    ///     - the [`ShaderModule`] **must** have an entry point with `name`, which is in the
    ///       [`MeshEXT`] stage.
    /// - If `fragment` is `Some`, and:
    ///     - if it is a shader:
    ///         - it **must** be in the [`Fragment`] shader stage, or
    ///     - if it is a `(shader_module, name)` tuple:
    ///         - the [`ShaderModule`] **must** have an entry point with `name`, which is in the
    ///           [`Fragment`] stage.
    ///
    /// [`TaskEXT`]: ShaderStage::TaskEXT
    /// [`MeshEXT`]: ShaderStage::MeshEXT
    /// [`Fragment`]: ShaderStage::Fragment
    #[inline]
    pub fn mesh<'a, S>(
        task: Option<S>,
        mesh: S,
        fragment: Option<S>
    ) -> Result<Self, ShaderReflectError>
        where S: Into<ShaderReflectArgs<'a>>
    {
        let args: Option<ShaderReflectArgs> = task
            .map(Into::into);
        let task = args
            .into_iter()
            .try_fold(
                None,
                |_, args| {
                    let task = args.reflect(ShaderStage::TaskEXT)?;
                    if !task.module()
                        .device()
                        .is_feature_enabled(vk_ext::task_shader::NAME)
                    {
                        Err(ShaderReflectError::TaskShaderNotEnabled)?
                    }
                    Ok(Some(task))
                }
            )?;
        let args: ShaderReflectArgs = mesh.into();
        let mesh = args.reflect(ShaderStage::MeshEXT)?;
        if !mesh
            .module()
            .device()
            .is_feature_enabled(vk_ext::mesh_shader::NAME)
        {
            Err(ShaderReflectError::MeshShaderNotEnabled)?
        }
        let args: Option<ShaderReflectArgs> = fragment
            .map(Into::into);
        let fragment = args
            .into_iter()
            .try_fold(
                None,
                |_, args| {
                    Ok(Some(args.reflect(ShaderStage::Fragment)?))
                }
            )?;
        Ok(Self {
            inner: PipelineLayoutTypeInner::Mesh { task, mesh, fragment },
        })
    }

    /// Returns an iterator over all the shaders.
    #[inline]
    pub fn shaders(&self) -> impl Iterator<Item = &'_ Shader> {
        let mut shaders = ArrayVec::<&Shader, 3>::new();
        match &self.inner {
            PipelineLayoutTypeInner::Compute { compute } => unsafe {
                shaders.push_unchecked(compute);
            },
            PipelineLayoutTypeInner::Vertex { vertex, fragment } => unsafe {
                shaders.push_unchecked(vertex);
                if let Some(fragment) = fragment {
                    shaders.push_unchecked(fragment);
                }
            },
            PipelineLayoutTypeInner::Mesh { task, mesh, fragment } => unsafe {
                if let Some(task) = task {
                    shaders.push_unchecked(task);
                }
                shaders.push_unchecked(mesh);
                if let Some(fragment) = fragment {
                    shaders.push(fragment);
                }
            }
        }
        shaders.into_iter()
    }
}

/// Additional attributes for a [`PipelineLayout`] binding.
#[derive(Clone, Copy)]
pub struct PipelineLayoutBindingAttributes<'a> {
    /// The set of the binding.
    pub set: u32,
    /// The binding within [`set`][Self::set].
    pub binding: u32,
    /// Overrides the name of this resource.
    pub override_name: Option<&'a str>,
    /// Whether to convert the [`DescriptorType`] to [`InlineUniformBlock`].
    ///
    /// # Valid usage
    /// - The [`vk_13::inline_uniform_block`] feature **must** be enabled.
    /// - This is only valid if the [`DescriptorType`] is [`UniformBuffer`].
    ///
    /// [`InlineUniformBlock`]: DescriptorType::InlineUniformBlock
    /// [`UniformBuffer`]: DescriptorType::UniformBuffer
    pub inline_uniform_block: bool,
    /// Fills a hole caused by a runtime-sized array of resources.
    ///
    /// If this is `None`, it is equivalent to setting this to `1`.
    pub fill_runtime_hole: Option<NonZeroU32>,
    /// Fills a hole in the struct size, if the struct has a runtime-sized array.
    pub fill_struct_hole: Option<NonZeroU32>,
}

/// Additional attributes for a [`PipelineLayout`] (descriptor) set.
#[derive(Clone, Copy)]
pub struct PipelineLayoutSetAttributes {
    /// The set these attributes target.
    pub set: u32,
    /// Whether the set is a push descriptor set and is directly pushed to the [`CommandBuffer`]
    /// instead of allocated from a pool.
    ///
    /// # Valid usage
    /// If this is `true`:
    /// - the [`vk_14::push_descriptor`] feature **must** be enabled, and
    /// - the total number descriptors **must** be less than [`max_push_descriptors`], and
    /// - the set **must** not contain a descriptor with the [`InlineUniformBlock`] descriptor type.
    ///
    /// [`max_push_descriptors`]: vk_14::push_descriptor::Properties::max_push_descriptors
    /// [`InlineUniformBlock`]: DescriptorType::InlineUniformBlock
    pub push_descriptor: bool,
}

/// Specifies parameters for creating an [`PipelineLayout`].
///
/// This **can** be used to create a [`PipelineLayout`] with [`Device::create_pipeline_layout`].
pub struct PipelineLayoutAttributes<'a> {
    /// An optional debug tag of the pipeline layout.
    pub tag: Option<&'a str>,
    /// The type of this layout.
    ///
    /// # Valid usage
    /// - Each [`set`]/[`binding`] pair in the resources of the shaders contained in this type
    ///   **must** be unique.
    ///
    /// [`set`]: Uniform::set
    /// [`binding`]: Uniform::binding
    pub ty: PipelineLayoutType,
    /// Specifies additional attributes for specific sets.
    pub set_attributes: &'a [PipelineLayoutSetAttributes],
    /// Specifies additional attributes for specific `set/binding` pairs.
    pub binding_attributes: &'a [PipelineLayoutBindingAttributes<'a>],
}

impl PipelineLayoutAttributes<'_> {

    /// Calculates how much extra (CPU) memory the created [`PipelineLayout`] will allocate.
    pub fn calculate_allocation_size(&self) -> usize {
        let mut resource_count = 0;
        let mut pc_count = 0;
        let mut set_count = 0;
        for s in self.ty
            .shaders()
        {
            resource_count += s.resources().len();
            s.resources()
                .iter()
                .map(|s| s.set as usize)
                .max()
                .inspect(|&max| {
                    set_count = (set_count).max(max + 1);
                });
            pc_count += s.push_constant_range().is_some() as usize;
        }
        size_of::<Inner>()
            + resource_count * size_of::<ShaderResource>()
            + pc_count * size_of::<PushConstantRange>()
            + set_count * size_of::<NonNull<vk::DescriptorSetLayout>>()
            + self.tag.map(size_of_val).unwrap_or_default()
    }
}

struct Inner {
    device: Device,
    ref_count: AtomicU64,
    ty: PipelineLayoutType,
    handle: Mutex<vk::PipelineLayout>,
    resources: NonNull<[ShaderResource]>,
    push_constants: NonNull<[PushConstantRange]>,
    set_layouts: NonNull<[vk::DescriptorSetLayout]>,
    pack_layout: alloc::Layout,
}

/// A validated set of [shaders].
///
/// This is used for creating pipelines and allocating shader resources.
///
/// [shaders]: Shader
#[must_use]
pub struct PipelineLayout {
    inner: NonNull<Inner>,
    tag: Option<NonNull<[u8]>>,
    allocator: AnyAllocator,
}

unsafe impl Send for PipelineLayout {}
unsafe impl Sync for PipelineLayout {}

impl Clone for PipelineLayout {

    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            self.inner
                .as_ref()
                .ref_count
                .add(1, atomic::Ordering::AcqRel);
        }
        Self {
            inner: self.inner,
            tag: self.tag,
            allocator: self.allocator.clone(),
        }
    }
}

impl Drop for PipelineLayout {

    #[inline]
    fn drop(&mut self) {
        unsafe {
            let ref_count = self.inner.as_ref()
                .ref_count
                .fetch_sub(1, atomic::Ordering::AcqRel);
            if ref_count == 1 {
                self.inner().device.use_allocation_callbacks(|cb| self.inner()
                    .device
                    .functions()
                    .destroy_pipeline_layout(
                        *self.inner()
                            .handle
                            .lock(),
                        cb,
                    )
                );
                self.inner().resources.drop_in_place();
                for layout in self
                    .inner()
                    .set_layouts.iter()
                    .copied()
                {
                    self.inner().device.use_allocation_callbacks(|cb|
                        self.inner()
                            .device
                            .functions()
                            .destroy_descriptor_set_layout(
                                layout,
                                cb,
                            )
                    );
                }
                let pack_layout = self.inner.as_ref().pack_layout;
                self.inner.drop_in_place();
                if let Some(tag) = self.tag() {
                    log::info!("destroyed `PipelineLayout` `{tag}`")
                }
                self.allocator
                    .deallocate(self.inner.cast(), pack_layout);
            }
        }
    }
}

impl PipelineLayout {

    pub(crate) fn new(
        device: Device,
        attributes: PipelineLayoutAttributes<'_>,
        allocator: AnyAllocator,
        tmp_allocator: &(impl Allocator + ?Sized),
    ) -> Result<Self, PipelineLayoutError> {
        if let Some(tag) = attributes.tag {
            log::info!("validating `PipelineLayout` `{tag}`");
        }
        let shaders: ArrayVec<&Shader, 3> = attributes.ty
            .shaders()
            .collect();
        let mut resource_count = 0;
        let mut push_constant_count = 0;
        let mut set_count = 0;
        for shader in &shaders {
            resource_count += shader.resources().len();
            shader
                .resources()
                .iter()
                .map(|r| r.set)
                .max()
                .inspect(|&max| set_count = set_count.max(max + 1));
            push_constant_count += shader.push_constant_range().is_some() as usize;
        }
        let mut inner = NonNull::<Inner>::dangling();
        let mut set_layouts = NonNull::<vk::DescriptorSetLayout>::dangling();
        let mut resources = NonNull::<ShaderResource>::dangling();
        let mut push_constants = NonNull::<PushConstantRange>::dangling();
        let mut tag_slice_ptr = None;
        let allocation =
        if let Some(tag) = attributes.tag {
            let mut tag_ptr = NonNull::dangling();
            let allocation = pack_alloc(
                &allocator,
                [
                    pack_block(&mut inner, 1).unwrap(),
                    pack_block(&mut set_layouts, push_constant_count).unwrap(),
                    pack_block(&mut resources, resource_count).unwrap(),
                    pack_block(&mut push_constants, push_constant_count).unwrap(),
                    pack_block(&mut tag_ptr, tag.len()).unwrap(),
                ]
            ).expect("allocation with `allocator` failed");
            unsafe {
                tag.as_ptr()
                    .copy_to_nonoverlapping(
                        tag_ptr.as_ptr(),
                        tag.len()
                    );
            }
            tag_slice_ptr = Some(NonNull::slice_from_raw_parts(
                tag_ptr,
                tag.len(),
            ));
            allocation
        } else {
            pack_alloc(
                &allocator,
                [
                    pack_block(&mut inner, 1).unwrap(),
                    pack_block(&mut set_layouts, push_constant_count).unwrap(),
                    pack_block(&mut resources, resource_count).unwrap(),
                    pack_block(&mut push_constants, push_constant_count).unwrap(),
                ]
            ).expect("allocation with `allocator` failed")
        };
        assert!(allocation.0.addr() == inner.addr());
        let alloc_guard = DropGuard::new(
            allocation,
            |a| unsafe {
                allocator.deallocate(a.0, a.1);
            }
        );
        let mut vecs = DropGuard::new(
            unsafe {(
                FixedVec32::from_parts(resources, 0, resource_count as u32),
                FixedVec32::from_parts(push_constants, 0, push_constant_count as u32),
                FixedVec32::from_parts(set_layouts, 0, set_count),
                true,
            )},
            |(mut a, b, c, clear)| {
                // only `ShaderResource` needs drop.
                if clear {
                    a.clear();
                }
                forget(a);
                forget(b);
                forget(c);
            }
        );
        static SHADER_STAGES: [ShaderStage; ShaderStage::VARIANT_COUNT] = [
            ShaderStage::Vertex,
            ShaderStage::Fragment,
            ShaderStage::Compute,
            ShaderStage::TaskEXT,
            ShaderStage::MeshEXT,
        ];
        let mut descriptor_counts = [
            [0u32; ShaderStage::VARIANT_COUNT];
            DescriptorType::VARIANT_COUNT
        ];
        let mut total_inline_uniform_block_size = 0u32;
        let max_push_constant_size = device
            .gpu_info()
            .limits
            .max_push_constants_size();
        for shader in &shaders {
            if let Some(&pc) = shader.push_constant_range() {
                if !pc.offset.is_multiple_of(4) {
                    Err(PipelineLayoutError::InvalidPushConstantOffset {
                        offset: pc.offset,
                        stage: shader.shader_stage(),
                    })?
                }
                if pc.size == 0 ||
                    !pc.size.is_multiple_of(4)
                {
                    Err(PipelineLayoutError::InvalidPushConstantSize {
                        size: pc.size,
                        stage: shader.shader_stage(),
                    })?
                }
                if pc.offset + pc.size > max_push_constant_size {
                    Err(PipelineLayoutError::PushConstantOutOfRange {
                        offset: pc.offset,
                        size: pc.size,
                        max_size: max_push_constant_size,
                        stage: shader.shader_stage(),
                    })?
                }
                vecs.1.push(pc);
            }
            for resource in shader.resources() {
                if let Some((set, binding)) = vecs.0
                    .iter()
                    .find(|r| r.set == resource.set && r.binding == resource.binding)
                    .map(|u| (u.set, u.binding))
                {
                    Err(PipelineLayoutError::DuplicateSetBinding { set, binding })?
                }
                let mut resource = resource.clone();
                if let Some(attrs) = attributes
                    .binding_attributes
                    .iter().find(|a| a.binding == resource.binding && a.set == resource.set)
                {
                    if let Some(name) = attrs.override_name {
                        resource.name = CompactStr::new(name);
                    }
                    match resource.count {
                        ShaderResourceCount::Static(_) => {},
                        ShaderResourceCount::Runtime { declared } => {
                            if let Some(resolve) = attrs.fill_runtime_hole {
                                resource.count = ShaderResourceCount::Static(
                                    declared * resolve.get(),
                                );
                            } else {
                                resource.count = ShaderResourceCount::Static(
                                    declared,
                                );
                            }
                        },
                    }
                    let struct_size = resource.struct_size
                        .as_ref()
                        .iter().try_fold(None, |_, size_hint| {
                            Ok(Some(match size_hint {
                                reflect::TypeSizeHint::Static(size) => *size,
                                reflect::TypeSizeHint::Struct(hole) => hole.resolve(
                                    attrs.fill_struct_hole.ok_or_else(|| PipelineLayoutError::UnresolvedStructSize {
                                        resource_name: resource.name.clone()
                                    })?.get() as usize
                                ),
                                _ => unreachable!(),
                            }))
                        })?;
                    resource.struct_size = struct_size.map(reflect::TypeSizeHint::Static);
                    if attrs.inline_uniform_block
                    {
                        if resource.ty != DescriptorType::UniformBuffer {
                            Err(PipelineLayoutError::InvalidInlineUniformBlock {
                                name: resource.name.clone(),
                                found_type: resource.ty,
                            })?
                        }
                        let feature = device.get_feature(vk_13::inline_uniform_block::NAME)
                            .ok_or(PipelineLayoutError::InlineUniformBlockNotEnabled)?;
                        resource.ty = DescriptorType::InlineUniformBlock;
                        let ShaderResourceCount::Static(count) = &mut resource.count else {
                            unsafe { ::core::hint::unreachable_unchecked() }
                        };
                        let struct_size = *count as usize * struct_size.unwrap();
                        let mut struct_size: u32 = struct_size.try_into().map_err(|_| PipelineLayoutError::StructSizeTooBig {
                            resource_name: resource.name.clone(),
                            resolved_size: struct_size,
                        })?;
                        struct_size = struct_size.next_multiple_of(4);
                        if struct_size > feature.max_size {
                            Err(PipelineLayoutError::InlineUniformBlockTooBig {
                                resource_name: resource.name.clone(),
                                resolved_size: struct_size,
                                max_size: feature.max_size,
                            })?
                        }
                        *count = struct_size;
                        total_inline_uniform_block_size = total_inline_uniform_block_size.saturating_add(struct_size);
                    }
                } else {
                    match resource.count {
                        ShaderResourceCount::Static(_) => {},
                        ShaderResourceCount::Runtime { declared } => {
                            resource.count = ShaderResourceCount::Static(
                                declared
                            );
                        },
                    }
                }
                descriptor_counts[resource.ty as usize][resource.shader_stage as usize] += 1;
                vecs.0.push(resource);
            }
        }
        let mut per_stage_counts = [0u32; ShaderStage::VARIANT_COUNT];
        for counts in descriptor_counts.into_iter() {
            for (i, count) in counts.into_iter().enumerate() {
                per_stage_counts[i] += count;
            }
        }
        let max_resources = device
            .gpu_info()
            .limits
            .max_per_stage_resources();
        for (i, count) in per_stage_counts.into_iter().enumerate() {
            if count > max_resources {
                Err(PipelineLayoutError::TooManyResourcesInStage {
                    count,
                    max_count: max_resources,
                    stage: SHADER_STAGES[i],
                })?
            }
        }
        if let Some(feature) = device.get_feature(vk_13::inline_uniform_block::NAME) {
            let max_per_stage = feature.max_per_stage_blocks;
            let inline_uniform_block_counts = descriptor_counts[DescriptorType::InlineUniformBlock as usize];
            for (i, count) in inline_uniform_block_counts.into_iter().enumerate() {
                if count > max_per_stage {
                    Err(PipelineLayoutError::TooManyDescriptorsInStage {
                        count,
                        max_count: max_per_stage,
                        stage: SHADER_STAGES[i],
                        descriptor_types: &[DescriptorType::InlineUniformBlock],
                    })?
                }
            }
            let total_count: u32 = inline_uniform_block_counts.into_iter().sum();
            if total_count > feature.max_descriptor_set_blocks {
                Err(PipelineLayoutError::TooManyDescriptors {
                    count: total_count,
                    max_count: feature.max_descriptor_set_blocks,
                    descriptor_types: &[DescriptorType::InlineUniformBlock],
                })?
            }
            if let Some(max_total_size) = device.gpu_info().limits.max_inline_uniform_total_size()
                && max_total_size < total_inline_uniform_block_size
            {
                Err(PipelineLayoutError::MaxInlineUniformTotalSizeExceeded {
                    max_total_size,
                    total_size: total_inline_uniform_block_size,
                })?
            }
        }
        let limits = &device.gpu_info().limits;
        for (types, max_per_stage, max_all) in [
            (
                &[DescriptorType::Sampler, DescriptorType::CombinedImageSampler] as &'static [DescriptorType],
                GpuLimits::max_per_stage_descriptor_samplers as fn(&GpuLimits) -> u32,
                GpuLimits::max_descriptor_set_samplers as fn(&GpuLimits) -> u32,
            ),
            (
                &[DescriptorType::UniformBuffer],
                GpuLimits::max_per_stage_descriptor_uniform_buffers,
                GpuLimits::max_descriptor_set_uniform_buffers,
            ),
            (
                &[DescriptorType::StorageBuffer],
                GpuLimits::max_per_stage_descriptor_storage_buffers,
                GpuLimits::max_descriptor_set_storage_buffers,
            ),
            (
                &[
                    DescriptorType::CombinedImageSampler,
                    DescriptorType::SampledImage,
                    DescriptorType::UniformTexelBuffer,
                ],
                GpuLimits::max_per_stage_descriptor_sampled_images,
                GpuLimits::max_descriptor_set_sampled_images,
            ),
            (
                &[
                    DescriptorType::StorageImage,
                    DescriptorType::StorageTexelBuffer,
                ],
                GpuLimits::max_per_stage_descriptor_storage_images,
                GpuLimits::max_descriptor_set_storage_images,
            ),
        ] {
            let max_per_stage = max_per_stage(limits);
            let max_all = max_all(limits);
            let mut per_stage_counts = [0u32; ShaderStage::VARIANT_COUNT];
            for &ty in types {
                for (i, count) in descriptor_counts[ty as usize]
                    .into_iter()
                    .enumerate()
                {
                    per_stage_counts[i] += count;
                }
            }
            for (i, count) in per_stage_counts
                .into_iter()
                .enumerate()
            {
                if count > max_per_stage {
                    Err(PipelineLayoutError::TooManyDescriptorsInStage {
                        count,
                        max_count: max_per_stage,
                        stage: SHADER_STAGES[i],
                        descriptor_types: types,
                    })?
                }
            }
            let all_counts: u32 = per_stage_counts.into_iter().sum();
            if all_counts > max_all {
                Err(PipelineLayoutError::TooManyDescriptors {
                    count: all_counts,
                    max_count: max_all,
                    descriptor_types: types,
                })?
            }
        }
        vecs.2.resize(set_count, vk::DescriptorSetLayout::null());
        let set_layouts = NonNull::slice_from_raw_parts(
            set_layouts,
            vecs.2.len() as usize,
        );
        let set_guard = DropGuard::new(
            set_layouts,
            |ptr| unsafe {
                for &layout in ptr.iter() {
                    device.use_allocation_callbacks(|cb| {
                        device.functions()
                            .destroy_descriptor_set_layout(
                                layout,
                                cb,
                            );
                    })
                }
            }
        );
        let v = &mut *vecs;
        for (i, set) in v.2.iter_mut().enumerate_u32() {
            let attrs = attributes
                .set_attributes
                .iter()
                .find(|s| s.set == i)
                .copied()
                .unwrap_or(PipelineLayoutSetAttributes {
                    set: i,
                    push_descriptor: false,
                });
            let max_push_descriptors =
                if attrs.push_descriptor
                {
                    let Some(feature) = device.get_feature(vk_14::push_descriptor::NAME) else {
                        Err(PipelineLayoutError::PushDescriptorNotEnabled)?
                    };
                    feature.max_push_descriptors
                } else {
                    0
                };
            let binding_count = v.0
                .iter()
                .filter(|r| r.set == i)
                .count_u32();
            let mut bindings = FixedVec32
                ::with_capacity_in(binding_count, &tmp_allocator)
                .unwrap();
            let mut num_push_descriptors = 0;
            for r in v.0
                .iter()
                .filter(|r| r.set == i)
            {
                let descriptor_count = __variant_some!(
                    ShaderResourceCount::Static = r.count
                ).unwrap();
                if attrs.push_descriptor {
                    if r.ty == DescriptorType::InlineUniformBlock {
                        Err(PipelineLayoutError::InvalidPushDescriptorType {
                            descriptor_type: r.ty,
                        })?
                    }
                    num_push_descriptors += descriptor_count;
                }
                bindings.push(vk::DescriptorSetLayoutBinding {
                    binding: r.binding,
                    descriptor_type: r.ty.into(),
                    descriptor_count,
                    stage_flags: r.shader_stage.into(),
                    ..Default::default()
                });
            }
            if num_push_descriptors > max_push_descriptors {
                Err(PipelineLayoutError::TooManyPushDescriptors {
                    count: num_push_descriptors,
                    max_count: max_push_descriptors,
                })?
            }
            let create_info = vk::DescriptorSetLayoutCreateInfo {
                flags: if attrs.push_descriptor {
                    vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR
                } else {
                    default()
                },
                ..Default::default()
            }.bindings(&bindings);
            *set = device.use_allocation_callbacks(|cb| unsafe {
                device.functions()
                    .create_descriptor_set_layout(
                        &create_info,
                        cb,
                    ).describe_err("failed to create `VkDescriptorSetLayout")
                    .map_err(PipelineLayoutError::Internal)
            })?;
        }
        let mut pcs = FixedVec32::with_capacity_in(
            vecs.1.len(),
            tmp_allocator
        ).unwrap();
        pcs.extend(vecs.1.iter().map(|pc| vk::PushConstantRange {
            stage_flags: pc.shader_stage.into(),
            offset: pc.offset,
            size: pc.size,
        }));
        let create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&vecs.2)
            .push_constant_ranges(&pcs);
        let handle = device.use_allocation_callbacks(|cb| unsafe {
            device.functions()
                .create_pipeline_layout(
                    &create_info,
                    cb,
                ).describe_err("failed to create `VkPipelineLayout`")
                .map_err(PipelineLayoutError::Internal)
        })?;
        vecs.3 = false;
        let pack_layout = DropGuard::dismiss(alloc_guard).1;
        DropGuard::dismiss(set_guard);
        drop(shaders);
        unsafe {
            inner.write(Inner {
                device,
                ref_count: AtomicU64::new(1),
                ty: attributes.ty,
                handle: Mutex::new(handle),
                resources: NonNull::slice_from_raw_parts(
                    resources,
                    vecs.0.len() as usize,
                ),
                push_constants: NonNull::slice_from_raw_parts(
                    push_constants,
                    vecs.1.len() as usize,
                ),
                set_layouts,
                pack_layout,
            });
        }
        if let Some(tag) = attributes.tag {
            log::info!("created `PipelineLayout` `{tag}`");
        }
        Ok(Self {
            inner,
            tag: tag_slice_ptr,
            allocator,
        })
    }

    #[inline]
    unsafe fn inner(&self) -> &Inner {
        unsafe {
            self.inner.as_ref()
        }
    }

    /// Returns the tag assigned to this pipeline layout.
    #[inline]
    pub fn tag(&self) -> Option<&str> {
        let tag = self.tag?;
        Some(unsafe {
            str::from_utf8_unchecked(
                tag.as_ref()
            )
        })
    }

    /// Returns the [`Device`] used to create this layout.
    #[inline]
    pub fn device(&self) -> &Device {
        unsafe {
            &self.inner().device
        }
    }

    /// The type of this pipeline layout as specified in [`PipelineLayoutAttributes::ty`].
    #[inline]
    pub fn ty(&self) -> &PipelineLayoutType {
        unsafe {
            &self.inner().ty
        }
    }
   
    /// Returns a slice over all [shader resources] in this pipeline layout.
    ///
    /// [shader resources]: ShaderResource
    #[inline]
    pub fn resources(&self) -> &[ShaderResource] {
        unsafe {
            self.inner().resources
                .as_ref()
        }
    }

    /// Returns a slice over all [push constant ranges] in this pipeline layout.
    ///
    /// [push constant ranges]: PushConstantRange
    #[inline]
    pub fn push_constant_ranges(&self) -> &[PushConstantRange] {
        unsafe {
            self.inner().push_constants
                .as_ref()
        }
    }
}
