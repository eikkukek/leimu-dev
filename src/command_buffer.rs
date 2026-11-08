use core::marker::PhantomData;

use bytemuck::Zeroable;

use tuhka::vk;

/// A Vulkan command buffer handle.
///
/// This structure has the same memory layout as a raw `VkCommandBuffer` handle, but is marked with
/// a lifetime and is explicitly not [`Send`] or [`Sync`] to indicate that this is just a raw handle
/// that shouldn't be stored or sent across threads.
#[repr(transparent)]
#[derive(Default, Clone, Copy, Zeroable)]
pub struct CommandBuffer<'a> {
    handle: vk::CommandBuffer,
    _marker: PhantomData<&'a *const ()>,
}

impl<'a> CommandBuffer<'a> {

    /// Extracts the raw `VkCommandBuffer` handle.
    #[inline]
    pub fn handle(self) -> vk::CommandBuffer {
        self.handle
    }
}
