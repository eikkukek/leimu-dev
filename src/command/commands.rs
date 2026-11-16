use tuhka::vk;

use crate::*;

/// This can be used to record commands with a [`CommandBuffer`].
///
/// To start recording use the [`CommandBuffer::record`] command.
pub struct CommandBufferRecorder<'buffer> {
    pub(super) command_buffer: &'buffer mut CommandBuffer,
}

impl<'buffer> CommandBufferRecorder<'buffer> {

    #[inline]
    pub(super) unsafe fn new(buffer: &'buffer mut CommandBuffer) -> Self {
        unsafe {
            let inheritance_info = vk::CommandBufferInheritanceInfo::default();
            buffer.pool().device()
                .functions()
                .begin_command_buffer(
                    buffer.handle(),
                    &vk::CommandBufferBeginInfo::default()
                    .inheritance_info(Some(&inheritance_info)),
                ).expect("`vkBeginCommandBuffer` failed");
        }
        Self { command_buffer: buffer, }
    }

    #[inline]
    pub(crate) fn transition_image_state(
        &mut self,
        image_view: ImageView,
        state: ImageState,
    ) {
        let mut state_cache = self.command_buffer.state_cache
            .take()
            .unwrap();
        unsafe {
            state_cache.insert_image(
                image_view.image().clone(),
                state,
                image_view.subresource_range(),
            );
        }
        self.command_buffer.state_cache = Some(state_cache);
    }

    /// Finishes recording commands.
    #[inline]
    pub fn finish(self) {}
}

impl<'a> Drop for CommandBufferRecorder<'a> {

    fn drop(&mut self) {
        unsafe {
            self.command_buffer
                .pool()
                .device()
                .functions()
                .end_command_buffer(
                    self.command_buffer.handle(),
                ).expect("`vkEndCommandBuffer` failed");
            self.command_buffer.is_executable = true;
            self.command_buffer.semaphore_value += 1;
        }
    }
}
