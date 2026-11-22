use super::*;

/// A lifetimed image-view handle.
#[repr(transparent)]
pub struct ImageViewHandle<'a> {
    image_view: &'a ImageView,
}

impl<'a> ImageViewHandle<'a> {

    /// Creates `self` from a reference to `image_view`.
    #[inline]
    pub fn from_ref(image_view: &'a ImageView) -> Self {
        Self { image_view }
    }

    #[inline]
    pub(crate) fn get(&self) -> &'a ImageView {
        self.image_view
    }
}

/// An [`ImageView`] handle trait.
pub trait HasImageView {

    /// Gets the [`ImageViewHandle`].
    fn handle(&self) -> ImageViewHandle<'_>;
}

pub(crate) trait HasImageViewExt: HasImageView {
    
    #[inline]
    fn get_view(&self) -> &ImageView {
        HasImageView
            ::handle(self)
            .get()
    }

    #[inline]
    fn format(&self) -> Format {
        self.get_view().format()
    }

    #[inline]
    fn aspects(&self) -> ImageAspects {
        self.get_view().aspects()
    }
    
    #[inline]
    fn image(&self) -> &Image {
        self.get_view().image()
    }

    #[inline]
    fn device(&self) -> &Device {
        self.get_view().device()
    }

    #[inline]
    fn subresource_range(&self) -> vk::ImageSubresourceRange {
        self.get_view().subresource_range()
    }
}

impl<T> HasImageViewExt for T
    where T: ?Sized + HasImageView
{}

/// Specifies parameters for [`ImageView`] creation.
///
/// This **can** be used to create an [`ImageView`] with [`Device::create_image_view`].
///
/// # Valid usage
/// In addition to the valid usages documented in each each field:
/// - the [`Image`] used to create this view **must** have been created with at least one
///   of the following usages:
///     - [`SAMPLED`],
///     - [`STORAGE`],
///     - [`COLOR_ATTACHMENT`],
///     - [`DEPTH_STENCIL_ATTACHMENT`], or
///     - [`INPUT_ATTACHMENT`].
///
/// [`SAMPLED`]: ImageUsages::SAMPLED
/// [`STORAGE`]: ImageUsages::STORAGE
/// [`COLOR_ATTACHMENT`]: ImageUsages::COLOR_ATTACHMENT
/// [`DEPTH_STENCIL_ATTACHMENT`]: ImageUsages::DEPTH_STENCIL_ATTACHMENT
/// [`INPUT_ATTACHMENT`]: ImageUsages::INPUT_ATTACHMENT
pub type ImageViewAttributes<'a> = attributes::ImageViewAttributes<&'a str>;

impl ImageViewAttributes<'_> {

    /// Calculates how much extra (CPU) memory the created [`ImageView`] will allocate.
    #[inline]
    pub fn calculate_allocation_size(&self) -> usize {
        size_of::<Inner>() + self.tag.map(|l| l.len()).unwrap_or_default()
    }
}

struct Inner
{
    image: Image,
    handle: vk::ImageView,
    format: Format,
    true_range: vk::ImageSubresourceRange,
    dimensions: ImageViewDim,
    ref_count: AtomicU64,
    pack_layout: alloc::Layout,
}

/// Represents an image view of an [`Image`].
///
/// To create an image view, use [`Device::create_image_view`].
pub struct ImageView
{
    inner: NonNull<Inner>,
    tag: (Option<NonNull<u8>>, usize),
    allocator: AnyAllocator,
}

unsafe impl Send for ImageView {}
unsafe impl Sync for ImageView {}

impl ImageView
{
    pub(crate) fn new(
        image: Image,
        attributes: ImageViewAttributes<'_>,
        allocator: AnyAllocator,
    ) -> Result<Self, Error> {
        if let Some(tag) = attributes.tag {
            log::info!("validating parameters for `ImageView` `{tag}`");
        }
        let require_one_of = 
            ImageUsages::SAMPLED |
            ImageUsages::STORAGE |
            ImageUsages::COLOR_ATTACHMENT |
            ImageUsages::DEPTH_STENCIL_ATTACHMENT;
        if !image.usage().intersects(
            require_one_of
        )
        {
            Err(err!(
                "`image` is missing all of {require_one_of} usages",
            ))?
        }
        let mut view_format = image.format();
        if let Some(f) = attributes.format &&
            f != view_format
        {
            if !image.view_formats().contains(&f) {
                Err(err!(
                    "`format` is not the format of the `image` and is not included in `view_formats`",
                ))?
            }
            view_format = f;
        }
        if image.samples() != SampleCountBit::X1 &&
            !matches!(attributes.dimensions, ImageViewDim::Dim2 | ImageViewDim::Dim2Array)
        {
            Err(err!(
                "the samples of `image` is not `X1` and `attributes.dimensions` is not `Dim2` or `Dim2Array`",
            ))?
        }
        let view_compatibility = image.view_compatibility();
        let image_dim = image.dimensions();
        let view_dim = attributes.dimensions;
        match image_dim {
            ImageDim::Dim1 => {
                if !matches!(view_dim, ImageViewDim::Dim2 | ImageViewDim::Dim1Array) {
                    Err(err!(
                        "image's dimensions is `Dim1`, but image view's dimensions is not `Dim1` or `Dim1Array`",
                    ))?
                }
            },
            ImageDim::Dim2 => {
                if !matches!(view_dim, ImageViewDim::Dim2 | ImageViewDim::Dim2Array | ImageViewDim::Cube | ImageViewDim::CubeArray) {
                    Err(err!(
                        "image's dimensions is `Dim2`, but image view's dimensions is not `Dim2`, `Dim2Array`, `Cube` or `CubeArray`",
                    ))?
                }
            },
            ImageDim::Dim3 => {
                if !matches!(view_dim, ImageViewDim::Dim3 | ImageViewDim::Dim2 | ImageViewDim::Dim2Array) {
                    Err(err!(
                        "image's dimensions is `Dim3`, but image view's dimensions is not `Dim3`, `Dim2` or `Dim2Array`",
                    ))?
                }
            },
        }
        let image_aspects = image.aspects();
        let mut view_aspects = image_aspects;
        if !attributes.range.aspects.is_empty() {
            let diff = attributes.range.aspects.difference(image_aspects);
            if !diff.is_empty() {
                Err(err!(
                    "`range.aspects` contains aspects ({diff}) not present in the image",
                ))?
            }
            view_aspects = attributes.range.aspects;
        } 
        let image_layers = image.array_layers();
        let image_levels = image.mip_levels();
        if attributes.range.base_mip_level >= image_levels {
            Err(err!(
                "`base_mip_level` is greater than or equal to image's `mip_levels` {image_levels}",
            ))?
        }
        let level_count =
            if let Some(levels) = attributes.range.level_count {
                if attributes.range.base_mip_level + levels.get() > image_levels {
                    Err(err!(
                        "`base_mip_level + level_count` is greater than image's `mip_levels` {image_levels}",
                    ))?
                }
                levels.get()
            } else {
                image_levels - attributes.range.base_mip_level
            };
        let (vk_subresource_range, true_range) =
        if image_dim != ImageDim::Dim3 {
            if attributes.range.base_array_layer >= image_layers {
                Err(err!(
                    "`base_array_layer` is greater than or equal to image's `array_layers` {image_layers}",
                ))?
            }
            let layer_count =
                if let Some(layers) = attributes.range.layer_count {
                    if attributes.range.base_array_layer + layers.get() > image_layers {
                        Err(err!(
                            "`base_array_layer + layer_count` is greater than image's `array_layers` {image_layers}",
                        ))?
                    }
                    layers.get()
                } else {
                    image_layers - attributes.range.base_array_layer
                };
            match view_dim {
                ImageViewDim::Dim1 | ImageViewDim::Dim2 => if layer_count != 1 {
                    Err(err!(
                        "`dimensions` is a non-arrayed, non-cube format but calculated layer count is not `1`",
                    ))?
                },
                ImageViewDim::Cube => {
                    if !view_compatibility.contains(ImageViewCompatibility::CUBE) {
                        Err(err!(
                            "`dimensions` is `Cube`, but image was not created with the `CUBE` view compatibility",
                        ))?
                    }
                    if layer_count != 6 {
                        Err(err!(
                            "`dimensions` is `Cube`, but calculated layer count is not `6`",
                        ))?
                    }
                },
                ImageViewDim::CubeArray => {
                    if !image.device()
                        .is_feature_enabled(vk_10::image_cube_array::NAME)
                    {
                        Err(err!(
                            "`dimensions` is `CubeArray`, but the `vk_10::image_cube_array` feature is not enabled",
                        ))?
                    }
                    if !view_compatibility.contains(ImageViewCompatibility::CUBE) {
                        Err(err!(
                            "`dimensions` is `CubeArray`, but image was not created with the `CUBE` view compatibility",
                        ))?
                    }
                    if !layer_count.is_multiple_of(6) {
                        Err(err!(
                            "`dimensions` is `CubeArray`, but calculated layer count is not a multiple of `6`",
                        ))?
                    }
                },
                ImageViewDim::Dim1Array |
                ImageViewDim::Dim2Array => {},
                ImageViewDim::Dim3 => unreachable!(),
            }
            if view_compatibility.contains(ImageViewCompatibility::REINTERPRET_TEXEL_BLOCKS)
                && !view_format.is_compressed()
            {
                if level_count != 1 {
                    Err(err!(
                        "image was created with `REINTERPRET_TEXEL_BLOCKS` and `format` is an uncompressed format, but level count is not `1`",
                    ))?
                }
                if layer_count != 1 {
                    Err(err!(
                        "image was created with `REINTERPRET_TEXEL_BLOCKS` and `format` is an uncompressed format, but layer count is not `1`",
                    ))?
                }
            }
            let true_range = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::from_raw(view_aspects.bits()),
                base_mip_level: attributes.range.base_mip_level,
                level_count,
                base_array_layer: attributes.range.base_array_layer,
                layer_count,
            };
            (true_range, true_range)
        } else {
            if view_compatibility.contains(ImageViewCompatibility::REINTERPRET_TEXEL_BLOCKS)
                && !view_format.is_compressed()
                && level_count != 1
            {
                Err(err!(
                    "image was created with `REINTERPRET_TEXEL_BLOCKS` and `format` is an uncompressed format, but level count is not `1`",
                ))?
            }
            if matches!(view_dim, ImageViewDim::Dim2 | ImageViewDim::Dim2Array)
            {
                if !view_compatibility.contains(ImageViewCompatibility::DIM3_SLICE_AS_ARRAY) {
                    Err(err!(
                        "image's dimensions is `Dim3` and view_dim is `Dim2*`, but the image was not created with `DIM3_SLICE_AS_ARRAY`",
                    ))?
                }
                if level_count != 1 {
                    Err(err!(
                        "image's dimensions is `Dim3` and view's dimensions is `Dim2*`, but calculated level count is not `1`",
                    ))?
                }
                let lod_depth = image.extent().lod(attributes.range.base_mip_level).depth;
                if attributes.range.base_array_layer >= lod_depth {
                    Err(err!(
                        "`base_array_layer` is greater than or equal to the calculated depth ({lod_depth}) at `base_mip_level`",
                    ))?
                }
                let layer_count =
                    if let Some(layers) = attributes.range.layer_count {
                        if attributes.range.base_array_layer + layers.get() > lod_depth {
                            Err(err!(
                                "`base_array_layer + layer_count` is greater than the calculated depth ({lod_depth}) at `base_mip_level`",
                            ))?
                        }
                        layers.get()
                    } else {
                        lod_depth - attributes.range.base_array_layer
                    };
                if layer_count > 1 && matches!(view_dim, ImageViewDim::Dim2) {
                    Err(err!(
                        "`dimensions` is `Dim2`, but calculated layer count is greater than `1`",
                    ))?
                }
                let mut true_range = vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::from_raw(view_aspects.bits()),
                    base_mip_level: attributes.range.base_mip_level,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                };
                (vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::from_raw(view_aspects.bits()))
                    .base_mip_level(attributes.range.base_mip_level)
                    .level_count(level_count)
                    .base_array_layer(attributes.range.base_array_layer)
                    .layer_count(layer_count), true_range)
            } else {
                core::assert_matches!(view_dim, ImageViewDim::Dim3);
                if attributes.range.base_array_layer != 0 {
                    Err(err!(
                        "`dimensions` is `Dim3`, but `base_array_layer` is not `0`",
                    ))?
                }
                if attributes.range.layer_count.is_some_and(|l| l.get() != 1) {
                    Err(err!(
                        "`dimensions` is `Dim3`, but `layer_count` is not `Some(1)` or `None`",
                    ))?
                }
                let mut true_range = vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::from_raw(view_aspects.bits()),
                    base_mip_level: attributes.range.base_mip_level,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                };
                (true_range, true_range)
            }
        };
        let create_info = vk::ImageViewCreateInfo::default()
            .image(image.handle())
            .view_type(view_dim.into())
            .format(view_format.into())
            .components(attributes.component_mapping.into())
            .subresource_range(vk_subresource_range);
        let handle = unsafe { image.device()
            .functions()
            .create_image_view(&create_info, None)
            .describe_err("failed to create `VkImageView`")?
        };
        let guard = DropGuard::new((&image, handle),
            |(image, handle)| unsafe {
                image.device()
                    .functions()
                    .destroy_image_view(handle, None);
            }
        );
        let mut inner = NonNull::dangling();
        let mut tag_ptr = None;
        let mut tag_len = 0;
        let pack_allocation = if let Some(tag) = attributes.tag {
            tag_len = tag.len();
            let tag_ptr = tag_ptr.insert(NonNull::dangling());
            let allocation = pack_alloc(
                &allocator,
                [
                    pack_block(&mut inner, 1).unwrap(),
                    pack_block(tag_ptr, tag.len()).unwrap(),
                ]
            ).expect("allocation failed");
            unsafe {
                tag.as_ptr()
                    .copy_to_nonoverlapping(
                        tag_ptr.as_ptr(),
                        tag.len(),
                    );
            }
            allocation
        } else {
            pack_alloc(
                &allocator,
                [
                    pack_block(&mut inner, 1).unwrap(),
                ]
            ).expect("allocation failed")
        };
        assert!(pack_allocation.0.addr() == inner.addr());
        DropGuard::dismiss(guard);
        unsafe {
            inner.write(Inner {
                image,
                handle,
                format: view_format,
                dimensions: view_dim,
                true_range,
                ref_count: AtomicU64::new(1),
                pack_layout: pack_allocation.1,
            });
        }
        if let Some(tag) = attributes.tag {
            log::info!("created `ImageView` `{tag}`");
        }
        Ok(Self { inner, tag: (tag_ptr, tag_len), allocator })
    }

    unsafe fn inner(&self) -> &Inner {
        unsafe {
            self.inner.as_ref()
        }
    }

    /// Gets the tag assigned to this image view.
    #[inline]
    pub fn tag(&self) -> Option<&str> {
        let tag_ptr = self.tag.0?;
        Some(unsafe {
            str::from_utf8_unchecked(
                slice::from_raw_parts(
                    tag_ptr.as_ptr(),
                    self.tag.1,
                )
            )
        })
    }

    /// Returns the [`Format`] of this image view.
    ///
    /// This is specified in [`ImageViewAttributes::format`], where a `None` value indicates the
    /// format of the `image` used to create this view.
    #[inline]
    pub fn format(&self) -> Format {
        unsafe {
            self.inner().format
        }
    }

    /// Returns the [`ImageAspects`] of this view.
    pub fn aspects(&self) -> ImageAspects {
        ImageAspects::from_bits_retain(self.subresource_range().aspect_mask.as_raw())
    }

    /// The [`Image`] used to create this image view.
    #[inline]
    pub fn image(&self) -> &Image {
        unsafe {
            &self.inner().image
        }
    }

    /// The [`Device`] used to create this image view.
    #[inline]
    pub fn device(&self) -> &Device {
        self.image().device()
    }

    /// The dimensions of this image view.
    #[inline]
    pub fn dimensions(&self) -> ImageViewDim {
        unsafe { self.inner().dimensions }
    }

    #[inline]
    pub(crate) fn handle(&self) -> vk::ImageView {
        unsafe {
            self.inner().handle
        }
    }

    #[inline]
    pub(crate) fn subresource_range(&self) -> vk::ImageSubresourceRange {
        unsafe { self.inner().true_range }
    }
}

impl Clone for ImageView
{

    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            self.inner.as_ref().ref_count
                .add(1, atomic::Ordering::AcqRel);
        }
        Self {
            inner: self.inner,
            tag: self.tag,
            allocator: self.allocator.clone(),
        }
    }
}

impl Drop for ImageView
{

    #[inline]
    fn drop(&mut self) {
        unsafe {
            let ref_count = self.inner.as_ref()
                .ref_count
                .fetch_sub(1, atomic::Ordering::AcqRel);
            if ref_count == 1 {
                self.device()
                    .functions()
                    .destroy_image_view(self.handle(), None);
                let pack_layout = self.inner.as_ref().pack_layout;
                self.inner.drop_in_place();
                if let Some(tag) = self.tag() {
                    log::info!("destroyed `ImageView` `{tag}`")
                }
                self.allocator
                    .deallocate(self.inner.cast(), pack_layout);
            }
        }
    }
}

impl HasImageView for ImageView {
    
    #[inline]
    fn handle(&self) -> ImageViewHandle<'_> {
        ImageViewHandle::from_ref(self)
    }
}
