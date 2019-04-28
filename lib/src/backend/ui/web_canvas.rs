use super::UiBackend;
use web_sys::HtmlCanvasElement;

pub struct WebCanvasBackend {
    canvas: HtmlCanvasElement,
}

impl WebCanvasBackend {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, Box<std::error::Error>> {
        use log::Level;
        console_log::init_with_level(Level::Trace)?;

        Ok(Self {
            canvas
        })
    }
}

impl UiBackend for WebCanvasBackend {
    fn poll_events(&mut self) -> bool {
        true
    }
}
