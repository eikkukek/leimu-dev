use core::{
    ffi::CStr,
    borrow::Borrow,
};
use tuhka::vk;

use crate::*;

use sync::Arc;
use spirv::{op, reflect::{self, ResourceType}};
use mem::{vec32, alloc_vec::Vec32};

/// Specifies the number of shader resources in a binding.
///
/// The count may be a [statically known count][1] or a [count that's resolved at runtime][2].
///
/// [1]: Self::Static
/// [2]: Self::Runtime
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShaderResourceCount {
    /// A statically known count.
    Static(u32),
    /// A count that's resolved at runtime.
    Runtime {
        /// The "declared" count of the resource.
        ///
        /// This is the count that would result if the runtime array had a length of `1`.
        declared: u32
    },
}
impl From<reflect::ResourceCount> for ShaderResourceCount {
    #[inline]
    fn from(value: reflect::ResourceCount) -> Self {
        match value {
            reflect::ResourceCount::Static(count) => {
                Self::Static(count as u32)
            },
            reflect::ResourceCount::Runtime { declared } => {
                Self::Runtime { declared: declared as u32 }
            },
        }
    }
}

/// A shader resource.
#[derive(Clone)]
pub struct ShaderResource {
    /// The shader stage of this reource.
    pub shader_stage: ShaderStage,
    /// The name of this resource.
    pub name: CompactStr,
    /* not supported for now
    /// The value of an optional input attachment index.
    pub input_attachment_index: Option<u32>,
    */
    /// The `DescriptorSet` of this resource.
    pub set: u32,
    /// The `Binding` of this resource.
    pub binding: u32,
    /// The [`DescriptorType`] of this resource.
    pub ty: DescriptorType,
    /// The number of these resources.
    ///
    /// This could be either static or defined during runtime.
    pub count: ShaderResourceCount,
    /// The size of the struct if this resource is a block.
    pub struct_size: Option<reflect::TypeSizeHint>,
}

/// A shader push constant range.
#[derive(Clone, Copy)]
pub struct PushConstantRange {
    /// The shader stage of this range.
    pub shader_stage: ShaderStage,
    /// The offset of this range.
    pub offset: u32,
    /// The size of this range.
    pub size: u32,
}
impl From<PushConstantRange> for vk::PushConstantRange {    
    #[inline]
    fn from(value: PushConstantRange) -> Self {
        Self {
            stage_flags: value.shader_stage.into(),
            offset: value.offset,
            size: value.size,
        }
    }
}
static_assertions::const_assert!(!::core::mem::needs_drop::<PushConstantRange>());

struct Inner {
    module: ShaderModule,
    tag: Option<CompactStr>,
    shader_stage: ShaderStage,
    name: CompactCStr,
    resources: Vec32<ShaderResource>,
    spec_constants: Vec<(u32, ScalarType)>,
    push_constant_range: Option<PushConstantRange>,
}

/// A reflected shader in an entry point.
#[derive(Clone)]
pub struct Shader {
    inner: Arc<Inner>,
}

impl Shader {

    pub(super) fn new(
        module: ShaderModule,
        tag: Option<&str>,
        shader_stage: ShaderStage,
        name: &str,
    ) -> Result<Self, ShaderReflectError> {
        if let Some(tag) = tag {
            log::info!("reflecting `Shader` `{tag}`")
        }
        let mut reflector = spirv::reflect::Reflector::new(
            spirv::Module::new(module.spirv()).unwrap()
        ).unwrap();
        reflector.set_entry_point(
            name,
            shader_stage.into(),
        ).map_err(|_| ShaderReflectError::EntryPointNotFound(shader_stage, name.into()))?;
        let mut resources = vec32![];
        for (resource_type, descriptor_type) in [
            (ResourceType::UniformBuffer, DescriptorType::UniformBuffer),
            (ResourceType::StorageBuffer, DescriptorType::StorageBuffer),
            //(ResourceType::InputAttachment, DescriptorType::InputAttachment), not supported for
            //now
            (ResourceType::StorageImage, DescriptorType::StorageImage),
            (ResourceType::CombinedImageSampler, DescriptorType::CombinedImageSampler),
            (ResourceType::SeparateImage, DescriptorType::SampledImage),
            (ResourceType::SeparateSampler, DescriptorType::Sampler),
            (ResourceType::UniformTexelBuffer, DescriptorType::UniformTexelBuffer),
            (ResourceType::StorageTexelBuffer, DescriptorType::StorageTexelBuffer)
        ] {
            for resource in reflector
                .resources_for_type(resource_type)
                .map_err(ShaderReflectError::ReflectError)?
            {
                let resource = resource.map_err(ShaderReflectError::ReflectError)?;
                let mut set = None;
                let mut binding = None;
                for dec in reflector.decorations(resource.variable_id) {
                    if let op::DecorationOperands::DescriptorSet { descriptor_set } = dec.decoration {
                        set = Some(descriptor_set);
                    } else if let op::DecorationOperands::Binding { binding_point } = dec.decoration {
                        binding = Some(binding_point);
                    }
                }
                let name = resource.name
                    .map(|str| str.as_str().into())
                    .unwrap_or_default();
                let Some(set) = set else {
                    return Err(ShaderReflectError::NoDescriptorSet(
                        name,
                    ))
                };
                let Some(binding) = binding else {
                    return Err(ShaderReflectError::NoDescriptorBinding(
                        name,
                    ))
                };
                resources.push(ShaderResource {
                    shader_stage,
                    set,
                    binding,
                    ty: descriptor_type,
                    name,
                    count: resource.count.into(),
                    struct_size: match reflector
                        .type_description(resource.base_type_id)
                        .map_err(ShaderReflectError::ReflectError)?
                        .size_hint
                    {
                        x if matches!(x, reflect::TypeSizeHint::Static(_) | reflect::TypeSizeHint::Struct(_)) => {
                            Some(x)
                        },
                        _ => None,
                    }
                });
            }
        }
        let mut push_constant_ranges = vec32![];
        for pc in reflector
            .resources_for_type(ResourceType::PushConstant)
            .map_err(ShaderReflectError::ReflectError)?
        {
            let pc = pc.map_err(ShaderReflectError::ReflectError)?;
            push_constant_ranges.push(PushConstantRange {
                shader_stage,
                offset: pc.offset.unwrap(),
                size: reflector
                    .type_description(pc.base_type_id)
                    .map_err(ShaderReflectError::ReflectError)?
                    .size_hint.declared() as u32,
            });
        }
        if push_constant_ranges.len() > 1 {
            Err(ShaderReflectError::TooManyPushConstants)?
        }
        let spec_constants = reflector
            .spec_constants()
            .filter_map(Result::ok)
            .map(|spec_constant| {
                (
                    spec_constant.spec_id,
                    spec_constant.ty,
                )
            }).collect();
        let mut name_bytes = name.as_bytes().to_owned();
        name_bytes.push(0);
        if let Some(tag) = tag {
            log::info!("created `Shader` `{tag}`")
        }
        Ok(Self {
            inner: Inner {
                module,
                spec_constants,
                tag: tag.map(CompactStr::new),
                shader_stage,
                name: CompactCStr::new(::core::ffi::CStr
                    ::from_bytes_until_nul(&name_bytes)
                    .unwrap()
                ),
                resources,
                push_constant_range: push_constant_ranges.first().copied(),
            }.arced()
        })
    }

    /// Returns a reference to the [`ShaderModule`] used to create this shader.
    #[inline]
    pub fn module(&self) -> &ShaderModule {
        &self.inner.module
    }

    /// Returns the debug tag of this shader.
    #[inline]
    pub fn tag(&self) -> Option<&str> {
        self.inner.tag.as_deref()
    }

    /// Returns the [`ShaderStage`] of this shader.
    #[inline]
    pub fn shader_stage(&self) -> ShaderStage{
        self.inner.shader_stage
    }

    /// Gets the entry point of this shader as a `(shader_stage, name)` tuple.
    #[inline]
    pub fn entry_point(&self) -> (ShaderStage, &CStr){
        (self.inner.shader_stage, &self.inner.name)
    }

    /// Returns a slice over all [resources] in this shader.
    ///
    /// [resources]: ShaderResource
    #[inline]
    pub fn resources(&self) -> &[ShaderResource] {
        &self.inner.resources
    }
    
    /// Returns the push constant range of this shader, if any.
    #[inline]
    pub fn push_constant_range(&self) -> Option<&PushConstantRange> {
        self.inner.push_constant_range.as_ref()
    }

    /// Returns a slice over the `SpecId`s and types of all specialization constants in this shader.
    #[inline]
    pub fn spec_constants(&self) -> &[(u32, ScalarType)] {
        &self.inner.spec_constants
    }
}

enum ReflectArgsInner<'a> {
    Shader(Shader),
    ModuleAndName(ShaderModule, &'a str)
}
/// Opaque structure containing arguments used for shader reflection.
///
/// This utilized in [`PipelineLayoutType`] where you can pass either a [`Shader`] or a
/// `(ShaderModule, entry_point_name)` tuple as arguments for creating the type.
///
/// # Examples
/// ``` rust
/// # use leimu::{ShaderModule, ShaderReflectArgs, ShaderStage};
/// # let shader_module: ShaderModule = {
/// #   return;
/// # };
/// let args: ShaderReflectArgs = (shader_module, "main").into();
/// let shader = args
///     .reflect(ShaderStage::Compute)
///     .expect("reflection failed");
/// let args: ShaderReflectArgs = shader.into();
/// args.reflect(ShaderStage::Fragment)
///     .map(|_| {})
///     .expect_err("stage should be `Compute`");
/// ```
pub struct ShaderReflectArgs<'a> {
    inner: ReflectArgsInner<'a>,
}
impl ShaderReflectArgs<'_> {

    /// Tries to reflect these arguments in `shader_stage`.
    #[inline]
    pub fn reflect(self, shader_stage: ShaderStage) -> Result<Shader, ShaderReflectError> {
        match self.inner {
            ReflectArgsInner::Shader(shader) => {
                if shader.shader_stage() != shader_stage {
                    Err(ShaderReflectError::MismatchedShaderStage {
                        expected: shader_stage,
                        found: shader.shader_stage(),
                    })?
                }
                Ok(shader)
            },
            ReflectArgsInner::ModuleAndName(module, name) => {
                Shader::new(module, None, shader_stage, name)
            },
        }
    }
}
impl From<Shader> for ShaderReflectArgs<'_> {
    #[inline]
    fn from(value: Shader) -> Self {
        Self {
            inner: ReflectArgsInner::Shader(value),
        }
    }
}
impl<'a, M, S> From<(M, &'a S)> for ShaderReflectArgs<'a>
    where
        M: Borrow<ShaderModule>,
        S: AsRef<str> + ?Sized
{
    #[inline]
    fn from(value: (M, &'a S)) -> Self {
        Self {
            inner: ReflectArgsInner::ModuleAndName(
                value.0.borrow().clone(),
                value.1.as_ref(),
            ),
        }
    }
}
