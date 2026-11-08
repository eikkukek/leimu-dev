use smallvec::SmallVec;

use crate::*;

use mem::alloc_vec::Vec32;

pub(crate) struct PipelineStateContextDynamic<'a> {
    dynamic_states: &'a HashSet<PipelineStateKind>,
    deny_non_zero_instance: &'a mut Vec32<u32>,
}

impl PipelineStateContextDynamic<'_> {

    #[inline]
    pub fn is_state_dynamic(&self, kind: PipelineStateKind) -> bool {
        self.dynamic_states.contains(&kind)
    }
}

pub(crate) enum PipelineStateContextInner<'a> {
    Dynamic(PipelineStateContextDynamic<'a>),
    Static(),
}

impl<'a> PipelineStateContextInner<'a> {

    #[inline]
    pub(crate) fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic(_))
    }

    #[inline]
    pub(crate) fn is_dynamic_and(&self, f: impl Fn(&PipelineStateContextDynamic) -> bool) -> bool {
        if let Self::Dynamic(d) = self
        {
            f(d)
        } else {
            false
        }
    }

    #[inline]
    pub fn get_dynamic(&self) -> Option<&PipelineStateContextDynamic<'a>> {
        if let Self::Dynamic(d) = self {
            Some(d)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_dynamic_mut(&mut self) -> Option<&mut PipelineStateContextDynamic<'a>> {
        if let Self::Dynamic(d) = self {
            Some(d)
        } else {
            None
        }
    }
}

/// The context in which to check for the validity of [`pipeline_state`].
pub struct PipelineStateContext<'a> {
    inner: PipelineStateContextInner<'a>,
    color_attachment_count: u32,
}

impl<'a> PipelineStateContext<'a> {

    #[inline]
    pub(crate) fn inner(&self) -> &PipelineStateContextInner<'a> {
        &self.inner
    }

    #[inline]
    pub(crate) fn inner_mut(&mut self) -> &mut PipelineStateContextInner<'a> {
        &mut self.inner
    }
}

/// A trait for [`pipeline_state`] objects.
pub trait PipelineStateObject: Default {

    /// The kind of this object.
    const KIND: PipelineStateKind;

    /// Checks the validity of this state in `context`.
    fn check_validity(
        &self,
        device: &Device,
        ctx: &mut PipelineStateContext,
    ) -> Result<(), Error>;

    /// Set this state dynamically
    ///
    /// Panics if allocation with `allocator` fails.
    ///
    /// # Safety
    /// This does not check for validity of usage, just records the command.
    unsafe fn cmd_set_dynamic(
        &self,
        device: &Device,
        command_buffer: CommandBuffer<'_>,
        allocator: &AnyAlloc,
    );
}

macro_rules! state {
    (
        $(#[$meta:meta])+
        pub struct $name:ident {$(
            $(#[doc = $field_doc:literal])+
            #[default($field_default:expr, $field_default_str:expr)]
            pub $field_name:ident: $field_ty:ty,
        )*}

        fn check_validity($self_ref_1:tt $self_1:ident, $($arg_name_1:ident: $arg_ty_1:ty),* $(,)?) -> $fn_ret_1:ty
            $fn_block_1:block

        unsafe fn cmd_set_dynamic($self_ref_2:tt $self_2:ident, $($arg_name_2:ident: $arg_ty_2:ty),*, $(,)?)
            $fn_block_2:block
    ) => {
        $(#[$meta])+
        /// # Default values
        $(
            #[doc = concat!(
                "[`",
                stringify!($field_name),
                "`][Self::",
                stringify!($field_name),
                "]: `",
                $field_default_str,
                "`",
            )]
        )*
        pub struct $name {$(
            $(#[doc = $field_doc])+
            pub $field_name: $field_ty,
        )+}

        impl $name {

            const NAME: &'static str = stringify!($name);
        }

        impl Default for $name {

            fn default() -> Self {
                Self {$(
                    $field_name: $field_default,
                )+}
            }
        }

        impl PipelineStateObject for $name {

            const KIND: PipelineStateKind = PipelineStateKind::$name;

            fn check_validity($self_ref_1 $self_1, $($arg_name_1: $arg_ty_1),*) -> $fn_ret_1
                $fn_block_1

            unsafe fn cmd_set_dynamic($self_ref_2 $self_2, $($arg_name_2: $arg_ty_2),*)
                $fn_block_2
        }
    };
}

macro_rules! enable_state {
    {
        $(#[doc = $doc:literal])+
        pub struct $name:ident {
            $(#[doc = $field_doc:literal])+
            pub value: bool,
        }

        cmd = $($cmd:ident)? $([$($cmd_alt:tt)*])?
    } => {

        state! {
            $(#[doc = $doc])+
            #[repr(C)]
            #[derive(Clone, Copy, NoUninit, ByteHash, ByteEq)]
            pub struct $name {
                $(#[doc = $field_doc])+
                #[default(false, "false")]
                pub value: bool,
            }

            fn check_validity(
                &self,
                _device: &Device,
                ctx: &mut PipelineStateContext,
            ) -> Result<(), Error> {
                if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                    Err(state_not_dynamic_err(Self::NAME))?
                }
                Ok(())
            }

            $(
                unsafe fn cmd_set_dynamic(
                    &self,
                    device: &Device,
                    command_buffer: CommandBuffer<'_>,
                    _allocator: &AnyAlloc,
                )
                {
                    unsafe {
                        device.functions()
                            .$cmd(
                                command_buffer.handle(),
                                self.value,
                            );
                    }
                }
            )?

            $(
                $($cmd_alt)*
            )?
        }
    };
}

fn state_not_dynamic_err(state: &str) -> Error {
    err!(
        "attempting to set the `{state}` state dynamically when it is not dynamic within the pipeline",
    )
}

/// Contains all available [`pipeline state objects`][PipelineStateObject].
pub mod pipeline_state {
    use super::*;

    use core::num::NonZeroU32;

    use bitflags::Flags;

    use bytemuck::{NoUninit, ByteEq, ByteHash};

    use crate::mem::{iter::CollectIn, int::NonZeroInteger, alloc_vec::FixedVec32};

    use tuhka::vk::{StencilFaceFlags, SampleCountFlags};

    use crate::vec::VecWrapper;

    state! {
        /// Specifies the width of rasterized line segments for a pipeline.
        #[repr(C)]
        #[derive(Clone, Copy, NoUninit, ByteEq, ByteHash)]
        pub struct LineWidth {
            /// The width of rasterized line segments.
            ///
            /// # Valid usage
            /// If the [`wide_lines`][1] feature is not enabled, this **must** be set to `1.0`.
            ///
            /// [1]: vk_10::wide_lines
            #[default(1.0, "1.0")]
            pub line_width: f32,
        }


        fn check_validity(&self, device: &Device, ctx: &mut PipelineStateContext) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            if self.line_width != 1.0 &&
                device.get_feature(vk_10::wide_lines::NAME)
                    .is_none_or(|&e| !e)
            {
                Err(err!(
                    "`line_width` is not `1.0` but the `wide_lines` feature is not enabled"
                ))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            unsafe {
                device.functions().cmd_set_line_width(
                    command_buffer.handle(),
                    self.line_width
                );
            }
        }
    }
    state! {
        /// Specifies depth bias factors and clamp.
        #[repr(C)]
        #[derive(Clone, Copy, NoUninit, ByteEq, ByteHash)]
        pub struct DepthBias {
            /// A scalar controlling the constant depth value added to each fragment.
            ///
            /// The default value is `0.0`.
            #[default(0.0, "0.0")]
            pub constant_factor: f32,
            /// The maximum (or minimum) depth bias of a fragemnt.
            ///
            /// The default value is `0.0`.
            ///
            /// # Valid usage
            /// If the [`depth_bias_clamp`][1] feature is not enabled, this **must** be set to
            /// `0.0`.
            ///
            /// [1]: vk_10::depth_bias_clamp
            #[default(0.0, "0.0")]
            pub clamp: f32,
            /// A scalar factor applied to a fragment's slope in depth bias calculations.
            #[default(0.0, "0.0")]
            pub slope_factor: f32,
        }

        fn check_validity(
            &self,
            device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            if self.clamp != 0.0 &&
                device.get_feature(vk_10::depth_bias_clamp::NAME)
                    .is_none_or(|&e| e)
            {
                Err(err!(
                    "`clamp` is not `0.0` and the `depth_bias_clamp` feature is not enabled"
                ))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            unsafe {
                device.functions().cmd_set_depth_bias(
                    command_buffer.handle(),
                    self.constant_factor,
                    self.clamp,
                    self.slope_factor,
                );
            }
        }
    }
    state! {
        /// An array of four values used as the `R`, `G`, `B` and `A` components of the blend
        /// constant that are used in blending, depending on the [`blend factor`][BlendFactor].
        #[repr(C)]
        #[derive(Clone, Copy, NoUninit, ByteEq, ByteHash)]
        pub struct BlendConstants {
            /// The blend constants.
            #[default([0.0; 4], "[0.0; 4]")]
            pub constants: [f32; 4],
        }


        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            unsafe {
                device.functions().cmd_set_blend_constants(
                    command_buffer.handle(),
                    &self.constants,
                );
            }
        }
    }
    state! {
        /// Specifies the minimum and maximum depth bounds used in depth bounds tests.
        #[repr(C)]
        #[derive(Clone, Copy, NoUninit, ByteEq, ByteHash)]
        pub struct DepthBounds {
            /// The minimum depth bound.
            #[default(0.0, "0.0")]
            pub min: f32,
            /// The maximum depth bound.
            #[default(1.0, "1.0")]
            pub max: f32,
        }

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            if !matches!(self.min, 0.0..=1.0) {
                Err(err!(
                    "`min` depth bound `{}` is not in the range `0.0..=1.0",
                    self.min,
                ))?
            }
            if !matches!(self.max, 0.0..=1.0) {
                Err(err!(
                    "`max` depth bound `{}` is not in the range `0.0..=1.0",
                    self.max,
                ))?
            }
            if self.min > self.max {
                Err(err!(
                    "`min` depth bound `{}` is greater than `max` depth bound `{}`",
                    self.min, self.max,
                ))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            unsafe {
                device.functions().cmd_set_depth_bounds(
                    command_buffer.handle(),
                    self.min,
                    self.max,
                );
            }
        }
    }
    state! {
        /// Specifies stencil operation compare mask.
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct StencilCompareMask {
            /// Specifies the compare mask of the front face.
            ///
            /// If this is set dynamically, a [`None`] value means that the compare mask is not
            /// updated for the front facing state. Otherwise, a [`None`] value is the same as
            /// setting the mask to zero.
            #[default(None, "None")]
            pub front: Option<u32>,
            /// Specifies the compare mask of the back face.
            ///
            /// If this is set dynamically, a [`None`] value means that the compare mask is not
            /// updated for the back facing state. Otherwise, a [`None`] value is the same as
            /// setting the mask to zero.
            #[default(None, "None")]
            pub back: Option<u32>,
        }

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            if self.front == self.back &&
                let Some(mask) = self.front
            {
                unsafe {
                    device.functions().cmd_set_stencil_compare_mask(
                        command_buffer.handle(),
                        StencilFaceFlags::FRONT_AND_BACK,
                        mask,
                    );
                }
                return
            }
            if let Some(mask) = self.front {
                unsafe {
                    device.functions().cmd_set_stencil_compare_mask(
                        command_buffer.handle(),
                        StencilFaceFlags::FRONT,
                        mask,
                    );
                }
            }
            if let Some(mask) = self.back {
                unsafe {
                    device.functions().cmd_set_stencil_compare_mask(
                        command_buffer.handle(),
                        StencilFaceFlags::BACK,
                        mask,
                    );
                }
            }
        }
    }
    state! {
        /// Specifies stencil operation write mask.
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct StencilWriteMask {
            /// Specifies the write mask of the front face.
            ///
            /// If this is set dynamically, a [`None`] value means that the write mask is not
            /// updated for the front facing state. Otherwise, a [`None`] value is the same as
            /// setting the mask to zero.
            #[default(None, "None")]
            pub front: Option<u32>,
            /// Specifies the write mask of the back face.
            ///
            /// If this is set dynamically, a [`None`] value means that the write mask is not
            /// updated for the back facing state. Otherwise, a [`None`] value is the same as
            /// setting the mask to zero.
            #[default(None, "None")]
            pub back: Option<u32>,
        }

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            if self.front == self.back &&
                let Some(mask) = self.front
            {
                unsafe {
                    device.functions().cmd_set_stencil_write_mask(
                        command_buffer.handle(),
                        StencilFaceFlags::FRONT_AND_BACK,
                        mask,
                    );
                }
                return
            }
            if let Some(mask) = self.front {
                unsafe {
                    device.functions().cmd_set_stencil_write_mask(
                        command_buffer.handle(),
                        StencilFaceFlags::FRONT,
                        mask,
                    );
                }
            }
            if let Some(mask) = self.back {
                unsafe {
                    device.functions().cmd_set_stencil_write_mask(
                        command_buffer.handle(),
                        StencilFaceFlags::BACK,
                        mask,
                    );
                }
            }
        }
    }
    state! {
        /// Specifies stencil operation reference value that is used in unsigned stencil comparison.
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct StencilReference {
            /// Specifies the reference of the front face.
            ///
            /// If this is set dynamically, a [`None`] value means that the reference is not
            /// updated for the front facing state. Otherwise, a [`None`] value is the same as
            /// setting the mask to zero.
            #[default(None, "None")]
            pub front: Option<u32>,
            /// Specifies the reference mask of the back face.
            ///
            /// If this is set dynamically, a [`None`] value means that the reference is not
            /// updated for the back facing state. Otherwise, a [`None`] value is the same as
            /// setting the mask to zero.
            #[default(None, "None")]
            pub back: Option<u32>,
        }

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            if self.front == self.back &&
                let Some(reference) = self.front
            {
                unsafe {
                    device.functions().cmd_set_stencil_reference(
                        command_buffer.handle(),
                        StencilFaceFlags::FRONT_AND_BACK,
                        reference,
                    );
                }
                return
            }
            if let Some(reference) = self.front {
                unsafe {
                    device.functions().cmd_set_stencil_reference(
                        command_buffer.handle(),
                        StencilFaceFlags::FRONT,
                        reference,
                    );
                }
            }
            if let Some(reference) = self.back {
                unsafe {
                    device.functions().cmd_set_stencil_reference(
                        command_buffer.handle(),
                        StencilFaceFlags::BACK,
                        reference,
                    );
                }
            }
        }
    }
    pub use crate::CullMode;
    impl PipelineStateObject for CullMode {

        const KIND: PipelineStateKind = PipelineStateKind::CullMode;

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err("CullMode"))?
            }
            let unknown_bits = self.difference(CullMode::all());
            if !unknown_bits.is_empty() {
                Err(err!("unknown cull mode bits `{unknown_bits}` detected"))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            unsafe {
                device.functions()
                    .cmd_set_cull_mode(
                        command_buffer.handle(),
                        (*self).into(),
                    );
            }
        }
    }
    pub use crate::FrontFace;
    impl PipelineStateObject for FrontFace {

        const KIND: PipelineStateKind = PipelineStateKind::FrontFace;

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err("FrontFace"))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            unsafe {
                device.functions()
                    .cmd_set_front_face(
                        command_buffer.handle(),
                        (*self).into(),
                    );
            }
        }
    }
    pub use crate::PrimitiveTopology;

    impl PipelineStateObject for PrimitiveTopology {

        const KIND: PipelineStateKind = PipelineStateKind::PrimitiveTopology;

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err("PrimitiveTopology"))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            unsafe {
                device.functions()
                    .cmd_set_primitive_topology(
                        command_buffer.handle(),
                        (*self).into(),
                    );
            }
        }
    }
    /* Specified with vkCmdBindVertexBuffers2
        pub struct VertexInputBindingStride;
    */
    enable_state! {
        /// Specifies whether depth test is enabled.
        pub struct DepthTestEnable {
            /// Whether to enable depth testing.
            pub value: bool,
        }
        cmd = cmd_set_depth_test_enable
    }
    enable_state! {
        /// Specifies whether depth writes are enabled.
        pub struct DepthWriteEnable {
            /// Whether to enable depth writes.
            pub value: bool,
        }

        cmd = cmd_set_depth_write_enable
    }
    state! {
        /// Specifies depth comparison operator.
        #[repr(C)]
        #[derive(Clone, Copy, NoUninit, ByteHash, ByteEq)]
        pub struct DepthCompareOp {
            /// The operator.
            #[default(CompareOp::Never, "CompareOp::Never")]
            pub op: CompareOp,
        }

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            unsafe {
                device.functions()
                    .cmd_set_depth_compare_op(
                        command_buffer.handle(),
                        self.op.into(),
                    );
            }
        }
    }
    enable_state! {
        /// Specifies whether depth bounds test is enabled.
        pub struct DepthBoundsTestEnable {
            /// Whether to enable depth bounds test.
            pub value: bool,
        }

        cmd = cmd_set_depth_bounds_test_enable
    }
    enable_state! {
        /// Specifies whether stencil test is enabled.
        pub struct StencilTestEnable {
            /// Whether to enable stencil test.
            pub value: bool,
        }

        cmd = cmd_set_stencil_test_enable
    }
    state! {
        /// Specifies stencil operation.
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct StencilOp {
            /// Specifies the stencil operations of the front face.
            ///
            /// If this is set dynamically, a [`None`] value means that the operations are not
            /// updated for the front facing state. Otherwise, a [`None`] value is the same as
            /// setting `front` to the default value of [`StencilOpState`].
            #[default(None, "None")]
            pub front: Option<StencilOpState>,
            /// Specifies the stencil operations of the back face.
            ///
            /// If this is set dynamically, a [`None`] value means that the operations are not
            /// updated for the back facing state. Otherwise, a [`None`] value is the same as
            /// setting `back` to the default value of [`StencilOpState`].
            #[default(None, "None")]
            pub back: Option<StencilOpState>,
        }

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            if self.front == self.back &&
                let Some(op) = self.front
            {
                unsafe {
                    device.functions().cmd_set_stencil_op(
                        command_buffer.handle(),
                        StencilFaceFlags::FRONT_AND_BACK,
                        op.fail_op.into(),
                        op.pass_op.into(),
                        op.depth_fail_op.into(),
                        op.compare_op.into(),
                    );
                }
                return
            }
            if let Some(op) = self.front {
                unsafe {
                    device.functions().cmd_set_stencil_op(
                        command_buffer.handle(),
                        StencilFaceFlags::FRONT,
                        op.fail_op.into(),
                        op.pass_op.into(),
                        op.depth_fail_op.into(),
                        op.compare_op.into(),
                    );
                }
            }
            if let Some(op) = self.back {
                unsafe {
                    device.functions().cmd_set_stencil_op(
                        command_buffer.handle(),
                        StencilFaceFlags::BACK,
                        op.fail_op.into(),
                        op.pass_op.into(),
                        op.depth_fail_op.into(),
                        op.compare_op.into(),
                    );
                }
            }
        }
    }
    enable_state! {
        /// Specifies primitives are discarded before the rasterization stage.
        pub struct RasterizerDiscardEnable {
            /// Whether to enable rasterizer discard.
            pub value: bool,
        }

        cmd = cmd_set_rasterizer_discard_enable
    }
    enable_state! {
        /// Specifies to bias fragment depth values.
        pub struct DepthBiasEnable {
            /// Whether to enable depth bias.
            pub value: bool,
        }

        cmd = cmd_set_depth_bias_enable
    }
    enable_state! {
        /// Specifies whether a special vertex index value is treated as restarting the assembly of primitive.
        pub struct PrimitiveRestartEnable {
            /// Whether to enable primitive restart.
            pub value: bool,
        }

        cmd = cmd_set_primitive_restart_enable
    }
    state! {

        /// Specifies line stipple options.
        #[repr(C)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct LineStipple {
            /// The repeat factor used in stippled line rasterization.
            ///
            /// # Valid usage
            /// This must be in the range `1..256`
            #[default(1, "1")]
            pub factor: u32,
            /// The bit pattern used in stippled line rasterization.
            #[default(0, "0")]
            pub pattern: u16,
        }

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            if !matches!(self.factor, 1..256) {
                Err(err!(
                    "`LineStipple` `factor` is not in the range `1..256`"
                ))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        )
        {
            unsafe {
                device.functions()
                    .cmd_set_line_stipple(
                    command_buffer.handle(),
                    self.factor,
                    self.pattern
                );
            }
        }
    }
    state! {

        /// Specifies the vertex input state of a pipeline.
        #[derive(Clone, PartialEq, Eq, Hash)]
        pub struct VertexInput {
            /// A set of vertex input bindings specifying unique bindings with unique binding
            /// numbers used by [`attributes`][Self::attributes].
            ///
            /// # Valid usage
            /// - Each binding number **must** be unique within the set of bindings.
            #[default(Sorted::new(), "empty set")]
            pub bindings: Sorted<VertexInputBindingDescription>,
            /// A set of vertex input attributes mapping shader input variable locations to
            /// [`bindings`][Self::bindings] and defining the memory layout of those locations.
            ///
            /// # Valid usage
            /// - Each binding number in this set **must** be included in
            ///   [`bindings`][Self::bindings].
            /// - Each `location` in this set **must** be unique.
            #[default(Sorted::new(), "empty set")]
            pub attributes: Sorted<VertexInputAttributeDescription>,
        }

        fn check_validity(
            &self,
            device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            let mut zero_divisor = false;
            let mut any_input_rate_divisor = self.bindings
                .first()
                .is_some_and(|b| { zero_divisor = b.divisor == 0; b.divisor != 1});
            for i in 0..self.bindings.len().saturating_sub(1) {
                let binding = self.bindings[i + 1];
                zero_divisor |= binding.divisor == 0;
                any_input_rate_divisor |= binding.divisor != 1;
                if self.bindings[i].binding == binding.binding {
                    Err(err!(
                        "binding `{}` is repeated", self.bindings[i].binding
                    ))?
                }
            }
            if let Some(attr) = self.attributes.first() &&
                !self.bindings.iter().any(|b| attr.binding == b.binding)
            {
                Err(err!(
                    "attribute with location `{}` references an undefined binding `{}`",
                    attr.location, attr.binding
                ))?
            }
            for i in 0..self.attributes.len().saturating_sub(1) {
                let attr = self.attributes[i + 1];
                if self.attributes[i].location == attr.location {
                    Err(err!(
                        "location `{}` is repeated", self.attributes[i].location,
                    ))?
                }
                if !self.bindings.iter().any(|b| b.binding == attr.binding) {
                    Err(err!(
                        "attribute with location `{}` references an undefined binding `{}`",
                        attr.location, attr.binding
                    ))?
                }
            }
            if self.bindings.len() > device.gpu_info().limits.max_vertex_input_bindings() as usize {
                Err(err!(
                    "number of bindings `{}` is greater than `max_vertex_input_bindings` `{}`",
                    self.bindings.len(), device.gpu_info().limits.max_vertex_input_bindings(),
                ))?
            }
            if self.attributes.len() > device.gpu_info().limits.max_vertex_input_attributes() as usize {
                Err(err!(
                    "number of attributes `{}` is greater than `max_vertex_input_bindings` `{}`",
                    self.attributes.len(), device.gpu_info().limits.max_vertex_input_attributes(),
                ))?
            }
            if any_input_rate_divisor {
                let feature = device
                    .get_feature(vk_14::vertex_attribute_instance_rate_divisor::NAME)
                    .ok_or_else(|| err!(
                        "`divisor` is not `1` and `vertex_attribute_instance_rate_divisor` feature is not enabled"
                    ))?;
                if zero_divisor {
                    if !feature.enabled_features().zero_divisor {
                        Err(err!(
                            "`divisor` is zero and `vertex_attribute_instance_divisor::zero_divisor` feature is not enabled"
                        ))?
                    }
                    if !feature.properties().supports_non_zero_first_instance &&
                        let Some(d) = ctx.inner_mut().get_dynamic_mut()
                    {
                        for b in self.bindings.iter() {
                            if b.divisor == 0 {
                                d.deny_non_zero_instance.push(b.binding);
                            }
                        }
                    }
                } 
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            allocator: &AnyAlloc,
        ) {
            use tuhka::vk;
            let feature = device
                .get_feature(vk_ext::vertex_input_dynamic_state::NAME)
                .unwrap();
            let mut bindings = FixedVec32::with_capacity_in(
                self.bindings.len() as u32,
                allocator,
            ).expect("allocation failed");
            let mut attributes = FixedVec32::with_capacity_in(
                self.attributes.len() as u32,
                allocator,
            ).expect("allocation failed");
            bindings.extend(self.bindings.iter().map(|binding| vk::VertexInputBindingDescription2EXT
                ::default()
                .binding(binding.binding)
                .stride(binding.stride)
                .input_rate(binding.input_rate.into())
                .divisor(binding.divisor)
            ));
            attributes.extend(self.attributes.iter().map(|attr| vk::VertexInputAttributeDescription2EXT
                ::default()
                .location(attr.location)
                .binding(attr.binding)
                .format(attr.format.into())
                .offset(attr.offset)
            ));
            unsafe {
                feature.functions(device)
                    .cmd_set_vertex_input(
                        command_buffer.handle(),
                        &bindings,
                        &attributes
                    );
            }
        }
    }
    state! {

        /// Specifies the number of control points per tessellation patch.
        #[repr(C)]
        #[derive(Clone, Copy, NoUninit, ByteEq, ByteHash)]
        pub struct PatchControlPoints {
            /// The number of control points per patch.
            ///
            /// This **must** be less than or equal to [`max_tessellation_patch_size`][1].
            ///
            /// [1]: GpuLimits::max_tessellation_patch_size
            #[default(NonZeroU32::ONE, "1")]
            pub num_points: NonZeroU32,
        }

        fn check_validity(
            &self,
            device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            if self.num_points.get() > device.gpu_info().limits.max_tessellation_patch_size() {
                Err(err!(
                    "the number of patch control points `{}` is greater than the maximum `{}`",
                    self.num_points, device.gpu_info().limits.max_tessellation_patch_size()
                ))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_13::extended_dynamic_state2::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_patch_control_points(
                        command_buffer.handle(),
                        self.num_points.get()
                    );
            }
        }
    }
    pub use crate::LogicOp;
    impl PipelineStateObject for LogicOp {

        const KIND: PipelineStateKind = PipelineStateKind::LogicOp;

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err("LogicOp"))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_13::extended_dynamic_state2::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_logic_op(
                        command_buffer.handle(),
                        (*self).into()
                    );
            }
        }
    }
    pub use crate::TessellationDomainOrigin;
    impl PipelineStateObject for TessellationDomainOrigin {

        const KIND: PipelineStateKind = PipelineStateKind::TessellationDomainOrigin;

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error>
        {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err("TessellationDomainOrigin"))?
            }
            Ok(())
        }
        
        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_tessellation_domain_origin(
                        command_buffer.handle(),
                        (*self).into()
                    );
            }
        }
    }
    state! {
        /// Specifies whether depth clamping is enabled.
        pub struct DepthClampEnable {
            /// Whether to enable depth clamping.
            ///
            /// If the [`depth_clamp`][1] feature is not enabled, this **must** be `false`.
            ///
            /// [1]: vk_10::depth_clamp
            #[default(false, "false")]
            pub value: bool,
        }

        fn check_validity(
            &self,
            device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            if self.value &&
                device.get_feature(vk_10::depth_clamp::NAME)
                .is_none_or(|&f| !f)
            {
                Err(err!(
                    "`DepthClampEnable` is `true`, but the `depth_clamp` feature is not enabled"
                ))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_depth_clamp_enable(
                        command_buffer.handle(),
                        self.value
                    );
            }
        }
    }
    pub use crate::PolygonMode;
    impl PipelineStateObject for PolygonMode {

        const KIND: PipelineStateKind = PipelineStateKind::PolygonMode;

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err("PolygonMode"))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_polygon_mode(
                        command_buffer.handle(),
                        (*self).into()
                    );
            }
        }
    }
    state! {
        /// Specifies number of samples used in rasterization.
        #[repr(C)]
        #[derive(Clone, Copy, NoUninit, ByteEq, ByteHash)]
        pub struct RasterizationSamples {
            /// The number of samples.
            #[default(SampleCountBit::X1, "SampleCountBit::X1")]
            pub sample_count: SampleCountBit,
        }

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_rasterization_samples(
                        command_buffer.handle(),
                        self.sample_count.into()
                    );
            }
        }
    }
    state! {
        /// Specifies a sample mask, which is compared against the [coverage mask][1] in
        /// multisample rendering.
        ///
        /// If the bit associated with a sample index `i` is set to `0`, the coverage mask bit for `i` is also
        /// set to `0`.
        ///
        /// The sample index `i` associated with a sample mask bit is given by the formula
        /// `i = 32 * w + b` where `w` is an array index into the mask and `b` is the bit number of
        /// the element at index `w`.
        ///
        /// Setting [`mask`][2] to `None` is equivalent to setting the mask to `[u32::MAX; 2]`
        /// (all bits set).
        ///
        /// [1]: https://docs.vulkan.org/spec/latest/chapters/primsrast.html#primsrast-multisampling-coverage-mask
        /// [2]: Self::mask
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct SampleMask {
            /// alksdjkal
            #[default(None, "None")]
            pub mask: Option<[u32; 2]>,
        }

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            unsafe {
                let mask = self.mask.unwrap_or([u32::MAX; 2]);
                feature.functions(device)
                    .cmd_set_sample_mask(
                        command_buffer.handle(),
                        SampleCountFlags::TYPE_64,
                        Some(&mask),
                    );
            }
        }
    }
    state! {
        /// Controls whether a temporary coverage value is generated based on the alpha component of a
        /// fragment’s first color output in multisample rendering.
        ///
        /// This is described in detail in the [`multisample coverage section`][1] of the Vulkan docs.
        ///
        /// [1]: https://docs.vulkan.org/spec/latest/chapters/fragops.html#fragops-covg
        pub struct AlphaToCoverageEnable {
            /// Whether to enable alpha to coverage.
            #[default(false, "false")]
            pub value: bool,
        }

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_alpha_to_coverage_enable(
                        command_buffer.handle(),
                        self.value
                    );
            }
        }
    }
    state! {
        /// Controls whether the alpha component of a fragment’s first color output is replaced
        /// with one in multisample rendering.
        ///
        /// This is described in detail in the [`multisample coverage section`][1] of the Vulkan docs.
        ///
        /// [1]: https://docs.vulkan.org/spec/latest/chapters/fragops.html#fragops-covg
        pub struct AlphaToOneEnable {
            /// Whether to enable alpha to coverage.
            ///
            /// If the [`alpha_to_one`][1] feature is not enabled, this **must** be `false`.
            ///
            /// [1]: vk_10::alpha_to_one
            #[default(false, "false")]
            pub value: bool,
        }

        fn check_validity(
            &self,
            device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            if self.value &&
                device.get_feature(vk_10::alpha_to_one::NAME)
                .is_none_or(|&v| !v)
            {
                Err(err!(
                    "`AlphaToOneEnable` is `true` but the `alpha_to_one` feature is not enabled."
                ))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_alpha_to_coverage_enable(
                        command_buffer.handle(),
                        self.value
                    );
            }
        }
    }
    state! {
        /// Specifies whether to apply logical operations instead of regular blending in rendering.
        ///
        /// If this is enabled, [`LogicOp`] specifies an operation between a fragment's color value
        /// and the existing value in the color attachment.
        pub struct LogicOpEnable {
            /// Whether to enable logical operations.
            ///
            /// If the [`logic_op`][1] feature is not enabled, this **must** be set to `false`.
            ///
            /// [1]: vk_10::logic_op
            #[default(false, "false")]
            pub value: bool,
        }

        fn check_validity(
            &self,
            device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err(Self::NAME))?
            }
            if self.value &&
                device.get_feature(vk_10::logic_op::NAME)
                .is_none_or(|&v| !v)
            {
                Err(err!(
                    "`LogicOpEnable` is `true` but the `logic_op` feature is not enabled."
                ))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_logic_op_enable(
                        command_buffer.handle(),
                        self.value
                    );
            }
        }
    }
    #[doc(hidden)]
    pub struct ColorBlendEnableMarker;
    /// Specifies whether to enable color blending for each color attachment.
    ///
    /// # Valid usage
    /// - The length of this state **must** be equal to th number of color attachments in the
    ///   pipeline.
    pub type ColorBlendEnable = VecWrapper<SmallVec<[bool; 8]>, ColorBlendEnableMarker>;
    impl PipelineStateObject for ColorBlendEnable {

        const KIND: PipelineStateKind = PipelineStateKind::ColorBlendEnable;

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error> {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err("ColorBlendEnable"))?
            }
            if self.len() as u32 != ctx.color_attachment_count
            {
                Err(err!(
                    "`ColorBlendEnable` number of values `{}` is not equal to pipeline color attachment count `{}`",
                    self.len(), ctx.color_attachment_count,
                ))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            allocator: &AnyAlloc,
        ) {
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            let values: FixedVec32<_, _> = self
                .iter()
                .map(|&v| v as u32)
                .collect_in(allocator)
                .expect("allocation failed");
            unsafe {
                feature.functions(device)
                    .cmd_set_color_blend_enable(
                        command_buffer.handle(),
                        0,
                        &values,
                    );
            }
        }
    }
    /// Specifies attachment color blend equations for a pipeline.
    pub type ColorBlendEquations = VecWrapper<SmallVec<[ColorBlendEquation; 2]>>;
    impl PipelineStateObject for ColorBlendEquations {

        const KIND: PipelineStateKind = PipelineStateKind::ColorBlendEquation;

        fn check_validity(
            &self,
            device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error>
        {
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err("ColorBlendEquation"))?
            }
            if self.len() as u32 != ctx.color_attachment_count
            {
                Err(err!(
                    "`ColorBlendEquations` number of values `{}` is not equal to pipeline color attachment count `{}`",
                    self.len(), ctx.color_attachment_count,
                ))?
            }
            if self.iter().any(|eq| eq.uses_second_source()) 
            {
                if device.get_feature(vk_10::dual_src_blend::NAME)
                    .is_none_or(|&e| !e)
                {
                    Err(err!("`ColorBlendEquations` uses dual source blending, but the `dual_src_blend` feature is not enabled"))?
                }
                if ctx.color_attachment_count > device.gpu_info().limits.max_fragment_dual_src_attachments() {
                    Err(err!(
                        "`ColorBlendEquations` uses dual source blending and color attachment count `{}` {} `{}`",
                        self.len(),
                        "is greater than `max_fragment_dual_src_attachments`",
                        device.gpu_info().limits.max_fragment_dual_src_attachments(),
                    ))?
                } 
            }
            Ok(())
        }
        
        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            use tuhka::vk::ColorBlendEquationEXT;
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_color_blend_equation(
                        command_buffer.handle(),
                        0,
                        bytemuck::cast_slice::<ColorBlendEquation, ColorBlendEquationEXT>(self)
                    );
            }
        }
    }
    /// Specifies which color components are enabled for writing per attachment.
    pub type ColorWriteMask = VecWrapper<SmallVec<[ColorComponents; 5]>>;
    impl PipelineStateObject for ColorWriteMask {

        const KIND: PipelineStateKind = PipelineStateKind::ColorWriteMask;

        fn check_validity(
            &self,
            _device: &Device,
            ctx: &mut PipelineStateContext,
        ) -> Result<(), Error>
        {   
            if ctx.inner().is_dynamic_and(|d| !d.is_state_dynamic(Self::KIND)) {
                Err(state_not_dynamic_err("ColorWriteMask"))?
            }
            if self.len() != ctx.color_attachment_count as usize {
                Err(err!(
                    "`ColorWriteMask` number of values `{}` is not equal to pipeline color attachment count `{}`",
                    self.len(), ctx.color_attachment_count,
                ))?
            }
            if let Some((i, bits)) = self.iter().enumerate().find(|(_, mask)| mask.contains_unknown_bits()) {
                Err(err!(
                    "color attachment write mask at index `{i}` contains unknown bits {}",
                    bits.difference(ColorComponents::all()),
                ))?
            }
            Ok(())
        }

        unsafe fn cmd_set_dynamic(
            &self,
            device: &Device,
            command_buffer: CommandBuffer<'_>,
            _allocator: &AnyAlloc,
        ) {
            use tuhka::vk::ColorComponentFlags;
            let feature = device
                .get_feature(vk_ext::extended_dynamic_state3::NAME)
                .unwrap();
            unsafe {
                feature.functions(device)
                    .cmd_set_color_write_mask(
                        command_buffer.handle(),
                        0,
                        bytemuck::cast_slice::<ColorComponents, ColorComponentFlags>(self)
                    );
            }
        }
    }
    pub struct LineRasterizationMode;
    pub struct LineStippleEnable;
}
