use core::{
    ptr::NonNull,
    mem::forget,
    slice,
};

use tuhka::vk;

use crate::*;

use sync::atomic::{self, AtomicU64};

use mem::alloc::{self, LocalAlloc, pack_alloc, pack_block};

/// Parameters of a newly created [`Buffer`].
///
/// This **can** be used to create a [`Buffer`] with [`Device::create_buffer`].
///
/// # Valid usage
/// The valid parameters of this structure is described in each field.
#[derive(Clone, Copy)]
pub struct BufferAttributes<'a> {
    /// An optional debug label of the buffer.
    pub label: Option<&'a str>,
    /// The size of the buffer.
    ///
    /// # Valid usage
    /// This **must** be in the range to 1..[`max_buffer_size`][1].
    ///
    /// [1]: GpuLimits::max_buffer_size
    pub size: DeviceSize,
    /// What the buffer is used for.
    pub usage: BufferUsages,
    /// Specifies what type of memory this buffer is backed by.
    pub memory_type: MemoryType,
}

impl BufferAttributes<'_> {

    /// Calculates how much extra (CPU) memory the created [`Buffer`] will allocate.
    ///
    /// This is equal to `40 + label.len()`.
    #[inline]
    pub fn calculate_allocation_size(&self) -> usize {
        40 + self.label.map(|l| l.len()).unwrap_or_default()
    }
}

/// A GPU-accessible buffer.
///
/// To create a buffer, use [`Device::create_buffer`].
pub struct Buffer<A: LocalAlloc + Send + Sync = ()> {
    device: Device,
    handle: vk::Buffer,
    memory: NonNull<MemoryAllocation>,
    label: (Option<NonNull<u8>>, usize),
    ref_count: NonNull<AtomicU64>,
    pack_layout: alloc::Layout,
    allocator: A,
}

impl<A> Buffer<A>
    where A: LocalAlloc + Send + Sync
{

    pub(crate) fn new(
        device: Device,
        attributes: BufferAttributes<'_>,
        allocator: A,
    ) -> Result<Self, Error>
    {
        if attributes.size < 1 ||
            attributes.size > device.gpu_info().limits.max_buffer_size()
        {
            Err(err!(
                "buffer `size` `{}` is not in the range `1..max_buffer_size` (`1..{}`)",
                attributes.size,
                device.gpu_info().limits.max_buffer_size(),
            ))?
        }
        let unknown_usage = attributes.usage.difference(BufferUsages::all());
        if !unknown_usage.is_empty() {
            Err(err!(
                "unknown buffer usage bits `{:b}` set",
                unknown_usage.bits(),
            ))?
        }
        let (usage, usage64) = if let Some(bits) = attributes.usage.as_32_bit() {
            (vk::BufferUsageFlags::from_raw(bits), None)
        } else {
            (
                vk::BufferUsageFlags::empty(),
                Some(vk::BufferUsageFlags2::from_raw(attributes.usage.as_64_bit()))
            )
        };
        let mut flags2_create_info = None;
        let mut create_info = vk::BufferCreateInfo::default()
            .size(attributes.size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        if let Some(usage) = usage64  {
            if !device.is_extension_enbled(tuhka::khr::maintenance5::NAME) {
                Err(err!(
                    "`BufferUsages` doesn't fit in the 32-bit range and the {:?} extension is not enabled",
                    tuhka::khr::maintenance5::NAME,
                ))?
            }
            let info = flags2_create_info.insert(vk::BufferUsageFlags2CreateInfo
                ::default()
                .usage(usage)
            );
            create_info = create_info.push_next(info);
        }
        let handle = unsafe {
            device.functions()
                .create_buffer(&create_info, None)
        }.describe_err("failed to create `VkBuffer`")?;
        struct Guard<'a>(vk::Buffer, &'a Device);
        impl Drop for Guard<'_> {

            #[inline]
            fn drop(&mut self) {
                unsafe {
                    self.1.functions()
                        .destroy_buffer(self.0, None);
                }
            }
        }
        let guard = Guard(handle, &device);
        let memory_requirements = unsafe {
            device.functions()
                .get_buffer_memory_requirements(handle)
        };
        let memory_type_indices = attributes.memory_type
            .get_memory_type_indices(device.gpu_info());
        let memory = device.allocate_memory(
            memory_requirements.size,
            memory_requirements.alignment,
            &memory_type_indices
        ).describe_err("failed to allocate memory for buffer")?;
        unsafe {
            device.functions()
                .bind_buffer_memory(
                    handle,
                    memory.handle,
                    memory.offset
                ).describe_err("failed to bind buffer memory")?
        }
        let mut ref_count = NonNull::dangling();
        let mut memory_ptr = NonNull::dangling();
        let mut label_ptr = None;
        let mut label_len = 0;
        let pack_allocation = if let Some(label) = attributes.label {
            label_len = label.len();
            let label_ptr = label_ptr.insert(NonNull::dangling());
            let allocation = pack_alloc(
                &allocator,
                [
                    pack_block(&mut ref_count, 1).unwrap(),
                    pack_block(&mut memory_ptr, 1).unwrap(),
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
                    pack_block(&mut ref_count, 1).unwrap(),
                    pack_block(&mut memory_ptr, 1).unwrap(),
                ]
            ).expect("allocation failed")
        };
        assert!(pack_allocation.0.addr() == ref_count.addr());
        unsafe {
            ref_count.write(AtomicU64::new(1));
            memory_ptr.write(memory);
        }
        forget(guard);
        if let Some(label) = attributes.label {
            log::trace!("created `Buffer` `{label}`")
        }
        Ok(Self {
            device,
            handle,
            memory: memory_ptr,
            label: (label_ptr, label_len),
            ref_count,
            pack_layout: pack_allocation.1,
            allocator,
        })
    }

    /// Retrieves the assigned label of this buffer.
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

    /// Gets the a device address of this buffer, which may be used in a shader to access the
    /// buffer's memory.
    ///
    /// # Valid usage
    /// The [`buffer_device_address`] feature **must** be enabled, and the buffer **must** have
    /// been created with the [`SHADER_DEVICE_ADDRESS`] usage.
    ///
    /// [`buffer_device_address`]: vk_12::buffer_device_address
    /// [`SHADER_DEVICE_ADDRESS`]: BufferUsages::SHADER_DEVICE_ADDRESS
    pub fn shader_device_address(&self) -> Result<ShaderDeviceAddress, Error> {
        todo!()
    }
}

impl<A> Clone for Buffer<A>
    where A: Clone + LocalAlloc + Send + Sync
{

    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            self.ref_count
                .as_ref()
                .add(1, atomic::Ordering::Release);
        }
        Self {
            device: self.device.clone(),
            handle: self.handle,
            memory: self.memory,
            label: self.label,
            ref_count: self.ref_count,
            pack_layout: self.pack_layout,
            allocator: self.allocator.clone(),
        }
    }
}

impl<A> Drop for Buffer<A>
    where A: LocalAlloc + Send + Sync
{

    #[inline]
    fn drop(&mut self) {
        unsafe {
            let ref_count = self.ref_count
                .as_ref()
                .fetch_sub(1, atomic::Ordering::AcqRel);
            if ref_count == 1 {
                self.device
                    .functions()
                    .destroy_buffer(self.handle, None);
                self.device
                    .deallocate_memory(self.memory.read());
                if let Some(label) = self.label() {
                    log::trace!("destroyed `Buffer` `{label}`")
                }
                self.allocator
                    .deallocate(self.ref_count.cast(), self.pack_layout);
            }
        }
    }
}

pub struct BufferViewAttributes {
    pub offset: DeviceSize,
}
