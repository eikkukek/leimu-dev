use core::{
    str,
    slice,
    ptr::NonNull,
    borrow::Borrow,
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
use mem::{
    pack::{pack_alloc, pack_block},
    alloc_vec::FixedVec32,
};

/// Specifies the type of a [`ShaderSet`].
pub enum ShaderSetType<T>
    where T: Borrow<Shader>
{
    /// Specifies a simple compute shader.
    Compute {
        /// The compute shader.
        ///
        /// # Valid usage
        /// The stage of the shader **must** be [`ShaderStage::Compute`].
        compute: T,
    },
    /// Specifies a vertex shading set.
    Vertex {
        /// The vertex shader.
        ///
        /// # Valid usage
        /// The stage of the shader **must** be [`ShaderStage::Vertex`].
        vertex: T,
        /// An optional fragment shader.
        ///
        /// # Valid usage
        /// If this is `Some`:
        /// - the stage of the shader **must** be [`ShaderStage::Fragment`].
        fragment: Option<T>,
    },
    /// Specifies a mesh shading set.
    ///
    /// # Valid usage
    /// The [`vk_ext::mesh_shader`] feature **must** be enabled.
    Mesh {
        /// An optional task shader.
        ///
        /// # Valid usage
        /// If this is `Some`:
        /// - the stage of the shader **must** be [`ShaderStage::TaskEXT`], and
        /// - the [`vk_ext::task_shader`] feature **must** be enabled
        task: Option<T>,
        /// The mesh shader.
        ///
        /// # Valid usage
        /// The stage of the shader **must** be [`ShaderStage::MeshEXT`].
        mesh: T,
        /// An optional fragment shader.
        ///
        /// If this is `Some`:
        /// - the stage of the shader **must** be [`ShaderStage::Fragment`].
        fragment: Option<T>,
    }
}

fn check_shader_stage(shader: &Shader, expected: ShaderStage) -> Result<(), ShaderSetError> {
    if shader.shader_stage() != expected {
        Err(ShaderSetError::MismatchedShaderStage {
            tag: shader.tag().map(Into::into),
            expected,
            found: shader.shader_stage(),
        })?
    }
    Ok(())
}

impl<T> ShaderSetType<T>
    where T: Borrow<Shader>
{

    /// Maps `T` to `U`.
    #[inline]
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> ShaderSetType<U>
        where U: Borrow<Shader>,
    {
        match self {
            Self::Compute { compute } => {
                ShaderSetType::Compute { compute: f(compute) }
            },
            Self::Vertex { vertex, fragment } => {
                ShaderSetType::Vertex {
                    vertex: f(vertex),
                    fragment: fragment.map(&mut f),
                }
            },
            Self::Mesh { task, mesh, fragment } => {
                ShaderSetType::Mesh {
                    task: task.map(&mut f),
                    mesh: f(mesh),
                    fragment: fragment.map(&mut f),
                }
            },
        }
    }
   
    /// Returns an iterator over all the shaders.
    ///
    /// Returns an error if there's a shader stage mismatch.
    #[inline]
    pub fn shaders(&self) -> Result<impl Iterator<Item = &'_ Shader>, ShaderSetError> {
        let mut shaders = ArrayVec::<&Shader, 3>::new();
        match self {
            Self::Compute { compute } => unsafe {
                let compute = compute.borrow();
                check_shader_stage(compute, ShaderStage::Compute)?;
                shaders.push_unchecked(compute);
            },
            Self::Vertex { vertex, fragment } => unsafe {
                let vertex = vertex.borrow();
                check_shader_stage(vertex, ShaderStage::Vertex)?;
                shaders.push_unchecked(vertex);
                if let Some(fragment) = fragment {
                    let fragment = fragment.borrow();
                    check_shader_stage(fragment, ShaderStage::Fragment)?;
                    shaders.push_unchecked(fragment);
                }
            },
            Self::Mesh { task, mesh, fragment } => unsafe {
                if let Some(task) = task {
                    let task = task.borrow();
                    check_shader_stage(task, ShaderStage::TaskEXT)?;
                    shaders.push_unchecked(task);
                }
                let mesh = mesh.borrow();
                check_shader_stage(mesh, ShaderStage::MeshEXT)?;
                shaders.push_unchecked(mesh);
                if let Some(fragment) = fragment {
                    let fragment = fragment.borrow();
                    check_shader_stage(fragment, ShaderStage::Fragment)?;
                    shaders.push(fragment);
                }
            }
        }
        Ok(shaders.into_iter())
    }

    /// Returns an iterator over all the shaders without checking the validity of their shader
    /// stages.
    #[inline]
    pub fn shaders_unchecked(&self) -> impl Iterator<Item = &'_ Shader> {
        let mut shaders = ArrayVec::<&Shader, 3>::new();
        match self {
            Self::Compute { compute } => unsafe {
                let compute = compute.borrow();
                shaders.push_unchecked(compute);
            },
            Self::Vertex { vertex, fragment } => unsafe {
                let vertex = vertex.borrow();
                shaders.push_unchecked(vertex);
                if let Some(fragment) = fragment {
                    let fragment = fragment.borrow();
                    shaders.push_unchecked(fragment);
                }
            },
            Self::Mesh { task, mesh, fragment } => unsafe {
                if let Some(task) = task {
                    let task = task.borrow();
                    shaders.push_unchecked(task);
                }
                let mesh = mesh.borrow();
                shaders.push_unchecked(mesh);
                if let Some(fragment) = fragment {
                    let fragment = fragment.borrow();
                    shaders.push(fragment);
                }
            }
        }
        shaders.into_iter()
    }
}

/// Additional attributes for a [`ShaderSet`] binding.
#[derive(Clone, Copy)]
pub struct ShaderSetBindingAttributes<'a> {
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

/// Specifies parameters for creating an [`ShaderSetAttributes`].
///
/// This **can** be used to create a [`ShaderSet`] with [`Device::create_shader_set`].
pub struct ShaderSetAttributes<'a> {
    /// An optional debug tag of the shader set.
    pub tag: Option<&'a str>,
    /// The type of this set.
    ///
    /// # Valid usage
    /// - Each [`set`]/[`binding`] pair in the resources of the shaders contained in this type
    ///   **must** be unique.
    ///
    /// [`set`]: Uniform::set
    /// [`binding`]: Uniform::binding
    pub ty: ShaderSetType<&'a Shader>,
    /// Specifies additional attributes for specific `set`/`binding` pairs.
    pub binding_attributes: &'a [ShaderSetBindingAttributes<'a>],
}

impl ShaderSetAttributes<'_> {

    /// Calculates how much extra (CPU) memory the created [`ShaderSet`] will allocate.
    pub fn calculate_allocation_size(&self) -> usize {
        let mut resource_count = 0;
        let mut pc_count = 0;
        for s in self.ty
            .shaders_unchecked()
        {
            resource_count += s.resources().len();
            pc_count += s.push_constant_range().is_some() as usize;
        }
        size_of::<Inner>()
            + resource_count * size_of::<ShaderResource>()
            + pc_count * size_of::<PushConstantRange>()
            + self.tag.map(size_of_val).unwrap_or_default()
    }
}

struct Inner {
    device: Device,
    ref_count: AtomicU64,
    pipeline_layout: Mutex<Option<vk::PipelineLayout>>,
    pack_layout: alloc::Layout,
}

/// A validated set of [shaders].
///
/// This is used for creating pipelines and allocating shader resources.
///
/// [shaders]: Shader
pub struct ShaderSet {
    inner: NonNull<Inner>,
    tag: Option<NonNull<[u8]>>,
    resources: NonNull<[ShaderResource]>,
    push_constants: NonNull<[PushConstantRange]>,
    allocator: AnyAllocator,
}

unsafe impl Send for ShaderSet {}
unsafe impl Sync for ShaderSet {}

impl Clone for ShaderSet {

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
            resources: self.resources,
            push_constants: self.push_constants,
            allocator: self.allocator.clone(),
        }
    }
}

impl Drop for ShaderSet {

    #[inline]
    fn drop(&mut self) {
        unsafe {
            let ref_count = self.inner.as_ref()
                .ref_count
                .fetch_sub(1, atomic::Ordering::AcqRel);
            if ref_count == 1 {
                let layout = self.inner
                    .as_ref()
                    .pipeline_layout
                    .lock();
                if let Some(layout) = *layout
                {
                    self.inner
                        .as_ref()
                        .device
                        .functions()
                        .destroy_pipeline_layout(
                            layout,
                            None
                        );
                }
                drop(layout);
                self.resources.drop_in_place();
                let pack_layout = self.inner.as_ref().pack_layout;
                self.inner.drop_in_place();
                if let Some(tag) = self.tag() {
                    log::info!("destroyed `Image` `{tag}`")
                }
                self.allocator
                    .deallocate(self.inner.cast(), pack_layout);
            }
        }
    }
}

impl ShaderSet {

    pub(crate) fn new(
        device: Device,
        attributes: ShaderSetAttributes<'_>,
        allocator: AnyAllocator,
    ) -> Result<Self, ShaderSetError> {
        if let Some(tag) = attributes.tag {
            log::info!("validation `ShaderSet` `{tag}`");
        }
        let shaders: ArrayVec<&Shader, 3> = attributes.ty
            .shaders()?
            .collect();
        let mut resource_count = 0;
        let mut push_constant_count = 0;
        for shader in &shaders {
            resource_count += shader.resources().len();
            push_constant_count += shader.push_constant_range().is_some() as usize;
        }
        let mut inner = NonNull::<Inner>::dangling();
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
                true,
            )},
            |(mut a, mut b, clear)| {
                if clear {
                    a.clear();
                    b.clear();
                }
                forget(a);
                forget(b);
            }
        );
        static SHADER_STAGES: [ShaderStage; ShaderStage::VARIANT_COUNT] = [
            ShaderStage::Vertex,
            ShaderStage::Fragment,
            ShaderStage::Compute,
            ShaderStage::TaskEXT,
            ShaderStage::MeshEXT,
        ];
        let mut descriptor_counts = [[0u32; ShaderStage::VARIANT_COUNT]; DescriptorType::VARIANT_COUNT];
        let mut total_inline_uniform_block_size = 0u32;
        let max_push_constant_size = device
            .gpu_info()
            .limits
            .max_push_constants_size();
        for shader in &shaders {
            if let Some(&pc) = shader.push_constant_range() {
                if !pc.offset.is_multiple_of(4) {
                    Err(ShaderSetError::InvalidPushConstantOffset {
                        offset: pc.offset,
                        stage: shader.shader_stage(),
                    })?
                }
                if pc.size == 0 ||
                    !pc.size.is_multiple_of(4)
                {
                    Err(ShaderSetError::InvalidPushConstantSize {
                        size: pc.size,
                        stage: shader.shader_stage(),
                    })?
                }
                if pc.offset + pc.size > max_push_constant_size {
                    Err(ShaderSetError::PushConstantOutOfRange {
                        offset: pc.offset,
                        size: pc.size,
                        max_size: pc.size,
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
                    Err(ShaderSetError::DuplicateSetBinding { set, binding })?
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
                                    declared * resolve.get()
                                );
                            }
                        },
                    }
                    if attrs.inline_uniform_block
                    {
                        if resource.ty != DescriptorType::UniformBuffer {
                            Err(ShaderSetError::InvalidInlineUniformBlock {
                                name: resource.name.clone(),
                                found_type: resource.ty,
                            })?
                        }
                        let feature = device.get_feature(vk_13::inline_uniform_block::NAME)
                            .ok_or(ShaderSetError::InlineUniformBlockNotEnabled)?;
                        resource.ty = DescriptorType::InlineUniformBlock;
                        let ShaderResourceCount::Static(count) = &mut resource.count else {
                            unsafe { ::core::hint::unreachable_unchecked() }
                        };
                        let struct_size = match resource.struct_size.as_ref().unwrap() {
                            leimu_spirv::reflect::TypeSizeHint::Static(size) => *size,
                            leimu_spirv::reflect::TypeSizeHint::Struct(hole) => hole.resolve(
                                attrs.fill_struct_hole.ok_or_else(|| ShaderSetError::UnresolvedStructSize {
                                    resource_name: resource.name.clone()
                                })?.get() as usize
                            ),
                            _ => unreachable!(),
                        };
                        resource.struct_size = Some(leimu_spirv::reflect::TypeSizeHint::Static(struct_size as usize));
                        let struct_size = *count as usize * struct_size;
                        let mut struct_size: u32 = struct_size.try_into().map_err(|_| ShaderSetError::StructSizeTooBig {
                            resource_name: resource.name.clone(),
                            resolved_size: struct_size,
                        })?;
                        struct_size = struct_size.next_multiple_of(4);
                        if struct_size > feature.max_size {
                            Err(ShaderSetError::InlineUniformBlockTooBig {
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
                Err(ShaderSetError::TooManyResourcesInStage {
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
                    Err(ShaderSetError::TooManyDescriptorsInStage {
                        count,
                        max_count: max_per_stage,
                        stage: SHADER_STAGES[i],
                        descriptor_types: &[DescriptorType::InlineUniformBlock],
                    })?
                }
            }
            let total_count: u32 = inline_uniform_block_counts.into_iter().sum();
            if total_count > feature.max_descriptor_set_blocks {
                Err(ShaderSetError::TooManyDescriptors {
                    count: total_count,
                    max_count: feature.max_descriptor_set_blocks,
                    descriptor_types: &[DescriptorType::InlineUniformBlock],
                })?
            }
            if let Some(max_total_size) = device.gpu_info().limits.max_inline_uniform_total_size()
                && max_total_size < total_inline_uniform_block_size
            {
                Err(ShaderSetError::MaxInlineUniformTotalSizeExceeded {
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
                    Err(ShaderSetError::TooManyDescriptorsInStage {
                        count,
                        max_count: max_per_stage,
                        stage: SHADER_STAGES[i],
                        descriptor_types: types,
                    })?
                }
            }
            let all_counts: u32 = per_stage_counts.into_iter().sum();
            if all_counts > max_all {
                Err(ShaderSetError::TooManyDescriptors {
                    count: all_counts,
                    max_count: max_all,
                    descriptor_types: types,
                })?
            }
        }
        vecs.2 = false;
        let pack_layout = DropGuard::dismiss(alloc_guard).1;
        unsafe {
            inner.write(Inner {
                device,
                ref_count: AtomicU64::new(1),
                pipeline_layout: Mutex::default(),
                pack_layout,
            });
        }
        Ok(Self {
            inner,
            tag: tag_slice_ptr,
            resources: NonNull::slice_from_raw_parts(
                resources,
                vecs.0.len() as usize,
            ),
            push_constants: NonNull::slice_from_raw_parts(
                push_constants,
                vecs.1.len() as usize,
            ),
            allocator,
        })
    }

    /// Gets the tag assigned to this shader set.
    #[inline]
    pub fn tag(&self) -> Option<&str> {
        let tag = self.tag?;
        Some(unsafe {
            str::from_utf8_unchecked(
                tag.as_ref()
            )
        })
    }
   
    /// Returns a slice over all [shader resources] in this shader set.
    ///
    /// [shader resources]: ShaderResource
    #[inline]
    pub fn resources(&self) -> &[ShaderResource] {
        unsafe {
            self.resources
                .as_ref()
        }
    }

    /// Returns a slice over all [push constant ranges] in this shader set.
    ///
    /// [push constant ranges]: PushConstantRange
    #[inline]
    pub fn push_constant_ranges(&self) -> &[PushConstantRange] {
        unsafe {
            self.push_constants
                .as_ref()
        }
    }
}
