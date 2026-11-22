use bytemuck::{
    NoUninit, ByteHash, ByteEq, Zeroable,
    CheckedBitPattern,
};

use tuhka::vk;

use crate::*;

/// Specifies polygon front-facing orientation.
///
/// Default is [`Ccw`][Self::Ccw].
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum FrontFace {
    /// Specifies that vertices that wind counterclockwise are considered front-facing.
    ///
    /// This is the default value.
    #[default]
    Ccw = 0,
    /// Specifies that vertices that wind clockwise are considered front-facing.
    Cw = 1,
}
unsafe impl NoUninit for FrontFace {}
impl From<FrontFace> for vk::FrontFace {

    #[inline]
    fn from(value: FrontFace) -> Self {
        Self::from_raw(value as i32)
    }
}
/// An enumeration of [pipeline states][crate::pipeline_state].
#[repr(i32)]
#[derive(Clone, Copy, Debug, CheckedBitPattern, Zeroable, ByteEq, ByteHash)]
pub enum PipelineStateKind {
    /// The [`kind`][1] of [`LineWidth`][pipeline_state::LineWidth].
    ///
    /// [1]: PipelineStateObject::KIND
    LineWidth = 0,
    /// The [`kind`][1] of [`DepthBias`][pipeline_state::DepthBias].
    ///
    /// [1]: PipelineStateObject::KIND
    DepthBias,
    /// The [`kind`][1] of [`BlendConstants`][pipeline_state::BlendConstants].
    ///
    /// [1]: PipelineStateObject::KIND
    BlendConstants,
    /// The [`kind`][1] of [`DepthBounds`][pipeline_state::DepthBounds].
    ///
    /// [1]: PipelineStateObject::KIND
    DepthBounds,
    /// The [`kind`][1] of [`StencilCompareMask`][pipeline_state::StencilCompareMask].
    ///
    /// [1]: PipelineStateObject::KIND
    StencilCompareMask,
    /// The [`kind`][1] of [`StencilWriteMask`][pipeline_state::StencilWriteMask].
    ///
    /// [1]: PipelineStateObject::KIND
    StencilWriteMask,
    /// The [`kind`][1] of [`StencilReference`][pipeline_state::StencilReference].
    ///
    /// [1]: PipelineStateObject::KIND
    StencilReference,
    /// The [`kind`][1] of [`CullMode`][pipeline_state::CullMode].
    ///
    /// [1]: PipelineStateObject::KIND
    CullMode,
    /// The [`kind`][1] of [`FrontFace`][pipeline_state::FrontFace].
    ///
    /// [1]: PipelineStateObject::KIND
    FrontFace,
    /// The [`kind`][1] of [`PrimitiveTopology`][pipeline_state::PrimitiveTopology].
    ///
    /// [1]: PipelineStateObject::KIND
    PrimitiveTopology,
    /// The [`kind`][1], whose state is determined with either by [`VertexInput`] state, or
    /// dynamically by `cmd_bind_vertex_buffers`.
    ///
    /// [1]: PipelineStateObject::KIND
    /// [`VertexInput`]: pipeline_state::VertexInput
    VertexInputBindingStride,
    /// The [`kind`][1] of [`DepthTestEnable`][pipeline_state::DepthTestEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    DepthTestEnable,
    /// The [`kind`][1] of [`DepthWriteEnable`][pipeline_state::DepthWriteEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    DepthWriteEnable,
    /// The [`kind`][1] of [`DepthCompareOp`][pipeline_state::DepthCompareOp].
    ///
    /// [1]: PipelineStateObject::KIND
    DepthCompareOp,
    /// The [`kind`][1] of [`DepthBoundsTestEnable`][pipeline_state::DepthBoundsTestEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    DepthBoundsTestEnable,
    /// The [`kind`][1] of [`StencilTestEnable`][pipeline_state::StencilTestEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    StencilTestEnable,
    /// The [`kind`][1] of [`StencilOp`][pipeline_state::StencilOp].
    ///
    /// [1]: PipelineStateObject::KIND
    StencilOp,
    /// The [`kind`][1] of [`RasterizerDiscardEnable`][pipeline_state::RasterizerDiscardEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    RasterizerDiscardEnable,
    /// The [`kind`][1] of [`DepthBiasEnable`][pipeline_state::DepthBiasEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    DepthBiasEnable,
    /// The [`kind`][1] of [`PrimitiveRestartEnable`][pipeline_state::PrimitiveRestartEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    PrimitiveRestartEnable,
    /// The [`kind`][1] of [`LineStipple`][pipeline_state::LineStipple].
    ///
    /// [1]: PipelineStateObject::KIND
    LineStipple,
    /// The [`kind`][1] of [`VertexInput`][pipeline_state::VertexInput].
    ///
    /// [1]: PipelineStateObject::KIND
    VertexInput,
    /// The [`kind`][1] of [`PatchControlPoints`][pipeline_state::PatchControlPoints].
    ///
    /// [1]: PipelineStateObject::KIND
    PatchControlPoints,
    /// The [`kind`][1] of [`LogicOp`][pipeline_state::LogicOp].
    ///
    /// [1]: PipelineStateObject::KIND
    LogicOp,
    /// The [`kind`][1] of [`TessellationDomainOrigin`][pipeline_state::TessellationDomainOrigin].
    ///
    /// [1]: PipelineStateObject::KIND
    TessellationDomainOrigin,
    /// The [`kind`][1] of [`DepthClampEnable`][pipeline_state::DepthClampEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    DepthClampEnable,
    /// The [`kind`][1] of [`PolygonMode`][pipeline_state::PolygonMode].
    ///
    /// [1]: PipelineStateObject::KIND
    PolygonMode,
    /// The [`kind`][1] of [`RasterizationSamples`][pipeline_state::RasterizationSamples].
    ///
    /// [1]: PipelineStateObject::KIND
    RasterizationSamples,
    /// The [`kind`][1] of [`SampleMask`][pipeline_state::SampleMask].
    ///
    /// [1]: PipelineStateObject::KIND
    SampleMask,
    /// The [`kind`][1] of [`AlphaToCoverageEnable`][pipeline_state::AlphaToCoverageEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    AlphaToCoverageEnable,
    /// The [`kind`][1] of [`AlphaToOneEnable`][pipeline_state::AlphaToOneEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    AlphaToOneEnable,
    /// The [`kind`][1] of [`LogicOpEnable`][pipeline_state::LogicOpEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    LogicOpEnable,
    /// The [`kind`][1] of [`ColorBlendEnable`][pipeline_state::ColorBlendEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    ColorBlendEnable,
    /// The [`kind`][1] of [`ColorBlendEquations`][pipeline_state::ColorBlendEquations].
    ///
    /// [1]: PipelineStateObject::KIND
    ColorBlendEquations,
    /// The [`kind`][1] of [`ColorWriteMask`][pipeline_state::ColorWriteMask].
    ///
    /// [1]: PipelineStateObject::KIND
    ColorWriteMask,
    /// The [`kind`][1] of [`LineRasterizationMode`][pipeline_state::LineRasterizationMode].
    ///
    /// [1]: PipelineStateObject::KIND
    LineRasterizationMode,
    /// The [`kind`][1] of [`LineStippleEnable`][pipeline_state::LineStippleEnable].
    ///
    /// [1]: PipelineStateObject::KIND
    LineStippleEnable,
}
unsafe impl NoUninit for PipelineStateKind {}
impl PipelineStateKind {

    /// The number of variants in this enum.
    pub const COUNT: usize = Self::LineStippleEnable as usize + 1;
}
impl From<PipelineStateKind> for vk::DynamicState {
    
    #[inline]
    fn from(value: PipelineStateKind) -> Self {
        static STATES: [vk::DynamicState; PipelineStateKind::COUNT] = {
            use PipelineStateKind::*;
            let mut states = [vk::DynamicState::from_raw(0); PipelineStateKind::COUNT];
            states[LineWidth as usize]                  = vk::DynamicState::LINE_WIDTH;
            states[DepthBias as usize]                  = vk::DynamicState::DEPTH_BIAS;
            states[BlendConstants as usize]             = vk::DynamicState::BLEND_CONSTANTS;
            states[DepthBounds as usize]                = vk::DynamicState::DEPTH_BOUNDS;
            states[StencilCompareMask as usize]         = vk::DynamicState::STENCIL_COMPARE_MASK;
            states[StencilWriteMask as usize]           = vk::DynamicState::STENCIL_WRITE_MASK;
            states[StencilReference as usize]           = vk::DynamicState::STENCIL_REFERENCE;
            states[CullMode as usize]                   = vk::DynamicState::CULL_MODE;
            states[FrontFace as usize]                  = vk::DynamicState::FRONT_FACE;
            states[PrimitiveTopology as usize]          = vk::DynamicState::PRIMITIVE_TOPOLOGY;
            states[VertexInputBindingStride as usize]   = vk::DynamicState::VERTEX_INPUT_BINDING_STRIDE;
            states[DepthTestEnable as usize]            = vk::DynamicState::DEPTH_TEST_ENABLE;
            states[DepthWriteEnable as usize]           = vk::DynamicState::DEPTH_WRITE_ENABLE;
            states[DepthCompareOp as usize]             = vk::DynamicState::DEPTH_COMPARE_OP;
            states[DepthBoundsTestEnable as usize]      = vk::DynamicState::DEPTH_BOUNDS_TEST_ENABLE;
            states[StencilTestEnable as usize]          = vk::DynamicState::STENCIL_TEST_ENABLE;
            states[StencilOp as usize]                  = vk::DynamicState::STENCIL_OP;
            states[RasterizerDiscardEnable as usize]    = vk::DynamicState::RASTERIZER_DISCARD_ENABLE;
            states[DepthBiasEnable as usize]            = vk::DynamicState::DEPTH_BIAS_ENABLE;
            states[PrimitiveRestartEnable as usize]     = vk::DynamicState::PRIMITIVE_RESTART_ENABLE;
            states[LineStipple as usize]                = vk::DynamicState::LINE_STIPPLE_EXT;
            states[VertexInput as usize]                = vk::DynamicState::VERTEX_INPUT_EXT;
            states[PatchControlPoints as usize]         = vk::DynamicState::PATCH_CONTROL_POINTS_EXT;
            states[LogicOp as usize]                    = vk::DynamicState::LOGIC_OP_EXT;
            states[TessellationDomainOrigin as usize]   = vk::DynamicState::TESSELLATION_DOMAIN_ORIGIN_EXT;
            states[DepthClampEnable as usize]           = vk::DynamicState::DEPTH_CLAMP_ENABLE_EXT;
            states[PolygonMode as usize]                = vk::DynamicState::POLYGON_MODE_EXT;
            states[RasterizationSamples as usize]       = vk::DynamicState::RASTERIZATION_SAMPLES_EXT;
            states[SampleMask as usize]                 = vk::DynamicState::SAMPLE_MASK_EXT;
            states[AlphaToCoverageEnable as usize]      = vk::DynamicState::ALPHA_TO_COVERAGE_ENABLE_EXT;
            states[AlphaToOneEnable as usize]           = vk::DynamicState::ALPHA_TO_ONE_ENABLE_EXT;
            states[LogicOpEnable as usize]              = vk::DynamicState::LOGIC_OP_ENABLE_EXT;
            states[ColorBlendEnable as usize]           = vk::DynamicState::COLOR_BLEND_ENABLE_EXT;
            states[ColorBlendEquations as usize]         = vk::DynamicState::COLOR_BLEND_EQUATION_EXT;
            states[ColorWriteMask as usize]             = vk::DynamicState::COLOR_WRITE_MASK_EXT;
            states[LineRasterizationMode as usize]      = vk::DynamicState::LINE_RASTERIZATION_MODE_EXT;
            states[LineStippleEnable as usize]          = vk::DynamicState::LINE_STIPPLE_ENABLE_EXT;
            states
        };
        STATES[value as usize]
    }
}

/// Specifies primitive topology.
///
/// The default value is [`TriangleList`][Self::TriangleList].
#[repr(i32)]
#[derive(Default, Clone, Copy, Debug, Zeroable, ByteEq, ByteHash)]
pub enum PrimitiveTopology {
    /// Separate point primitives.
    PointList = 0,
    /// Separate line primitives.
    LineList = 1,
    /// Connected line primitives with consecutive lines sharing a vertex.
    LineStrip = 2,
    /// Separate triangles.
    ///
    /// The default value.
    #[default]
    TriangleList = 3,
    /// Connected triangle primitives with consecutive triangles sharing an edge.
    TriangleStrip = 4,
    /// Connected triangle primitives with all triangles sharing a common vertex.
    TriangleFan = 5,
    /// Separate line primitives with adjacency.
    LineListWithAdj = 6,
    /// Connected line primitives with adjacency with consecutive primitives sharing three vertices.
    LineStripWithAdj = 7,
    /// Separate triangle primitives with adjacency.
    TriangleListWithAdj = 8,
    /// Separate triangle primitives with adjacency with consecutive triangles sharing an edge.
    TriangleStripWithAdj = 9,
    /// Separate patch primitives.
    PatchList = 10,
}
unsafe impl NoUninit for PrimitiveTopology {}
impl From<PrimitiveTopology> for vk::PrimitiveTopology {

    #[inline]
    fn from(value: PrimitiveTopology) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Specifies a comparison operator.
///
/// The default value is [`Never`][Self::Never]
#[repr(i32)]
#[derive(Default, Clone, Copy, Debug, Zeroable, ByteEq, ByteHash)]
pub enum CompareOp {
    /// Comparison always evaluates false.
    ///
    /// This is the default value.
    #[default]
    Never = 0,
    /// Comparison evaluates `reference < test`.
    Lt = 1,
    /// Comparison evaluates `reference == test`.
    Eq = 2,
    /// Comparison evaluates `reference <= test`.
    Le = 3,
    /// Comparison evaluates `reference > test`.
    Gt = 4,
    /// Comparison evaluates `reference != test`.
    Ne = 5,
    /// Comparison evaluates `reference >= test`.
    Ge = 6,
    /// Comparison always evaluates true.
    Always = 7,
}
unsafe impl NoUninit for CompareOp {}
impl From<CompareOp> for vk::CompareOp {

    #[inline]
    fn from(value: CompareOp) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Specifies a stencil comparison operator.
///
/// The default value is [`Keep`][Self::Keep].
#[repr(i32)]
#[derive(Default, Clone, Copy, Debug, Zeroable, ByteEq, ByteHash)]
pub enum StencilOp {
    /// Keeps the current value.
    ///
    /// This is the default value.
    #[default]
    Keep = 0,
    /// Sets the value to zero.
    Zero = 1,
    /// Sets the value to `reference`.
    Replace = 2,
    /// Increments the value and clamps it to the maximum representable unsigned value.
    SaturatingIncrement = 3,
    /// Decrements the value and clamps it to zero.
    SaturatingDecrement = 4,
    /// Applies bitwise-not to the current value.
    Invert = 5,
    /// Increments the current value and wraps to 0 if the current value is the maximum
    /// representable unsigned value.
    WrappingIncrement = 6,
    /// Decrements the current value and wraps to to the maximum representable unsigned value if the
    /// current value is zero.
    WrappingDecrement = 7,
}
unsafe impl NoUninit for StencilOp {}
impl From<StencilOp> for vk::StencilOp {

    #[inline]
    fn from(value: StencilOp) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Specifies at which rate vertex attributes are pulled from buffers.
///
/// The default value is [`Vertex`][Self::Vertex].
#[repr(i32)]
#[derive(Default, Clone, Copy, Debug, Zeroable, ByteEq, ByteHash)]
pub enum VertexInputRate {
    /// Specifies that attributes are pulled per vertex.
    ///
    /// This is the default value.
    #[default]
    Vertex = 0,
    /// Specifies that attributes are pulled per instance.
    Instance = 1,
}
unsafe impl NoUninit for VertexInputRate {}
impl From<VertexInputRate> for vk::VertexInputRate {

    #[inline]
    fn from(value: VertexInputRate) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Represents a single [`SampleCounts`] bit.
///
/// The default value is 1 sample per pixel.
#[repr(u32)]
#[derive(Default, Clone, Copy, CheckedBitPattern, ByteEq, ByteHash)]
pub enum SampleCountBit {
    /// 1 sample per pixel.
    ///
    /// This is the default value.
    #[default]
    X1  = 0x1,
    /// 2 samples per pixel.
    X2  = 0x2,
    /// 4 samples per pixel.
    X4  = 0x4,
    /// 8 samples per pixel.
    X8  = 0x8,
    /// 16 samples per pixel.
    X16 = 0x10,
    /// 32 samples per pixel.
    X32 = 0x20,
    /// 64 samples per pixel.
    X64 = 0x40,
}
unsafe impl NoUninit for SampleCountBit {}
impl SampleCountBit {
    /// Converts `self` to [`SampleCounts`]
    #[inline]
    pub const fn to_flags(self) -> SampleCounts {
        SampleCounts::from_bits_retain(self as u32)
    }
    /// Extracts the highest supported sample count from [`SampleCounts`].
    #[inline]
    pub fn highest_count(supported: SampleCounts) -> Option<Self> {
        bytemuck::checked::try_cast::<SampleCounts, Self>(supported.iter().last()?).ok()
    }
}
impl From<SampleCountBit> for vk::SampleCountFlags {

    #[inline]
    fn from(value: SampleCountBit) -> Self {
        Self::from_raw(value as u32)
    }
}
/// Specifies a resolve mode for a [`ResolveAttachment`].
#[repr(u32)]
#[derive(Clone, Copy, CheckedBitPattern, ByteEq, ByteHash, Debug)]
pub enum ResolveMode {
    /// Specifies the resolve operation is equal to the value of sample `0`.
    Zero = 0x1,
    /// Specifies that the resolve operation is the average of the sample values.
    Average = 0x2,
    /// Specifies that the resolve operation is the minimum of the sample values.
    Min = 0x4,
    /// Specifies that the resolve operation is the maximum of the sample values.
    Max = 0x8,
}
unsafe impl NoUninit for ResolveMode {}
impl ResolveMode {
    /// Converts `self` to [`ResolveModeFlags`].
    #[inline]
    pub const fn to_flags(self) -> ResolveModeFlags {
        ResolveModeFlags::from_bits_retain(self as u32)
    }
}
impl From<ResolveMode> for vk::ResolveModeFlags {
    #[inline]
    fn from(value: ResolveMode) -> Self {
        Self::from_raw(value as u32)
    }
}
/// A logical operation used for a blend state.
///
/// The default value is [`Clear`][Self::Clear].
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum LogicOp {
    /// Performs the operation `0`.
    ///
    /// This is the default value.
    #[default]
    Clear = 0,
    /// Performs the operation `output & current`.
    And = 1,
    /// Performs the operation `output & !current`.
    AndNot = 2,
    /// Performs the operation `output`.
    Copy = 3,
    /// Performs the operation `!output & current`.
    NotAnd = 4,
    /// Performs the operation `current`.
    Nop = 5,
    /// Performs the operation `output ^ current`.
    Xor = 6,
    /// Performs the operation `!(output | current)`.
    Nor = 8,
    /// Performs the operation `!(output ^ current)`.
    Equivalent = 9,
    /// Performs the operation `!current`.
    Not = 10,
    /// Performs the operation `output | !current`.
    OrNot = 11,
    /// Performs the operation `!output`.
    CopyNot = 12,
    /// Performs the operation `!output | current`.
    NotOr = 13,
    /// Performs the operation `!(output & current)`.
    Nand = 14,
    /// Performs the operation `!0`.
    Set = 15,
}
unsafe impl NoUninit for LogicOp {}
impl From<LogicOp> for vk::LogicOp {

    #[inline]
    fn from(value: LogicOp) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Describes tessellation domain origin.
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum TessellationDomainOrigin {
    /// Specifies that the origin of the domain space is upper left corner.
    ///
    /// This is the default value.
    #[default]
    UpperLeft = 0,
    /// Specifies that the origin of the domain space is lower left corner.
    LowerLeft = 1,
}
unsafe impl NoUninit for TessellationDomainOrigin {}
impl From<TessellationDomainOrigin> for vk::TessellationDomainOrigin {

    #[inline]
    fn from(value: TessellationDomainOrigin) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Controls polygon rasterization mode.
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum PolygonMode {
    /// Specifies that polygons are rendered normally.
    ///
    /// This is the default value.
    #[default]
    Fill = 0,
    /// Specifies that polygon edges are drawn as line segments.
    Line = 1,
    /// Specifies that polygon vertces are drawn as points.
    Point = 2,
}
unsafe impl NoUninit for PolygonMode {}
impl From<PolygonMode> for vk::PolygonMode {

    #[inline]
    fn from(value: PolygonMode) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Determines blending factors.
///
/// The blend factors are described in each variant, where:
///- `src0.color` and `src0.alpha` represent the first source's color components and alpha 
///  component,
///- `src1.color` and `src1.alpha` represent the second source's color components and alpha
///  component (dual source blending),
///- `dst.color` and `dst.alpha` represent the destination's color components and alpha
///  component, and
///- `c.color` and `c.alpha` represent the blend constant's color components and alpha
///  component.
///  The default value is [`Zero`][Self::Zero].
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum BlendFactor {
    /// `0` for all components.
    ///
    /// This is the default value.
    #[default]
    Zero = 0,
    /// `1` for all components.
    One = 1,
    /// `src0.color` color components and `src0.alpha` alpha component.
    SrcColor = 2,
    /// `1 - src0.color` cokor components and `1 - src0.alpha` alpha component.
    OneMinusSrcColor = 3,
    /// `dst.color` color components and `dst.alpha` alpha component.
    DstColor = 4,
    /// `1 - dst.color` color components and `1 - dst.alpha` alpha component.
    OneMinusDstColor = 5,
    /// `src.alpha` for all components.
    SrcAlpha = 6,
    /// `1 - src.alpha` for all components.
    OneMinusSrcAlpha = 7,
    /// `dst.alpha` for all components.
    DstAlpha = 8,
    /// `1 - dst.alpha` for all components.
    OneMinusDstAlpha = 9,
    /// `c.color` color components and `c.alpha` alpha component.
    ConstColor = 10,
    /// `1 - c.color` color components and `1 - c.alpha` alpha component.
    OneMinusConstColor = 11,
    /// `c.alpha` for all components.
    ConstAlpha = 12,
    /// `1 - c.alpha` for all components.
    OneMinusConstAlpha = 13,
    /// `src0.alpha.min(1 - dst.alpha)` for all components
    AlphaSaturate = 14,
    /// `src1.color` for color components and `src1.alpha` for alpha component.
    Src1Color = 15,
    /// `1- src1.color` for color components and `1 - src1.alpha` for alpha component.
    OneMinusSrc1Color = 16,
    /// `src1.alpha` for all components.
    Src1Alpha = 17,
    /// `1 - src1.alpha` for all components.
    OneMinusSrc1Alpha = 18,
}
unsafe impl NoUninit for BlendFactor {}
impl BlendFactor {
    /// Returns whether the blend factor uses the second source components (variants containing
    /// `Src1`).
    /// 
    /// If this returns `true`, the [`dual_src_blend`][1] feature **must** be enabled and the
    /// maximum number of color attachments in the pipeline **must** be less than or equal to
    /// [`max_fragment_dual_src_attachments`][2].
    ///
    /// [1]: vk_10::dual_src_blend
    /// [2]: GpuLimits::max_fragment_dual_src_attachments
    #[inline]
    pub fn uses_second_source(self) -> bool {
        const FIRST: i32 = BlendFactor::Src1Color as i32;
        const LAST: i32 = BlendFactor::OneMinusSrc1Alpha as i32;
        matches!(self as i32, FIRST..=LAST)
    }
}
impl From<BlendFactor> for vk::BlendFactor {
    #[inline]
    fn from(value: BlendFactor) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Determines blending operation.
///
/// The default value is [`Add`][Self::Add].
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum BlendOp {
    /// `src0 * src_blend_factor + dst * dst_blend_factor`
    #[default]
    Add = 0,
    /// `src0 * src_blend_factor - dst * dst_blend_factor`
    Sub = 1,
    /// `dst * dst_blend_factor - src0 * src_blend_factor`
    RevSub = 2,
    /// `src0.min(dst)`
    Min = 3,
    /// `src0.max(dst)`
    Max = 4,
}
unsafe impl NoUninit for BlendOp {}
impl From<BlendOp> for vk::BlendOp {
    #[inline]
    fn from(value: BlendOp) -> Self {
        Self::from_raw(value as i32)
    }
}
/// The type stored in an index buffer.
///
/// The default value is [`U16`][Self::U16].
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum IndexType {
    /// Specifies that indices are `u16` values.
    ///
    /// This is the default value.
    #[default]
    U16 = 0,
    /// Specifies that indices are `u32` values.
    U32 = 1,
    /// Specifies that indices are `u8` values.
    ///
    /// Using this requires the [`index_type_uint8`][1] feature.
    ///
    /// [1]: vk_14::index_type_uint8
    U8 = 1000265000,
}
unsafe impl NoUninit for IndexType {}
impl From<IndexType> for vk::IndexType {
    #[inline]
    fn from(value: IndexType) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Specifies the behaviour of sampling an image outside the image.
///
/// The default value is [`Repeat`][Self::Repeat]
///
/// For more detail about each wrapping mode, see the [Wrapping Operation] section of the Vulkan
/// docs.
///
/// [Wrapping Operation]: https://docs.vulkan.org/spec/latest/chapters/textures.html#textures-wrapping-operation
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum SamplerWrappingMode {
    /// Specifies that the repeat wrap mode is used.
    ///
    /// This is the default value.
    #[default]
    Repeat = 0,
    /// Specifies that the mirrored repeat wrap mode is used.
    MirroredRepeat = 1,
    /// Specifies that the clamp to edge wrap mode will be used.
    ClampToEdge = 2,
    /// Specifies that the clamp to border wrap mode will be used.
    ClampToBorder = 3,
    /// Specifies that the mirror clamp to edge wrap mode will be used.
    MirrorClampToEdge = 4,
}
unsafe impl NoUninit for SamplerWrappingMode {}
impl From<SamplerWrappingMode> for vk::SamplerAddressMode {
    #[inline]
    fn from(value: SamplerWrappingMode) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Represents number of image dimensions.
#[repr(i32)]
#[derive(Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum ImageDim {
    /// 1-dimensional image.
    Dim1 = 0,
    /// 2-dimensional image.
    Dim2 = 1,
    /// 3-dimensional image.
    Dim3 = 2,
}
unsafe impl NoUninit for ImageDim {}
impl From<ImageDim> for vk::ImageType {
    #[inline]
    fn from(value: ImageDim) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Represents number of image view dimensions and whether the view is arrayed.
#[repr(i32)]
#[derive(Clone, Copy, Zeroable, ByteEq, ByteHash, Debug)]
pub enum ImageViewDim {
    /// 1-dimensional view.
    Dim1 = 0,
    /// 2-dimensional view.
    Dim2 = 1,
    /// 3-dimensional view.
    Dim3 = 2,
    /// Cube view.
    Cube = 3,
    /// 1-dimensional arrayed view.
    Dim1Array = 4,
    /// 2-dimensional arrayed view.
    Dim2Array = 5,
    /// Arrayed cube view.
    CubeArray = 6,
}
unsafe impl NoUninit for ImageViewDim {}
impl From<ImageViewDim> for vk::ImageViewType {
    #[inline]
    fn from(value: ImageViewDim) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Specifies filters used for sampling operations.
///
/// The default value is [`Nearest`][Self::Nearest].
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash, Debug)]
pub enum SamplerFilter {
    /// Specify nearest filtering.
    ///
    /// This is the default value.
    #[default]
    Nearest = 0,
    /// Specify linear filtering.
    Linear = 1,
}
unsafe impl NoUninit for SamplerFilter {}
impl From<SamplerFilter> for vk::Filter {
    #[inline]
    fn from(value: SamplerFilter) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Specifies mipmap mode used for sampling operations.
///
/// The default value is [`Nearest`][Self::Nearest].
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash, Debug)]
pub enum SamplerMipmapMode {
    /// Specifies nearest filtering.
    ///
    /// This is the default value.
    #[default]
    Nearest = 0,
    /// Specifies linear filtering.
    Linear = 1,
}
unsafe impl NoUninit for SamplerMipmapMode {}
impl From<SamplerMipmapMode> for vk::SamplerMipmapMode {
    #[inline]
    fn from(value: SamplerMipmapMode) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Specifies border color used in sampling operations.
///
/// The default value is [`FloatTransparentBlack`][Self::FloatTransparentBlack].
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum SamplerBorderColor {
    /// A transparent black color in floating-point format.
    ///
    /// This is the default value.
    #[default]
    FloatTransparentBlack = 0,
    /// A transparent black color in integer format.
    IntTransparentBlack = 1,
    /// An opaque black color in floating-point format.
    FloatOpaqueBlack = 2,
    /// An opaque black color in integer format.
    IntOpaqueBlack = 3,
    /// An opaque white color in floating-point format.
    FloatOpaqueWhite = 4,
    /// An opaque white color in integer format.
    IntOpaqueWhite = 5,
}
unsafe impl NoUninit for SamplerBorderColor {}
impl From<SamplerBorderColor> for vk::BorderColor {
    #[inline]
    fn from(value: SamplerBorderColor) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Specifies an [`ImageView`] component swizzle.
///
/// The default value is [`Identity`][Self::Identity].
#[repr(i32)]
#[derive(Default, Clone, Copy, Zeroable, ByteEq, ByteHash)]
pub enum ComponentSwizzle {
    /// Specifies that the component swizzles its self.
    ///
    /// This is the default value.
    #[default]
    Identity = 0,
    /// Specifies that the component is set to zero.
    Zero = 1,
    /// Specifies that the component is set to `1` or `1.0` depending on the format of the image
    /// view.
    One = 2,
    /// Specifies that the component swizzles the `R` component.
    R = 3,
    /// Specifies that the component swizzles the `G` component.
    G = 4,
    /// Specifies that the component swizzles the `B` component.
    B = 5,
    /// Specifies that the component swizzles the alpha-component.
    A = 6,
}
unsafe impl NoUninit for ComponentSwizzle {}
impl From<ComponentSwizzle> for vk::ComponentSwizzle {
    #[inline]
    fn from(value: ComponentSwizzle) -> Self {
        Self::from_raw(value as i32)
    }
}
/// Specifies a single selected bit of [`CompositeAlphaFlags`] bits.
#[repr(u32)]
#[derive(Clone, Copy, ByteEq, ByteHash, Debug)]
pub enum CompositeAlphaBit {
    /// Specifies that the [`OPAQUE`] bit is selected.
    ///
    /// [`OPAQUE`]: CompositeAlphaFlags::OPAQUE
    Opaque = 0x1,
    /// Specifies that the [`PRE_MULTIPLIED`] bit is selected.
    ///
    /// [`PRE_MULTIPLIED`]: CompositeAlphaFlags::PRE_MULTIPLIED
    PreMultiplied = 0x2,
    /// Specifies that the [`POST_MULTIPLIED`] bit is selected.
    ///
    /// [`POST_MULTIPLIED`]: CompositeAlphaFlags::POST_MULTIPLIED
    PostMultiplied = 0x4,
    /// Specifies that the [`INHERIT`] bit is selected.
    ///
    /// [`INHERIT`]: CompositeAlphaFlags::INHERIT
    Inherit = 0x8,
}
unsafe impl NoUninit for CompositeAlphaBit {}
impl From<CompositeAlphaBit> for vk::CompositeAlphaFlagsKHR {
    #[inline]
    fn from(value: CompositeAlphaBit) -> Self {
        Self::from_raw(value as u32)
    }
}
/// Specifies an attachment load operation for attachments in [`RenderPassAttributes`].
#[repr(i32)]
#[derive(Clone, Copy, ByteEq, ByteHash, Debug)]
pub enum AttachmentLoadOp {
    /// Specifies that the previous contents of the image within the [render area] will be preserved
    /// as the initial values.
    ///
    /// [render area]: RenderArea
    Load     = 0,
    /// Specifies that the contents within the [render area] will be cleared to a uniform value, which
    /// is specified when a [render pass instance is begun].
    ///
    /// [render pass instance is begun]: CommandBufferRecorder::render_pass
    /// [render area]: RenderArea
    Clear    = 1,
    /// Specifies that the previous contents within the area need not be preserved; the contents of
    /// the attachment will be **undefined** inside the [render area].
    ///
    /// [render area]: RenderArea
    DontCare = 2,
}
impl From<AttachmentLoadOp> for vk::AttachmentLoadOp {
    #[inline]
    fn from(value: AttachmentLoadOp) -> Self {
        Self::from_raw(value as i32)
    }
}
unsafe impl NoUninit for AttachmentLoadOp {}
/// Specifies an attachment store operation for attachments in [`RenderPassAttributes`].
#[repr(i32)]
#[derive(Clone, Copy, ByteEq, ByteHash, Debug)]
pub enum AttachmentStoreOp {
    /// Specifies the contents generated during the render pass and within the [render area] are
    /// written to memory.
    ///
    /// [render area]: RenderArea
    Store   = 0,
    /// Specifies the contents within the [render area] are not needed after rendering, and may be
    /// discarded; the contents of the attachment will be undefined inside the [render area].
    ///
    /// [render area]: RenderArea
    DontCare = 1,
}
impl From<AttachmentStoreOp> for vk::AttachmentStoreOp {
    #[inline]
    fn from(value: AttachmentStoreOp) -> Self {
        Self::from_raw(value as i32)
    }
}
unsafe impl NoUninit for AttachmentStoreOp {}
/// Specifies how lines are rasterized in a pipeline.
///
/// The default value is [`Default`][Self::Default].
#[repr(i32)]
#[derive(Default, Clone, Copy, ByteEq, ByteHash, Debug)]
pub enum LineRasterizationMode {
    /// Equal to [`Rectangular`][Self::Rectangular] if [`GpuLimits::strict_lines`] is true,
    /// otherwise lines are drawn as non-`srict_lines` parallelograms.
    ///
    /// This is the default value.
    #[default]
    Default = 0,
    /// Specifies lines drawn as if they were rectangles extruded from the line.
    ///
    /// # Valid usage
    /// - The [`vk_14::rectangular_lines`] feature **must** be enabled.
    Rectangular = 1,
    /// Specifies lines drawn by determining which pixel diamonds the line intersects and exits.
    ///
    /// Described in detail in [here].
    ///
    /// # Valid usage
    /// - The [`vk_14::bresenham_lines`] feature **must** be enabled.
    ///
    /// [here]: https://docs.vulkan.org/spec/latest/chapters/primsrast.html#primsrast-lines-bresenham
    Bresenham = 2,
    /// Specifies lines drawn as if they were rectangles extruded from the line, with alpha falloff.
    ///
    /// # Valid usage
    /// - The [`vk_14::smooth_lines`] feature **must** be enabled.
    RectangularSmooth = 3,
}
impl From<LineRasterizationMode> for vk::LineRasterizationMode {
    #[inline]
    fn from(value: LineRasterizationMode) -> Self {
        Self::from_raw(value as i32)
    }
}
unsafe impl NoUninit for LineRasterizationMode {}
