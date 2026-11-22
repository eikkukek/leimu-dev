use std::rc::Rc;

use core::{
    task::{self, Waker, Poll},
    pin::Pin,
    ops::Deref,
    cell::Cell,
};

use portable_atomic as atomic;

use atomic::AtomicBool;

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
