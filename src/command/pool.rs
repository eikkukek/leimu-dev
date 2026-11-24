use core::{
    ptr::NonNull,
    mem::DropGuard,
};

use tuhka::vk;

use std::alloc::{self, Global, Allocator};

use leimu_mem::pack::{pack_alloc, pack_block};

use crate::*;

use sync::{Mutex, Arc};

struct Buffers {
    primary_buffers: NonNull<vk::CommandBuffer>,
    secondary_buffers: NonNull<vk::CommandBuffer>,
    used: NonNull<bool>,
    len: u32,
    pack_alloc: (NonNull<u8>, alloc::Layout),
}

impl Buffers {
    
    #[inline]
    unsafe fn primary_buffers(&self) -> &[vk::CommandBuffer] {
        unsafe {
            NonNull::slice_from_raw_parts(
                self.primary_buffers,
                self.len as usize
            ).as_ref()
        }
    }

    #[inline]
    unsafe fn secondary_buffers(&self) -> &[vk::CommandBuffer] {
        unsafe {
            NonNull::slice_from_raw_parts(
                self.secondary_buffers,
                self.len as usize
            ).as_ref()
        }
    } 

    #[inline]
    unsafe fn used_mut(&mut self) -> &mut [bool] {
        unsafe {
            NonNull::slice_from_raw_parts(
                self.used,
                self.len as usize
            ).as_mut()
        }
    }
}

struct Inner {
    tag: Option<CompactStr>,
    device: Device,
    queue: DeviceQueue,
    handle: vk::CommandPool,
    buffers: Mutex<Buffers>,
}

impl Drop for Inner {

    fn drop(&mut self) {
        unsafe {
            self.device
                .functions()
                .device_wait_idle()
                .expect("GPU hang");
            self.device.use_allocation_callbacks(|cb| self.device
                .functions()
                .destroy_command_pool(
                    self.handle,
                    cb,
                ),
            );
            if let Some(tag) = &self.tag {
                log::info!("destroyed `CommandPool` `{tag}`");
            }
        }
    }
}

/// A command pool, which **can** be used to allocate [command buffers] with
/// [`Self::allocate_command_buffer`].
///
/// To create a command pool, use [`Device::create_command_pool`].
///
/// [command buffers]: CommandBuffer
#[derive(Clone)]
pub struct CommandPool {
    inner: Arc<Inner>,
}

impl CommandPool {

    pub(crate) fn new(
        device: Device,
        tag: Option<&str>,
        queue: DeviceQueue,
    ) -> Result<Self, Error> {
        if let Some(tag) = tag {
            log::info!("validating `CommandPool` `{tag}`");
        }
        if device.id() != queue.device_id() {
            panic!("queue `{}` originates from a different device", queue.tag());
        }
        let create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue.family_index())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let handle = device.use_allocation_callbacks(|cb| unsafe { device
            .functions()
            .create_command_pool(
                &create_info,
                cb,
            )
        }).describe_err("failed to create `VkCommandPool`")?;
        Ok(Self {
            inner: Inner {
                tag: tag.map(Into::into),
                device,
                queue,
                handle,
                buffers: Mutex::new(Buffers {
                    primary_buffers: NonNull::dangling(),
                    secondary_buffers: NonNull::dangling(),
                    used: NonNull::dangling(),
                    len: 0,
                    pack_alloc: (NonNull::dangling(), alloc::Layout::new::<()>())
                }),
            }.arced(),
        })
    }

    fn grow(current: u32) -> u32 {
        if current == 0 {
            2
        } else {
            current * 2
        }
    }

    /// Gets the tag assigned to this command pool.
    #[inline]
    pub fn tag(&self) -> Option<&str> {
        self.inner.tag.as_deref()
    }

    /// Gets the [`Device`] this command pool was created with.
    #[inline]
    pub fn device(&self) -> &Device {
        &self.inner.device
    }

    /// Gets the [`DeviceQueue`] of this command pool.
    #[inline]
    pub fn queue(&self) -> &DeviceQueue {
        &self.inner.queue
    }

    /// Allocates a [`CommandBuffer`] from this pool.
    pub fn allocate_command_buffer(&self) -> Result<CommandBuffer, Error> {
        let mut buffers = self.inner.buffers.lock();
        if let Some((index, used)) = unsafe { buffers.used_mut() }
            .iter_mut()
            .enumerate()
            .find(|(_, used)| !**used)
        {
            *used = true;
            let &primary = unsafe { buffers.primary_buffers().get_unchecked(index) };
            let &secondary = unsafe { buffers.secondary_buffers().get_unchecked(index) };
            unsafe {
                Ok(CommandBuffer::new(
                    self.clone(),
                    primary,
                    secondary,
                    index as u32,
                ))
            }
        } else {
            let new_len = Self::grow(buffers.len);
            let len_diff = (new_len - buffers.len) as usize;
            let mut new_primary = NonNull::dangling();
            let mut new_secondary = NonNull::dangling();
            let mut new_used = NonNull::dangling();
            let new_alloc = pack_alloc(
                &Global,
                [
                    pack_block(&mut new_primary, new_len as usize).expect("layout error"),
                    pack_block(&mut new_secondary, new_len as usize).expect("layout error"),
                    pack_block(&mut new_used, new_len as usize).expect("layout error")
                ]
            ).unwrap();
            let guard = DropGuard::new((new_alloc.0, new_alloc.1),
                |alloc| unsafe {
                    Global.deallocate(
                        alloc.0, alloc.1
                    );
                },
            );
            unsafe {
                new_used.add(buffers.len as usize).write_bytes(0, len_diff);
                let new_primary = NonNull::slice_from_raw_parts(
                    new_primary.add(buffers.len as usize),
                    len_diff
                ).as_mut();
                let new_secondary = NonNull::slice_from_raw_parts(
                    new_secondary.add(buffers.len as usize),
                    len_diff
                ).as_mut();
                self.inner.device
                    .functions()
                    .allocate_command_buffers(
                        &vk::CommandBufferAllocateInfo::default()
                            .command_pool(self.inner.handle)
                            .level(vk::CommandBufferLevel::PRIMARY)
                            .command_buffer_count(new_primary.len() as u32),
                        new_primary,
                    ).describe_err("failed to allocate command buffers")?;
                self.inner.device
                    .functions()
                    .allocate_command_buffers(
                        &vk::CommandBufferAllocateInfo::default()
                            .command_pool(self.inner.handle)
                            .level(vk::CommandBufferLevel::SECONDARY)
                            .command_buffer_count(new_secondary.len() as u32),
                        new_secondary,
                    ).describe_err("failed to allocate command buffers")?;
            }
            DropGuard::dismiss(guard);
            unsafe {
                buffers.primary_buffers
                    .copy_to_nonoverlapping(
                        new_primary, buffers.len as usize,
                    );
                buffers.secondary_buffers
                    .copy_to_nonoverlapping(
                        new_secondary, buffers.len as usize,
                    );
                buffers.used
                    .copy_to_nonoverlapping(
                        new_used, buffers.len as usize,
                    );
                if buffers.pack_alloc.1.size() != 0 {
                    Global.deallocate(buffers.pack_alloc.0, buffers.pack_alloc.1);
                }
            }
            let index = buffers.len;
            *buffers = Buffers {
                primary_buffers: new_primary,
                secondary_buffers: new_secondary,
                used: new_used,
                len: new_len,
                pack_alloc: new_alloc
            };
            unsafe {
                buffers.used_mut()[index as usize] = true;
            }
            unsafe {
                Ok(CommandBuffer::new(
                    self.clone(),
                    buffers.primary_buffers()[index as usize],
                    buffers.secondary_buffers()[index as usize],
                    index
                ))
            }
        }
    }

    pub(crate) unsafe fn unuse_command_buffer(&self, index: u32) {
        unsafe {
            let mut buffers = self.inner.buffers.lock();
            buffers.used_mut()[index as usize] = false;
        }
    }
}
