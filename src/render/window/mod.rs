use std::time::{Duration, Instant};

use glutin::{
    dpi::{PhysicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};

use super::opengl::context::GlContext;

/// The game window. Handles window events,
/// management of the OpenGL context and key presses.
pub struct GameWindow {
    /// The OpenGL context.
    context: GlContext,
    /// The window size.
    window_size: PhysicalSize<u32>,
    /// Set to `None` as soon as `run` is called.
    event_loop: Option<EventLoop<()>>,
    // Delta time.
    delta_time: Duration,
    last_frame: Instant,
    /// Cursor grabbed?
    cursor_grabbed: bool,
}
impl GameWindow {
    /// Creates a new GL context and game window of size `s`.
    pub fn new(s: impl Into<Size> + Copy) -> Self {
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new().with_inner_size(s);

        let gl_window = ContextBuilder::new()
            .build_windowed(window, &event_loop)
            .unwrap();

        let gl_window = unsafe { gl_window.make_current().unwrap() };
        gl::load_with(|symbol| gl_window.get_proc_address(symbol));

        let mut context = GlContext::new(gl_window);
        Self {
            window_size: s.into().to_physical(1.),
            context,
            event_loop: Some(event_loop),
            delta_time: Duration::new(0, 0),
            last_frame: Instant::now(),
            cursor_grabbed: false,
        }
    }

    /// Begin running the event loop. This runs until the window closes.
    /// `event_handler` is ONLY called on events not handled by the window.
    pub fn run(mut self, mut event_handler: impl FnMut(&mut GameWindow, Event<()>) + 'static) {
        self.event_loop
            .take()
            .unwrap()
            .run(move |event, _, control_flow| {
                let current_frame = Instant::now();
                self.delta_time = current_frame - self.last_frame;
                self.last_frame = current_frame;
                *control_flow = ControlFlow::Poll;
                // we want to pass this one up the chain
                if let Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } = event
                {
                    unsafe {
                        gl::Viewport(0, 0, size.width as i32, size.height as i32);
                        self.window_size = size;
                    }
                }
                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        *control_flow = ControlFlow::Exit;
                    }
                    e => event_handler(&mut self, e),
                }
            });
    }

    /// Retrieves the OpenGL context for this window.
    pub fn context(&mut self) -> &mut GlContext {
        &mut self.context
    }

    /// Retrieves the current delta time.
    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    /// Is the cursor grabbed?
    pub fn cursor_grabbed(&self) -> bool {
        self.cursor_grabbed
    }

    /// Set whether the cursor is grabbed.
    pub fn set_cursor_grabbed(&mut self, state: bool) -> anyhow::Result<()> {
        self.cursor_grabbed = state;
        let w = self.context.get_context().window();
        w.set_cursor_grab(self.cursor_grabbed)?;
        w.set_cursor_visible(!self.cursor_grabbed);
        Ok(())
    }


    /// Get the window size.
    pub fn size(&self) -> PhysicalSize<u32> {
        self.window_size
    }
}
