use std::rc::Rc;

use core::{
    task::{self, Waker, Poll},
    pin::Pin,
    ops::Deref,
    sync::atomic::{self, AtomicBool},
    cell::Cell,
};

use leimu::{DescribeResult, raw_window_handle::HasDisplayHandle};

use leimu::mem::arena::Arena;

pub use winit;

use winit::{
    application::ApplicationHandler, event_loop::ActiveEventLoop,
};

pub enum Event {
    Unset,
    Resumed,
    WindowEvent {
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    },
}

struct Polling<'a> {
    polling: &'a AtomicBool,
    exiting: &'a AtomicBool,
}

impl Future for Polling<'_> {

    type Output = bool;

    fn poll(self: Pin<&mut Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        let polling = self.polling.fetch_not(atomic::Ordering::Relaxed);
        if polling {
            Poll::Ready(!self.exiting.load(atomic::Ordering::Relaxed))
        } else {
            Poll::Pending
        }
    }
}

struct Inner {
    polling: AtomicBool,
    exiting: AtomicBool,
    event_loop: Cell<*const ActiveEventLoop>,
    event: Cell<Event>,
}

#[derive(Clone)]
pub struct EventLoop {
    inner: Rc<Inner>,
}

impl EventLoop {

    #[inline]
    pub async fn poll(&mut self) -> bool {
        (&mut Polling { polling: &self.inner.polling, exiting: &self.inner.exiting, }).await
    }

    /// Takes the current event of the event loop, replacing the value with [`Event::Unset`].
    #[inline]
    pub fn event(&self) -> Event {
        self.inner.event.replace(Event::Unset)
    }
}

impl Deref for EventLoop {

    type Target = ActiveEventLoop;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let event_loop = self.inner.event_loop.get();
        assert!(!event_loop.is_null());
        unsafe { &*event_loop }
    }
}

type PinBoxFuture<Output> = Pin<Box<dyn Future<Output = Output>>>;

struct App<F, E>
    where F: AsyncFnOnce(EventLoop) -> Result<(), E> + 'static,
{
    resumed: bool,
    f: Option<F>,
    f_future: Option<PinBoxFuture<Result<(), E>>>,
    result: Option<Result<(), E>>,
    event_loop: EventLoop,
}

impl<F, E> App<F, E>
    where F: AsyncFnOnce(EventLoop) -> Result<(), E> + 'static,
{

    pub fn new(f: F) -> Self {
        Self {
            resumed: false,
            f: Some(f),
            f_future: None,
            result: None,
            event_loop: EventLoop {
                inner: Rc::new(Inner {
                    polling: AtomicBool::new(true),
                    exiting: AtomicBool::new(false),
                    event_loop: Cell::new(core::ptr::null()),
                    event: Cell::new(Event::Resumed),
                }),
            },
        }
    }

    fn poll_once(&mut self, event_loop: &ActiveEventLoop, event: Event) -> Option<Result<(), E>> {
        let f_future =
            if let Some(f) = &mut self.f_future {
                f
            } else {
                self.f_future.insert(Box::pin((self.f.take().unwrap())(self.event_loop.clone())))
            };
        self.event_loop.inner.event_loop.set(
            (event_loop as *const ActiveEventLoop).cast_mut(),
        );
        self.event_loop.inner.event.set(event);
        let mut cx = task::Context::from_waker(Waker::noop());
        let res = match f_future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => {
                Some(result)
            },
            Poll::Pending => None,
        };
        self.event_loop.inner.event_loop.set(core::ptr::null());
        res
    }
}

impl<F, E> ApplicationHandler for App<F, E>
    where F: AsyncFnOnce(EventLoop) -> Result<(), E>,
{

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.resumed {
            return
        }
        self.resumed = true;
        if let Some(result) = self.poll_once(event_loop, Event::Resumed) {
            self.result = Some(result);
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    )
    {
        let exiting = event_loop.exiting();
        self.event_loop.inner.exiting.store(exiting, atomic::Ordering::Relaxed);
        if exiting {
            return
        }
        if let Some(result) = self.poll_once(
            event_loop,
            Event::WindowEvent { window_id, event },
        ) {
            self.result = Some(result);
            event_loop.exit();
        }
    }
}

#[must_use]
pub fn run_app<F, E>(app: F) -> Option<Result<(), E>>
    where F: AsyncFnOnce(EventLoop) -> Result<(), E> + 'static
{
    let event_loop = winit::event_loop
        ::EventLoop::new().expect("failed to create `winit::event_loop::EventLoop");
    let mut app = App::new(app);
    event_loop.run_app(&mut app).expect("failed to run app");
    app.result
}

fn main() -> Result<(), leimu::Error> {
    env_logger::init();
    if let Some(res) = run_app::<_, leimu::Error>(async |mut event_loop| {
        let instance = leimu::Instance
            ::new_in_display(
                event_loop.display_handle().expect("failed to get display handle")
                    .as_raw()
            )?;
        let window = event_loop.create_window(winit::window::WindowAttributes::default()
            .with_title("test")
        ).describe_err("failed to create window")?;
        let surface = instance.create_surface(Some("test surface"), &window)?;
        let suitable_gpus = instance
            .find_suitable_gpus(leimu::DeviceAttributes {
                required_features: leimu::Features::from_iter([
                    leimu::vk_khr::swapchain::require(),
                    leimu::vk_core::scalar_block_layout::require(),
                    leimu::vk_core::shader_buffer_int64_atomics::require(),
                    leimu::vk_core::shader_int64::require(),
                ]),
            })?;
        let gpu = suitable_gpus.gpus()
            .find(|gpu| gpu.gpu_type == leimu::GpuType::Discrete)
            .unwrap_or(suitable_gpus.get_gpu(0));
        let present_family = suitable_gpus
            .get_gpu_present_support(gpu.index, &surface)
            .next().unwrap();
        let device = suitable_gpus.create_device(
            gpu.index,
            [leimu::DeviceQueueAttributes {
                queue_family_index: present_family.index,
                queue_priorities: vec![1.0],
                queue_tags: vec!["present queue"],
            }],
            [],
        )?;
        let comp_spv = include_bytes!("../comp.spv");
        device.create_shader_module(
            leimu::ShaderModuleAttributes {
                tag: Some("compute shader"),
                spirv: leimu::bytemuck::cast_slice(comp_spv),
            },
            None,
        ).describe_err("failed to validate shader")?;
        let present_queue = device.get_queue("present queue").unwrap();
        let swapchain = device.create_swapchain(
            Some("swapchain"),
            surface,
            present_queue.clone(),
        )?;
        let command_pool = device.create_command_pool(
            Some("main command pool"),
            present_queue.clone(),
        )?;
        let mut command_buffer = command_pool
            .allocate_command_buffer()?
            .with_tag("render pass command buffer");
        let tmp_arena = Arena
            ::with_fallback(1 << 16)
            .unwrap();
        while event_loop.poll().await {
            if let Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } = event_loop.event() {
                break;
            }
            let window_size = window.inner_size();
            swapchain.present(
                leimu::PresentParameters {
                    window_width: window_size.width,
                    window_height: window_size.height,
                    desired_image_count: 3,
                    clip: true,
                    force_recreate: false,
                    select_surface_format: &mut |formats| {
                        formats.iter()
                            .enumerate()
                            .find_map(|(i, format)|
                                if format.color_space == leimu::ColorSpace::SRGB_NONLINEAR_KHR
                                    && format.format == leimu::Format::R8g8b8a8_Srgb
                                {
                                    Some(i)
                                } else {
                                    None
                                }
                            ).unwrap_or(0)
                    },
                    select_composite_alpha: &mut |_| leimu::CompositeAlphaBit::Opaque
                },
                std::time::Duration::from_secs_f32(1.0),
                None,
                |image_view| {
                    let mut recorder = command_buffer.start_recording(
                        leimu::duration_from_secs(1.0),
                    )?;
                    let pass = recorder.render_pass(
                        leimu::RenderPassAttributes {
                            tag: Some("clear pass"),
                            render_area: leimu::RenderArea {
                                width: window_size.width,
                                height: window_size.height,
                                ..Default::default()
                            },
                            color_attachments: &[leimu::ColorAttachment::new(
                                &image_view,
                                None,
                                leimu::AttachmentLoadOp::Clear,
                                leimu::AttachmentStoreOp::Store,
                                Some(leimu::ClearColorValue::F32([
                                    1.0,
                                    35.0 / 255.0,
                                    82.0 / 255.0,
                                    1.0,
                                ]))
                            )?],
                            depth_stencil_attachment: leimu::DepthStencilAttachment::none(),
                            ..Default::default()
                        },
                        &tmp_arena,
                    )?;
                    pass.finish();
                    recorder.finish();
                    device.poll([&mut command_buffer], Some(&tmp_arena))?;
                    unsafe {
                        tmp_arena.clear();
                    }
                    Ok(())
                }
            )?;
            window.request_redraw();
        }
        Ok(())
    }) {
        res?
    }
    Ok(())
}
