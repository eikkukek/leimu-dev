use raw_window_handle::{HasWindowHandle, HasDisplayHandle};

use tuhka::{vk, window::create_surface};

use crate::*;

use sync::Arc;

/// An auto-trait for types that implement [`HasWindowHandle`] and [`HasDisplayHandle`].
pub trait WindowHandle: HasWindowHandle + HasDisplayHandle {}

impl<T> WindowHandle for T
    where T: HasWindowHandle + HasDisplayHandle
{}

struct Inner {
    tag: Option<CompactStr>,
    instance: Instance,
    handle: vk::SurfaceKHR,
}

impl Drop for Inner {

    fn drop(&mut self) {
        unsafe {
            self.instance
                .surface_functions()
                .unwrap()
                .destroy_surface(self.handle, None);
            if let Some(tag) = &self.tag {
                log::info!("destroyed `Surface` `{tag}`");
            }
        }
    }
}

/// A Vulkan surface attached to a window.
///
/// To create a surface, call [`Instance::create_surface`].
#[derive(Clone)]
#[must_use]
pub struct Surface<'a> {
    inner: Arc<Inner>,
    _window: &'a dyn WindowHandle,
}

impl<'a> Surface<'a> {

    pub(crate) fn new(
        instance: Instance,
        tag: Option<&str>,
        window: &'a dyn WindowHandle,
    ) -> Result<Self, Error> {
        if let Some(tag) = tag {
            log::info!("validating `Surface` `{tag}`");
        }
        let display_handle = window
            .display_handle()
            .map_err(|_| err!("failed to get display handle"))?
            .as_raw();
        if instance.display_handle().is_none_or(|d| !d.eq(&display_handle)) {
            Err(err!(
                "`window` seems to have a different display handle than `instance`",
            ))?
        }
        let handle = unsafe { create_surface(
            instance.functions(),
            display_handle,
            window.window_handle()
                .map_err(|_| err!("failed to get window handle"))?
                .as_raw(),
            None,
        ).describe_err("failed to create `VkSurfaceKHR")? };
        if let Some(tag) = tag {
            log::info!("created `Surface` `{tag}`");
        }
        Ok(Self {
            inner: Inner {
                tag: tag.map(|tag| tag.into()),
                instance,
                handle,
            }.arced(),
            _window: window,
        })
    }

    /// Gets the tag assigned to this surface.
    #[inline]
    pub fn tag(&self) -> Option<&str> {
        self.inner.tag.as_deref()
    }

    /// Gets the [`Instance`] used to create this surface.
    #[inline]
    pub fn instance(&self) -> &Instance {
        &self.inner.instance
    }

    #[inline]
    pub(crate) fn handle(&self) -> vk::SurfaceKHR {
        self.inner.handle
    }
}
