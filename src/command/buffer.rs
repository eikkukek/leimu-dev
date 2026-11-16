use core::{
    time::Duration,
    alloc::AllocError,
    mem::DropGuard,
};

use tuhka::vk;

use crate::*;

use mem::alloc_vec::FixedVec32;

use smallvec::{SmallVec, smallvec};

struct ImageRangeStates {
    states: SmallVec<[(ImageState, vk::ImageSubresourceRange); 4]>,
}

#[derive(Default)]
pub(super) struct ResourceStateCache {
    images: IndexMap<Image, ImageRangeStates>,
}

impl ResourceStateCache {

    /// Inserts an image range and state to this cache.
    ///
    /// # Safety
    /// `range` **must** be a valid range for `image'. The validity of `range` is not checked at
    /// any point.
    #[inline]
    pub unsafe fn insert_image(
        &mut self,
        image: Image,
        state: ImageState,
        range: vk::ImageSubresourceRange,
    ) {
        self.images
            .entry(image)
            .and_modify(|ranges| {
                ranges.states.push((state, range));
            }).or_insert_with(|| {
                ImageRangeStates { states: smallvec![(state, range)] }
            });
    }

    /// Returns a vector of signal semaphores, where the first semaphore is `cb_signal`.
    unsafe fn pipeline_barriers<A: Allocator>(
        &mut self,
        poll_id: u64,
        command_buffer: vk::CommandBuffer,
        additional_cap: u32,
        alloc: A,
    ) -> Result<FixedVec32<(vk::Semaphore, u64), A>, AllocError> {
        let mut signal_semaphores = FixedVec32::with_capacity_in(
            self.images.len() as u32 + additional_cap,
            alloc,
        ).map_err(|_| AllocError)?;
        for (image, ranges) in &self.images {
            let mut scoped = unsafe {
                image.scoped(poll_id, command_buffer)
            };
            for &(state, range) in &ranges.states {
                unsafe {
                    scoped.bitor_state(
                        state,
                        range
                    );
                }
            }
            if let Some(s) = scoped.signal_semaphore {
                signal_semaphores.push(s);
            }
        }
        Ok(signal_semaphores)
    }

    #[inline]
    unsafe fn reset(&mut self) {
        self.images.clear();
    }
}

/// A command buffer handle.
pub struct CommandBuffer {
    pub(super) pool: CommandPool,
    label: Option<CompactStr>,
    pub(super) signal_sync_scope: vk::PipelineStageFlags2,
    primary: vk::CommandBuffer,
    secondary: vk::CommandBuffer,
    index: u32,
    semaphore_handle: vk::Semaphore,
    pub(super) semaphore_value: u64,
    binary_semaphore: Option<vk::Semaphore>,
    pub(super) state_cache: Option<ResourceStateCache>,
    pub(super) is_executable: bool,
    is_polled: bool,
}

impl CommandBuffer {

    #[inline]
    pub(super) unsafe fn new(
        pool: CommandPool, 
        primary: vk::CommandBuffer,
        secondary: vk::CommandBuffer,
        index: u32,
    ) -> Self {
        let mut semaphore_type = vk::SemaphoreTypeCreateInfo
            ::default()
            .semaphore_type(vk::SemaphoreType::TIMELINE);
        let semaphore_handle = unsafe {
            pool.device()
                .functions()
                .create_semaphore(
                    &vk::SemaphoreCreateInfo::default()
                        .push_next(&mut semaphore_type),
                    None,
                ).expect("failed to create `vkSemaphore`")
        };
        Self {
            pool,
            label: None,
            signal_sync_scope: vk::PipelineStageFlags2::empty(),
            primary,
            secondary,
            index,
            semaphore_handle,
            semaphore_value: 0,
            binary_semaphore: None,
            is_executable: false,
            is_polled: false,
            state_cache: Some(default()),
        }
    }

    /// Assigns a debug label for this command buffer.
    #[inline]
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.into());
        self
    }

    #[inline]
    pub(crate) fn with_binary_semaphore(mut self) -> Self {
        let handle = unsafe { self.pool()
            .device()
            .functions()
            .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
            .expect("failed to create `VkSemaphore`")
        };
        self.binary_semaphore = Some(handle);
        self
    }

    #[inline]
    pub(crate) fn get_binary_semaphore(&self) -> Option<vk::Semaphore> {
        self.binary_semaphore
    }

    /// Gets the label assigned to this command buffer with [`Self::with_label`].
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Gets the [`CommandPool`] which allocated this command buffer.
    #[inline]
    pub fn pool(&self) -> &CommandPool {
        &self.pool
    }

    #[inline]
    pub(crate) fn handle(&self) -> vk::CommandBuffer {
        self.secondary
    }

    /// Blocks the thread waiting for the command buffer to be unused and resets the command buffer.
    ///
    /// If the command buffer was successfully reset this returns `Ok(true)`, or
    /// if `timeout` was reached this returns `Ok(false)`. Otherwise returns an [`Err`].
    fn wait_and_reset(&mut self, timeout: Duration) -> Result<bool, Error> {
        if !self.is_executable {
            self.is_polled = false;
            return Ok(true)
        }
        unsafe {
            let res = self.pool
                .device()
                .functions()
                .wait_semaphores(
                    &vk::SemaphoreWaitInfo::default()
                        .semaphores(&[self.semaphore_handle])
                        .values(&[self.semaphore_value]),
                    timeout.as_nanos() as u64,
                ).describe_err("`vkWaitSemaphores` failed")?;
            if res != vk::Result::SUCCESS {
                return Ok(false)
            }
            self.pool
                .device()
                .functions()
                .reset_command_buffer(
                    self.primary,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES
                ).expect("`vkResetCommandBuffer` failed");
            self.pool
                .device()
                .functions()
                .reset_command_buffer(
                    self.secondary,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES
                ).expect("`vkResetCommandBuffer` failed");
            self.is_executable = false;
            self.is_polled = false;
            self.signal_sync_scope = vk::PipelineStageFlags2::empty();
            self.state_cache
                .as_mut().unwrap()
                .reset();
            Ok(true)
        }
    }

    /// Records commands with the given closure `f`.
    ///
    /// This will block the current thread until the command buffer can be safely reused. If this
    /// was not passed to [`Device::poll`] after a previous recording, this may halt the thread
    /// completely.
    ///
    /// # Parameters
    /// - `timeout`: a timeout used for waiting for the command buffer to be unused.
    /// - `f`: the closure used to record commands. It takes a single argument
    ///   [`CommandBufferRecorder`], which can be used to call commands with command buffer.
    pub fn start_recording(
        &mut self,
        timeout: Duration,
    ) -> Result<CommandBufferRecorder<'_>, Error>
    {
        if !self.wait_and_reset(timeout)? {
            Err(err!(
                "`timeout` reached while waiting for the command buffer",
            ))?
        }
        unsafe {
            Ok(CommandBufferRecorder::new(self))
        }
    }

    pub(crate) fn poll<'a>(
        &'a mut self,
        poll_id: u64,
        alloc: &'a dyn Allocator,
    ) -> Result<CommandBufferPollResult<'a>, Error>
    {
        if !self.is_executable {
            Err(err!(
                "`CommandBuffer` is not in the executable state",
            ))?
        }
        if self.is_polled {
            Err(err!(
                "`CommandBuffer` has already been polled",
            ))?
        }
        self.is_polled = true;
        let device = self.pool.device();
        let cache = self.state_cache.as_mut().unwrap();
        unsafe {
            device
                .functions()
                .begin_command_buffer(
                    self.primary,
                    &vk::CommandBufferBeginInfo {
                        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                        ..Default::default()
                    }
                ).expect("`vkBeginCommandBuffer` failed");
            let _end_guard = DropGuard::new(
                (device, self.primary),
                |(device, handle)| {
                    device
                        .functions()
                        .end_command_buffer(handle)
                        .expect("`vkEndCommandBuffer` failed");
                }
            );
            let mut signal_semaphores = cache.pipeline_barriers(
                poll_id,
                self.primary,
                1 + self.binary_semaphore.is_some() as u32,
                alloc,
            ).describe_err("allocation with `alloc` failed")?;
            device.functions()
                .cmd_execute_commands(
                    self.primary,
                    &[self.secondary],
                );
            let mut wait_semaphores = FixedVec32::with_capacity_in(
                signal_semaphores.len(),
                alloc,
            ).describe_err("allocation with `alloc` failed")?;
            wait_semaphores.extend(signal_semaphores
                .iter()
                .copied()
                .map(|(s, v)| (s, v - 1 ))
            );
            signal_semaphores.push((self.semaphore_handle, self.semaphore_value));
            if let Some(bin) = self.binary_semaphore {
                signal_semaphores.push((bin, 0));
            }
            Ok(CommandBufferPollResult {
                command_buffer: self.primary,
                wait_scope: vk::PipelineStageFlags2::TOP_OF_PIPE, 
                signal_scope: self.signal_sync_scope,
                wait_semaphores,
                signal_semaphores,
            })
        }
    }
}

pub(crate) struct CommandBufferPollResult<'a> {
    pub command_buffer: vk::CommandBuffer,
    pub wait_scope: vk::PipelineStageFlags2,
    pub signal_scope: vk::PipelineStageFlags2,
    pub wait_semaphores: FixedVec32<(vk::Semaphore, u64), &'a dyn Allocator>,
    pub signal_semaphores: FixedVec32<(vk::Semaphore, u64), &'a dyn Allocator>,
}

impl Drop for CommandBuffer {

    fn drop(&mut self) {
        if !self.wait_and_reset(Duration::from_secs_f32(2.0)).ok()
            .is_some_and(|s| s)
        {
            unsafe {
                self.pool.device()
                    .functions()
                    .device_wait_idle().expect("GPU hang");
                self.pool
                    .device()
                    .functions()
                    .reset_command_buffer(
                        self.primary,
                        vk::CommandBufferResetFlags::RELEASE_RESOURCES
                    ).expect("`vkResetCommandBuffer` failed");
                self.pool
                    .device()
                    .functions()
                    .reset_command_buffer(
                        self.secondary,
                        vk::CommandBufferResetFlags::RELEASE_RESOURCES
                    ).expect("`vkResetCommandBuffer` failed");
            }
        }
        unsafe {
            self.pool.unuse_command_buffer(self.index);
            self.pool.device()
                .functions()
                .destroy_semaphore(
                    self.semaphore_handle,
                    None,
                );
            if let Some(semaphore) = self.binary_semaphore {
                self.pool.device()
                    .functions()
                    .destroy_semaphore(
                        semaphore,
                        None,
                    );
            }
        }
    }
}
