mod map;

use std::alloc;

use core::{
    ptr::NonNull,
    mem::DropGuard,
    slice,
};

use tuhka::vk;

use crate::*;

use sync::atomic::{self, AtomicU64};

use mem::pack::{pack_alloc, pack_block};

pub use map::*;

/// Parameters used to create a [`Buffer`].
///
/// This **can** be used to create a [`Buffer`] with [`Device::create_buffer`].
///
/// # Valid usage
/// The valid parameters of this structure is described in each field.
pub type BufferAttributes<'a> = attributes::BufferAttributes<&'a str>;

impl BufferAttributes<'_> {

    /// Calculates how much extra (CPU) memory the created [`Buffer`] will allocate.
    ///
    /// Equal to to `72 + label.len()`.
    #[inline]
    pub fn calculate_allocation_size(&self) -> usize {
        size_of::<Inner>() + self.label.map(|l| l.len()).unwrap_or_default()
    }
}

struct Inner {
    device: Device,
    handle: vk::Buffer,
    ref_count: AtomicU64,
    usage: BufferUsages,
    size: DeviceSize,
    memory: MemoryAllocation,
    pack_layout: alloc::Layout,
}

/// A GPU-accessible buffer.
///
/// To create a buffer, use [`Device::create_buffer`].
pub struct Buffer {
    label: (Option<NonNull<u8>>, usize),
    inner: NonNull<Inner>,
    allocator: AnyAllocator,
}

impl Buffer
{

    pub(crate) fn new(
        device: Device,
        attributes: BufferAttributes<'_>,
        allocator: AnyAllocator,
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
        if attributes.usage.is_empty() {
            Err(err!(
                "`usage` is empty"
            ))?
        } else if !unknown_usage.is_empty() {
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
            if device.get_feature(vk_14::maintenance5::NAME).is_none() {
                Err(err!(
                    "`BufferUsages` doesn't fit in the 32-bit range, but the `maintenance5` feature is not enabled",
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
        let mut guard = DropGuard::new((&device, handle, None),
            |(device, handle, memory)| unsafe {
                device.functions()
                    .destroy_buffer(handle, None);
                if let Some(mem) = memory {
                    device.deallocate_memory(mem);
                }
            }
        );
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
        guard.2 = Some(memory);
        unsafe {
            device.functions()
                .bind_buffer_memory(
                    handle,
                    memory.handle,
                    memory.offset
                ).describe_err("failed to bind buffer memory")?
        }
        let mut inner = NonNull::dangling();
        let mut label_ptr = None;
        let mut label_len = 0;
        let pack_allocation = if let Some(label) = attributes.label {
            label_len = label.len();
            let label_ptr = label_ptr.insert(NonNull::dangling());
            let allocation = pack_alloc(
                &allocator,
                [
                    pack_block(&mut inner, 1).unwrap(),
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
                ]
            ).expect("allocation failed")
        };
        assert!(pack_allocation.0.addr() == inner.addr());
        DropGuard::dismiss(guard);
        unsafe {
            inner.write(Inner {
                device,
                handle,
                ref_count: AtomicU64::new(1),
                memory,
                usage: attributes.usage,
                size: attributes.size,
                pack_layout: pack_allocation.1,
            });
        }
        if let Some(label) = attributes.label {
            log::info!("created `Buffer` `{label}`")
        }
        Ok(Self {
            inner,
            label: (label_ptr, label_len),
            allocator,
        })
    }

    /// Gets the [`Device`] used to create this buffer.
    #[inline]
    pub fn device(&self) -> &Device {
        unsafe {
            &self.inner.as_ref().device
        }
    }

    /// Gets the label assigned to this buffer.
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

    /// Gets the size of this buffer, as specified in [`BufferAttributes::size`].
    #[inline]
    pub fn size(&self) -> DeviceSize {
        unsafe {
            self.inner
                .as_ref()
                .size
        }
    }

    /// Gets the usage of this buffer, as specified in [`BufferAttributes::usage`].
    #[inline]
    pub fn usage(&self) -> BufferUsages {
        unsafe {
            self.inner
                .as_ref()
                .usage
        }
    }

    #[inline]
    pub(crate) fn handle(&self) -> vk::Buffer {
        unsafe {
            self.inner
                .as_ref()
                .handle
        }
    }

    #[inline]
    pub(crate) fn memory(&self) -> &MemoryAllocation {
        unsafe {
            &self.inner
                .as_ref()
                .memory
        }
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

    /// Maps the buffer and passes [`MappedBuffer`] to the given closure, unmapping the memory
    /// after.
    ///
    /// 
    ///
    /// # Valid usage
    /// - The buffer memory backing **must** be mappable (e.g. by specifying `HostVisible*`
    ///   [`MemoryType`] during creation). It **may**  be mappable even with [`DeviceLocal`][1]
    ///   memory type, but you should **not** rely on that.
    ///
    /// [1]: MemoryType::DeviceLocal
    #[inline]
    pub fn map<F>(&self, map_mode: MapMode, f: F)
        where
            F: FnOnce(Result<MappedBuffer<'_>, Error>) -> Result<(), Error> + Send + 'static
    {
        let device = self.device();
        let memory = self.memory();
        if !device
            .gpu_info()
            .memory_properties
            .memory_types[memory.memory_type_index as usize]
            .property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
        {
            f(Err(err!(
                "`Buffer` `{}` is non-mappable",
                self.label().unwrap_or("unnamed"),
            ))).unwrap();
            return
        }
        struct UnmapGuard<'a>(&'a Buffer);
        impl Drop for UnmapGuard<'_> {
            #[inline]
            fn drop(&mut self) {
                unsafe {
                    self.0.device().functions()
                        .unmap_memory(self.0.memory().handle);
                }
            }
        }
        match unsafe { self.device().functions()
            .map_memory(
                memory.handle,
                memory.offset,
                memory.size,
                default(),
            )
        } {
            Ok(ptr) => {
                let _guard = UnmapGuard(self);
                let memory_range = vk::MappedMemoryRange::default()
                    .memory(memory.handle)
                    .offset(memory.offset)
                    .size(memory.size);
                unsafe {
                    if let Err(err) = device
                        .functions()
                        .invalidate_mapped_memory_ranges(&[memory_range])
                    {
                        f(Err(Error::new(Some(err), "failed to invalidate memory ranges")))
                            .unwrap();
                        return;
                    }
                }
                let mapped = MappedBuffer {
                    buffer: self,
                    ptr: NonNull::new(ptr).unwrap().cast(),
                    range: memory.size,
                    mode: map_mode,
                };
                f(Ok(mapped)).unwrap();
                unsafe {
                    device
                        .functions()
                        .flush_mapped_memory_ranges(&[memory_range])
                        .unwrap()
                }
            },
            Err(err) => {
                f(Err(Error::new(Some(err), "failed to map `VkMemory`")))
                    .unwrap();
            },
        }
    }
}

impl Clone for Buffer
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

impl Drop for Buffer
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
                    .destroy_buffer(self.handle(), None);
                self.device()
                    .deallocate_memory(*self.memory());
                if let Some(label) = self.label() {
                    log::info!("destroyed `Buffer` `{label}`")
                }
                let pack_layout = self.inner.as_ref().pack_layout;
                self.inner.drop_in_place();
                self.allocator
                    .deallocate(self.inner.cast(), pack_layout);
            }
        }
    }
}

pub struct BufferViewAttributes {
    pub offset: DeviceSize,
}
