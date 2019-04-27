use super::RenderBackend;
use crate::backend::ui::glutin::GlutinBackend;
use glium::Display;
use glutin::Context;

pub struct GliumRenderBackend {
    display: Display,
}

impl GliumRenderBackend {
    pub fn new(ui: &mut GlutinBackend) -> Result<GliumRenderBackend, Box<std::error::Error>> {
        let display = Display::from_gl_window(ui.take_context())?;
        Ok(GliumRenderBackend { display })
    }
}

impl RenderBackend for GliumRenderBackend {}
