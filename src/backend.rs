pub mod render;
pub mod ui;

use render::RenderBackend;
use ui::UiBackend;

#[cfg(not(target_arch = "wasm32"))]
pub fn build() -> Result<(Box<UiBackend>, Box<RenderBackend>), Box<std::error::Error>> {
    let mut ui = ui::glutin::GlutinBackend::new(500, 500)?;
    let renderer = render::glium::GliumRenderBackend::new(&mut ui)?;
    Ok((Box::new(ui), Box::new(renderer)))
}

#[cfg(target_arch = "wasm32")]
pub fn build() -> Result<(Box<UiBackend>, Box<RenderBackend>), Box<std::error::Error>> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("Expected window")?;
    let document = window.document().ok_or("Expected document")?;
    let canvas: web_sys::HtmlCanvasElement = document
        .get_element_by_id("fluster-canvas")
        .ok_or("Missing element")?
        .dyn_into()
        .map_err(|_| "Not a canvas")?;
    let (width, height) = (f64::from(canvas.width()), f64::from(canvas.height()));
    let context: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .map_err(|_| "Unable to make canvas context")?
        .ok_or("Unable to make canvas context")?
        .dyn_into()
        .map_err(|_| "Not a CanvasRenderingContext2d")?;

    let mut ui = ui::web_canvas::WebCanvasBackend::new(canvas)?;
    let renderer = render::web_canvas::WebCanvasRenderBackend::new(context, width, height);
    Ok((Box::new(ui), Box::new(renderer)))
}
