use super::UiBackend;
use glutin::{ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent, WindowedContext};

pub struct GlutinBackend {
    context: Option<WindowedContext>,
    events_loop: EventsLoop,
}

impl GlutinBackend {
    pub fn new(width: u32, height: u32) -> Result<GlutinBackend, Box<std::error::Error>> {
        let events_loop = EventsLoop::new();
        let window_builder = WindowBuilder::new().with_dimensions((width, height).into());
        let context = ContextBuilder::new().build_windowed(window_builder, &events_loop)?;
        Ok(GlutinBackend {
            context: Some(context),
            events_loop,
        })
    }

    pub fn take_context(&mut self) -> WindowedContext {
        self.context.take().unwrap()
    }
}

impl UiBackend for GlutinBackend {
    fn poll_events(&mut self) -> bool {
        let mut request_close = false;
        self.events_loop.poll_events(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => request_close = true,
            _ => (),
        });

        !request_close
    }
}
