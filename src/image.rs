mod view;
mod state;

use std::alloc;

use core::{
    mem::DropGuard,
    ptr::NonNull,
    slice,
    hash::{self, Hash},
    borrow::Borrow,
};

use bytemuck::{NoUninit, ByteEq, ByteHash, AnyBitPattern};
use bitflags::bitflags;

use tuhka::vk;

use crate::*;

use sync::{
    atomic::{AtomicU64, self},
    Mutex,
};

use mem::pack::{pack_alloc, pack_block};

pub use view::*;
pub(crate) use state::*;

bitflags! {
    /// A bitmask specifying what kinds of image views **can** be created of an [`Image`].
    #[repr(transparent)]
    #[derive(Default, Clone, Copy, NoUninit, ByteEq, ByteHash)]
    pub struct ImageViewCompatibility: u32 {
        /// Specfies that image views with the [`ImageViewDim::Cube`] and
        /// [`ImageViewDim::CubeArray`] dimensions **can** be created.
        ///
        /// # Valid usage
        /// If this bit is set:
        /// - [`dimensions`][ImageAttributes::dimensions] **must** be [`ImageDim::Dim2`],
        /// - [`extent`][ImageAttributes::extent] `width` and `height` **must** be equal, and
        /// - [`array_layers`][ImageAttributes::array_layers] **must** be greater than or equal to `6`.
        const CUBE = 0x1;
        /// Specifies that image views with the [`ImageViewDim::Dim2Array`] and
        /// [`ImageViewDim::Dim2`]  dimensions **can** be created when [`dimensions`][1]
        /// is [`ImageDim::Dim3`].
        ///
        /// # Valid usage
        /// If this bit is set, [`dimensions`][1] **must** be [`ImageDim::Dim3`].
        ///
        /// [1]: ImageAttributes::dimensions
        const DIM3_SLICE_AS_ARRAY = 0x2;
        /// Specifies that [`view_formats`][1] **can** include formats that are not [compatible],
        /// but are [size compatible] with [`format`][2].
        ///
        /// # Valid usage
        /// - If this bit is set, [`format`][2] **must** be a [compressed format].
        ///
        /// [1]: ImageAttributes::view_formats
        /// [2]: ImageAttributes::format
        /// [compatible]: Format::is_compatible_with
        /// [size compatible]: Format::is_size_compatible_with
        /// [compressed format]: Format::is_compressed
        const REINTERPRET_TEXEL_BLOCKS = 0x4;
    }
}

/// Specifies the extent of an [`Image`].
#[repr(C)]
#[derive(Clone, Copy, NoUninit, ByteEq, ByteHash, AnyBitPattern, Debug)]
pub struct ImageExtent {
    /// The width of the image.
    pub width: u32,
    /// The height of the image.
    pub height: u32,
    /// The depth of the image.
    pub depth: u32,
}

impl Default for ImageExtent {
    
    #[inline]
    fn default() -> Self {
        Self { width: 1, height: 1, depth: 1, }
    }
}

impl ImageExtent {

    /// Copies self and sets the width.
    #[must_use]
    #[inline]
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Copies self and sets the height.
    #[must_use]
    #[inline]
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Copies self and sets the depth.
    #[must_use]
    #[inline]
    pub fn depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    /// Returns whether any of `width`, `height` or `depth` is zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.width == 0 ||
        self.height == 0 ||
        self.depth == 0
    }

    /// Maps the extent with a closure.
    #[must_use]
    #[inline]
    pub fn map<F>(self, mut f: F) -> Self
        where F: FnMut(u32) -> u32
    {
        Self {
            width: f(self.width),
            height: f(self.height),
            depth: f(self.depth),
        }
    }

    /// Gets the extent of an image with these dimensions at `mip_level`.
    #[must_use]
    #[inline]
    pub fn lod(self, mip_level: u32) -> Self {
        self.map(|x| (x >> mip_level).max(1))
    }

    /// Clamps this extent with `min` and `max` component-wise.
    #[must_use]
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            width:  self.width.clamp(min.width, max.width),
            height: self.height.clamp(min.height, max.height),
            depth:  self.depth.clamp(min.depth, max.depth),
        }
    }
}

impl From<ImageExtent> for vk::Extent3D {
    #[inline]
    fn from(value: ImageExtent) -> Self {
        bytemuck::cast(value)
    }
}

impl From<vk::Extent3D> for ImageExtent {
    #[inline]
    fn from(value: vk::Extent3D) -> Self {
        bytemuck::cast(value)
    }
}

/// Specifies image creation limits for a set of parameters specified in
/// [`Device::get_image_creation_limits`].
///
/// The limits are documented in each field of this structure.
#[derive(Clone, Copy)]
pub struct ImageCreationLimits {
    /// The maximum number of [mip levels] that an [`Image`] of this type **can** have.
    ///
    /// [mip levels]: ImageAttributes::mip_levels
    pub max_mip_levels: u32,
    /// The maximum number of [array layers] that an [`Image`] of this type **can** have.
    ///
    /// [array layers]: ImageAttributes::array_layers
    pub max_array_layers: u32,
    /// The minimum amount of [array layers] that an [`Image`] of this type **can** have.
    ///
    /// This is either `1` or `6` depending on whether the image's view compatibility contains
    /// [`ImageViewCompatibility::CUBE`].
    pub min_array_layers: u32,
    /// The maximum component-wise extent that an [`Image`] of this type **can** have.
    pub max_extent: ImageExtent,
    /// Whether the image's [extent] **must** have the same width and height:
    ///
    /// [extent]: ImageAttributes::extent
    pub require_cube_extent: bool,
    /// A bitmask of valid [sample counts] that an [`Image`] of this type **can** have.
    ///
    /// [sample counts]: ImageAttributes::samples
    pub sample_counts: SampleCounts,
    pub(crate) create_flags: vk::ImageCreateFlags,
}

/// Specifies parameters for [`Image`] creation.
///
/// This **can** be used to create an [`Image`] with [`Device::create_image`].
///
/// # Valid usage
/// The valid parameters of this structure is described in each field and in
/// [`Device::query_image_creation_limits`].
pub type ImageAttributes<'a> = attributes::ImageAttributes<&'a str, &'a [Format]>;

impl ImageAttributes<'_> {

    /// Calculates how much extra (CPU) memory the created [`Image`] will allocate.
    #[inline]
    pub fn calculate_allocation_size(&self) -> usize {
        size_of::<Inner>() +
        ImageRangeState::calculate_allocation_size(
            self.format.aspects(),
            self.mip_levels,
            self.array_layers
        ) * 2 +
        size_of_val(self.view_formats) + self.label.map(|l| l.len()).unwrap_or_default()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HandleType {
    Created = 0,
    Swapchain,
}

crate::__id!(
    /// A unique [`Image`] identifier.
    ImageId
);

struct States {
    scoped_state: ImageRangeState,
    state: ImageRangeState,
}

struct Inner {
    device: Device,
    handle: vk::Image,
    id: ImageId,
    handle_type: HandleType,
    ref_count: AtomicU64,
    format: Format,
    mip_levels: u32,
    array_layers: u32,
    samples: SampleCountBit,
    usage: ImageUsages,
    stencil_usage: Option<ImageUsages>,
    dimensions: ImageDim,
    view_compatibility: ImageViewCompatibility,
    view_formats: (NonNull<Format>, usize),
    extent: ImageExtent,
    memory: MemoryAllocation,
    state: Mutex<States>,
    semaphore_handle: vk::Semaphore,
    semaphore_value: AtomicU64,
    last_poll_id: Mutex<u64>,
    pack_layout: alloc::Layout,
}

/// A GPU-accessible image/texture.
///
/// To create an image, use [`Device::create_image`].
pub struct Image {
    label: (Option<NonNull<u8>>, usize),
    inner: NonNull<Inner>,
    allocator: AnyAllocator,
}

impl Image
{

    pub(crate) fn new(
        device: Device,
        attributes: &ImageAttributes<'_>,
        allocator: AnyAllocator,
    ) -> Result<Self, Error> {
        if let Some(label) = attributes.label {
            log::info!("validating parameters for `Image` `{label}`");
        }
        let limits = device.query_image_creation_limits(
            attributes.format,
            attributes.dimensions,
            attributes.usage,
            attributes.stencil_usage,
            attributes.view_compatibility,
            attributes.view_formats
        ).describe_err("`get_image_creation_limits` failed")?;
        if attributes.mip_levels > limits.max_mip_levels {
            Err(err!(
                "`mip_levels` is greater than the maximum mip levels `{}` for an image of this type",
                limits.max_mip_levels,
            ))?
        }
        if attributes.array_layers > limits.max_array_layers {

            Err(err!(
                "`array_layers` is greater than the maximum array layers `{}` for an image of this type",
                limits.max_array_layers,
            ))?
        }
        if attributes.array_layers < limits.min_array_layers {
            Err(err!(
                "`array_layers` is less than the minimum array layers `{}` for an image of this type",
                limits.min_array_layers,
            ))?
        }
        if attributes.extent.width > limits.max_extent.width {
            Err(err!(
                "`extent.width` is greater than the maximum extent width `{}` for an image of this type",
                limits.max_extent.width,
            ))?
        }
        if attributes.extent.height > limits.max_extent.height {
            Err(err!(
                "`extent.height` is greater than the maximum extent height `{}` for an image of this type",
                limits.max_extent.height,
            ))?
        }
        if attributes.extent.depth > limits.max_extent.depth  {
            Err(err!(
                "`extent.depth` is greater than the maximum extent depth `{}` for an image of this type",
                limits.max_extent.depth,
            ))?
        }
        if attributes.extent.is_zero() {
            Err(err!(
                "`extent` is zero"
            ))?
        }
        if attributes.mip_levels == 0 {
            Err(err!(
                "`mip_levels` is zero"
            ))?
        }
        if !limits.sample_counts.contains(attributes.samples.to_flags())
        {
            Err(err!(
                "sample count `{}` is not supported for an image of this type (supported sample counts are `{}`)",
                attributes.samples.to_flags(), limits.sample_counts,
            ))?
        }
        if attributes.samples != SampleCountBit::X1
        {
            if attributes.dimensions != ImageDim::Dim2
                || limits.create_flags.contains(vk::ImageCreateFlags::CUBE_COMPATIBLE)
                || attributes.mip_levels != 1 {
                Err(err!(
                    "sample count `{}` is not supported for an image of this type (supported sample counts are `{}`)",
                    attributes.samples.to_flags(), SampleCounts::X1,
                ))?
            }
            if (attributes.usage.contains(ImageUsages::STORAGE)
                    || attributes.stencil_usage.is_some_and(|usage| usage.contains(ImageUsages::STORAGE))
                ) && !device.is_feature_enabled(vk_10::shader_storage_image_multisample::NAME)
            {
                Err(err!(
                    "`samples` is not `X1` and `usage` contains `STORAGE`, but the `vk_10::shader_storage_image_multisample` feature is not enabled",
                ))?
            }
        }
        if attributes.usage.contains(
            ImageUsages::COLOR_ATTACHMENT
                | ImageUsages::DEPTH_STENCIL_ATTACHMENT
        ) {
            let limits = &device.gpu_info().limits;
            if attributes.extent.width > limits.max_framebuffer_width() {
                Err(err!(
                    "`usage` indicates a framebuffer attachment, but `extent.width` is greater then `max_frame_buffer_width` `{}`",
                    limits.max_framebuffer_width(),
                ))?
            }
            if attributes.extent.height > limits.max_framebuffer_height() {
                Err(err!(
                    "`usage` indicates a framebuffer attachment, but `extent.height` is greater then `max_frame_buffer_height` `{}`",
                    limits.max_framebuffer_height(),
                ))?
            }
        }
        if limits.require_cube_extent && attributes.extent.width != attributes.extent.height {
            Err(err!(
                "`ImageViewCompatibility::CUBE` is set, but `extent.width != extent.height`",
            ))?
        }
        if attributes.format == Format::Undefined {
            Err(err!(
                "`format` is `Undefined",
            ))?
        }
        let mut view_info = None;
        let mut stencil_usage_info = None;
        let mut create_info = vk::ImageCreateInfo::default()
            .flags(limits.create_flags)
            .image_type(attributes.dimensions.into())
            .format(attributes.format.into())
            .extent(attributes.extent.into())
            .mip_levels(attributes.mip_levels)
            .array_layers(attributes.array_layers)
            .samples(attributes.samples.into())
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(attributes.usage.into())
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        if !attributes.view_formats.is_empty() {
            let view_info = view_info.insert(vk::ImageFormatListCreateInfo::default()
                .view_formats(bytemuck::cast_slice(attributes.view_formats)));
            create_info = create_info.push_next(view_info);
        }
        if let Some(usage) = attributes.stencil_usage {
            let stencil_usage_info = stencil_usage_info.insert(vk::ImageStencilUsageCreateInfo::default()
                .stencil_usage(usage.into())
            );
            create_info = create_info.push_next(stencil_usage_info);
        }
        let handle = unsafe { device.functions()
            .create_image(&create_info, None)
            .describe_err("failed to create `VkImage`")?
        };
        let mut guard = DropGuard::new((&device, handle, None),
            |(device, handle, memory)| unsafe {
                device.functions()
                    .destroy_image(handle, None);
                if let Some(mem) = memory {
                    device.deallocate_memory(mem);
                }
            }
        );
        let memory_requirements = unsafe {
            device.functions()
                .get_image_memory_requirements(handle)
        };
        let memory_type_indices = attributes.memory_type
            .get_memory_type_indices(device.gpu_info());
        let memory = device.allocate_memory(
            memory_requirements.size,
            memory_requirements.alignment,
            &memory_type_indices
        ).describe_err("failed to allocate memory for image")?;
        guard.2 = Some(memory);
        unsafe {
            device.functions()
                .bind_image_memory(
                    handle,
                    memory.handle,
                    memory.offset
                ).describe_err("failed to bind image memory")?
        }
        let mut inner = NonNull::dangling();
        let mut view_formats_ptr = NonNull::dangling();
        let mut label_ptr = None;
        let mut label_len = 0;
        let aspects = attributes.format
            .aspects();
        let states_len =
            aspects.bits().count_ones() as usize *
            attributes.mip_levels as usize *
            attributes.array_layers as usize;
        let mut scoped_states = NonNull::dangling();
        let mut states = NonNull::dangling();
        let pack_allocation = if let Some(label) = attributes.label {
            label_len = label.len();
            let label_ptr = label_ptr.insert(NonNull::dangling());
            let allocation = pack_alloc(
                &allocator,
                [
                    pack_block(&mut inner, 1).unwrap(),
                    pack_block(&mut scoped_states, states_len).unwrap(),
                    pack_block(&mut states, states_len).unwrap(),
                    pack_block(&mut view_formats_ptr, attributes.view_formats.len()).unwrap(),
                    pack_block(label_ptr, label.len()).unwrap(),
                ]
            ).expect("allocation failed");
            unsafe {
                label.as_ptr()
                    .copy_to_nonoverlapping(
                        label_ptr.as_ptr(),
                        label.len(),
                    );
            }
            allocation
        } else {
            pack_alloc(
                &allocator,
                [
                    pack_block(&mut inner, 1).unwrap(),
                    pack_block(&mut scoped_states, states_len).unwrap(),
                    pack_block(&mut states, states_len).unwrap(),
                    pack_block(&mut view_formats_ptr, attributes.view_formats.len()).unwrap(),
                ]
            ).expect("allocation failed")
        };
        assert!(pack_allocation.0.addr() == inner.addr());
        let mut timeline_info = vk::SemaphoreTypeCreateInfo
            ::default()
            .semaphore_type(vk::SemaphoreType::TIMELINE);
        let semaphore_handle = unsafe {
            device.functions()
                .create_semaphore(
                    &vk::SemaphoreCreateInfo
                        ::default()
                        .push_next(&mut timeline_info),
                    None,
                ).expect("failed to create `VkSemaphore`")
        };
        DropGuard::dismiss(guard);
        unsafe {
            attributes.view_formats.as_ptr()
                .copy_to_nonoverlapping(
                    view_formats_ptr.as_ptr(),
                    attributes.view_formats.len()
                );
            inner.write(Inner {
                device,
                id: ImageId::next(),
                handle,
                handle_type: HandleType::Created,
                ref_count: AtomicU64::new(1),
                format: attributes.format,
                mip_levels: attributes.mip_levels,
                array_layers: attributes.array_layers,
                samples: attributes.samples,
                usage: attributes.usage,
                stencil_usage: attributes.stencil_usage,
                dimensions: attributes.dimensions,
                view_compatibility: attributes.view_compatibility,
                view_formats: (view_formats_ptr, attributes.view_formats.len()),
                extent: attributes.extent,
                state: Mutex::new(States {
                    state: ImageRangeState::new(
                        states,
                        aspects,
                        attributes.mip_levels,
                        attributes.array_layers
                    ),
                    scoped_state: ImageRangeState::new(
                        scoped_states,
                        aspects,
                        attributes.mip_levels,
                        attributes.array_layers
                    ),
                }),
                semaphore_handle,
                semaphore_value: AtomicU64::new(0),
                last_poll_id: Mutex::new(1),
                memory,
                pack_layout: pack_allocation.1,
            });
        }
        if let Some(label) = attributes.label {
            log::info!("created `Image` `{label}`")
        }
        Ok(Self {
            label: (label_ptr, label_len),
            inner,
            allocator,
        })
    }

    /// Gets the unique [`ImageId`] assigned to this image.
    #[inline]
    pub fn id(&self) -> ImageId {
        self.inner().id
    }

    /// Gets the label assigned to this image.
    #[inline]
    pub fn label(&self) -> Option<&str> {
        let label_ptr = self.label.0?;
        Some(unsafe {
            str::from_utf8_unchecked(
                slice::from_raw_parts(
                    label_ptr.as_ptr(),
                    self.label.1,
                )
            )
        })
    }

    /// Gets the [`Device`] used to create this image.
    #[inline]
    pub fn device(&self) -> &Device {
        unsafe {
            &self.inner.as_ref().device
        }
    } 

    #[inline]
    fn inner(&self) -> &Inner {
        unsafe {
            self.inner
                .as_ref()
        }
    }

    /// Gets the format of this image, as specified in [`ImageAttributes::format`].
    #[inline]
    pub fn format(&self) -> Format {
        self.inner().format
    }

    /// Returns all valid [`ImageAspects`] for this image.
    #[inline]
    pub fn aspects(&self) -> ImageAspects {
        self.format().aspects()
    }

    /// Gets the mip levels of this image, as specified in [`ImageAttributes::mip_levels`].
    #[inline]
    pub fn mip_levels(&self) -> u32 {
        self.inner().mip_levels
    }

    /// Gets the array layers of this image, as specified in [`ImageAttributes::array_layers`].
    #[inline]
    pub fn array_layers(&self) -> u32 {
        self.inner().array_layers
    }

    /// Gets the number of samples per texel of this image, as specified in [`ImageAttributes::samples`].
    #[inline]
    pub fn samples(&self) -> SampleCountBit {
        self.inner().samples
    }

    /// Gets the usage of this image, as specified in [`ImageAttributes::usage`].
    #[inline]
    pub fn usage(&self) -> ImageUsages {
        self.inner().usage
    }

    /// Gets the optional separate stencil usage of this image, as specified in
    /// [`ImageAttributes::stencil_usage`].
    #[inline]
    pub fn stencil_usage(&self) -> Option<ImageUsages> {
        self.inner().stencil_usage
    }

    /// Gets the number of dimensions in this image, as specified in
    /// [`ImageAttributes::dimensions`]
    #[inline]
    pub fn dimensions(&self) -> ImageDim {
        self.inner().dimensions
    }

    /// Gets what kind of image views can be created of this image, as specified in
    /// [`ImageAttributes::view_compatibility`].
    #[inline]
    pub fn view_compatibility(&self) -> ImageViewCompatibility {
        self.inner().view_compatibility
    }

    /// Gets compatible formats that **can** be used by image views of this image, as specified
    /// in [`ImageAttributes::view_formats`].
    ///
    /// The actual format of the image can always be used to create image views of this image.
    #[inline]
    pub fn view_formats(&self) -> &[Format] {
        unsafe {
            let (ptr, len) = self.inner().view_formats;
            slice::from_raw_parts(
                ptr.as_ptr(),
                len
            )
        }
    }

    /// Gets the extent of this image, as specified in [`ImageAttributes::extent`].
    #[inline]
    pub fn extent(&self) -> ImageExtent {
        self.inner().extent
    }

    #[inline]
    pub(crate) fn handle(&self) -> vk::Image {
        self.inner().handle
    }

    #[inline]
    pub(crate) fn memory(&self) -> &MemoryAllocation {
        &self.inner().memory
    }

    pub(crate) unsafe fn scoped<'a>(
        &'a self,
        poll_id: u64,
        command_buffer: vk::CommandBuffer,
    ) -> ImageScoped<'a> {
        let mut last_poll_id = self.inner().last_poll_id.lock();
        let mut signal_semaphore = None;
        if *last_poll_id != poll_id {
            if *last_poll_id < poll_id {
                *last_poll_id = poll_id;
            }
            signal_semaphore = Some((
                self.inner().semaphore_handle,
                self.inner().semaphore_value.fetch_add(
                    1, atomic::Ordering::Relaxed
                ) + 1,
            ));
        }
        let mut lock = self.inner().state.lock();
        unsafe {
            lock.scoped_state.zero_states();
        }
        ImageScoped {
            image: &self,
            lock,
            command_buffer,
            signal_semaphore,
        }
    }
}

impl Clone for Image
{

    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            self.inner.as_ref().ref_count
                .add(1, atomic::Ordering::Release);
        }
        Self {
            inner: self.inner,
            label: self.label,
            allocator: self.allocator.clone(),
        }
    }
}

impl Drop for Image
{

    #[inline]
    fn drop(&mut self) {
        unsafe {
            let ref_count = self.inner.as_ref()
                .ref_count
                .fetch_sub(1, atomic::Ordering::AcqRel);
            if ref_count == 1 {
                if self.inner().handle_type == HandleType::Created {
                    self.device()
                        .functions()
                        .destroy_image(self.handle(), None);
                    self.device()
                        .deallocate_memory(*self.memory());
                }
                self.device()
                    .functions()
                    .destroy_semaphore(
                        self.inner().semaphore_handle,
                        None
                    );
                if let Some(label) = self.label() {
                    log::info!("destroyed `Image` `{label}`")
                }
                let pack_layout = self.inner.as_ref().pack_layout;
                self.inner.drop_in_place();
                self.allocator
                    .deallocate(self.inner.cast(), pack_layout);
            }
        }
    }
}

impl Image {

    pub(crate) unsafe fn from_swapchain_handle(
        device: Device,
        handle: vk::Image,
        format: Format,
        extent: ImageExtent,
    ) -> Self {
        let mut timeline_info = vk::SemaphoreTypeCreateInfo
            ::default()
            .semaphore_type(vk::SemaphoreType::TIMELINE);
        let semaphore_handle = unsafe {
            device.functions()
                .create_semaphore(
                    &vk::SemaphoreCreateInfo
                        ::default()
                        .push_next(&mut timeline_info),
                    None,
                ).expect("failed to create `VkSemaphore`")
        };
        let mut inner = NonNull::dangling();
        let aspects = format.aspects();
        let states_len = aspects.bits().count_ones() as usize;
        let mut scoped_states = NonNull::dangling();
        let mut states = NonNull::dangling();
        let pack_allocation = pack_alloc(
            &AnyAllocator::Global,
            [
                pack_block(&mut inner, 1).unwrap(),
                pack_block(&mut scoped_states, states_len).unwrap(),
                pack_block(&mut states, states_len).unwrap(),
            ]
        ).expect("allocation failed");
        assert!(pack_allocation.0.addr() == inner.addr());
        let aspects = format.aspects();
        unsafe {
            inner.write(Inner {
                device,
                id: ImageId::next(),
                handle,
                handle_type: HandleType::Swapchain,
                ref_count: AtomicU64::new(1),
                format,
                mip_levels: 1,
                array_layers: 1,
                samples: SampleCountBit::X1,
                usage: ImageUsages::COLOR_ATTACHMENT,
                stencil_usage: None,
                dimensions: ImageDim::Dim2,
                view_compatibility: default(),
                view_formats: (NonNull::dangling(), 0),
                extent,
                state: Mutex::new(States {
                    state: ImageRangeState::new(
                        states,
                        aspects,
                        1,
                        1,
                    ),
                    scoped_state: ImageRangeState::new_scoped(
                        scoped_states,
                        aspects,
                        1,
                        1,
                    ),
                }),
                semaphore_handle,
                semaphore_value: AtomicU64::new(0),
                last_poll_id: Mutex::new(1),
                memory: MemoryAllocation { handle: default(), offset: 0, size: 0, memory_type_index: 0, },
                pack_layout: pack_allocation.1,
            });
        }
        Self {
            label: (None, 0),
            inner,
            allocator: AnyAllocator::Global,
        }
    }
}

impl Hash for Image {

    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Borrow<ImageId> for Image {

    #[inline]
    fn borrow(&self) -> &ImageId {
        &self.inner().id
    }
}

impl PartialEq for Image {
    
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Image {}
