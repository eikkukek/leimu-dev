use std::time::Duration;

use core::mem::DropGuard;

use bytemuck::{ByteEq, ByteHash, NoUninit, CheckedBitPattern};

use tuhka::vk;

use crate::*;

use mem::{vec32, alloc_vec::{Vec32}};

use sync::{Arc, Mutex, MutexGuard};

pub use vk::ColorSpaceKHR as ColorSpace;

/// A swapchain image view acquired in the closure passed to [`Swapchain::present`].
pub struct SwapchainImageView<'a> {
    view: &'a ImageView,

}

impl HasImageView for SwapchainImageView<'_> {
 
    #[inline]
    fn handle(&self) -> ImageViewHandle<'_> {
        ImageViewHandle::from_ref(self.view)
    }
}

/// The format of a [`Surface`].
#[repr(C)]
#[derive(Clone, Copy, NoUninit, CheckedBitPattern, ByteEq, ByteHash)]
pub struct SurfaceFormat {
    /// The [`Format`] of the surface.
    pub format: Format,
    /// The color space of the surface.
    pub color_space: ColorSpace,
}

struct Fence {
    device: Device,
    handle: vk::Fence,
}

impl Fence {

    #[inline]
    pub fn signaled(device: Device) -> Self {
        Self {
            handle: unsafe {
                device.functions()
                    .create_fence(
                        &vk::FenceCreateInfo::default()
                            .flags(vk::FenceCreateFlags::SIGNALED),
                        None,
                    ).expect("failed to create `VkFence`")
            },
            device,
        } 
    }
}

impl Drop for Fence {

    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.device
                .functions()
                .destroy_fence(self.handle, None);
        }
    }
}

struct State {
    handle: Option<vk::SwapchainKHR>,
    image_views: Vec32<ImageView>,
    in_flight_fences: Vec32<Fence>,
    command_buffers: Vec32<CommandBuffer>,
    frame_index: u32,
    width: u32,
    height: u32,
    desired_image_count: u32,
    should_recreate: bool,
}

struct Inner<'a> {
    label: Option<CompactStr>,
    surface: Surface<'a>,
    device: Device,
    state: Mutex<State>,
    command_pool: CommandPool,
}

impl Drop for Inner<'_> {

    fn drop(&mut self) {
        let state = self.state.get_mut();
        if let Some(handle) = state.handle.take() {
            state.image_views.clear();
            let swapchain_functions = self.device
                .get_feature(vk_khr::swapchain::NAME)
                .unwrap()
                .functions(&self.device);
            unsafe {
                self.device.functions()
                    .device_wait_idle().ok();
                swapchain_functions.destroy_swapchain(
                    handle, None
                );
            }
            if let Some(label) = &self.label {
                log::info!("destroyed `Swapchain` `{label}`");
            }
        }
    }
}

/// Specifies parameters for [`Swapchain::present`].
pub struct PresentParameters<'a> {
    /// The current width of the window [surface] is created for.
    ///
    /// [surface]: Swapchain::surface
    pub window_width: u32,
    /// The current height of the window [surface] is created for.
    ///
    /// [surface]: Swapchain::surface
    pub window_height: u32,
    /// How many in-flight frames are desired.
    pub desired_image_count: u32,
    /// When this is set to `true`, obstructed pixels of the [surface] **may** be left unrendered.
    /// Fragment shaders may not execute for these pixels, and thus any side effects they would have
    /// had will also not occur.
    ///
    /// [surface]: Swapchain::surface
    pub clip: bool,
    /// Forces a swapchain recreation.
    pub force_recreate: bool,
    /// A user-defined closure used for selecting the surface format when the swapchain is created.
    ///
    /// The returned value is the index of the surface format in the argument slice. If the index is
    /// out of bounds, the swapchain creation will panic.
    pub select_surface_format: &'a mut dyn FnMut(&[SurfaceFormat]) -> usize,
    /// A user-defined closure selecting a [`CompositeAlphaBit`] from supported
    /// [`CompositeAlphaFlags`].
    ///
    /// If the selected bit is not contained in the argument bitmask, the swapchain creation will
    /// fail.
    pub select_composite_alpha: &'a mut dyn FnMut(CompositeAlphaFlags) -> CompositeAlphaBit,
}

/// A Vulkan swapchain.
///
/// To create a swapchain, use [`Device::create_swapchain`].
pub struct Swapchain<'a> {
    inner: Arc<Inner<'a>>,
}

impl<'a> Swapchain<'a> {

    pub(crate) fn new(
        label: Option<&str>,
        surface: Surface<'a>,
        device: Device,
        present_queue: DeviceQueue,
    ) -> Result<Self, Error> {
        if surface.instance().id() != device.instance().id() {
            panic!("`surface` instance id is not the same as `self` instance id")
        }
        if let Some(label) = label {
            log::trace!("validating `Swapchain` `{label}`");
        }
        if device
            .get_feature(vk_khr::swapchain::NAME)
            .is_none()
        {
            Err(err!(
                "attempting to create `Swapchain` when the `swapchain` feature is not enabled",
            ))?
        }
        let command_pool = device.create_command_pool(None, present_queue)
            .describe_err("failed to create command pool")?;
        if unsafe {
            surface.instance()
                .surface_functions().unwrap()
                .get_physical_device_surface_support(
                    device.gpu_info().handle,
                    command_pool.queue().family_index(),
                    surface.handle()
                )
        }.ok().is_none_or(|supported| !supported)
        {
            Err(err!(
                "queues in queue family index `{}` cannot be used for presentation",
                command_pool.queue().family_index(),
            ))?
        }
        if let Some(label) = label {
            log::trace!("created `Swapchain` `{label}`");
        }
        Ok(Self {
            inner: Inner {
                label: label.map(Into::into),
                surface,
                device,
                state: Mutex::new(State {
                    handle: None,
                    image_views: vec32![],
                    in_flight_fences: vec32![],
                    command_buffers: vec32![],
                    frame_index: 0,
                    width: 0,
                    height: 0,
                    desired_image_count: 1,
                    should_recreate: true,
                }),
                command_pool,
            }.arced()
        })
    }

    /// Gets the label assigned to this swapchain.
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.inner.label.as_deref()
    }

    /// Gets the [`Surface`] used to create this swapchain.
    #[inline]
    pub fn surface(&self) -> &Surface<'a> {
        &self.inner.surface
    }

    /// Gets the [`Device`] used to create this swapchain.
    #[inline]
    pub fn device(&self) -> &Device {
        &self.inner.device
    }

    fn create_swapchain(
        &self,
        lock: &mut MutexGuard<'_, State>,
        params: &mut PresentParameters<'_>,
    ) -> Result<(), Error> {
        if let Some(label) = self.label() {
            log::trace!("validating `Swapchain` {label}");
        }
        let surface = self.surface();
        let surface_functions = surface
            .instance()
            .surface_functions()
            .unwrap();
        let swapchain_functions = self.device()
            .get_feature(vk_khr::swapchain::NAME)
            .unwrap()
            .functions(self.device());
        if let Some(handle) = lock.handle.take() {
            unsafe {
                self.device()
                    .functions()
                    .device_wait_idle()
                    .describe_err("GPU hang")?;
                swapchain_functions.destroy_swapchain(handle, None);
            }
        }
        lock.image_views.clear();
        let physical_device = self.device().gpu_info().handle;
        let capabilities = unsafe { surface_functions
            .get_physical_device_surface_capabilities(
                physical_device,
                surface.handle()
            )
        }.describe_err(||
            if let Some(label) = self.surface().label() {
                format!("failed to get GPU surface capabilities from `Surface` {label}")
            } else {
                "failed to get GPU surface capabilities".to_string()
            }
        )?;
        let present_modes = unsafe {
            let len = surface_functions.get_physical_device_surface_present_modes_len(
                physical_device,
                surface.handle(),
            ).describe_err(
                if let Some(label) = self.surface().label() {
                    format!("failed to get GPU surface present modes from `Surface` {label}")
                } else {
                    "failed to get GPU surface present modes".to_string()
                }
            )?.0;
            let mut present_modes = Vec32::with_exact_capacity(len);
            present_modes.set_len(len);
            surface_functions.get_physical_device_surface_present_modes(
                physical_device,
                surface.handle(),
                &mut present_modes,
            ).unwrap();
            present_modes
        };
        let present_mode = present_modes
            .iter()
            .copied()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX_KHR)
            .unwrap_or(vk::PresentModeKHR::FIFO_KHR);

        let image_count = lock.desired_image_count
            .clamp(
                capabilities.min_image_count,
                if capabilities.max_image_count == 0 {
                    capabilities.min_image_count
                } else {
                    capabilities.max_image_count
                }
            );
        let surface_formats = unsafe {
            let len = surface_functions.get_physical_device_surface_formats_len(
                physical_device,
                surface.handle(),
            ).describe_err(
                if let Some(label) = self.label() {
                    format!("failed to get GPU surface present modes from `Surface` {label}")
                } else {
                    "failed to get GPU surface present modes".to_string()
                })?.0;
            let mut formats = Vec32::with_exact_capacity(len);
            formats.set_len(len);
            surface_functions.get_physical_device_surface_formats(
                physical_device,
                surface.handle(),
                &mut formats,
            ).unwrap();
            formats
        };
        let surface_format = surface_formats[
            (params.select_surface_format)(bytemuck::checked::cast_slice(&surface_formats))
        ];
        let format: Format = bytemuck::checked::cast(surface_format.format);
        let colors_space = surface_format.color_space;
        let width = params.window_width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        );
        let height = params.window_height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        );
        let composite_alpha = (params.select_composite_alpha)(
            CompositeAlphaFlags::from_bits_retain(capabilities.supported_composite_alpha.as_raw())
        );
        let vk_composite_alpha: vk::CompositeAlphaFlagsKHR = composite_alpha.into();
        if !capabilities.supported_composite_alpha.contains(vk_composite_alpha) {
            Err(err!(
                "selected `CompositeAlphaBit` {composite_alpha:?} is not supported on this surface",
            ))?
        }
        if let Some(label) = self.label() {
            log::trace!("selected min image count {image_count} for `Swapchain` {label}");
            log::trace!("selected present mode {present_mode} for `Swapchain` {label}");
            log::trace!("selected image format {format} in color space {colors_space} for `Swapchain` {label}");
            log::trace!("selected image width {width} for `Swapchain` {label}");
            log::trace!("selected image height {height} for `Swapchain` {label}");
            log::trace!("selected composite alpha {composite_alpha:?} for `Swapchain` {label}");
        }
        if !capabilities.supported_usage_flags.contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            Err(err!(
                "can't use swapchain's images as a color attachment",
            ))?
        }
        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.handle())
            .min_image_count(image_count)
            .image_format(format.into())
            .image_color_space(colors_space)
            .image_extent(vk::Extent2D::default().width(width).height(height))
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk_composite_alpha)
            .present_mode(present_mode)
            .clipped(params.clip);
        let handle = unsafe { swapchain_functions
            .create_swapchain(&create_info, None)
        }.describe_err("failed to create `VkSwapchain`")?;
        let guard = DropGuard::new(
            handle,
            |handle| unsafe {
                swapchain_functions
                    .destroy_swapchain(handle, None);
            },
        );
        let image_handles = unsafe {
            let len = swapchain_functions
                .get_swapchain_images_len(handle)
                .describe_err("failed to create swapchain images")?.0;
            let mut images = Vec32::with_exact_capacity(len);
            images.set_len(len);
            swapchain_functions.get_swapchain_images(handle, &mut images).unwrap();
            images
        };
        let device = self.device();
        let image_views: Vec32<_> = image_handles
            .into_iter()
            .map(|handle| {
                let image = unsafe {
                    Image::from_swapchain_handle(
                        device.clone(),
                        handle,
                        format,
                        ImageExtent::default()
                            .width(width)
                            .height(height)
                    )
                };
                device.create_image_view(
                    image,
                    ImageViewAttributes {
                        label: None,
                        format: None,
                        dimensions: ImageViewDim::Dim2,
                        range: default(),
                        component_mapping: default(),
                    },
                    Global,
                ).expect("failed to create swapchain image view")
            }).collect();
        lock.in_flight_fences.resize_with(
            image_count,
            || Fence::signaled(device.clone()),
        );
        lock.command_buffers.try_resize_with(image_count,
            || self.inner.command_pool
                .allocate_command_buffer()
                .map(CommandBuffer::with_binary_semaphore)
        ).describe_err("failed to allocate command buffer")?;
        DropGuard::dismiss(guard);
        lock.frame_index = 0;
        lock.handle = Some(handle);
        lock.image_views = image_views;
        lock.width = width;
        lock.height = height;
        lock.desired_image_count = params.desired_image_count;
        lock.should_recreate = false;
        Ok(())
    }

    fn acquire_next_image(
        &self,
        mut params: PresentParameters<'_>,
        timeout: Duration,
    ) -> Result<(u32, MutexGuard<'_, State>), Error>
    { 
        let mut lock = self.inner.state.lock();
        if lock.handle.is_none() ||
            lock.width != params.window_width ||
            lock.height != params.window_height ||
            lock.desired_image_count != params.desired_image_count ||
            params.force_recreate
        {
            lock.desired_image_count = params.desired_image_count;
            self.create_swapchain(&mut lock, &mut params)?;
        }
        let handle = lock.handle.unwrap();
        let fence = lock.in_flight_fences[lock.frame_index as usize].handle;
        let device = self.device();
        let swapchain_device = device
            .get_feature(vk_khr::swapchain::NAME)
            .unwrap()
            .functions(device);
        let timeout_nanos = timeout.as_nanos() as u64;
        unsafe { 
            device.functions()
                .reset_fences(&[fence]).ok();
            let index = match swapchain_device
                .acquire_next_image(
                    handle,
                    timeout_nanos,
                    vk::Semaphore::null(),
                    fence,
                )
            {
                Ok((index, res)) => {
                    if res == vk::Result::SUBOPTIMAL_KHR {
                        lock.should_recreate = true;
                    } else if res != vk::Result::SUCCESS {
                        Err(Error::new(
                            Some(res),
                            "failed to acquire next image",
                        ))?
                    }
                    index
                },
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        return self.acquire_next_image(params, timeout)
                    } else {
                        Err(Error::new(
                            Some(err),
                            "failed to acquire next image",
                        ))?
                    }
                },
            };
            let res = device.functions()
                .wait_for_fences(
                    &[fence],
                    true,
                    timeout.as_nanos() as u64,
                ).describe_err("failed to wait for fence")?;
            if res == vk::Result::TIMEOUT {
                Err(err!(
                    "timeout `{} seconds` reached when waiting for fences",
                    timeout.as_secs_f64(),
                ))?
            }
            Ok((index, lock))
        }
    }

    /// Presents to the [`Surface`] of this swapchain.
    pub fn present<F>(
        &self,
        present_parameters: PresentParameters<'_>,
        timeout: Duration,
        alloc: AnyAllocator,
        f: F,
    ) -> Result<(), Error>
        where
            F: FnOnce(SwapchainImageView<'_>) -> Result<(), Error>,
    {
        let device = self.device();
        let (image_index, mut lock) = self.acquire_next_image(present_parameters, timeout)?;
        let view = lock.image_views[image_index as usize].clone();
        f(SwapchainImageView { view: &view })?;
        // Wait for these operations to finish.
        let command_pool = &self.inner.command_pool;
        let queue = command_pool.queue();
        let lock = &mut *lock;
        let command_buffer = &mut lock
            .command_buffers[image_index as usize];
        let mut recorder = command_buffer.start_recording(timeout)?;
        recorder.transition_image_state(
            view,
            ImageState {
                stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                access: vk::AccessFlags2::COLOR_ATTACHMENT_READ,
                layout: vk::ImageLayout::PRESENT_SRC_KHR,
                queue_family_index: queue.family_index(),
            },
        );
        recorder.finish();
        let _poll = device.poll([&mut *command_buffer], alloc)?;
        lock.frame_index = (lock.frame_index + 1) % lock.in_flight_fences.len();
        let swapchain_functions = device
            .get_feature(vk_khr::swapchain::NAME)
            .unwrap()
            .functions(device);
        let swapchains = &[lock.handle.unwrap()];
        let image_indices = &[image_index];
        unsafe {
            match swapchain_functions
                .queue_present(
                    queue.handle(),
                    &vk::PresentInfoKHR::default()
                        .swapchains(swapchains)
                        .image_indices(image_indices)
                        .wait_semaphores(&[command_buffer.get_binary_semaphore().unwrap()])
                )
            {
                Ok(res) => {
                    if res == vk::Result::SUBOPTIMAL_KHR {
                        lock.should_recreate = true;
                    }
                    Ok(())
                },
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        lock.should_recreate = true;
                        Ok(())
                    } else {
                        Err(Error::new(
                            Some(err),
                            "failed to present"
                        ))
                    }
                }
            }
        }
    }
}
