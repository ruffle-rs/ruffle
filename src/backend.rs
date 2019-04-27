pub mod render;
pub mod ui;

use render::RenderBackend;
use ui::UiBackend;

pub fn build() -> Result<(Box<UiBackend>, Box<RenderBackend>), Box<std::error::Error>> {
    let mut ui = ui::glutin::GlutinBackend::new(500, 500)?;
    let renderer = render::glium::GliumRenderBackend::new(&mut ui)?;
    Ok((Box::new(ui), Box::new(renderer)))
}
