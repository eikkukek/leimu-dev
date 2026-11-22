use bytemuck::{Zeroable, NoUninit, ByteEq, ByteHash};
use tuhka::vk;

use crate::*;

use spirv::op;


/// Specifies the type of a shader descriptor.
///
/// The default value is [`Sampler`][Self::Sampler].
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash, Debug)]
pub enum DescriptorType {
    /// Specifies a sampler descriptor.
    /// # SPIR-V type
    /// `OpTypeSampler`.
    #[default]
    Sampler = 0,
    /// Specifies combined image/sampler descriptor
    /// # SPIR-V type
    /// `OpTypeSampledImage`.
    CombinedImageSampler,
    /// Specifies a sampled image descriptor.
    /// # SPIR-V type
    /// `OpTypeImage` with `sampled = 1`.
    SampledImage,
    /// Specifies a storage image descriptor.
    /// # SPIR-V type
    /// `OpTypeImage` with `sampled = 2`.
    StorageImage,
    /// Specifies a uniform texel buffer descriptor 
    /// # SPIR-V type
    /// `OpTypeImage` with `dim = Buffer` and `sampled = 1`.
    UniformTexelBuffer,
    /// Specifies a storage texel buffer descriptor.
    /// # SPIR-V type
    /// `OpTypeImage` with `dim = Buffer` and `sampled = 2`
    StorageTexelBuffer,
    /// Specifies a uniform buffer descriptor.
    /// # SPIR-V type
    /// Block decorated `OpTypeStruct` in a variable of `Uniform` storage class.
    UniformBuffer,
    /// Specifies a storage buffer descriptor.
    /// # SPIR-V type
    /// Block decorated `OpTypeStruct` in a variable of `StorageBuffer` storage class.
    StorageBuffer,
    /// Specifies an input inline uniform block.
    /// # SPIR-V type
    /// Defined identically as [`UniformBuffer`][Self::UniformBuffer]
    InlineUniformBlock,
}
unsafe impl NoUninit for DescriptorType {}

impl DescriptorType {
    /// The number of variants in this enum.
    pub const VARIANT_COUNT: usize = Self::InlineUniformBlock as usize + 1; 
}

/// An enumeration of shader stages.
#[repr(u32)]
#[derive(Clone, Copy, Zeroable, ByteEq, ByteHash, Debug)]
pub enum ShaderStage {
    /// Specifies the vertex stage.
    Vertex = 0,
    /// Specifies the fragment stage.
    Fragment,
    /// Specifies the compute stage.
    Compute,
    /// Specifies the task stage.
    TaskEXT,
    /// Specifies the mesh stage.
    MeshEXT,
}
unsafe impl NoUninit for ShaderStage {}

impl ShaderStage {
    /// The number of variants in this enum.
    pub const VARIANT_COUNT: usize = Self::MeshEXT as usize + 1; 
}
impl From<ShaderStage> for op::ExecutionModel {
    #[inline]
    fn from(value: ShaderStage) -> Self {
        static EXEC_MODELS: [op::ExecutionModel; ShaderStage::VARIANT_COUNT] = [
            op::ExecutionModel::VERTEX,
            op::ExecutionModel::FRAGMENT,
            op::ExecutionModel::GLCOMPUTE,
            op::ExecutionModel::TASK_EXT,
            op::ExecutionModel::MESH_EXT,
        ];
        EXEC_MODELS[value as usize]
    }
}
impl From<ShaderStage> for vk::ShaderStageFlags {
    #[inline]
    fn from(value: ShaderStage) -> Self {
        static STAGES: [vk::ShaderStageFlags; ShaderStage::VARIANT_COUNT] = [
            vk::ShaderStageFlags::VERTEX,
            vk::ShaderStageFlags::FRAGMENT,
            vk::ShaderStageFlags::COMPUTE,
            vk::ShaderStageFlags::TASK_EXT,
            vk::ShaderStageFlags::MESH_EXT,
        ];
        STAGES[value as usize]
    }
}
