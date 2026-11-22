use tuhka::vk;

use crate::*;

use super::ResourceStateCache;

use mem::alloc_vec::FixedVec32;

/// Specifies an area a [render pass] covers.
///
/// # Valid usage
/// - [`width`] and [`height`] **must** be greater than `0`.
/// - [`width`] + [`offset_x`] **must** be less than or equal to [`max_framebuffer_width`].
/// - [`height`] + [`offset_y`] **must** be less than or equal to [`max_framebuffer_height`].
/// - Each image view of the attachments in [`RenderPassAttributes`] **must** have a width greater than or
///   equal to [`width`] + [`offset_x`]..
/// - Each image view of the attachments in [`RenderPassAttributes`] **must** have a height greater than or
///   equal to [`height`] + [`offset_y`].
///
/// [render pass]: RenderPassAttributes
/// [`width`]: Self::width
/// [`height`]: Self::height
/// [`offset_x`]: Self::offset_x
/// [`offset_y`]: Self::offset_y
/// [`max_framebuffer_width`]: GpuLimits::max_framebuffer_width
/// [`max_framebuffer_height`]: GpuLimits::max_framebuffer_height
#[derive(Clone, Copy)]
pub struct RenderArea {
    /// The `x` offset to the start of the area.
    ///
    /// The default value is `0`.
    pub offset_x: u32,
    /// The `y` offset to the start of the area.
    ///
    /// The default value is `0`.
    pub offset_y: u32,
    /// The width of the area.
    ///
    /// The default value is `1`.
    pub width: u32,
    /// The height of the area.
    ///
    /// The default value is `1`.
    pub height: u32,
}

impl Default for RenderArea {

    #[inline]
    fn default() -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            width: 1,
            height: 1,
        }
    }
}

/// Specifies to how many or which layers to render to in a [render pass].
///
/// The default value is [`Count`][Self::Count] with the value set to `1`.
///
/// If the [`multiview`] feature is not enabled, this **must** be [`Count`][Self::Count].
///
/// [render pass]: RenderPassAttributes
/// [`multiview`]: vk_11::multiview
#[derive(Clone, Copy)]
pub enum RenderLayers {
    /// Specifies a number of layers to render to.
    ///
    /// # Valid usage
    /// - This **must** be between `1` and [`max_framebuffer_layers`] inclusive.
    /// - Each attachment in [`RenderPassAttributes`] **must** have at least this many layers.
    ///
    /// [`max_framebuffer_layers`]: GpuLimits::max_framebuffer_layers
    Count(u32),
    /// Specifies a bitmask of layers to render to.
    ///
    /// # Valid usage
    /// - This **must** not be `0`.
    /// - The index of the most significant bit of this mask:
    ///     - **must** be less than [`max_multiview_count`], and it
    ///     - **must** be less than the number of layers in each attachment in [`RenderPassAttributes`].
    ///
    /// [`max_framebuffer_layers`]: GpuLimits::max_framebuffer_layers
    /// [`max_multiview_count`]: GpuLimits::max_multiview_count
    Mask(u32),
}

impl Default for RenderLayers {

    #[inline]
    fn default() -> Self {
        Self::Count(1)
    }
}

/// An attachment to resolve to in multisampled rendering.
///
/// To create an attachment use [`Self::new`].
#[derive(Clone, Copy)]
pub struct ResolveAttachment<'view> {
    image_view: &'view dyn HasImageView,
    resolve_mode: ResolveMode,
}

impl<'view> ResolveAttachment<'view> {

    /// Creates a new resolve attachment.
    ///
    /// # Valid usage
    /// - The [`Image`] of `image_view` **must** have a [sample count] of [`X1`][SampleCountBit::X1].
    /// - If `image_view` has [a non-integer] [color format], `resolve_mode` **must** be
    ///   [`ResolveMode::Average`].
    /// - If `image_view` has [an integer] [color format], `resolve_mode` **must** be
    ///   [`ResolveMode::Zero`].
    ///
    /// Additional validation rules apply when this attachment is the `resolve_attachment` of a
    /// depth/stencil attachment.
    ///
    /// [sample count]: Image::samples
    /// [a non-integer]: NumericFormat::is_integer
    /// [color format]: Format::numeric_format_color
    /// [an integer]: NumericFormat::is_integer
    #[inline]
    pub fn new(
        image_view: &'view dyn HasImageView,
        resolve_mode: ResolveMode,
    ) -> Result<Self, Error> {
        if image_view.image().samples() != SampleCountBit::X1 {
            Err(err!(
                "The `Image` of `image_view` doesn't have sample count of one",
            ))?
        }
        if image_view.aspects().contains(ImageAspects::COLOR) {
            let format = image_view.format().numeric_format_color().unwrap();
            if format.is_integer()
            {
                if resolve_mode != ResolveMode::Zero {
                    Err(err!(
                        "`format` of `image_view` is an integer format, but `resolve_mode` is not `Zero`",
                    ))?
                }
            } else if resolve_mode != ResolveMode::Average {
                Err(err!(
                    "`format` of `image_view` is a non-integer format, but `resolve_mode` is not `Average`",
                ))?
            }
        }
        Ok(Self { image_view, resolve_mode })
    }
}

/// Specifies the clear value of a [`ColorAttachment`] when `load_op` is
/// [`AttachmentLoadOp::Clear`].
///
/// # Valid usage
/// - If the [`Format`] of the attachment [is a floating point format], this **must** be [`F32`].
/// - If the [`Format`] of the attachment [is a signed integer format], this **must** be [`I32`].
/// - If the [`Format`] of the attachment [is an unsigned integer format], this **must** be [`U32`].
///
/// [is a floating point format]: NumericFormat::is_floating_point
/// [is a signed integer format]: NumericFormat::is_signed_integer
/// [is an unsigned integer format]: NumericFormat::is_unsigned_integer
/// [`F32`]: Self::F32
/// [`I32`]: Self::I32
/// [`U32`]: Self::U32
pub enum ClearColorValue {
    /// Specifies a four-component floating point color.
    F32([f32; 4]),
    /// Specifies a four-component signed integer color.
    I32([i32; 4]),
    /// Specifies a four-component unsigned integer color.
    U32([u32; 4]),
}

/// A color attachment used in [`RenderPassAttributes`].
///
/// To create an attachment use [`Self::new`].
#[derive(Clone, Copy)]
pub struct ColorAttachment<'view> {
    image_view: &'view dyn HasImageView,
    resolve_attachment: Option<ResolveAttachment<'view>>,
    load_op: AttachmentLoadOp,
    store_op: AttachmentStoreOp,
    clear_value: vk::ClearColorValue,
}

impl<'view> ColorAttachment<'view> {

    /// Creates a new color attachment.
    ///
    /// # Parameters
    /// - `image_view`: The [`ImageView`] of this attachment.
    /// - `resolve_attachment`: A [`ResolveAttachment`] used in multisampled rendering.
    /// - `load_op`: An [`AttachmentLoadOp`] value specifying how the attachment is loaded.
    /// - `store_op`: An [`AttachmentStoreOp`] value specifying how the attachment is stored.
    /// - `clear_value`:  An optional [`ClearColorValue`] value specifying the value used when
    ///   `load_op` is [`AttachmentLoadOp::Clear`].
    ///
    /// # Valid usage
    /// - The [`Image`] of `image_view` **must** have been created with the [`COLOR_ATTACHMENT`].
    /// - If `resolve_attachment` is `Some`:
    ///     - the [`Image`] of `image_view` **must** have sample count greater than one, and
    ///     - the [`Image`] of the image view used to create that attahment **must** have
    ///       been created with the [`COLOR_ATTACHMENT`] usage, and
    ///     - the [`Format`] of its image view **must** be the same as that of `image_view`.
    /// - If `load_op` is [`AttachmentLoadOp::Clear`], `clear_value` **must** be `Some`.
    /// - If `clear_value` is `Some`, the valid usage [`ClearColorValue`] applies here.
    ///
    /// [`COLOR_ATTACHMENT`]: ImageUsages::COLOR_ATTACHMENT
    pub fn new(
        image_view: &'view dyn HasImageView,
        resolve_attachment: Option<ResolveAttachment<'view>>,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
        clear_value: Option<ClearColorValue>,
    ) -> Result<Self, Error> {
        if !image_view
            .image()
            .usage()
            .contains(ImageUsages::COLOR_ATTACHMENT)
        {
            Err(err!(
                "the `Image` of `image_view` was not created with the `COLOR_ATTACHMENT` usage",
            ))?
        }
        if let Some(attachment) = &resolve_attachment  
        {
            if image_view.image().samples() == SampleCountBit::X1 {
                Err(err!(
                    "`resolve_attachment` is `Some`, but the `Image` of `image_view` has a sample count of `X1`",
                ))?
            }
            if !attachment
                .image_view
                .image()
                .usage()
                .contains(ImageUsages::COLOR_ATTACHMENT)
            {
                Err(err!(
                    "the `Image` of the `image_view` of `resolve_attachment` was not created with the `COLOR_ATTACHMENT` usage",
                ))?
            }
            if attachment.image_view.format() != image_view.format() {
                Err(err!(
                    "`image_view` and the `image_view of `resolve_attachment` have different formats",
                ))?
            }
        }
        let mut vk_clear_value = vk::ClearColorValue {
            float32: [0.0; 4]
        };
        if let Some(clear_value) =  clear_value {
            let numeric_format = image_view.format().numeric_format_color().unwrap();
            if numeric_format.is_floating_point() {
                if let ClearColorValue::F32(value) = clear_value {
                    vk_clear_value = vk::ClearColorValue {
                        float32: value,
                    };
                } else {
                    Err(err!(
                        "`numeric_format_color` is a floating point format, but `clear_value` is not `F32`",
                    ))?
                }
            } else if numeric_format.is_signed_integer() {
                if let ClearColorValue::I32(value) = clear_value {
                    vk_clear_value = vk::ClearColorValue {
                        int32: value,
                    };
                } else {
                    Err(err!(
                        "`numeric_format_color` is a signed integer format, but `clear_value` is not `I32`",
                    ))?
                }
            } else if numeric_format.is_unsigned_integer() {
                if let ClearColorValue::U32(value) = clear_value {
                    vk_clear_value = vk::ClearColorValue {
                        uint32: value,
                    };
                } else {
                    Err(err!(
                        "`numeric_format_color` is an unsigned integer format, but `clear_value` is not `U32`",
                    ))?
                }
            } else {
                // no enumerated format is `Sfixed5` or `Bool` currently
                unreachable!()
            }
        } else if load_op == AttachmentLoadOp::Clear {
            Err(err!(
                "`load_op` is `Clear`, but `clear_value` is `None`"
            ))?
        }
        Ok(Self {
            image_view,
            resolve_attachment,
            load_op,
            store_op,
            clear_value: vk_clear_value,
        })
    }

    fn to_attachment(
        &self,
        state_cache: &mut ResourceStateCache,
        queue_family_index: u32,
    ) -> (vk::RenderingAttachmentInfo<'static>, u32, u32, u32) {
        unsafe {
            state_cache.insert_image(
                self.image_view.image().clone(),
                ImageState {
                    stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
                        | vk::AccessFlags2::COLOR_ATTACHMENT_READ,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    queue_family_index,
                },
                self.image_view.subresource_range()
            );
        }
        let mut attachment = vk::RenderingAttachmentInfo {
            image_view: self.image_view.get_view().handle(),
            image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            load_op: self.load_op.into(),
            store_op: self.store_op.into(),
            clear_value: vk::ClearValue {
                color: self.clear_value,
            },
            ..Default::default()
        };
        let ImageExtent { mut width, mut height, .. } = self.image_view.image().extent();
        let mut layer_count = self.image_view.subresource_range().layer_count;
        if let Some(ResolveAttachment { image_view, resolve_mode }) = &self.resolve_attachment {
            unsafe {
                state_cache.insert_image(
                    image_view.image().clone(),
                    ImageState {
                        stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                        access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
                            | vk::AccessFlags2::COLOR_ATTACHMENT_READ,
                        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        queue_family_index,
                    },
                    image_view.subresource_range(),
                );
            }
            attachment.resolve_mode = (*resolve_mode).into();
            attachment.resolve_image_view = image_view.get_view().handle();
            attachment.resolve_image_layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
            let extent = image_view.image().extent();
            width = width.min(extent.width);
            height = height.min(extent.height);
            layer_count = layer_count.min(image_view.subresource_range().layer_count);
        }
        (attachment, width, height, layer_count)
    }
}

/// Specifies the clear value of a [`DepthStencilAttachment`] when `load_op` is
/// [`AttachmentLoadOp::Clear`].
#[derive(Clone, Copy)]
pub struct ClearDepthStencilValue {
    /// The clear value of the depth aspect.
    pub depth: f32,
    /// The clear value of the stencil aspect.
    pub stencil: u32,
}

#[derive(Default, Clone)]
enum DepthStencilInternal<'view> {
    #[default]
    None,
    Depth {
        image_view: &'view dyn HasImageView,
        resolve_attachment: Option<ResolveAttachment<'view>>,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
    },
    Stencil {
        image_view: &'view dyn HasImageView,
        resolve_attachment: Option<ResolveAttachment<'view>>,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
    },
    DepthStencil {
        image_view: &'view dyn HasImageView,
        resolve_attachment: Option<(&'view dyn HasImageView, ResolveMode, ResolveMode)>,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
    }
}

/// Contains the depth and stencil attachments of a [`RenderPassAttributes`].
///
/// To create this attachment, see [`none`], [`depth`], [`stencil`] and [`depth_stencil`].
///
/// [`none`]: Self::none
/// [`depth`]: Self::depth
/// [`stencil`]: Self::stencil
/// [`depth_stencil`]: Self::depth_stencil
#[derive(Default, Clone)]
pub struct DepthStencilAttachment<'veiw> {
    inner: DepthStencilInternal<'veiw>,
    clear_value: vk::ClearDepthStencilValue,
}

impl<'veiw> DepthStencilAttachment<'veiw> {
    
    /// Creates an attachment with neither depth or stencil attachments.
    ///
    /// Equal to `Self::builder().build().unwrap()`.
    #[inline]
    pub fn none() -> Self {
        Self::default()
    }

    /// Creates a depth-only attachment.
    ///
    /// # Parameters
    /// - `image_view`: The [`ImageView`] of this attachment.
    /// - `resolve_attachment`: A [`ResolveAttachment`] used in multisampled rendering.
    /// - `load_op`: An [`AttachmentLoadOp`] value specifying how the attachment is loaded.
    /// - `store_op`: An [`AttachmentStoreOp`] value specifying how the attachment is stored.
    /// - `clear_value`:  An optional [`ClearDepthStencilValue`] value specifying the value used when
    ///   `load_op` is [`AttachmentLoadOp::Clear`].
    ///
    /// # Valid usage
    /// - The [`Image`] of `image_view` **must** have been created with the
    ///   [`DEPTH_STENCIL_ATTACHMENT`] usage.
    /// - `image_view` **must** have a [depth aspect].
    /// - If `resolve_attachment` is `Some`:
    ///     - the [`Image`] of `image_view` **must** have sample count greater than one, and
    ///     - the [`Image`] of the `image_view` used to create that attahment **must**
    ///       have been created with the [`DEPTH_STENCIL_ATTACHMENT`] usage, and
    ///     - the [`Format`] of its image view **must** be the same as that of `image_view` and,
    ///     - the `resolve_mode` **must** be supported as specified by
    ///       [`GpuDepthStencilResolveLimits::supported_depth_resolve_modes`].
    /// - If `load_op` is [`AttachmentLoadOp::Clear`], `clear_value` **must** be `Some`.
    ///
    /// [`DEPTH_STENCIL_ATTACHMENT`]: ImageUsages::DEPTH_STENCIL_ATTACHMENT
    /// [depth aspect]: ImageAspects::DEPTH
    pub fn depth(
        image_view: &'veiw dyn HasImageView,
        resolve_attachment: Option<ResolveAttachment<'veiw>>,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
        clear_value: Option<ClearDepthStencilValue>,
    ) -> Result<Self, Error> {
        if !image_view.image()
            .usage()
            .contains(ImageUsages::DEPTH_STENCIL_ATTACHMENT)
        {
            Err(err!(
                "the `Image` of `image_view` was not created with the `DEPTH_STENCIL_ATTACHMENT` usage",
            ))?
        }
        if !image_view.aspects().contains(ImageAspects::DEPTH) {
            Err(err!(
                "`image_view` doesn't have a `DEPTH` aspect",
            ))?
        }
        if let Some(attachment) = &resolve_attachment
        {
            if image_view.image().samples() == SampleCountBit::X1 {
                Err(err!(
                    "`resolve_attachment` is `Some`, but the `Image` of `image_view` has a sample count of `X1`",
                ))?
            }
            if !attachment
                .image_view
                .image()
                .usage()
                .contains(ImageUsages::DEPTH_STENCIL_ATTACHMENT)
            {
                Err(err!(
                    "the `Image` of the `image_view` of `resolve_attachment` was not created with the `DEPTH_STENCIL_ATTACHMENT` usage",
                ))?
            }
            if attachment.image_view.format() != image_view.format() {
                Err(err!(
                    "`image_view` and the `image_view of `resolve_attachment` have different formats",
                ))?
            }
            let supported_modes = image_view
                .device()
                .gpu_info()
                .limits
                .depth_stencil_resolve_limits()
                .supported_depth_resolve_modes;
            if !supported_modes.contains(attachment.resolve_mode.to_flags())
            {
                Err(err!(
                    "the `{:?}` resolve mode is not supported for depth attachments (supported modes are `{supported_modes}`)",
                    attachment.resolve_mode,
                ))?
            }
        }
        let mut vk_clear_value = vk::ClearDepthStencilValue {
            depth: 0.0,
            stencil: 0,
        };
        if let Some(clear_value) = clear_value {
            vk_clear_value = vk::ClearDepthStencilValue {
                depth: clear_value.depth,
                stencil: clear_value.stencil,
            };
        } else if load_op == AttachmentLoadOp::Clear {
            Err(err!(
                "`load_op` is `Clear`, but `clear_value` is `None`",
            ))?
        }
        Ok(Self { inner: DepthStencilInternal::Depth {
            image_view,
            resolve_attachment,
            load_op,
            store_op,
        }, clear_value: vk_clear_value})
    }

    /// Creates a stencil-only attachment.
    ///
    /// # Parameters
    /// - `image_view`: The [`ImageView`] of this attachment.
    /// - `resolve_attachment`: A [`ResolveAttachment`] used in multisampled rendering.
    /// - `load_op`: An [`AttachmentLoadOp`] value specifying how the attachment is loaded.
    /// - `store_op`: An [`AttachmentStoreOp`] value specifying how the attachment is stored.
    /// - `clear_value`:  An optional [`ClearDepthStencilValue`] value specifying the value used when
    ///   `load_op` is [`AttachmentLoadOp::Clear`].
    ///
    /// # Valid usage
    /// - The [`Image`] of `image_view` **must** have been created with the
    ///   [`DEPTH_STENCIL_ATTACHMENT`] usage.
    /// - `image_view` **must** have a [stencil aspect].
    /// - If `resolve_attachment` is `Some`:
    ///     - the [`Image`] of `image_view` **must** have sample count greater than one, and
    ///     - the [`Image`] of the `image_view` used to create that attahment **must**
    ///       have been created with the [`DEPTH_STENCIL_ATTACHMENT`] usage, and
    ///     - the [`Format`] of its image view **must** be the same as that of `image_view` and,
    ///     - the `resolve_mode` **must** be supported as specified by
    ///       [`GpuDepthStencilResolveLimits::supported_stencil_resolve_modes`].
    /// - If `load_op` is [`AttachmentLoadOp::Clear`], `clear_value` **must** be `Some`.
    ///
    /// [stencil aspect]: ImageAspects::STENCIL
    /// [`DEPTH_STENCIL_ATTACHMENT`]: ImageUsages::DEPTH_STENCIL_ATTACHMENT
    #[inline]
    pub fn stencil(
        image_view: &'veiw dyn HasImageView,
        resolve_attachment: Option<ResolveAttachment<'veiw>>,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
        clear_value: Option<ClearDepthStencilValue>,
    ) -> Result<Self, Error> {
        if !image_view.image()
            .usage()
            .contains(ImageUsages::DEPTH_STENCIL_ATTACHMENT)
        {
            Err(err!(
                "the `Image` of `image_view` was not created with the `DEPTH_STENCIL_ATTACHMENT` usage",
            ))?
        }
        if !image_view.aspects().contains(ImageAspects::STENCIL) {
            Err(err!(
                "`image_view` doesn't have a `STENCIL` aspect",
            ))?
        }
        if let Some(attachment) = &resolve_attachment
        {
            if image_view.image().samples() == SampleCountBit::X1 {
                Err(err!(
                    "`resolve_attachment` is `Some`, but the `Image` of `image_view` has a sample count of `X1`",
                ))?
            }
            if !attachment
                .image_view
                .image()
                .usage()
                .contains(ImageUsages::DEPTH_STENCIL_ATTACHMENT)
            {
                Err(err!(
                    "the `Image` of the `image_view` of `resolve_attachment` was not created with the `DEPTH_STENCIL_ATTACHMENT` usage",
                ))?
            }
            if attachment.image_view.format() != image_view.format() {
                Err(err!(
                    "`image_view` and the `image_view of `resolve_attachment` have different formats",
                ))?
            }
            let supported_modes = image_view
                .device()
                .gpu_info()
                .limits
                .depth_stencil_resolve_limits()
                .supported_stencil_resolve_modes;
            if !supported_modes.contains(attachment.resolve_mode.to_flags())
            {
                Err(err!(
                    "the `{:?}` resolve mode is not supported for stencil attachments (supported modes are `{supported_modes}`)",
                    attachment.resolve_mode,
                ))?
            }
        }
        let mut vk_clear_value = vk::ClearDepthStencilValue {
            depth: 0.0,
            stencil: 0,
        };
        if let Some(clear_value) = clear_value {
            vk_clear_value = vk::ClearDepthStencilValue {
                depth: clear_value.depth,
                stencil: clear_value.stencil,
            };
        } else if load_op == AttachmentLoadOp::Clear {
            Err(err!(
                "`load_op` is `Clear`, but `clear_value` is `None`",
            ))?
        }
        Ok(Self { inner: DepthStencilInternal::Stencil {
            image_view,
            resolve_attachment,
            load_op,
            store_op,
        }, clear_value: vk_clear_value})
    }

    /// Creates a depth/stencil attachment.
    ///
    /// # Parameters
    /// - `image_view`: the image view of the depth/stencil attachment.
    /// - `resolve_attachment`: an optional `(image_view, depth_resolve_mode, stencil_resolve_mode)`
    ///   tuple specifying a resolve image view and resolve modes used to resolve the attachment in
    ///   multisampled rendering.
    /// - `load_op`: specifies how the attachment is loaded.
    /// - `store_op`: specifies how the attachment is stored.
    /// - `clear_value`:  An optional [`ClearDepthStencilValue`] value specifying the value used when
    ///   `load_op` is [`AttachmentLoadOp::Clear`].
    ///
    /// # Valid usage
    /// - The [`Image`] of `image_view` **must** have been created with the
    ///   [`DEPTH_STENCIL_ATTACHMENT`] usage.
    /// - `image_view` **must** have both a [depth aspect] and a [stencil aspect].
    /// - If `resolve_attachment` is `Some`:
    ///     - the [`Image`] of `image_view` **must** have sample count greater than one, and
    ///     - the [`Image`] of `resolve_attachment.0` **must** have a [sample count] of [`X1`][SampleCountBit::X1] and
    ///       it **must** have been created with the [`DEPTH_STENCIL_ATTACHMENT`] usage,
    ///     - the [`Format`] of `resolve_attachment.0` **must** be the same as that of `image_view`, and
    ///     - `resolve_attachment.1` **must** be a supported [`ResolveMode`] as specified by
    ///       [`GpuDepthStencilResolveLimits::supported_depth_resolve_modes`], and
    ///     - `resolve_attachment.2` **must** be supported [`ResolveMode`] as specified by
    ///       [`GpuDepthStencilResolveLimits::supported_stencil_resolve_modes`], and
    ///     - if [`GpuDepthStencilResolveLimits::independent_resolve`] is not supported
    ///       `resolve_attachment.1` and `resolve_attachment.2` **must** be equal.
    /// - If `load_op` is [`AttachmentLoadOp::Clear`], `clear_value` **must** be `Some`.
    ///
    /// [depth aspect]: ImageAspects::DEPTH
    /// [stencil aspect]: ImageAspects::STENCIL
    /// [`DEPTH_STENCIL_ATTACHMENT`]: ImageUsages::DEPTH_STENCIL_ATTACHMENT
    pub fn depth_stencil(
        image_view: &'veiw dyn HasImageView,
        resolve_attachment: Option<(&'veiw dyn HasImageView, ResolveMode, ResolveMode)>,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
        clear_value: Option<ClearDepthStencilValue>,
    ) -> Result<Self, Error> {
        if !image_view.image()
            .usage()
            .contains(ImageUsages::DEPTH_STENCIL_ATTACHMENT)
        {
            Err(err!(
                "the `Image` of `image_view` was not created with the `DEPTH_STENCIL_ATTACHMENT` usage",
            ))?
        }
        let aspects = image_view.aspects();
        if !aspects.contains(ImageAspects::DEPTH) {
            Err(err!(
                "`image_view` doesn't have a `DEPTH` aspect",
            ))?
        }
        if !aspects.contains(ImageAspects::STENCIL) {
            Err(err!(
                "`image_view` doesn't have a `STENCIL` aspect",
            ))?
        }
        let format = image_view.format();
        if let Some((resolve_view, depth_mode, stencil_mode)) = &resolve_attachment {
            if image_view.image().samples() == SampleCountBit::X1 {
                Err(err!(
                    "`resolve_attachment` is `Some`, but the `Image` of `image_view` has a sample count of `X1`",
                ))?
            }
            let resolve_image = resolve_view.image();
            if resolve_image.samples() != SampleCountBit::X1 {
                Err(err!(
                    "`resolve_attachment.0` doesn't have a sample count of one",
                ))?
            }
            if !resolve_image.usage().contains(ImageUsages::DEPTH_STENCIL_ATTACHMENT) {
                Err(err!(
                    "the `Image` of `resolve_attachment.0` was not created with the `DEPTH_STENCIL_ATTACHMENT` usage",
                ))?
            }
            if resolve_view.format() != format {
                Err(err!(
                    "`resolve_attachment.0` and `image_view` have different formats",
                ))?
            }
            let limits = image_view
                .device()
                .gpu_info()
                .limits
                .depth_stencil_resolve_limits();
            if !limits.supported_depth_resolve_modes.contains(depth_mode.to_flags()) {
                Err(err!(
                    "the `{depth_mode:?}` resolve mode is not supported for depth attachments (supported modes are `{}`)",
                    limits.supported_depth_resolve_modes,
                ))?
            }
            if !limits.supported_stencil_resolve_modes.contains(stencil_mode.to_flags()) {
                Err(err!(
                    "the `{stencil_mode:?}` resolve mode is not supported for stencil attachments (supported modes are `{}`)",
                    limits.supported_stencil_resolve_modes,
                ))?
            }
            if depth_mode != stencil_mode &&
                !limits.independent_resolve
            {
                Err(err!(
                    "depth resolve mode and stencil resolve mode are different, but `independent_resolve` is not supported",
                ))?
            }
        }
        let mut vk_clear_value = vk::ClearDepthStencilValue {
            depth: 0.0,
            stencil: 0,
        };
        if let Some(clear_value) = clear_value {
            vk_clear_value = vk::ClearDepthStencilValue {
                depth: clear_value.depth,
                stencil: clear_value.stencil,
            };
        } else if load_op == AttachmentLoadOp::Clear {
            Err(err!(
                "`load_op` is `Clear`, but `clear_value` is `None`",
            ))?
        }
        Ok(Self { inner: DepthStencilInternal::DepthStencil {
            image_view,
            resolve_attachment,
            load_op,
            store_op,
        }, clear_value: vk_clear_value,})
    }

    /// Gets the number of samples in the attachments, if this is not [`none`].
    ///
    /// [`none`]: Self::none
    #[inline]
    pub fn samples(&self) -> Option<SampleCountBit> {
        Some(match &self.inner {
            DepthStencilInternal::None => None?,
            DepthStencilInternal::Depth { image_view, .. } => image_view
                .image().samples(),
            DepthStencilInternal::Stencil { image_view, .. } => image_view
                .image().samples(),
            DepthStencilInternal::DepthStencil { image_view, .. } =>
                image_view.image().samples(),
        })
    }

    fn to_attachments(
        &self,
        state_cache: &mut ResourceStateCache,
        queue_family_index: u32,
    ) -> (
        Option<vk::RenderingAttachmentInfo<'static>>,
        Option<vk::RenderingAttachmentInfo<'static>>,
        u32, u32, u32,
    ) {
        match &self.inner {
            DepthStencilInternal::None => (None, None, u32::MAX, u32::MAX, u32::MAX),
            DepthStencilInternal::Depth { image_view, resolve_attachment, store_op, load_op, } => {
                let layout = vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL;
                unsafe {
                    state_cache.insert_image(
                        image_view.image().clone(),
                        ImageState {
                            stage: vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                            access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
                                | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ,
                            layout,
                            queue_family_index,
                        },
                        image_view.subresource_range(),
                    );
                }
                let mut attachment = vk::RenderingAttachmentInfo {
                    image_view: image_view.get_view().handle(),
                    image_layout: layout,
                    load_op: (*load_op).into(),
                    store_op: (*store_op).into(),
                    clear_value: vk::ClearValue { depth_stencil: self.clear_value },
                    ..Default::default()
                };
                let ImageExtent { mut width, mut height, .. } = image_view.image().extent();
                let mut layer_count = image_view.subresource_range().layer_count;
                if let Some(ResolveAttachment { image_view, resolve_mode }) = resolve_attachment {
                    attachment.resolve_mode = (*resolve_mode).into();
                    attachment.resolve_image_view = image_view.get_view().handle();
                    unsafe {
                        state_cache.insert_image(
                            image_view.image().clone(),
                            ImageState {
                                stage: vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                                access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
                                    | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ,
                                layout,
                                queue_family_index,
                            },
                            image_view.subresource_range(),
                        );
                    }
                    attachment.resolve_image_layout = layout;
                    let extent = image_view.image().extent();
                    width = width.min(extent.width);
                    height = height.min(extent.height);
                    layer_count = layer_count.min(image_view.subresource_range().layer_count);
                }
                (
                    Some(attachment),
                    None,
                    width, height,
                    layer_count,
                )
            },
            DepthStencilInternal::Stencil { image_view, resolve_attachment, load_op, store_op } => {
                let layout = vk::ImageLayout::STENCIL_ATTACHMENT_OPTIMAL;
                unsafe {
                    state_cache.insert_image(
                        image_view.image().clone(),
                        ImageState {
                            stage: vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                            access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
                                | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ,
                            layout,
                            queue_family_index,
                        },
                        image_view.subresource_range(),
                    );
                }
                let mut attachment = vk::RenderingAttachmentInfo {
                    image_view: image_view.get_view().handle(),
                    image_layout: layout,
                    load_op: (*load_op).into(),
                    store_op: (*store_op).into(),
                    clear_value: vk::ClearValue { depth_stencil: self.clear_value },
                    ..Default::default()
                };
                let ImageExtent { mut width, mut height, .. } = image_view.image().extent();
                let mut layer_count = image_view.subresource_range().layer_count;
                if let Some(ResolveAttachment { image_view, resolve_mode }) = resolve_attachment {
                    attachment.resolve_mode = (*resolve_mode).into();
                    attachment.resolve_image_view = image_view.get_view().handle();
                    unsafe {
                        state_cache.insert_image(
                            image_view.image().clone(),
                            ImageState {
                                stage: vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                                access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
                                    | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ,
                                layout,
                                queue_family_index,
                            },
                            image_view.subresource_range(),
                        );
                    }
                    attachment.resolve_image_layout = layout;
                    let extent = image_view.image().extent();
                    width = width.min(extent.width);
                    height = height.min(extent.height);
                    layer_count = layer_count.min(image_view.subresource_range().layer_count);
                }
                (
                    None,
                    Some(attachment),
                    width, height,
                    layer_count,
                )
            },
            DepthStencilInternal::DepthStencil { image_view, resolve_attachment, load_op, store_op } => {
                let layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
                let [mut depth_attachment, mut stencil_attachment ] = [vk::RenderingAttachmentInfo {
                    image_view: image_view.get_view().handle(),
                    image_layout: layout,
                    load_op: (*load_op).into(),
                    store_op: (*store_op).into(),
                    clear_value: vk::ClearValue { depth_stencil: self.clear_value },
                    ..Default::default()
                }; 2];
                unsafe {
                    state_cache.insert_image(
                        image_view.image().clone(),
                        ImageState {
                            stage: vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                            access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
                                | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ,
                            layout,
                            queue_family_index,
                        },
                        image_view.subresource_range(),
                    );
                }
                let ImageExtent { mut width, mut height, .. } = image_view.image().extent();
                let mut layer_count = image_view.subresource_range().layer_count;
                if let Some((image_view, depth_mode, stencil_mode)) = resolve_attachment {
                    unsafe {
                        state_cache.insert_image(
                            image_view.image().clone(),
                            ImageState {
                                stage: vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                                access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
                                    | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ,
                                layout,
                                queue_family_index,
                            },
                            image_view.subresource_range(),
                        );
                    }
                    depth_attachment.resolve_mode = (*depth_mode).into();
                    depth_attachment.resolve_image_view = image_view.get_view().handle();
                    depth_attachment.resolve_image_layout = layout;
                    stencil_attachment.resolve_mode = (*stencil_mode).into();
                    stencil_attachment.resolve_image_view = image_view.get_view().handle();
                    stencil_attachment.resolve_image_layout = layout;
                    let extent = image_view.image().extent();
                    width = width.min(extent.width);
                    height = height.min(extent.height);
                    layer_count = layer_count.min(image_view.subresource_range().layer_count);
                }
                (
                    Some(depth_attachment),
                    Some(stencil_attachment),
                    width, height,
                    layer_count,
                )
            },
        }
    }
}

/// Parameters of a [render pass].
///
/// [render pass]: CommandBufferRecorder::render_pass
#[derive(Default)]
pub struct RenderPassAttributes<'a, 'b> {
    /// An optional tag of this render pass.
    pub tag: Option<&'b str>,
    /// The area rendered to.
    ///
    /// # Valid usage
    /// The valid usage of this field is documented in [`RenderArea`].
    pub render_area: RenderArea,
    /// How many layers or which layers to render to.
    ///
    /// # Valid usage
    /// The valid usage of this field is documented in [`RenderLayers`].
    pub layers: RenderLayers,
    /// Specifies the color attachments of this render pass.
    ///
    /// These are the `Output` variables in a fragment shader.
    ///
    /// # Valid usage
    /// In addition to the valid usages documented in [`ColorAttachment::new`]:
    /// - Each color attachment **must** have the same number of samples, and that number **must**
    ///   be the same as the number of samples in [`depth_stencil_attachments`], and
    /// - the sample count of these attachments **must** be contained in
    ///   [`GpuLimits::framebuffer_color_samples`]
    ///
    /// [`depth_stencil_attachments`]: Self::depth_stencil_attachments
    pub color_attachments: &'a [ColorAttachment<'b>],
    /// Specifies the depth and stencil attachments.
    ///
    /// # Valid usage
    /// In addition to the valid usages documented in [`DepthStencilAttachment`]:
    /// - Th number of samples queried with [`DepthStencilAttachment::samples`] **must** be the
    ///   same as the sample count of each attachment in [`color_attachments`].
    ///
    /// [`color_attachments`]: Self::color_attachments
    pub depth_stencil_attachment: DepthStencilAttachment<'b>,
}

/// An actively recording render pass.
///
/// To record a render pass use the [`CommandBufferRecorder::render_pass`] method.
pub struct ActiveRenderPass<'a, 'b, 'buffer> {
    tag: Option<&'b str>,
    color_attachments: FixedVec32<ColorAttachment<'b>, &'b dyn Allocator>,
    depth_stencil_attachment: DepthStencilAttachment<'b>,
    recorder: &'a mut CommandBufferRecorder<'buffer>,
}

impl<'buffer> CommandBufferRecorder<'buffer> {

    /// Starts recording a render pass.
    pub fn render_pass<'b>(
        &mut self,
        attributes: RenderPassAttributes<'_, 'b>,
        allocator: &'b dyn Allocator,
    ) -> Result<ActiveRenderPass<'_, 'b, 'buffer>, Error>
    {
        if let Some(tag) = &attributes.tag {
            log::trace!("validating render pass `{tag}`");
        }
        self.command_buffer.signal_sync_scope |= vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT;
        if attributes.render_area.width == 0 {
            Err(err!(
                "render area `width` is 0",
            ))?
        }
        if attributes.render_area.height == 0 {
            Err(err!(
                "render area `height` is 0",
            ))?
        }
        let max_x = attributes.render_area.offset_x + attributes.render_area.width;
        let max_y = attributes.render_area.offset_y + attributes.render_area.height;
        let limits = &self.command_buffer.pool().device().gpu_info().limits;
        if max_x > limits.max_framebuffer_width() {
            Err(err!(
                "`offset_x + width` of `render_area` is greater than `max_framebuffer_width` {}",
                limits.max_framebuffer_width(),
            ))?
        }
        if max_y > limits.max_framebuffer_height() {
            Err(err!(
                "`offset_y + height` of `render_area` is greater than `max_framebuffer_height` {}",
                limits.max_framebuffer_height(),
            ))?
        }
        let mut state_cache = self.command_buffer.state_cache
            .take()
            .unwrap();
        let queue_family_index = self.command_buffer
            .pool()
            .queue()
            .family_index();
        let device = self.command_buffer
            .pool()
            .device();
        let mut pass_sample_count = attributes.depth_stencil_attachment
            .samples();
        let (min_layers, vk_layer_count, vk_view_mask) = match attributes.layers {
            RenderLayers::Count(count) => {
                if count == 0 {
                    Err(err!(
                        "`RenderLayers::Count` is `0`",
                    ))?
                }
                if count > device
                    .gpu_info()
                    .limits.max_framebuffer_layers()
                {
                    Err(err!(
                        "`RenderLayers:Count` is greater than maximum supported layers `{}`",
                        device.gpu_info()
                            .limits.max_framebuffer_layers(),
                    ))?
                }
                (count, count, 0)
            },
            RenderLayers::Mask(mask) => {
                if !device
                    .is_feature_enabled(vk_11::multiview::NAME)
                {
                    Err(err!(
                        "`layers` is `RenderLayers::Mask`, but the `vk_11::multiview` feature is not enabled",
                    ))?
                }
                if mask == 0 {
                    Err(err!(
                        "`RenderLayers::Mask` is `0`",
                    ))?
                }
                let msb = 32 - mask.leading_zeros();
                if msb >= device
                    .gpu_info()
                    .limits.max_multiview_count()
                {
                    Err(err!(
                        "the index of the most signifcant bit of `RenderLayers::Mask` is {}",
                        format_args!("not less than `max_multiview_count` {}",
                            device
                            .gpu_info()
                            .limits.max_multiview_count()
                        ),
                    ))?
                }
                (msb + 1, 0, mask)
            },
        };
        let (depth_attachment, stencil_attachment, width, height, layers) = attributes.depth_stencil_attachment
            .to_attachments(&mut state_cache, queue_family_index);
        if max_x > width {
            Err(err!(
                "`offset_x + width` of `render_area` is greater than depth/stencil attachment width",
            ))?
        }
        if max_y > height {
            Err(err!(
                "`offset_y + height` of `render_area` is greater than depth/stencil attachment height",
            ))?
        }
        if layers < min_layers {
            Err(err!(
                "depth/stencil attachment has less than the minimum amount of layers (`{min_layers}`) for this pass"
            ))?
        }
        let mut color_attachments = FixedVec32::with_capacity_in(
            attributes.color_attachments.len() as u32,
            allocator
        ).describe_err("allocation with `alloc` failed")?;
        for (i, attachment) in attributes.color_attachments.iter().enumerate() {
            let att_sample_count = attachment.image_view
                .image().samples();
            let pass = *pass_sample_count.get_or_insert(att_sample_count);
            if att_sample_count != pass {
                Err(err!(
                    "color attachment at index `{i}` has a different sample count from other attachments",
                ))?
            }
            let (att, width, height, layers) = attachment.to_attachment(&mut state_cache, queue_family_index);
            if max_x > width {
                Err(err!(
                    "`offset_x + width` of `render_area` is greater than color attachment width at index `{i}`",
                ))?
            }
            if max_y > height {
                Err(err!(
                    "`offset_y + height` of `render_area` is greater than color attachment height at index `{i}`",
                ))?
            }
            if layers < min_layers {
                Err(err!(
                    "color attachment at index `{i}` has less than the minimum amount of layers (`{min_layers}`) for this pass"
                ))?
            }
            color_attachments.push(att);
        }
        let rendering_info = vk::RenderingInfo {
            render_area: vk::Rect2D {
                offset: vk::Offset2D {
                    x: attributes.render_area.offset_x as i32,
                    y: attributes.render_area.offset_y as i32,
                },
                extent: vk::Extent2D {
                    width: attributes.render_area.width,
                    height: attributes.render_area.height,
                },
            },
            layer_count: vk_layer_count,
            view_mask: vk_view_mask,
            ..Default::default()
        }.color_attachments(&color_attachments)
        .depth_attachment(depth_attachment.as_ref())
        .stencil_attachment(stencil_attachment.as_ref());
        if let Some(tag) = &attributes.tag {
            log::trace!("beginning render pass `{tag}`");
        }
        unsafe {
            device
                .functions()
                .cmd_begin_rendering(
                    self.command_buffer.handle(),
                    &rendering_info
                );
        }
        self.command_buffer.state_cache = Some(state_cache);
        Ok(ActiveRenderPass {
            tag: attributes.tag,
            color_attachments: attributes.color_attachments
                .iter()
                .copied()
                .collect_in(allocator)
                .describe_err("allocation with `alloc` failed")?,
            depth_stencil_attachment: attributes.depth_stencil_attachment,
            recorder: self,
        })
    }
}

impl<'a, 'b, 'buffer> ActiveRenderPass<'a, 'b, 'buffer> {

    /// Finishes recording the pass
    #[inline]
    pub fn finish(self) {}
}

impl<'a, 'b, 'buffer> Drop for ActiveRenderPass<'a, 'b, 'buffer> {

    #[inline]
    fn drop(&mut self) {
        unsafe {
            let handle = self.recorder.command_buffer.handle();
            if let Some(tag) = self.tag {
                log::trace!("ending render pass `{tag}`");
            }
            self.recorder
                .command_buffer
                .pool()
                .device()
                .functions()
                .cmd_end_rendering(handle);
        }
    }
}
